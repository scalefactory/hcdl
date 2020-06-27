// signature: Check GPG signatures
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::shasums::Shasums;
use anyhow::{
    anyhow,
    Result,
};
use bytes::{
    buf::BufExt,
    Bytes,
};
use gpgrv::Keyring;
use std::fs::File;
use std::io::{
    prelude::*,
    BufReader,
};
use std::path::PathBuf;

const HASHICORP_GPG_KEY_FILENAME: &str = "hashicorp.asc";

#[derive(Debug, PartialEq)]
pub struct Signature {
    // This is the signature of the shasums file.
    signature: Bytes,

    // The GPG key that the signature was signed with.
    gpg_key: String,
}

impl Signature {
    pub fn new(signature: Bytes) -> Result<Self> {
        let gpg_key = get_gpg_key()?;

        let signature = Self {
            signature: signature,
            gpg_key:   gpg_key,
        };

        Ok(signature)
    }

    // Dead code allowed here because we use this during testing.
    #[allow(dead_code)]
    pub fn new_with_gpg_key(signature: Bytes, gpg_key: String) -> Self {
        Self {
            signature: signature,
            gpg_key:   gpg_key,
        }
    }

    pub fn check(&self, shasums: &Shasums) -> Result<()> {
        let mut keyring = Keyring::new();
        let gpg_key     = BufReader::new(self.gpg_key.as_bytes());

        // compat handles error returned by failure crate
        match keyring.append_keys_from_armoured(gpg_key) {
            Ok(_)  => Ok(()),
            Err(e) => Err(e.compat()),
        }?;

        let shasums   = BufReader::new(shasums.content().as_bytes());
        let signature = self.signature.clone().reader();

        match gpgrv::verify_detached(signature, shasums, &keyring) {
            Ok(_)  => Ok(()),
            Err(e) => Err(e.compat()),
        }?;

        Ok(())
    }
}

// Read a file's content into a String
fn read_file_content(path: &PathBuf) -> Result<String> {
    let file         = File::open(&path)?;
    let mut reader   = BufReader::new(file);
    let mut contents = String::new();

    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

// Location and read the GPG key.
fn get_gpg_key() -> Result<String> {
    // During tests we short circuit the path discovery to just take the
    // GPG key from the test-data directory.
    let path = if cfg!(test) {
        let test_data_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
        );

        let mut path = PathBuf::new();
        path.push(test_data_dir);
        path.push(HASHICORP_GPG_KEY_FILENAME);
        path
    }
    else {
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

        path
    };

    let contents = read_file_content(&path)?;

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Read a file's contents into Bytes
    fn read_file_bytes(path: &PathBuf) -> Result<Bytes> {
        let file         = File::open(&path)?;
        let mut reader   = BufReader::new(file);
        let mut contents = Vec::new();

        reader.read_to_end(&mut contents)?;

        Ok(Bytes::from(contents))
    }

    #[test]
    fn test_signature_check() {
        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
        );

        let gpg_key_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            HASHICORP_GPG_KEY_FILENAME,
        )).to_path_buf();

        let signature_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.12.26_SHA256SUMS.sig",
        )).to_path_buf();

        let gpg_key_content   = read_file_content(&gpg_key_file_path).unwrap();
        let signature_content = read_file_bytes(&signature_file_path).unwrap();
        let signature         = Signature::new_with_gpg_key(
            Bytes::from(signature_content),
            gpg_key_content,
        );

        let shasums_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.12.26_SHA256SUMS",
        )).to_path_buf();

        let shasums_content = read_file_content(&shasums_file_path).unwrap();
        let shasums         = Shasums::new(shasums_content);

        let res = signature.check(&shasums);

        assert!(res.is_ok())
    }
}
