// signature: Check GPG signatures
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use bytes::{
    buf::BufExt,
    Bytes,
};
use gpgrv::Keyring;
use std::io::BufReader;
use super::shasums::Shasums;

#[cfg(not(feature = "embedded_gpg_key"))]
use anyhow::anyhow;

#[cfg(not(feature = "embedded_gpg_key"))]
use std::fs::File;

#[cfg(not(feature = "embedded_gpg_key"))]
use std::io::prelude::*;

#[cfg(feature = "embedded_gpg_key")]
const HASHICORP_GPG_KEY: &'static str = include_str!("../gpg/hashicorp.asc");

#[cfg(not(feature = "embedded_gpg_key"))]
const HASHICORP_GPG_KEY_FILENAME: &'static str = "hashicorp.asc";

pub struct Signature {
    signature: Bytes,
}

impl Signature {
    pub fn new(signature: Bytes) -> Self {
        Self {
            signature: signature,
        }
    }

    pub fn check(&self, shasums: &Shasums) -> Result<()> {
        let mut keyring = Keyring::new();
        let gpg_key     = get_gpg_key()?;
        let gpg_key     = BufReader::new(gpg_key.as_bytes());

        // compat handles error returned by failure crate
        match keyring.append_keys_from_armoured(gpg_key) {
            Ok(_)  => Ok(()),
            Err(e) => Err(e.compat()),
        }?;

        let shasums = BufReader::new(shasums.content().as_bytes());

        match gpgrv::verify_detached(
            self.signature.clone().reader(),
            shasums,
            &keyring,
        ) {
            Ok(_)  => Ok(()),
            Err(e) => Err(e.compat()),
        }?;

        Ok(())
    }
}

#[cfg(feature = "embedded_gpg_key")]
fn get_gpg_key() -> Result<String> {
    Ok(HASHICORP_GPG_KEY.to_owned())
}

#[cfg(not(feature = "embedded_gpg_key"))]
fn get_gpg_key() -> Result<String> {
    let mut path = match dirs::data_dir() {
        Some(dir) => Ok(dir),
        None      => Err(anyhow!("Couldn't find shared data directory")),
    }?;

    // Ensure that the data dir exists
    if !path.exists() || !path.is_dir() {
        let msg = anyhow!(
            "Data directory {} does not exist or is not a directory",
            path.display(),
        );

        return Err(msg);
    }

    path = path.join(env!("CARGO_PKG_NAME"));
    path = path.join(HASHICORP_GPG_KEY_FILENAME);

    // Ensure that the GPG key exists
    if !path.exists() || !path.is_file() {
        let msg = anyhow!(
            "GPG key file {} does not exist or is not a file",
            path.display()
        );

        return Err(msg);
    }

    let file         = File::open(&path)?;
    let mut reader   = BufReader::new(file);
    let mut contents = String::new();

    reader.read_to_string(&mut contents)?;

    Ok(contents)
}
