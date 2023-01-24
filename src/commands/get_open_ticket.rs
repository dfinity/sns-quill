use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};
use candid::Encode;

#[derive(candid::CandidType, candid::Deserialize)]
struct GetOpenTicketArg {}

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let req = sign_ingress_with_request_status_query(
        pem,
        "get_open_ticket",
        Encode!(&GetOpenTicketArg {})?,
        TargetCanister::Swap(sns_canister_ids.swap_canister_id.get().0),
    )?;
    Ok(vec![req])
}
