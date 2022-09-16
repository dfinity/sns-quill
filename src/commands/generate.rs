use crate::lib::{get_identity, mnemonic_to_pem, AnyhowResult};
use anyhow::{bail, Context};
use bip39::{Language, Mnemonic};
use clap::Parser;
use dialoguer::Password;
use rand::{rngs::OsRng, RngCore};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct GenerateOpts {
    /// Number of words: 12 or 24.
    #[clap(long, default_value = "12")]
    words: u32,

    /// File to write the seed phrase to.
    #[clap(long)]
    out_seed_file: Option<PathBuf>,

    /// File to write the PEM to.
    #[clap(long, default_value("identity.pem"))]
    to_pem_file: PathBuf,

    #[clap(long)]
    pem_file: Option<Dummy>, // fake param to ensure it is not accidentally used instead of --to-pem-file

    /// A file containing a seed phrase to generate the key from.
    #[clap(long, conflicts_with("out-seed-file"))]
    from_seed_file: Option<PathBuf>,

    /// Overwrite any existing seed file.
    #[clap(long, requires("out-seed-file"))]
    overwrite_seed_file: bool,

    /// Overwrite any existing PEM file.
    #[clap(long)]
    overwrite_pem_file: bool,

    /// Disable PEM encryption.
    #[clap(long)]
    disable_encryption: bool,
}

#[derive(Debug)]
struct Dummy;

impl FromStr for Dummy {
    type Err = &'static str;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("--pem-file is incorrect for this command (did you mean --to-pem-file?)")
    }
}

/// Generate or recover mnemonic seed phrase and/or PEM file.
pub fn exec(opts: GenerateOpts) -> AnyhowResult {
    if !opts.overwrite_pem_file && opts.to_pem_file.exists() {
        bail!("PEM file exists and overwrite is not set.");
    }
    let bytes = match opts.words {
        12 => 16,
        24 => 32,
        _ => bail!("Words must be 12 or 24."),
    };
    let mnemonic = match &opts.from_seed_file {
        Some(phrase_file) => {
            Mnemonic::parse(fs::read_to_string(phrase_file)?).context("Failed to parse mnemonic")?
        }
        None => {
            let mut key = vec![0u8; bytes];
            OsRng.fill_bytes(&mut key);
            Mnemonic::from_entropy_in(Language::English, &key).unwrap()
        }
    };
    let password = (!opts.disable_encryption)
        .then(|| {
            Password::new()
                .with_prompt("Enter a password to encrypt the PEM")
                .with_confirmation(
                    "Re-enter the password to confirm",
                    "Passwords did not match",
                )
                .interact()
        })
        .transpose()?;
    let password = password.as_deref().map(str::as_bytes);
    // argon2 is not needed, the algorithm already uses scrypt
    let pem = mnemonic_to_pem(&mnemonic, password).context("Failed to convert mnemonic to PEM")?;
    let ident = get_identity(&pem, password)?;
    let mut phrase = mnemonic
        .word_iter()
        .collect::<Vec<&'static str>>()
        .join(" ");
    phrase.push('\n');
    match opts.out_seed_file {
        Some(file) if file != Path::new("-") => {
            if !opts.overwrite_seed_file && file.exists() {
                bail!("Seed file exists and overwrite is not set.")
            }
            fs::write(file, phrase)?;
        }
        _ => {
            if opts.from_seed_file.is_none() {
                eprintln!("Your seed phrase: {phrase}\nThis can be used to reconstruct your key in case of emergency, so write it down and store it in a safe place.")
            }
        }
    }
    if opts.to_pem_file == Path::new("-") {
        println!("{pem}");
    } else {
        fs::write(opts.to_pem_file, &pem)?;
    }
    let (principal_id, account_id) = crate::commands::public::get_ids(Some(ident))?;
    println!("Principal id: {}", principal_id);
    println!("NNS account id: {}", account_id);
    Ok(())
}
