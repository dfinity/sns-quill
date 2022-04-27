use crate::{
    lib::{
        parse_neuron_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        TargetCanister,
    },
    AnyhowResult, SnsCanisterIds,
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

// These constants are copied from src/governance.rs
pub const ONE_DAY_SECONDS: u32 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u32 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u32 = ONE_YEAR_SECONDS / 12;

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
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: ConfigureDissolveDelayOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    require_mutually_exclusive(
        opts.start_dissolving,
        opts.stop_dissolving,
        &opts.additional_dissolve_delay_seconds,
    )?;

    let neuron_id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = PrincipalId::from(sns_canister_ids.governance_canister_id).0;

    let mut msgs = Vec::new();

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
        })?;
        msgs.push(args);
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                    additional_dissolve_delay_seconds: match additional_dissolve_delay_seconds
                        .as_ref()
                    {
                        "ONE_DAY" => ONE_DAY_SECONDS,

                        "ONE_WEEK" => ONE_DAY_SECONDS * 7,
                        "TWO_WEEKS" => ONE_DAY_SECONDS * 7 * 2,
                        "THREE_WEEKS" => ONE_DAY_SECONDS * 7 * 3,
                        "FOUR_WEEKS" => ONE_DAY_SECONDS * 7 * 4,

                        "ONE_MONTH" => ONE_MONTH_SECONDS,
                        "TWO_MONTHS" => ONE_MONTH_SECONDS * 2,
                        "THREE_MONTHS" => ONE_MONTH_SECONDS * 3,
                        "FOUR_MONTHS" => ONE_MONTH_SECONDS * 4,
                        "FIVE_MONTHS" => ONE_MONTH_SECONDS * 5,
                        "SIX_MONTHS" => ONE_MONTH_SECONDS * 6,
                        "SEVEN_MONTHS" => ONE_MONTH_SECONDS * 7,
                        "EIGHT_MONTHS" => ONE_MONTH_SECONDS * 8,
                        "NINE_MONTHS" => ONE_MONTH_SECONDS * 9,
                        "TEN_MONTHS" => ONE_MONTH_SECONDS * 10,
                        "ELEVEN_MONTHS" => ONE_MONTH_SECONDS * 11,

                        "ONE_YEAR" => ONE_YEAR_SECONDS,
                        "TWO_YEARS" => ONE_YEAR_SECONDS * 2,
                        "THREE_YEARS" => ONE_YEAR_SECONDS * 3,
                        "FOUR_YEARS" => ONE_YEAR_SECONDS * 4,
                        "FIVE_YEARS" => ONE_YEAR_SECONDS * 5,
                        "SIX_YEARS" => ONE_YEAR_SECONDS * 6,
                        "SEVEN_YEARS" => ONE_YEAR_SECONDS * 7,
                        "EIGHT_YEARS" => ONE_YEAR_SECONDS * 8,

                        s => s
                            .parse::<u32>()
                            .expect("Failed to parse the dissolve delay"),
                    }
                }))
            })),
        })?;
        msgs.push(args);
    };

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(sign_ingress_with_request_status_query(
            private_key_pem,
            "manage_neuron",
            args,
            TargetCanister::Governance(governance_canister_id),
        )?);
    }
    Ok(generated)
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
