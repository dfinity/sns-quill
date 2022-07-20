use candid::Encode;
use clap::Parser;
use ic_sns_wasm::pb::v1::ListDeployedSnsesRequest;

use crate::lib::{AnyhowResult, TargetCanister};

use super::send::send_unsigned_ingress;

/// Lists all SNSes that have been deployed by the NNS.
#[derive(Parser)]
pub struct ListDeployedSnsesOpts {
    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(opts: ListDeployedSnsesOpts) -> AnyhowResult {
    let arg = Encode!(&ListDeployedSnsesRequest {})?;
    send_unsigned_ingress(
        "list_deployed_snses",
        arg,
        opts.dry_run,
        TargetCanister::SnsW,
    )
    .await?;
    Ok(())
}
