use crate::{
    commands::send::send_unsigned_ingress, commands::transfer::HexSubaccount, lib::TargetCanister,
    AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_icrc1::Account;

/// Signs a ledger account-balance query call.
#[derive(Parser)]
pub struct AccountBalanceOpts {
    /// The principal of the account to query.
    principal: PrincipalId,

    /// The subaccount of the account to query. For example: e000d80101
    #[clap(long)]
    subaccount: Option<HexSubaccount>,

    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(sns_canister_ids: &SnsCanisterIds, opts: AccountBalanceOpts) -> AnyhowResult {
    let ledger_canister_id = PrincipalId::from(sns_canister_ids.ledger_canister_id).0;

    let args = Account {
        owner: opts.principal,
        subaccount: opts.subaccount.map(|sub| sub.0),
    };

    send_unsigned_ingress(
        "icrc1_balance_of",
        Encode!(&args)?,
        opts.dry_run,
        TargetCanister::Ledger(ledger_canister_id),
    )
    .await?;

    Ok(())
}
