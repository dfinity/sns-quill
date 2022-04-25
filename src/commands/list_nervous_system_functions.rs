use crate::{
    commands::send::send_unsigned_ingress, lib::TargetCanister, AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;

/// Queries governance to list NervousSystemFunctions registered with the governance canister
#[derive(Parser)]
pub struct ListNervousSystemFunctionsOpts {
    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(
    sns_canister_ids: &SnsCanisterIds,
    opts: ListNervousSystemFunctionsOpts,
) -> AnyhowResult {
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let args = Encode!(&())?;

    send_unsigned_ingress(
        "list_nervous_system_functions",
        args,
        opts.dry_run,
        TargetCanister::Governance(governance_canister_id),
    )
    .await?;

    Ok(())
}
