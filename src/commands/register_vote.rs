use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::{parse_neuron_id, TargetCanister};
use crate::{AnyhowResult, SnsCanisterIds};
use anyhow::{anyhow, Error};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::manage_neuron::RegisterVote;
use ic_sns_governance::pb::v1::{manage_neuron, ManageNeuron, ProposalId, Vote};

/// Signs a ManageNeuron message to register a vote for a proposal. Registering a vote will
/// update the ballot of the given proposal and could trigger followees to vote. When
/// enough votes are cast or enough time passes, the proposal will either be rejected or
/// adopted and executed.
#[derive(Parser)]
pub struct RegisterVoteOpts {
    /// The id of the neuron to configure as a hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    neuron_id: String,

    #[clap(long)]
    /// The id of the proposal to voted on
    proposal_id: u64,

    #[clap(long)]
    /// The vote to be cast on the proposal [y/n]
    vote: String,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: RegisterVoteOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let vote = match opts.vote.as_str() {
        "y" => Ok(Vote::Yes),
        "n" => Ok(Vote::No),
        _ => Err(anyhow!(
            "Unsupported vote supplied to --vote. Supported values: ['y', 'n']"
        )),
    }?;

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::RegisterVote(RegisterVote {
            proposal: Some(ProposalId {
                id: opts.proposal_id
            }),
            vote: vote as i32
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
