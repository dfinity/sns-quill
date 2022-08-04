use crate::lib::{mnemonic_to_pem, AnyhowResult};
use anyhow::{bail, Context};
use bip39::{Language, Mnemonic};
use clap::Parser;
use rand::{rngs::OsRng, RngCore};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct GenerateOpts {
    /// Number of words: 12 or 24.
    #[clap(long, default_value = "12")]
    words: u32,

    /// File to write the seed phrase to.
    #[clap(long, default_value = "seed.txt")]
    seed_file: PathBuf,

    /// File to write the PEM to.
    #[clap(long)]
    pem_file: Option<PathBuf>,

    /// A seed phrase in quotes to use to generate the PEM file.
    #[clap(long)]
    phrase: Option<String>,

    /// Overwrite any existing seed file.
    #[clap(long)]
    overwrite_seed_file: bool,

    /// Overwrite any existing PEM file.
    #[clap(long)]
    overwrite_pem_file: bool,
}

/// Generate or recover mnemonic seed phrase and/or PEM file.
pub fn exec(opts: GenerateOpts) -> AnyhowResult {
    if opts.seed_file.exists() && !opts.overwrite_seed_file {
        bail!("Seed file exists and overwrite is not set.");
    }
    if let Some(path) = &opts.pem_file {
        if path.exists() && !opts.overwrite_pem_file {
            bail!("PEM file exists and overwrite is not set.");
        }
    }
    let bytes = match opts.words {
        12 => 16,
        24 => 32,
        _ => bail!("Words must be 12 or 24."),
    };
    let mnemonic = match opts.phrase {
        Some(phrase) => Mnemonic::parse(phrase).context("Failed to parse mnemonic")?,
        None => {
            let mut key = vec![0u8; bytes];
            OsRng.fill_bytes(&mut key);
            Mnemonic::from_entropy_in(Language::English, &key).unwrap()
        }
    };
    let pem = mnemonic_to_pem(&mnemonic).context("Failed to convert mnemonic to PEM")?;
    let mut phrase = mnemonic
        .word_iter()
        .collect::<Vec<&'static str>>()
        .join(" ");
    phrase.push('\n');
    std::fs::write(opts.seed_file, phrase)?;
    if let Some(path) = opts.pem_file {
        std::fs::write(path, &pem)?;
    }
    let (principal_id, account_id) = crate::commands::public::get_ids(&pem)?;
    println!("Principal id: {}", principal_id);
    println!("Account id: {}", account_id);
    Ok(())
}
