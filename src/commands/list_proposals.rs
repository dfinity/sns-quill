use crate::{
    commands::send::send_unsigned_ingress, lib::TargetCanister, AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{ListProposals, ProposalId};

/// Queries governance to to list proposals submitted to the governance canister
#[derive(Parser)]
pub struct ListProposalsOpts {
    /// Limit the number of Proposals returned in each page, from 1 to 100.
    /// If a value outside of this range is provided, 100 will be used
    #[clap(long)]
    limit: u32,

    /// The optional id of the proposal to start the page at as an unsigned 64 bit integer.
    ///
    /// This should be set to the last proposal of the previously returned page and
    /// will not be included in the current page.
    /// If this is specified, then only the proposals that have a proposal ID strictly
    /// lower than the specified one are returned. If this is not specified
    /// then the list of proposals starts with the most recent proposal's ID.
    #[clap(long)]
    before_proposal: Option<u64>,

    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(sns_canister_ids: &SnsCanisterIds, opts: ListProposalsOpts) -> AnyhowResult {
    let before_proposal = opts.before_proposal.map(|id| ProposalId { id });

    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let args = Encode!(&ListProposals {
        limit: opts.limit,
        before_proposal,
        exclude_type: vec![],
        include_reward_status: vec![],
        include_status: vec![],
    })?;

    send_unsigned_ingress(
        "list_proposals",
        args,
        opts.dry_run,
        TargetCanister::Governance(governance_canister_id),
    )
    .await?;

    Ok(())
}
