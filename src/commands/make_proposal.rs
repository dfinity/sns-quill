use crate::{
    lib::{
        parse_neuron_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        TargetCanister,
    },
    AnyhowResult, SnsCanisterIds,
};
use anyhow::{Context, Error};
use candid::{Decode, Encode, IDLArgs};
use clap::Parser;
use ic_sns_governance::pb::v1::{manage_neuron, ManageNeuron, Proposal};

/// Signs a ManageNeuron message to submit a proposal. With this command, neuron holders
/// can submit proposals (such as a Motion Proposal) to be voted on by other neuron
/// holders.
#[derive(Parser)]
pub struct MakeProposalOpts {
    /// The id of the neuron making the proposal as a hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    proposer_neuron_id: String,

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
    proposal: Option<String>,

    /// Path to the WASM file to be installed onto the target canister.
    #[clap(long)]
    proposal_path: Option<String>,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: MakeProposalOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_id = parse_neuron_id(opts.proposer_neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let proposal_string = match opts.proposal {
        Some(proposal) => proposal,
        None => String::from_utf8(
            std::fs::read(opts.proposal_path.unwrap())
                .context("Unable to read --proposal-path.")?,
        )?,
    };
    let proposal = parse_proposal_from_candid_string(proposal_string)?;

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::MakeProposal(proposal))
    })?;

    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "manage_neuron",
        args,
        TargetCanister::Governance(governance_canister_id),
    )?;

    Ok(vec![msg])
}

fn parse_proposal_from_candid_string(proposal_candid: String) -> AnyhowResult<Proposal> {
    let args: IDLArgs = proposal_candid.parse()?;
    let args: Vec<u8> = args.to_bytes()?;
    Decode!(args.as_slice(), Proposal).map_err(Error::msg)
}
