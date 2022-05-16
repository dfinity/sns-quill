use crate::commands::{get_nervous_system_parameters, sns_canisters_summary};
use crate::lib::signing::IngressWithRequestId;
use crate::lib::AnyhowResult;
use crate::SnsCanisterIds;

/// Prints a summary with general info about the sns:
///  - Canisters info: cycles, ownership and more info about the Governance, Ledger, Root,
///    and Dapp canisters.
///  - Nervous System Parameters: Like the cost of rejected proposal, the initial voting time, and
///    the reward distribution period.
pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = vec![];
    msgs.extend(sns_canisters_summary::exec(
        private_key_pem,
        sns_canister_ids,
    )?);
    msgs.extend(get_nervous_system_parameters::exec(sns_canister_ids)?);
    Ok(msgs)
}
