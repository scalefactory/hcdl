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

const HASHICORP_GPG_KEY: &'static str = include_str!("../gpg/hashicorp.asc");

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
        let gpg_key = BufReader::new(HASHICORP_GPG_KEY.as_bytes());

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
