use std::str::FromStr;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    IdsOpt, PemOpts, QrOpt, SnsCanisterIds,
};
use anyhow::{anyhow, bail, Context, Error};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ledger_canister::{AccountIdentifier, Memo, Tokens, TransferArgs, DEFAULT_TRANSFER_FEE};

/// Signs a ledger transfer update call.
#[derive(Parser)]
pub struct TransferOpts {
    /// The AccountIdentifier of the destination account. For example: d5662fbce449fbd4adb4b9aff6c59035bd93e7c2eff5010a446ebc3dd81007f8
    pub to: String,

    /// Amount of governance tokens to transfer (with up to 8 decimal digits after decimal point)
    #[clap(long)]
    pub amount: ParsedTokens,

    /// An arbitrary number associated with a transaction. The default is 0
    #[clap(long)]
    pub memo: Option<u64>,

    /// The amount that the caller pays for the transaction, default is 10_000 e8s. Specify this amount
    /// when using an SNS that sets its own transaction fee
    #[clap(long)]
    pub fee: Option<ParsedTokens>,

    #[clap(flatten)]
    pem: PemOpts,
    #[clap(flatten)]
    sns_canister_ids: IdsOpt,
    #[clap(flatten)]
    qr: QrOpt,
}

pub fn exec(opts: TransferOpts) -> AnyhowResult {
    let amount = opts.amount.0;
    let private_key_pem = opts.pem.to_pem()?;
    let sns_canister_ids = opts.sns_canister_ids.to_ids()?;
    let fee = opts.fee.map_or(DEFAULT_TRANSFER_FEE, |fee| fee.0);
    let memo = Memo(opts.memo.unwrap_or(0));
    let to_account_identifier = AccountIdentifier::from_hex(&opts.to).map_err(Error::msg)?;
    let msg = sign_transfer(
        &sns_canister_ids,
        &private_key_pem,
        &to_account_identifier,
        amount,
        fee,
        memo,
    )?;
    super::print_vec(opts.qr.qr, &[msg])?;
    Ok(())
}

pub fn sign_transfer(
    sns_canister_ids: &SnsCanisterIds,
    private_key_pem: &str,
    to: &AccountIdentifier,
    amount: Tokens,
    fee: Tokens,
    memo: Memo,
) -> AnyhowResult<IngressWithRequestId> {
    let args = Encode!(&TransferArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to: to.to_address(),
        created_at_time: None,
    })?;
    let ledger_canister_id = PrincipalId::from(sns_canister_ids.ledger_canister_id).0;
    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "transfer",
        args,
        TargetCanister::Ledger(ledger_canister_id),
    )?;
    Ok(msg)
}

fn new_tokens(tokens: u64, e8s: u64) -> AnyhowResult<Tokens> {
    Tokens::new(tokens, e8s)
        .map_err(|err| anyhow!(err))
        .context("Cannot create new Tokens")
}

pub struct ParsedTokens(pub Tokens);

impl FromStr for ParsedTokens {
    type Err = anyhow::Error;
    fn from_str(amount: &str) -> AnyhowResult<Self> {
        let parse_u64 = |s: &str| {
            s.parse::<u64>()
                .context("Failed to parse Tokens as unsigned integer")
        };
        match &amount.split('.').collect::<Vec<_>>().as_slice() {
            [tokens] => new_tokens(parse_u64(tokens)?, 0).map(Self),
            [tokens, e8s] => {
                let mut e8s = e8s.to_string();
                // Pad e8s with zeros on the right so that its length is 8.
                while e8s.len() < 8 {
                    e8s.push('0');
                }
                let e8s = &e8s[..8];
                new_tokens(parse_u64(tokens)?, parse_u64(e8s)?).map(Self)
            }
            _ => bail!("Cannot parse amount {}", amount),
        }
    }
}
