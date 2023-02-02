use candid::Encode;
use clap::Parser;

use crate::{
    commands::transfer::HexSubaccount,
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};

/// Get the sale ticket of the caller. If there is no open ticket yet, create a new ticket with specified arguments.
#[derive(Parser)]
pub struct GetSaleTicketOpts {
    /// The amount of ICP tokens in e8s.
    #[clap(long)]
    amount_icp_e8s: u64,

    /// The subaccount of the account to create sale ticket. For example: e000d80101
    #[clap(long)]
    subaccount: Option<HexSubaccount>,
}

// TODO: SDK-954 - use ic_sns_swap when it is available
#[derive(candid::CandidType, candid::Deserialize)]
struct NewSaleTicketRequest {
    amount_icp_e8s: u64,
    subaccount: Option<[u8; 32]>,
}

#[derive(candid::CandidType, candid::Deserialize)]
struct GetOpenTicketArg {}

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: GetSaleTicketOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let req1 = sign_ingress_with_request_status_query(
        pem,
        "get_open_ticket",
        Encode!(&GetOpenTicketArg {})?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;

    let message = NewSaleTicketRequest {
        amount_icp_e8s: opts.amount_icp_e8s,
        subaccount: opts.subaccount.map(|sub| sub.0),
    };
    let req2 = sign_ingress_with_request_status_query(
        pem,
        "new_sale_ticket",
        Encode!(&message)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    Ok(vec![req1, req2])
}
