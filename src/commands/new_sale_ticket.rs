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

/// Attempts to create a new Ticket for the caller.
#[derive(Parser)]
pub struct NewSaleTicketOpts {
    // TODO: check this description
    /// The amount of ICP tokens in e8s.
    #[clap(long)]
    amount_icp_e8s: u64,

    /// The subaccount of the account to create sale ticket. For example: e000d80101
    #[clap(long)]
    subaccount: Option<HexSubaccount>,
}

// FIXME: use ic_sns_swap when it is available
#[derive(candid::CandidType, candid::Deserialize)]
struct NewSaleTicketRequest {
    amount_icp_e8s: u64,
    subaccount: Option<[u8; 32]>,
}

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: NewSaleTicketOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let message = NewSaleTicketRequest {
        amount_icp_e8s: opts.amount_icp_e8s,
        subaccount: opts.subaccount.map(|sub| sub.0),
    };
    let req = sign_ingress_with_request_status_query(
        pem,
        "new_sale_ticket",
        Encode!(&message)?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    Ok(vec![req])
}
