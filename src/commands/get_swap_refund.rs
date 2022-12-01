use candid::Encode;
use clap::Parser;
use ic_sns_swap::pb::v1::ErrorRefundIcpRequest;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};

use super::public::get_ids;

/// Signs a message to request a refund from the SNS swap canister.
/// If the swap was aborted or failed, or some of your contributed ICP never made it into a neuron,
/// this command can retrieve your unused ICP, minus transaction fees.
#[derive(Parser)]
pub struct GetSwapRefundOpts;

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    _: GetSwapRefundOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (principal, _) = get_ids(&Some(pem.to_string()))?;
    let message = ErrorRefundIcpRequest {
        source_principal_id: Some(principal.into()),
    };
    let req = sign_ingress_with_request_status_query(
        pem,
        "error_refund_icp",
        Encode!(&message)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    Ok(vec![req])
}
