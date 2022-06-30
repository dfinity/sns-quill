use candid::Encode;
use clap::Parser;
use ic_sns_sale::pb::v1::ErrorRefundIcpRequest;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};

use super::transfer;

/// Signs a message to request a refund from the SNS swap canister.
/// If the swap was aborted or failed, or some of your contributed ICP never made it into a neuron,
/// this command can retrieve your unused ICP, minus transaction fees.
#[derive(Parser)]
pub struct RefundOpts {
    /// The amount of ICP to request a refund for.
    #[clap(long)]
    amount: String,
    /// The expected transaction fee. If omitted, defaults to 0.0001 ICP.
    #[clap(long)]
    fee: Option<String>,
}

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: RefundOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let tokens = transfer::parse_tokens(&opts.amount)?.get_e8s();
    let fee = opts
        .fee
        .map(|fee| anyhow::Ok(transfer::parse_tokens(&fee)?.get_e8s()))
        .transpose()?
        .unwrap_or(10_000);
    let message = ErrorRefundIcpRequest {
        icp_e8s: tokens,
        fee_override_e8s: fee,
    };
    let req = sign_ingress_with_request_status_query(
        pem,
        "error_refund_icp",
        Encode!(&message)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    Ok(vec![req])
}
