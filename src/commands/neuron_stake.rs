use crate::{
    commands::transfer,
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        TargetCanister,
    },
    AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::{
    manage_neuron,
    manage_neuron::{
        claim_or_refresh::{By, MemoAndController},
        ClaimOrRefresh,
    },
    ManageNeuron,
};
use ic_types::Principal;
use ledger_canister::{AccountIdentifier, Subaccount};

/// Signs messages needed to stake governance tokens for a neuron. First, neuron-stake will sign
/// a ledger transfer to a subaccount of the Governance canister calculated from the
/// provided private key and memo. Second, neuron-stake will sign a ManageNeuron message for
/// Governance to claim the neuron for the principal derived from the provided private key.
#[derive(Parser)]
pub struct NeuronStakeOpts {
    /// The amount of tokens to be transferred to the Governance canister's ledger subaccount
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
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: NeuronStakeOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (controller, _) = crate::commands::public::get_ids(&Some(private_key_pem.to_string()))?;
    let neuron_subaccount = get_neuron_subaccount(&controller, opts.memo);

    let governance_canister_id = PrincipalId::from(sns_canister_ids.governance_canister_id);
    let account = AccountIdentifier::new(governance_canister_id, Some(neuron_subaccount));

    // If amount is provided, sign a transfer message that will transfer tokens from the principal's
    // account on the ledger to a subaccount of the governance canister.
    let mut messages = match &opts.amount {
        Some(amount) => transfer::exec(
            private_key_pem,
            sns_canister_ids,
            transfer::TransferOpts {
                to: account.to_hex(),
                amount: amount.clone(),
                fee: opts.fee,
                memo: Some(opts.memo.to_string()),
            },
        )?,
        _ => Vec::new(),
    };

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
        private_key_pem,
        "manage_neuron",
        args,
        TargetCanister::Governance(governance_canister_id.0),
    )?);

    Ok(messages)
}

/// Compute the subaccount for a given Principal and nonce. This function _must_ correspond to
/// how the governance canister computes the subaccount.
pub fn get_neuron_subaccount(controller: &Principal, nonce: u64) -> Subaccount {
    use openssl::sha::Sha256;
    let mut data = Sha256::new();
    data.update(&[0x0c]);
    data.update(b"neuron-stake");
    data.update(controller.as_slice());
    data.update(&nonce.to_be_bytes());
    Subaccount(data.finish())
}
