#![warn(unused_extern_crates)]

use crate::lib::AnyhowResult;
use anyhow::{anyhow, bail, Context};
use bip39::Mnemonic;
use clap::{crate_version, Args, Parser};
use ic_base_types::CanisterId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

mod commands;
mod lib;

/// Cold wallet toolkit for interacting with a Service Nervous System's Ledger & Governance canisters.
#[derive(Parser)]
#[clap(name("sns-quill"), version = crate_version!())]
pub struct CliOpts {
    #[clap(subcommand)]
    command: commands::Command,
}

#[derive(Args)]
pub struct PemOpts {
    /// Path to your PEM file (use "-" for STDIN)
    #[clap(long)]
    pem_file: Option<PathBuf>,

    /// Path to your seed file (use "-" for STDIN)
    #[clap(long)]
    seed_file: Option<PathBuf>,
}

impl PemOpts {
    fn to_pem(&self) -> AnyhowResult<String> {
        read_pem(self.pem_file.as_deref(), self.seed_file.as_deref())
    }
}

#[derive(Args)]
pub struct IdsOpt {
    /// Path to the JSON file containing the SNS cluster's canister ids. This is a JSON
    /// file containing a JSON map of canister names to canister IDs.
    ///
    /// For example,
    /// {
    ///   "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
    ///   "ledger_canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
    ///   "root_canister_id": "r7inp-6aaaa-aaaaa-aaabq-cai"
    ///   "dapp_canister_id_list": [
    ///.      "qoctq-giaaa-aaaaa-aaaea-cai"
    ///.   ],
    /// }
    #[clap(long)]
    canister_ids_file: PathBuf,
}

impl IdsOpt {
    fn to_ids(&self) -> AnyhowResult<SnsCanisterIds> {
        read_sns_canister_ids(&self.canister_ids_file)
    }
}

#[derive(Args)]
pub struct QrOpt {
    /// Output the result(s) as UTF-8 QR codes.
    #[clap(long)]
    qr: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnsCanisterIds {
    pub governance_canister_id: CanisterId,
    pub ledger_canister_id: CanisterId,
    pub root_canister_id: CanisterId,
    pub swap_canister_id: CanisterId,
    pub dapp_canister_id_list: Vec<CanisterId>,
}

fn main() -> AnyhowResult {
    let opts = CliOpts::parse();
    commands::dispatch(opts.command)
}

/// Get PEM from the file if provided, or try to convert from the seed file
fn read_pem(pem_file: Option<&Path>, seed_file: Option<&Path>) -> AnyhowResult<String> {
    match (pem_file, seed_file) {
        (Some(pem_file), _) => read_file(&pem_file, "PEM"),
        (_, Some(seed_file)) => {
            let seed = read_file(&seed_file, "seed")?;
            let mnemonic = parse_mnemonic(&seed)?;
            let mnemonic = lib::mnemonic_to_pem(&mnemonic)?;
            Ok(mnemonic)
        }
        _ => bail!("Either the PEM file or the seed file must be provided"),
    }
}

/// Tries to load canister IDs from file_path, which is a JSON formatted file containing a map
/// from the following (string) keys to canister ID strings:
///
///   1. governance_canister_id
///   2. ledger_canister_id
///   3. root_canister_id
///   4. dapp_canister_id_list (array)
///
/// If the file is malformed, Err is returned. Else, return the parsed struct.
fn read_sns_canister_ids(file_path: &Path) -> AnyhowResult<SnsCanisterIds> {
    let file = File::open(file_path).context("Could not open the SNS Canister Ids file")?;
    let ids: HashMap<String, Value> =
        serde_json::from_reader(file).context("Could not parse the SNS Canister Ids file")?;

    let governance_canister_id = parse_canister_id("governance_canister_id", &ids)?;
    let ledger_canister_id = parse_canister_id("ledger_canister_id", &ids)?;
    let root_canister_id = parse_canister_id("root_canister_id", &ids)?;
    let swap_canister_id = parse_canister_id("swap_canister_id", &ids)?;

    let dapp_canister_id_list = parse_dapp_canister_id_list("dapp_canister_id_list", &ids)?;

    Ok(SnsCanisterIds {
        governance_canister_id,
        ledger_canister_id,
        swap_canister_id,
        root_canister_id,
        dapp_canister_id_list,
    })
}

fn parse_canister_id(
    key_name: &str,
    canister_id_map: &HashMap<String, Value>,
) -> AnyhowResult<CanisterId> {
    let value = canister_id_map.get(key_name).ok_or_else(|| {
        anyhow!(
            "'{}' is not present in --canister-ids-file <file>",
            key_name
        )
    })?;
    if let Value::String(str) = value {
        let canister_id = CanisterId::from_str(str)
            .map_err(|err| anyhow!("Could not parse CanisterId of '{}': {}", key_name, err))?;
        Ok(canister_id)
    } else {
        Err(anyhow!("Couldnt read {} as a string", key_name))
    }
}

fn parse_dapp_canister_id_list(
    key_name: &str,
    canister_id_map: &HashMap<String, Value>,
) -> AnyhowResult<Vec<CanisterId>> {
    let value = canister_id_map.get(key_name).ok_or_else(|| {
        anyhow!(
            "'{}' is not present in --canister-ids-file <file>",
            key_name
        )
    })?;
    let mut canister_id_vec: Vec<CanisterId> = vec![];
    match value {
        Value::Array(id_array) => {
            for id in id_array {
                if let Value::String(str) = id {
                    let canister_id = CanisterId::from_str(str).map_err(|err| {
                        anyhow!("Could not parse {} as a CanisterId: {}", str, err)
                    })?;
                    canister_id_vec.push(canister_id);
                }
            }
            Ok(canister_id_vec)
        }
        _ => Err(anyhow!("Failed to parse field {} as an Array", key_name)),
    }
}

fn parse_mnemonic(phrase: &str) -> AnyhowResult<Mnemonic> {
    Mnemonic::parse(phrase).context("Couldn't parse the seed phrase as a valid mnemonic. {:?}")
}

fn read_file(path: impl AsRef<Path>, name: &str) -> AnyhowResult<String> {
    let path = path.as_ref();
    if path == Path::new("-") {
        let mut buffer = String::new();
        use std::io::Read;
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map(|_| buffer)
            .context(format!("Couldn't read {} from STDIN", name))
    } else {
        std::fs::read_to_string(path).with_context(|| format!("Couldn't read {} file", name))
    }
}

#[test]
fn test_read_pem_from_pem_file() {
    use std::io::Write;

    let mut pem_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let content = "pem";
    pem_file
        .write_all(content.as_bytes())
        .expect("Cannot write to temp file");

    let res = read_pem(Some(pem_file.path()), None);

    assert_eq!(content, res.expect("read_pem from pem file"));
}

#[test]
fn test_read_pem_from_seed_file() {
    use std::io::Write;

    let mut seed_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let phrase = "ozone drill grab fiber curtain grace pudding thank cruise elder eight about";
    seed_file
        .write_all(phrase.as_bytes())
        .expect("Cannot write to temp file");
    let mnemonic = lib::mnemonic_to_pem(&Mnemonic::parse(phrase).unwrap()).unwrap();

    let pem = read_pem(None, Some(seed_file.path())).expect("Unable to read seed_file");

    assert_eq!(mnemonic, pem);
}

#[test]
fn test_read_pem_from_non_existing_file() {
    let dir = tempfile::tempdir().expect("Cannot create temp dir");
    let non_existing_file = dir.path().join("non_existing_pem_file");

    read_pem(Some(&non_existing_file), None).unwrap_err();

    read_pem(None, Some(&non_existing_file)).unwrap_err();
}

#[test]
fn test_read_canister_ids_from_file() {
    use std::io::Write;

    let mut canister_ids_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let expected_canister_ids = SnsCanisterIds {
        governance_canister_id: CanisterId::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
        ledger_canister_id: CanisterId::from_str("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        root_canister_id: CanisterId::from_str("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap(),
        swap_canister_id: CanisterId::from_str("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap(),
        dapp_canister_id_list: vec![
            CanisterId::from_str("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            CanisterId::from_str("qoctq-giaaa-aaaaa-aaaea-cai").unwrap(),
        ],
    };

    let json_str = serde_json::to_string(&expected_canister_ids).unwrap();

    write!(canister_ids_file, "{}", json_str).expect("Cannot write to tmp file");

    let actual_canister_ids =
        read_sns_canister_ids(canister_ids_file.path()).expect("Unable to read canister_ids_file");

    assert_eq!(actual_canister_ids, expected_canister_ids);
}

#[test]
fn test_read_canister_ids_from_file_empty_dapp_canister_id_list() {
    use std::io::Write;

    let mut canister_ids_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let expected_canister_ids = SnsCanisterIds {
        governance_canister_id: CanisterId::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
        ledger_canister_id: CanisterId::from_str("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        root_canister_id: CanisterId::from_str("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap(),
        swap_canister_id: CanisterId::from_str("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap(),
        dapp_canister_id_list: vec![],
    };

    let json_str = serde_json::to_string(&expected_canister_ids).unwrap();

    write!(canister_ids_file, "{}", json_str).expect("Cannot write to tmp file");

    let actual_canister_ids =
        read_sns_canister_ids(canister_ids_file.path()).expect("Unable to read canister_ids_file");

    assert_eq!(actual_canister_ids, expected_canister_ids);
}

#[test]
fn test_canister_ids_from_non_existing_file() {
    let dir = tempfile::tempdir().expect("Cannot create temp dir");
    let non_existing_file = dir.path().join("non_existing_pem_file");

    read_sns_canister_ids(&non_existing_file).unwrap_err();
}

#[test]
fn test_canister_ids_from_malformed_canister_id() {
    use std::io::Write;

    let mut canister_ids_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let raw_json = r#"{"governance_canister_id": "Not a valid canister id","ledger_canister_id": "Not a valid canister id","root_canister_id": "Not a valid canister id"}"#;
    write!(canister_ids_file, "{}", raw_json).expect("Cannot write to tmp file");

    read_sns_canister_ids(canister_ids_file.path()).unwrap_err();
}

#[test]
fn test_canister_ids_from_missing_key() {
    use std::io::Write;

    let mut canister_ids_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let raw_json = r#"{"ledger_canister_id": "Not a valid canister id","root_canister_id": "Not a valid canister id"}"#;
    write!(canister_ids_file, "{}", raw_json).expect("Cannot write to tmp file");

    read_sns_canister_ids(canister_ids_file.path()).unwrap_err();
}
