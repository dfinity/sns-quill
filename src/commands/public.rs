use crate::lib::{get_account_id, get_identity, require_pem, AnyhowResult};
use anyhow::anyhow;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_nervous_system_common::ledger;
use ic_sns_governance::pb::v1::{NeuronId};
use candid::Principal;
use icp_ledger::AccountIdentifier;

#[derive(Parser)]
pub struct PublicOpts {
    /// Principal to get public ids from
    #[clap(long)]
    principal_id: Option<String>,

    /// Memo used when calculating SNS Neuron Id
    #[clap(long)]
    memo: Option<u64>,
}

/// Prints the account and the principal ids.
pub fn exec(pem: &Option<String>, opts: PublicOpts) -> AnyhowResult {
    let (principal_id, account_id, sns_neuron_id) = get_public_ids(pem, &opts)?;
    println!("Principal id: {}", principal_id.to_text());
    println!("NNS account id: {}", account_id);
    if let Some(sns_neuron_id) = sns_neuron_id {
        println!("SNS neuron id (memo = {}): {}", opts.memo.unwrap(), sns_neuron_id)
    }
    Ok(())
}

/// Returns the account id and the principal id if the private key was provided.
fn get_public_ids(
    pem: &Option<String>,
    opts: &PublicOpts,
) -> AnyhowResult<(Principal, AccountIdentifier, Option<NeuronId>)> {
    match (opts.principal_id.as_ref(), opts.memo) {
        (Some(principal_id), None) => {
            let principal_id = Principal::from_text(principal_id)?;
            Ok((principal_id, get_account_id(principal_id)?, None))
        },
        (Some(principal_id), Some(memo)) => {
            let principal_id = Principal::from_text(principal_id)?;
            Ok((principal_id, get_account_id(principal_id)?, Some(get_neuron_id(principal_id, memo))))
        },
        (None, None) => {
            let pem = require_pem(pem)?;
            let principal_id = get_identity(&pem).sender().map_err(|e| anyhow!(e))?;
            Ok((principal_id, get_account_id(principal_id)?, None))
        },
        (None, Some(memo)) => {
            let pem = require_pem(pem)?;
            let principal_id = get_identity(&pem).sender().map_err(|e| anyhow!(e))?;
            Ok((principal_id, get_account_id(principal_id)?, Some(get_neuron_id(principal_id, memo))))
        },
    }
}

/// Returns the account id and the principal id if the private key was provided.
pub fn get_ids(pem: &Option<String>) -> AnyhowResult<(Principal, AccountIdentifier)> {
    let pem = require_pem(pem)?;
    let principal_id = get_identity(&pem).sender().map_err(|e| anyhow!(e))?;
    Ok((principal_id, get_account_id(principal_id)?))
}

pub fn get_neuron_id(principal_id: Principal, memo: u64) -> NeuronId {
    NeuronId::from(ledger::compute_neuron_staking_subaccount_bytes(PrincipalId::from(principal_id), memo))
}
