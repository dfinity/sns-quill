use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_swap::pb::v1::RefreshBuyerTokensRequest;
use ledger_canister::{AccountIdentifier, Memo, SendArgs, Subaccount, Tokens};

use crate::{
    lib::{signing::sign_ingress_with_request_status_query, AnyhowResult, TargetCanister},
    IdsOpt, PemOpts, QrOpt,
};

use super::transfer::ParsedTokens;

/// Signs messages needed to participate in the initial token swap. This operation consists of two messages:
/// First, `amount` ICP is transferred to the swap canister on the NNS ledger, under the subaccount for your principal.
/// Second, the swap canister is notified that the transfer has been made.
/// Once the swap has been finalized, if it was successful, you will receive your neurons automatically.
#[derive(Parser)]
pub struct SwapOpts {
    /// The amount of ICP to transfer. Your neuron's share of the governance tokens at sale finalization will be proportional to your share of the contributed ICP.
    #[clap(long, requires("memo"), required_unless_present("notify-only"))]
    amount: Option<ParsedTokens>,
    /// An arbitrary number used to identify the NNS block this transfer was made in.
    #[clap(long)]
    memo: Option<u64>,
    /// If this flag is specified, then no transfer will be made, and only the notification message will be generated.
    /// This is useful if there was an error previously submitting the notification which you have since rectified, or if you have made the transfer with another tool.
    #[clap(long)]
    notify_only: bool,

    #[clap(flatten)]
    pem: PemOpts,
    #[clap(flatten)]
    sns_canister_ids: IdsOpt,
    #[clap(flatten)]
    qr: QrOpt,
}

pub fn exec(opts: SwapOpts) -> AnyhowResult {
    let sns_canister_ids = opts.sns_canister_ids.to_ids()?;
    let pem = opts.pem.to_pem()?;
    let (controller, _) = crate::commands::public::get_ids(&pem)?;
    let mut messages = vec![];
    if !opts.notify_only {
        let subaccount = Subaccount::from(&PrincipalId(controller));
        let account_id =
            AccountIdentifier::new(sns_canister_ids.swap_canister_id.get(), Some(subaccount));
        let amount = opts.amount.unwrap().0;
        let request = SendArgs {
            amount,
            created_at_time: None,
            from_subaccount: None,
            fee: Tokens::from_e8s(10_000),
            memo: Memo(opts.memo.unwrap()),
            to: account_id,
        };
        messages.push(sign_ingress_with_request_status_query(
            &pem,
            "send_dfx",
            Encode!(&request)?,
            TargetCanister::IcpLedger,
        )?)
    }
    let refresh = RefreshBuyerTokensRequest {
        buyer: controller.to_text(),
    };
    messages.push(sign_ingress_with_request_status_query(
        &pem,
        "refresh_buyer_tokens",
        Encode!(&refresh)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?);
    super::print_vec(opts.qr.qr, &messages)?;
    Ok(())
}
