use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::{AnyhowResult, TargetCanister};
use crate::SnsCanisterIds;
use candid::Encode;
use ic_base_types::PrincipalId;

/// Outputs info about the sns canisters: Governance, Ledger, Root and canisters belonging to the
/// controlled dapp. Info includes cycles, ownership and more.
pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let dapp_canister_id_list = sns_canister_ids.dapp_canister_id_list.clone();
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
