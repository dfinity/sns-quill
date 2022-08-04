use anyhow::anyhow;
use candid::Encode;
use clap::{ArgEnum, Parser};
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::{
    manage_neuron::{AddNeuronPermissions, Command, RemoveNeuronPermissions},
    ManageNeuron, NeuronPermissionList, NeuronPermissionType,
};

use crate::{
    lib::{
        parse_neuron_id, signing::sign_ingress_with_request_status_query, AnyhowResult,
        TargetCanister,
    },
    IdsOpt, PemOpts, QrOpt,
};

/// Signs a ManageNeuron message to add or remove permissions for a principal to/from a neuron.
/// This will selectively enable/disable that principal to do a variety of management tasks for the neuron, including voting and disbursing.
#[derive(Parser)]
pub struct NeuronPermissionOpts {
    /// Whether to add or remove permissions.
    #[clap(arg_enum)]
    subcommand: Subcmd,
    /// The id of the neuron to configure as a hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    neuron_id: String,
    /// The principal to change the permissions of.
    #[clap(long)]
    principal: PrincipalId,
    /// The permissions to add to/remove from the principal. You can specify multiple in one command.
    #[clap(
        long,
        multiple_values(true),
        use_value_delimiter(true),
        arg_enum,
        min_values(1),
        required(true)
    )]
    permissions: Vec<NeuronPermissionType>,

    #[clap(flatten)]
    pem: PemOpts,
    #[clap(flatten)]
    canister_ids: IdsOpt,
    #[clap(flatten)]
    qr: QrOpt,
}

#[derive(ArgEnum, Clone)]
enum Subcmd {
    Add,
    Remove,
}

pub fn exec(opts: NeuronPermissionOpts) -> AnyhowResult {
    let pem = opts.pem.to_pem()?;
    let canister_ids = opts.canister_ids.to_ids()?;
    let id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = id.subaccount().map_err(|e| anyhow!(e))?;
    let permission_list = NeuronPermissionList {
        permissions: opts.permissions.into_iter().map(|x| x as i32).collect(),
    };
    let req = ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(if let Subcmd::Add = opts.subcommand {
            Command::AddNeuronPermissions(AddNeuronPermissions {
                principal_id: Some(opts.principal),
                permissions_to_add: Some(permission_list),
            })
        } else {
            Command::RemoveNeuronPermissions(RemoveNeuronPermissions {
                principal_id: Some(opts.principal),
                permissions_to_remove: Some(permission_list),
            })
        }),
    };
    let msg = sign_ingress_with_request_status_query(
        &pem,
        "manage_neuron",
        Encode!(&req)?,
        TargetCanister::Governance(canister_ids.governance_canister_id.get().0),
    )?;
    super::print_vec(opts.qr.qr, &[msg])?;
    Ok(())
}
