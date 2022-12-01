use anyhow::Error;
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::{
    manage_neuron::{Command, StakeMaturity},
    ManageNeuron,
};

use crate::{
    lib::{
        parse_neuron_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, TargetCanister,
    },
    SnsCanisterIds,
};

/// Signs a ManageNeuron message to stake a percentage of a neuron's maturity.
///
/// A neuron's total stake is the combination of its staked governance tokens and staked maturity.
#[derive(Parser)]
pub struct StakeMaturityOpts {
    /// The percentage of the current maturity to stake (1-100).
    #[clap(long, value_parser = 1..100)]
    percentage: i64,
    /// The id of the neuron to configure as a hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    neuron_id: String,
}

pub fn exec(
    pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: StakeMaturityOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;

    let governance_canister_id = PrincipalId::from(sns_canister_ids.governance_canister_id);

    let command = ManageNeuron {
        command: Some(Command::StakeMaturity(StakeMaturity {
            percentage_to_stake: Some(opts.percentage as u32),
        })),
        subaccount: neuron_subaccount.to_vec(),
    };

    let message = sign_ingress_with_request_status_query(
        pem,
        "manage_neuron",
        Encode!(&command)?,
        TargetCanister::Governance(governance_canister_id.0),
    )?;
    Ok(vec![message])
}
