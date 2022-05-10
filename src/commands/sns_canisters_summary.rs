use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::{AnyhowResult, TargetCanister};
use crate::SnsCanisterIds;
use candid::Encode;
use clap::Parser;
use ic_base_types::{CanisterId, PrincipalId};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct SnsCanistersSummaryOps {
    /// Canister id of the dapp.
    #[clap(long)]
    pub canister_id: Option<String>,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: SnsCanistersSummaryOps,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let dapp_canister_id_list = match &opts.canister_id {
        Some(str) => {
            vec![CanisterId::from_str(&str)?]
        }
        None => sns_canister_ids.dapp_canister_id_list.clone(),
    };
    let root_canister_id = PrincipalId::from(sns_canister_ids.root_canister_id).0;
    let args = Encode!(&dapp_canister_id_list)?;

    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "get_sns_canisters_summary",
        args,
        TargetCanister::Root(root_canister_id),
    )?;

    Ok(vec![msg])
}
