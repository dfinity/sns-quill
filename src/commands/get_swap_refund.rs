use candid::Encode;
use clap::Parser;
use ic_sns_swap::pb::v1::ErrorRefundIcpRequest;

use crate::{
    lib::{signing::sign_ingress_with_request_status_query, AnyhowResult, TargetCanister},
    IdsOpt, PemOpts, QrOpt,
};

use super::transfer::ParsedTokens;

/// Signs a message to request a refund from the SNS swap canister.
/// If the swap was aborted or failed, or some of your contributed ICP never made it into a neuron,
/// this command can retrieve your unused ICP, minus transaction fees.
#[derive(Parser)]
pub struct GetSwapRefundOpts {
    /// The amount of ICP to request a refund for.
    #[clap(long)]
    amount: ParsedTokens,
    /// The expected transaction fee. If omitted, defaults to 0.0001 ICP.
    #[clap(long)]
    fee: Option<ParsedTokens>,

    #[clap(flatten)]
    pem: PemOpts,
    #[clap(flatten)]
    sns_canister_ids: IdsOpt,
    #[clap(flatten)]
    qr: QrOpt,
}

pub fn exec(opts: GetSwapRefundOpts) -> AnyhowResult {
    let pem = opts.pem.to_pem()?;
    let sns_canister_ids = opts.sns_canister_ids.to_ids()?;
    let tokens = opts.amount.0.get_e8s();
    let fee = opts.fee.map(|fee| fee.0.get_e8s()).unwrap_or(10_000);
    let message = ErrorRefundIcpRequest {
        icp_e8s: tokens,
        fee_override_e8s: fee,
    };
    let req = sign_ingress_with_request_status_query(
        &pem,
        "error_refund_icp",
        Encode!(&message)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    super::print_vec(opts.qr.qr, &[req])?;
    Ok(())
}
