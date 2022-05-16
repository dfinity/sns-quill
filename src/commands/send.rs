use crate::{
    commands::request_status,
    lib::{
        get_ic_url, parse_query_response, read_from_file,
        signing::{CallType, Ingress, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
};
use anyhow::{anyhow, Context};
use candid::Decode;
use clap::Parser;
use ic_agent::{
    agent::{http_transport::ReqwestHttpReplicaV2Transport, ReplicaV2Transport},
    RequestId,
};
use ic_base_types::PrincipalId;
use ic_ic00_types::CanisterStatusResultV2;
use ic_sns_governance::pb::v1::{ManageNeuronResponse, NervousSystemParameters};
use ledger_canister::{BlockHeight, Tokens, TransferError};
use std::str::FromStr;

/// Sends a signed message or a set of messages.
#[derive(Parser)]
pub struct SendOpts {
    /// Path to the signed message. Use "-" for STDIN.
    file_name: String,

    /// Will display the signed message, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Skips confirmation and sends the message directly.
    #[clap(long)]
    yes: bool,
}

pub async fn exec(opts: SendOpts) -> AnyhowResult {
    let json = read_from_file(&opts.file_name)?;
    if let Ok(val) = serde_json::from_str::<Ingress>(&json) {
        send(&val, &opts).await?;
    } else if let Ok(vals) = serde_json::from_str::<Vec<Ingress>>(&json) {
        for msg in vals {
            send(&msg, &opts).await?;
        }
    } else if let Ok(vals) = serde_json::from_str::<Vec<IngressWithRequestId>>(&json) {
        for tx in vals {
            send_ingress_and_check_status(&tx, &opts).await?;
        }
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }
    Ok(())
}

pub async fn send_unsigned_ingress(
    method_name: &str,
    args: Vec<u8>,
    dry_run: bool,
    target_canister: TargetCanister,
) -> AnyhowResult {
    let msg = crate::lib::signing::sign("", method_name, args, target_canister)?;
    let ingress = msg.message;
    send(
        &ingress,
        &SendOpts {
            file_name: Default::default(), // Not used.
            yes: false,
            dry_run,
        },
    )
    .await
}

/// Submits a ingress message to the Internet Computer and retrieves a reply.
async fn send_ingress_and_check_status(
    message: &IngressWithRequestId,
    opts: &SendOpts,
) -> AnyhowResult {
    send(&message.ingress, opts).await?;
    if opts.dry_run {
        return Ok(());
    }
    let (_, _, method_name, _) = message.ingress.parse()?;
    let result = request_status::submit(&message.request_status).await?;
    print_response(result, method_name)?;
    Ok(())
}

/// Sends a message to the Internet Computer.
async fn send(message: &Ingress, opts: &SendOpts) -> AnyhowResult {
    let (sender, canister_id, method_name, args) = message.parse()?;

    println!("Sending message with\n");
    println!("  Call type:   {}", message.call_type);
    println!("  Sender:      {}", sender);
    println!("  Canister id: {}", canister_id);
    println!("  Method name: {}", method_name);
    println!("  Arguments:   {}", args);

    if opts.dry_run {
        return Ok(());
    }

    if message.call_type == CallType::Update && !opts.yes {
        println!("\nDo you want to send this message? [y/N]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !["y", "yes"].contains(&input.to_lowercase().trim()) {
            std::process::exit(0);
        }
    }

    let transport = ReqwestHttpReplicaV2Transport::create(get_ic_url())?;
    let content = hex::decode(&message.content)?;

    match message.call_type {
        CallType::Query => {
            let response = parse_query_response(transport.query(canister_id, content).await?)?;
            print_response(response, method_name)?;
        }
        CallType::Update => {
            let request_id = RequestId::from_str(
                &message
                    .clone()
                    .request_id
                    .context("Cannot get request_id from the update message")?,
            )?;
            let formatted_request_id = format!("0x{}", String::from(request_id));
            println!("Request ID: {}", formatted_request_id);
            transport.call(canister_id, content, request_id).await?;
        }
    }
    Ok(())
}

enum SupportedResponse {
    ManageNeuron,
    Transfer,
    AccountBalance,
    SnsCanistersSummary,
    NervousSystemParameters,
}

impl FromStr for SupportedResponse {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<SupportedResponse, Self::Err> {
        match input {
            "account_balance" => Ok(SupportedResponse::AccountBalance),
            "transfer" => Ok(SupportedResponse::Transfer),
            "manage_neuron" => Ok(SupportedResponse::ManageNeuron),
            "get_sns_canisters_summary" => Ok(SupportedResponse::SnsCanistersSummary),
            "get_nervous_system_parameters" => Ok(SupportedResponse::NervousSystemParameters),
            unsupported_response => Err(anyhow!(
                "{} is not a supported response",
                unsupported_response
            )),
        }
    }
}

fn print_response(blob: Vec<u8>, method_name: String) -> AnyhowResult {
    let response_type = SupportedResponse::from_str(method_name.as_str())?;

    match response_type {
        SupportedResponse::AccountBalance => {
            let response = Decode!(blob.as_slice(), Tokens)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::Transfer => {
            let response = Decode!(blob.as_slice(), Result<BlockHeight, TransferError>)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::ManageNeuron => {
            let response = Decode!(blob.as_slice(), ManageNeuronResponse)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::SnsCanistersSummary => {
            let response: Vec<(String, PrincipalId, CanisterStatusResultV2)> = Decode!(
                blob.as_slice(),
                Vec<(String, PrincipalId, CanisterStatusResultV2)>
            )?;
            for (name, principal, canister_status) in response.into_iter() {
                println!("\nCanister Summary:");
                println!("name: {}", name);
                println!("id: {}", principal);
                // TODO(alejandro), add field controllers, NNS1-1396.
                println!("controller: {}", canister_status.controller());
                println!("status: {}", canister_status.status());
                println!(
                    "module_hash: {}",
                    &canister_status
                        .module_hash()
                        .map_or("None".to_string(), hex::encode)
                );
                println!("memory_size: {}", canister_status.memory_size());
                println!("cycles: {}", canister_status.cycles());
                println!(
                    "freezing_threshold: {}",
                    canister_status.freezing_threshold()
                );
            }
        }
        SupportedResponse::NervousSystemParameters => {
            let response = Decode!(blob.as_slice(), NervousSystemParameters)?;
            println!("\nNervous System Parameters:");
            println!(
                "reject_cost_e8: {}",
                response
                    .reject_cost_e8s
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "transaction_fee_e8: {}",
                response
                    .transaction_fee_e8s
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "max_proposals_to_keep_per_action: {}",
                response
                    .max_proposals_to_keep_per_action
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "initial_voting_period: {}",
                response
                    .initial_voting_period
                    .map_or("None".to_string(), |field| field.to_string())
            );
            // TODO(alejandro): improve the display of default followees, NNS1-1395.
            println!(
                "default_followees: {:#?}",
                response
                    .default_followees
                    .map_or("None".to_string(), |field| format!("{:#?}", field))
            );
            println!(
                "max_number_of_neurons: {}",
                response
                    .max_number_of_neurons
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "neuron_minimum_dissolve_delay_to_vote_seconds: {}",
                response
                    .neuron_minimum_dissolve_delay_to_vote_seconds
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "max_followees_per_action: {}",
                response
                    .max_followees_per_action
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "max_dissolve_delay_seconds:  {}",
                response
                    .max_dissolve_delay_seconds
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "max_neuron_age_for_age_bonus:  {}",
                response
                    .max_neuron_age_for_age_bonus
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "reward_distribution_period_seconds: {}",
                response
                    .reward_distribution_period_seconds
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "max_number_of_proposals_with_ballots: {}",
                response
                    .max_number_of_proposals_with_ballots
                    .map_or("None".to_string(), |field| field.to_string())
            );
            println!(
                "neuron_claimer_permissions: {}",
                response
                    .neuron_claimer_permissions
                    .map_or("None".to_string(), |field| format!("{:#?}", field))
            );
            println!(
                "neuron_grantable_permissions: {}",
                response
                    .neuron_grantable_permissions
                    .map_or("None".to_string(), |field| format!("{:#?}", field))
            );
            println!(
                "max_number_of_principals_per_neuron: {}",
                response
                    .max_number_of_principals_per_neuron
                    .map_or("None".to_string(), |field| field.to_string())
            );
        }
    }

    Ok(())
}
