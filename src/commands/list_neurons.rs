use crate::{
    commands::send::send_unsigned_ingress,
    lib::{parse_neuron_id, TargetCanister},
    AnyhowResult, SnsCanisterIds,
};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::ListNeurons;
use std::str::FromStr;

/// Queries governance to list neurons claimed in the governance canister
#[derive(Parser)]
pub struct ListNeuronsOpts {
    /// Limit the number of Neurons returned in each page, from 1 to 100.
    /// If a value outside of this range is provided, 100 will be used
    #[clap(long)]
    limit: u32,

    /// The optional id of the neuron to start the page at as a hex encoded string.
    /// For example: 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    ///
    /// This should be set to the last neuron of the previously returned page
    /// and will not be included in the next page. If not specified, the page
    /// will start at the "0th" neuron
    #[clap(long)]
    start_page_at: Option<String>,

    /// An optional PrincipalId that when specified, ListNeurons will only return
    /// neurons where the provided PrincipalId has some NeuronPermission associated
    /// with it
    #[clap(long)]
    of_principal: Option<String>,

    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(sns_canister_ids: &SnsCanisterIds, opts: ListNeuronsOpts) -> AnyhowResult {
    let start_page_at = match opts.start_page_at {
        None => None,
        Some(neuron_id_string) => Some(parse_neuron_id(neuron_id_string)?),
    };

    let of_principal = match opts.of_principal {
        None => None,
        Some(of_principal_string) => Some(PrincipalId::from_str(of_principal_string.as_str())?),
    };

    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let args = Encode!(&ListNeurons {
        limit: opts.limit,
        start_page_at,
        of_principal,
    })?;

    send_unsigned_ingress(
        "list_neurons",
        args,
        opts.dry_run,
        TargetCanister::Governance(governance_canister_id),
    )
    .await?;

    Ok(())
}
