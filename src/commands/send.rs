use crate::{
    commands::request_status,
    lib::{
        get_ic_url, parse_query_response, read_from_file,
        signing::{CallType, Ingress, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
};
use anyhow::{anyhow, Context};
use candid::{Decode, Nat};
use clap::Parser;
use ic_agent::{
    agent::{http_transport::ReqwestHttpReplicaV2Transport, ReplicaV2Transport},
    RequestId,
};
use ic_icrc1::endpoints::TransferError;
use ic_sns_governance::pb::v1::ManageNeuronResponse;
use ic_sns_root::GetSnsCanistersSummaryResponse;
use ic_sns_swap::pb::v1::RefreshBuyerTokensResponse;
use ic_sns_wasm::pb::v1::ListDeployedSnsesResponse;
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
    let msg = crate::lib::signing::sign_ingress_with_request_status_query(
        "",
        method_name,
        args,
        target_canister,
    )?;
    send_ingress_and_check_status(
        &msg,
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
    ListDeployedSnses,
    IcpTransfer,
    RefreshBuyerTokens,
    GetSnsCanistersSummary,
    GetOpenTicket,
}

impl FromStr for SupportedResponse {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<SupportedResponse, Self::Err> {
        match input {
            "icrc1_balance_of" => Ok(SupportedResponse::AccountBalance),
            "icrc1_transfer" => Ok(SupportedResponse::Transfer),
            "list_deployed_snses" => Ok(SupportedResponse::ListDeployedSnses),
            "manage_neuron" => Ok(SupportedResponse::ManageNeuron),
            "send_dfx" => Ok(SupportedResponse::IcpTransfer),
            "refresh_buyer_tokens" => Ok(SupportedResponse::RefreshBuyerTokens),
            "get_sns_canisters_summary" => Ok(SupportedResponse::GetSnsCanistersSummary),
            "get_open_ticket" => Ok(SupportedResponse::GetOpenTicket),
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
            let response = Decode!(blob.as_slice(), candid::Nat)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::Transfer => {
            let response = Decode!(blob.as_slice(), Result<Nat, TransferError>)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::ManageNeuron => {
            let response = Decode!(blob.as_slice(), ManageNeuronResponse)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::ListDeployedSnses => {
            let response = Decode!(blob.as_slice(), ListDeployedSnsesResponse)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::IcpTransfer => {
            let response = Decode!(blob.as_slice(), u64)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::RefreshBuyerTokens => {
            let response = Decode!(blob.as_slice(), RefreshBuyerTokensResponse)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::GetSnsCanistersSummary => {
            let response = Decode!(blob.as_slice(), GetSnsCanistersSummaryResponse)?;
            println!("Response: {:#?\n}", response);
        }
        SupportedResponse::GetOpenTicket => {
            let response = Decode!(blob.as_slice(), GetOpenTicketResponse)?;
            println!("Response: {:#?\n}", response);
        }
    }

    Ok(())
}

// FIXME: use ic_sns_swap when it is available
#[derive(Debug, candid::CandidType, candid::Deserialize)]
struct GetOpenTicketResponse {}
