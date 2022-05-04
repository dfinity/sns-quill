use crate::{
    commands::send::send_unsigned_ingress, lib::TargetCanister, AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{GetProposal, ProposalId};

#[derive(Parser)]
pub struct GetNervousSystemParameterOpts {
    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(sns_canister_ids: &SnsCanisterIds, opts: GetNervousSystemParameterOpts) -> AnyhowResult {
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let args = Encode!(&())?;


    send_unsigned_ingress(
        "get_nervous_system_parameters",
        args,
        opts.dry_run,
        TargetCanister::Governance(governance_canister_id),
    )
        .await?;

    Ok(())
}
