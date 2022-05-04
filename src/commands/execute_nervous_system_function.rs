use crate::{
    lib::{
        parse_neuron_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        TargetCanister,
    },
    AnyhowResult, SnsCanisterIds,
};
use anyhow::Error;
use candid::{Decode, Encode, IDLArgs};
use clap::Parser;
use ic_sns_governance::pb::v1::{ExecuteNervousSystemFunction, manage_neuron, ManageNeuron, Proposal, proposal};

/// Signs a ManageNeuron message to submit a proposal. With this command, neuron holders
/// can submit proposals (such as a Motion Proposal) to be voted on by other neuron
/// holders.
#[derive(Parser)]
pub struct ExecuteNervousSystemFunctionOpts {
    /// The id of the neuron to configure as a hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    neuron_id: String,

    function_id: u64,

    /// The proposal to be submitted. The proposal must be formatted as a string
    /// wrapped candid record.
    ///
    /// For example:
    /// '(
    ///     record {
    ///         title="SNS Launch";
    ///         url="https://dfinity.org";
    ///         summary="A motion to start the SNS";
    ///         action=opt variant {
    ///             Motion=record {
    ///                 motion_text="I hereby raise the motion that the use of the SNS shall commence";
    ///             }
    ///         };
    ///     }
    /// )'
    #[clap(long)]
    payload: String,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: ExecuteNervousSystemFunctionOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let proposal = parse_payload_from_candid_string(opts.payload)?;

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::MakeProposal(Proposal {
            title: "".to_string(),
            summary: "".to_string(),
            url: "".to_string(),
            action: Some(proposal::Action::ExecuteNervousSystemFunction(ExecuteNervousSystemFunction {
                function_id: opts.function_id,
                payload: proposal,
            }))
        }))
    })?;

    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "manage_neuron",
        args,
        TargetCanister::Governance(governance_canister_id),
    )?;

    Ok(vec![msg])
}

fn parse_payload_from_candid_string(candid_string: String) -> AnyhowResult<Vec<u8>> {
    let args: IDLArgs = candid_string.parse()?;
    Ok(args.to_bytes()?)
}
