use crate::lib::signing::{sign_query_as_ingress_with_request_id, IngressWithRequestId};
use crate::lib::{AnyhowResult, TargetCanister};
use crate::SnsCanisterIds;
use candid::Encode;
use ic_base_types::PrincipalId;

pub fn exec(sns_canister_ids: &SnsCanisterIds) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let governance_canister_id = PrincipalId::from(sns_canister_ids.governance_canister_id).0;
    let args = Encode!()?;

    let msg = sign_query_as_ingress_with_request_id(
        "",
        "get_nervous_system_parameters",
        args,
        TargetCanister::Governance(governance_canister_id),
    )?;
    Ok(vec![msg])
}
