use std::str::FromStr;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};
use anyhow::{anyhow, bail, ensure, Context};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_icrc1::{Account, endpoints::TransferArg, Memo, Subaccount};
use ic_ledger_core::Tokens;

/// Signs a ledger transfer update call.
#[derive(Default, Parser)]
pub struct TransferOpts {
    /// The principal of the destination account.
    pub to_principal: PrincipalId,

    /// The subaccount of the destination account. For example: e000d80101
    #[clap(long)]
    pub to_subaccount: Option<HexSubaccount>,

    /// Amount of governance tokens to transfer (with up to 8 decimal digits after decimal point)
    #[clap(long, validator(tokens_amount_validator))]
    pub amount: String,

    /// An arbitrary number associated with a transaction. The default is 0
    #[clap(long, validator(memo_validator))]
    pub memo: Option<String>,

    /// The amount that the caller pays for the transaction, default is 10_000 e8s. Specify this amount
    /// when using an SNS that sets its own transaction fee
    #[clap(long, validator(tokens_amount_validator))]
    pub fee: Option<String>,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: TransferOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let amount = parse_tokens(&opts.amount)
        .context("Cannot parse amount")?
        .get_e8s()
        .into();
    let fee = opts
        .fee
        .map(|fee| {
            anyhow::Ok(
                parse_tokens(&fee)
                    .context("Cannot parse fee")?
                    .get_e8s()
                    .into(),
            )
        })
        .transpose()?;
    let memo = opts
        .memo
        .map(|memo| {
            memo.parse::<u64>()
                .context("Failed to parse memo as unsigned integer")
        })
        .transpose()?
        .map(|memo| Memo::from(memo));
    let ledger_canister_id = PrincipalId::from(sns_canister_ids.ledger_canister_id).0;
    let to_subaccount = opts.to_subaccount.map(|sub| sub.0);
    let args = TransferArg {
        memo,
        amount,
        fee,
        from_subaccount: None,
        created_at_time: None,
        to: Account {
            owner: opts.to_principal,
            subaccount: to_subaccount,
        }
    };

    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "icrc1_transfer",
        Encode!(&args)?,
        TargetCanister::Ledger(ledger_canister_id),
    )?;

    Ok(vec![msg])
}

fn new_tokens(tokens: u64, e8s: u64) -> AnyhowResult<Tokens> {
    Tokens::new(tokens, e8s)
        .map_err(|err| anyhow!(err))
        .context("Cannot create new Tokens")
}

pub fn parse_tokens(amount: &str) -> AnyhowResult<Tokens> {
    let parse_u64 = |s: &str| {
        s.parse::<u64>()
            .context("Failed to parse Tokens as unsigned integer")
    };
    match &amount.split('.').collect::<Vec<_>>().as_slice() {
        [tokens] => new_tokens(parse_u64(tokens)?, 0),
        [tokens, e8s] => {
            let mut e8s = e8s.to_string();
            // Pad e8s with zeros on the right so that its length is 8.
            while e8s.len() < 8 {
                e8s.push('0');
            }
            let e8s = &e8s[..8];
            new_tokens(parse_u64(tokens)?, parse_u64(e8s)?)
        }
        _ => bail!("Cannot parse amount {}", amount),
    }
}

fn tokens_amount_validator(tokens: &str) -> AnyhowResult<()> {
    parse_tokens(tokens).map(|_| ())
}

fn memo_validator(memo: &str) -> Result<(), String> {
    if memo.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Memo must be an unsigned integer".to_string())
}

#[derive(Debug)]
pub struct HexSubaccount(pub Subaccount);

impl FromStr for HexSubaccount {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = hex::decode(&s)?;
        ensure!(
            parsed.len() <= 32,
            "Subaccounts must be less than 32 bytes (64 characters)"
        );
        let mut sub = [0; 32];
        sub[32 - parsed.len()..].copy_from_slice(&parsed);
        Ok(Self(sub))
    }
}
