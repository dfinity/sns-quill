use candid::Encode;
use clap::Parser;
use ic_sns_swap::pb::v1::GetCanisterStatusRequest;

use crate::{
    lib::{AnyhowResult, TargetCanister},
    IdsOpt,
};

use super::send::send_unsigned_ingress;

/// Fetches the status of the canisters in the SNS. This includes their controller, running status, canister settings,
/// cycle balance, memory size, daily cycle burn rate, and module hash, along with their principals.
#[derive(Parser)]
pub struct StatusOpts {
    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,

    #[clap(flatten)]
    ids: IdsOpt,
}

pub async fn exec(opts: StatusOpts) -> AnyhowResult {
    let root_canister_id = opts.ids.to_ids()?.root_canister_id.get().0;
    let arg = Encode!(&GetCanisterStatusRequest {})?;
    send_unsigned_ingress(
        "get_sns_canisters_summary",
        arg,
        opts.dry_run,
        TargetCanister::Root(root_canister_id),
    )
    .await?;
    Ok(())
}
