use crate::commands::sns_canisters_summary::SnsCanistersSummaryOps;
use crate::commands::{nervous_system_parameters, sns_canisters_summary};
use crate::lib::signing::IngressWithRequestId;
use crate::lib::AnyhowResult;
use crate::SnsCanisterIds;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct SnsSummaryOps {
    /// Canister id of the dapp.
    #[clap(long)]
    canister_id: Option<String>,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: SnsSummaryOps,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = vec![];
    msgs.extend(sns_canisters_summary::exec(
        private_key_pem,
        sns_canister_ids,
        SnsCanistersSummaryOps {
            canister_id: opts.canister_id.clone(),
        },
    )?);
    msgs.extend(nervous_system_parameters::exec(sns_canister_ids)?);
    Ok(msgs)
}
