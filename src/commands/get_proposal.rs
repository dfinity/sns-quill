use crate::{
    commands::send::send_unsigned_ingress, lib::TargetCanister, AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{GetProposal, ProposalId};

#[derive(Parser)]
pub struct GetProposalsOpts {
    #[clap(long)]
    proposal_id: u64,

    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(sns_canister_ids: &SnsCanisterIds, opts: GetProposalsOpts) -> AnyhowResult {
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let args = Encode!(&GetProposal {
        proposal_id: Some(ProposalId {
            id: opts.proposal_id
        })
    })?;

    send_unsigned_ingress(
        "get_proposal",
        args,
        opts.dry_run,
        TargetCanister::Governance(governance_canister_id),
    )
        .await?;

    Ok(())
}
