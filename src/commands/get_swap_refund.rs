use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_swap::pb::v1::ErrorRefundIcpRequest;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};

/// Signs a message to request a refund from the SNS swap canister.
/// If the swap was aborted or failed, or some of your contributed ICP never made it into a neuron,
/// this command can retrieve your unused ICP, minus transaction fees.
#[derive(Parser)]
pub struct GetSwapRefundOpts {
    /// The principal that made the ICP contribution and should be refunded. The ICP will be
    /// refunded to the main account of this Principal irrespective of the caller.
    #[clap(long)]
    principal: PrincipalId,
}

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: GetSwapRefundOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let message = ErrorRefundIcpRequest {
        source_principal_id: Some(opts.principal)
    };
    let req = sign_ingress_with_request_status_query(
        pem,
        "error_refund_icp",
        Encode!(&message)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    Ok(vec![req])
}
