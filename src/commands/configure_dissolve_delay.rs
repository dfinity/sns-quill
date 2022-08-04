use crate::{
    lib::{parse_neuron_id, signing::sign_ingress_with_request_status_query, TargetCanister},
    AnyhowResult, IdsOpt, PemOpts, QrOpt,
};
use anyhow::{anyhow, Error};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;

use ic_sns_governance::pb::v1::{
    manage_neuron,
    manage_neuron::{
        configure::Operation, Configure, IncreaseDissolveDelay, StartDissolving, StopDissolving,
    },
    ManageNeuron,
};

/// Signs a ManageNeuron message to configure the dissolve delay of a neuron. With this command
/// neuron holders can start dissolving, stop dissolving, or increase dissolve delay. The
/// dissolve delay of a neuron determines its voting power, its ability to vote, its ability
/// to make proposals, and other actions it can take (such as disbursing).
#[derive(Parser)]
pub struct ConfigureDissolveDelayOpts {
    /// The id of the neuron to configure as a hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    neuron_id: String,

    /// Additional number of seconds to add to the dissolve delay of a neuron. If the neuron is
    /// already dissolving and this argument is specified, the neuron will stop dissolving
    /// and begin aging
    #[clap(short, long)]
    additional_dissolve_delay_seconds: Option<String>,

    /// When this argument is specified, the neuron will go into the dissolving state and a
    /// countdown timer will begin. When the timer is exhausted (i.e. dissolve_delay_seconds
    /// amount of time has elapsed), the neuron can be disbursed
    #[clap(long)]
    start_dissolving: bool,

    /// When this argument is specified, the neuron will exit the dissolving state and whatever
    /// amount of dissolve delay seconds is left in the countdown timer is stored. A neuron's
    /// dissolve delay can be extended (for instance to increase voting power) by using the
    /// additional_dissolve_delay_seconds flag
    #[clap(long)]
    stop_dissolving: bool,

    #[clap(flatten)]
    pem: PemOpts,
    #[clap(flatten)]
    sns_canister_ids: IdsOpt,
    #[clap(flatten)]
    qr: QrOpt,
}

pub fn exec(opts: ConfigureDissolveDelayOpts) -> AnyhowResult {
    require_mutually_exclusive(
        opts.start_dissolving,
        opts.stop_dissolving,
        &opts.additional_dissolve_delay_seconds,
    )?;
    let private_key_pem = opts.pem.to_pem()?;
    let neuron_id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id =
        PrincipalId::from(opts.sns_canister_ids.to_ids()?.governance_canister_id).0;

    let mut args = Vec::new();

    if opts.stop_dissolving {
        args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
        })?;
    }

    if opts.start_dissolving {
        args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
        })?;
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let parsed_additional_dissolve_delay_seconds = additional_dissolve_delay_seconds
            .parse::<u32>()
            .expect("Failed to parse the dissolve delay");

        args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                    additional_dissolve_delay_seconds: parsed_additional_dissolve_delay_seconds
                }))
            })),
        })?;
    };

    let msg = sign_ingress_with_request_status_query(
        &private_key_pem,
        "manage_neuron",
        args,
        TargetCanister::Governance(governance_canister_id),
    )?;
    super::print_vec(opts.qr.qr, &[msg])?;
    Ok(())
}

fn require_mutually_exclusive(
    stop_dissolving: bool,
    start_dissolving: bool,
    additional_dissolve_delay_seconds: &Option<String>,
) -> AnyhowResult {
    match (stop_dissolving, start_dissolving, additional_dissolve_delay_seconds) {
        (true, false, None) => Ok(()),
        (false, true, None) => Ok(()),
        (false, false, Some(_)) => Ok(()),
        _ => Err(anyhow!("--stop-dissolving, --start-dissolving, --additional-dissolve-delay-seconds are mutually exclusive arguments"))
    }
}
