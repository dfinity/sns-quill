//! This module implements the command-line API.

use crate::lib::{qr, require_canister_ids, require_pem, AnyhowResult};
use anyhow::Context;
use clap::Parser;
use std::io::{self, Write};
use tokio::runtime::Runtime;

mod account_balance;
mod configure_dissolve_delay;
mod generate;
mod get_nervous_system_parameters;
mod make_proposal;
mod make_upgrade_canister_proposal;
mod public;
mod qrcode;
mod register_vote;
mod request_status;
mod send;
mod sns_canisters_summary;
mod stake_neuron;
mod summary;
mod transfer;

use crate::SnsCanisterIds;

#[derive(Parser)]
pub enum Command {
    /// Prints the principal id and the account id.
    PublicIds(public::PublicOpts),
    /// Queries a ledger account balance.
    AccountBalance(account_balance::AccountBalanceOpts),
    /// Signs a ledger transfer message to the provided 'to' account.
    Transfer(transfer::TransferOpts),
    /// Signs messages needed to stake governance tokens for a neuron. First, stake-neuron will sign
    /// a ledger transfer to a subaccount of the Governance canister calculated from the
    /// provided private key and memo. Second, stake-neuron will sign a ManageNeuron message for
    /// Governance to claim the neuron for the principal derived from the provided private key.
    StakeNeuron(stake_neuron::StakeNeuronOpts),
    /// Signs a ManageNeuron message to configure the dissolve delay of a neuron. With this command
    /// neuron holders can start dissolving, stop dissolving, or increase dissolve delay. The
    /// dissolve delay of a neuron determines its voting power, its ability to vote, its ability
    /// to make proposals, and other actions it can take (such as disbursing).
    ConfigureDissolveDelay(configure_dissolve_delay::ConfigureDissolveDelayOpts),
    /// Signs a ManageNeuron message to submit a proposal. With this command, neuron holders
    /// can submit proposals (such as a Motion Proposal) to be voted on by other neuron
    /// holders.
    MakeProposal(make_proposal::MakeProposalOpts),
    /// Signs a ManageNeuron message to register a vote for a proposal. Registering a vote will
    /// update the ballot of the given proposal and could trigger followees to vote. When
    /// enough votes are cast or enough time passes, the proposal will either be rejected or
    /// adopted and executed.
    RegisterVote(register_vote::RegisterVoteOpts),
    /// Generate a mnemonic seed phrase and generate or recover PEM.
    Generate(generate::GenerateOpts),
    /// Print QR Scanner dapp QR code: scan to start dapp to submit QR results.
    ScannerQRCode,
    /// Print QR code for data e.g. principal id.
    QRCode(qrcode::QRCodeOpts),
    /// Sends signed messages to the Internet computer.
    Send(send::SendOpts),
    /// Signs a ManageNeuron message to submit a UpgradeSnsControlledCanister
    /// proposal.
    MakeUpgradeCanisterProposal(make_upgrade_canister_proposal::MakeUpgradeCanisterProposalOpts),
    /// Prints a comprehensive overview of the sns. Includes sns canisters info and nervous system
    /// parameters.
    Summary,
    /// Outputs info about the sns canisters: Governance, Ledger, Root and canisters belonging to
    /// the controlled dapp. Info includes cycles, ownership and more.
    SnsCanistersSummary,
    /// Prints info about the system parameters in this sns. These include the cost of rejected
    /// proposal, the initial voting time, the reward distribution period, and others.
    GetNervousSystemParameters,
}

pub fn exec(
    private_key_pem: &Option<String>,
    sns_canister_ids: &Option<SnsCanisterIds>,
    qr: bool,
    cmd: Command,
) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds(opts) => public::exec(private_key_pem, opts),
        Command::AccountBalance(opts) => {
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            runtime.block_on(async { account_balance::exec(&canister_ids, opts).await })
        }
        Command::Transfer(opts) => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            transfer::exec(&pem, &canister_ids, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::StakeNeuron(opts) => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            stake_neuron::exec(&pem, &canister_ids, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::ConfigureDissolveDelay(opts) => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            configure_dissolve_delay::exec(&pem, &canister_ids, opts)
                .and_then(|out| print_vec(qr, &out))
        }
        Command::MakeProposal(opts) => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            make_proposal::exec(&pem, &canister_ids, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::RegisterVote(opts) => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            register_vote::exec(&pem, &canister_ids, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::Generate(opts) => generate::exec(opts),
        // QR code for URL: https://p5deo-6aaaa-aaaab-aaaxq-cai.raw.ic0.app/
        // Source code: https://github.com/ninegua/ic-qr-scanner
        Command::ScannerQRCode => {
            println!(
                "█████████████████████████████████████
█████████████████████████████████████
████ ▄▄▄▄▄ █▀█ █▄▀▄▀▄█ ▄ █ ▄▄▄▄▄ ████
████ █   █ █▀▀▀█ ▀▀█▄▀████ █   █ ████
████ █▄▄▄█ █▀ █▀▀██▀▀█ ▄ █ █▄▄▄█ ████
████▄▄▄▄▄▄▄█▄▀ ▀▄█ ▀▄█▄█▄█▄▄▄▄▄▄▄████
████▄▄▄▄ ▀▄  ▄▀▄ ▄ █▀▄▀▀▀ ▀ ▀▄█▄▀████
████▄█  █ ▄█▀█▄▀█▄  ▄▄ █ █   ▀█▀█████
████▄▀ ▀ █▄▄▄ ▄   █▄▀   █ ▀▀▀▄▄█▀████
████▄██▀▄▀▄▄ █▀█ ▄▄▄▄███▄█▄▀ ▄▄▀█████
████ ▀▄▀▄█▄▀▄▄▄▀█ ▄▄▀▄▀▀▀▄▀▀▀▄ █▀████
████ █▀██▀▄██▀▄█ █▀  █▄█▄▀▀  █▄▀█████
████▄████▄▄▄  ▀▀█▄▄██▄▀█ ▄▄▄ ▀   ████
████ ▄▄▄▄▄ █▄▄██▀▄▀ ▄█▄  █▄█ ▄▄▀█████
████ █   █ █  █▀▄▄▀▄ ▄▀▀▄▄▄ ▄▀ ▄▀████
████ █▄▄▄█ █ █▄▀▄██ ██▄█▀ ▄█  ▄ █████
████▄▄▄▄▄▄▄█▄▄▄▄▄▄██▄▄█▄████▄▄▄██████
█████████████████████████████████████
█████████████████████████████████████"
            );
            Ok(())
        }
        Command::QRCode(opts) => qrcode::exec(opts),
        Command::Send(opts) => runtime.block_on(async { send::exec(opts).await }),
        Command::MakeUpgradeCanisterProposal(opts) => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            make_upgrade_canister_proposal::exec(&pem, &canister_ids, opts)
                .and_then(|out| print_vec(qr, &out))
        }
        Command::Summary => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            summary::exec(&pem, &canister_ids).and_then(|out| print_vec(qr, &out))
        }
        Command::SnsCanistersSummary => {
            let pem = require_pem(private_key_pem)?;
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            sns_canisters_summary::exec(&pem, &canister_ids).and_then(|out| print_vec(qr, &out))
        }
        Command::GetNervousSystemParameters => {
            let canister_ids = require_canister_ids(sns_canister_ids)?;
            get_nervous_system_parameters::exec(&canister_ids).and_then(|out| print_vec(qr, &out))
        }
    }
}

// Using println! for printing to STDOUT and piping it to other tools leads to
// the problem that when the other tool closes its stream, the println! macro
// panics on the error and the whole binary crashes. This function provides a
// graceful handling of the error.
fn print<T>(arg: &T) -> AnyhowResult
where
    T: ?Sized + serde::ser::Serialize,
{
    if let Err(e) = io::stdout().write_all(serde_json::to_string(&arg)?.as_bytes()) {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn print_qr<T>(arg: &T, pause: bool) -> AnyhowResult
where
    T: serde::ser::Serialize,
{
    let json = serde_json::to_string(&arg)?;
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(json.as_bytes()).unwrap();
    let json = e.finish().unwrap();
    let json = base64::encode(json);
    qr::print_qr(json.as_str());
    if pause {
        let mut input_string = String::new();
        std::io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");
    }
    Ok(())
}

fn print_vec<T>(qr: bool, arg: &[T]) -> AnyhowResult
where
    T: serde::ser::Serialize,
{
    if !qr {
        print(arg)
    } else {
        for (i, a) in arg.iter().enumerate() {
            print_qr(&a, i != arg.len() - 1).context("Failed to print QR code")?;
        }
        Ok(())
    }
}
