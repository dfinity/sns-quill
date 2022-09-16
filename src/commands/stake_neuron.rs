use std::sync::Arc;

use crate::{
    commands::transfer::{self, HexSubaccount},
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        TargetCanister,
    },
    AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_agent::Identity;
use ic_base_types::PrincipalId;
use ic_nervous_system_common::ledger;
use ic_sns_governance::pb::v1::{
    manage_neuron,
    manage_neuron::{
        claim_or_refresh::{By, MemoAndController},
        ClaimOrRefresh,
    },
    ManageNeuron,
};

/// Signs messages needed to stake governance tokens for a neuron. First, stake-neuron will sign
/// a ledger transfer to a subaccount of the Governance canister calculated from the
/// provided private key and memo. Second, stake-neuron will sign a ManageNeuron message for
/// Governance to claim the neuron for the principal derived from the provided private key.
#[derive(Parser)]
pub struct StakeNeuronOpts {
    /// The amount of tokens in e8s to be transferred to the Governance canister's ledger subaccount
    /// (the neuron's AccountId) from the AccountId derived from the provided private key. This is
    /// known as a staking transfer. These funds will be returned when disbursing the neuron. If NOT
    /// specified, no transfer will be made, and only a neuron claim command will be signed. This
    /// is useful for situations where the transfer was initially made with some other command or
    /// tool
    #[clap(long)]
    amount: Option<String>,

    /// An arbitrary number used in calculating the neuron's subaccount. The memo must be unique among
    /// the neurons claimed for a single PrincipalId. More information on ledger accounts and
    /// subaccounts can be found here: https://smartcontracts.org/docs/integration/ledger-quick-start.html#_ledger_canister_overview
    #[clap(long)]
    memo: u64,

    /// The amount that the caller pays for the transaction, default is 10_000 e8s. Specify this amount
    /// when using an SNS that sets its own transaction fee
    #[clap(long)]
    fee: Option<String>,
}

pub fn exec(
    ident: Arc<dyn Identity>,
    sns_canister_ids: &SnsCanisterIds,
    opts: StakeNeuronOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (controller, _) = crate::commands::public::get_ids(Some(ident.clone()))?;
    let neuron_subaccount =
        ledger::compute_neuron_staking_subaccount(PrincipalId::from(controller), opts.memo);

    let governance_canister_id = PrincipalId::from(sns_canister_ids.governance_canister_id);

    let mut messages = Vec::new();

    // If amount is provided, sign a transfer message that will transfer tokens from the principal's
    // account on the ledger to a subaccount of the governance canister.
    if let Some(amount) = opts.amount {
        messages.extend(transfer::exec(
            ident.clone(),
            sns_canister_ids,
            transfer::TransferOpts {
                to_principal: governance_canister_id,
                to_subaccount: Some(HexSubaccount(neuron_subaccount.0)),
                amount,
                fee: opts.fee,
                memo: Some(opts.memo.to_string()),
            },
        )?)
    }

    // Sign a message claiming the neuron with funds staked to the previously calculated subaccount.
    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::MemoAndController(MemoAndController {
                memo: opts.memo,
                controller: Some(PrincipalId(controller)),
            }))
        }))
    })?;

    messages.push(sign_ingress_with_request_status_query(
        ident,
        "manage_neuron",
        args,
        TargetCanister::Governance(governance_canister_id.0),
    )?);

    Ok(messages)
}
