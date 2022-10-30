use candid::Encode;
use clap::Parser;
use ic_sns_root::GetSnsCanistersSummaryRequest;

use crate::{
    lib::{AnyhowResult, TargetCanister},
    SnsCanisterIds,
};

use super::send::send_unsigned_ingress;

/// Fetches the status of the canisters in the SNS. This includes their controller, running status, canister settings,
/// cycle balance, memory size, daily cycle burn rate, and module hash, along with their principals.
#[derive(Parser)]
pub struct StatusOpts {
    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(ids: &SnsCanisterIds, opts: StatusOpts) -> AnyhowResult {
    let root_canister_id = ids.root_canister_id.get().0;
    let arg = Encode!(&GetSnsCanistersSummaryRequest {
        update_canister_list: None,
    })?;
    send_unsigned_ingress(
        "get_sns_canisters_summary",
        arg,
        opts.dry_run,
        TargetCanister::Root(root_canister_id),
    )
    .await?;
    Ok(())
}
