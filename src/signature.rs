// signature: Check GPG signatures
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::shasums::Shasums;
use anyhow::Result;
use bytes::{
    Buf,
    Bytes,
};
use gpgrv::Keyring;
use std::io::BufReader;

#[cfg(not(feature = "embed_gpg_key"))]
use anyhow::anyhow;

#[cfg(any(test, not(feature = "embed_gpg_key")))]
use std::io::prelude::*;

#[cfg(any(test, not(feature = "embed_gpg_key")))]
use std::fs::File;

#[cfg(any(test, not(feature = "embed_gpg_key")))]
use std::path::PathBuf;

#[cfg(any(test, not(feature = "embed_gpg_key")))]
const HASHICORP_GPG_KEY_FILENAME: &str = "hashicorp.asc";

#[cfg(feature = "embed_gpg_key")]
const HASHICORP_GPG_KEY: &str = include_str!("../gpg/hashicorp.asc");

#[derive(Debug)]
pub struct Signature {
    // This is the signature of the shasums file.
    signature: Bytes,

    // The GPG key that the signature was signed with.
    gpg_key: String,

    // The GPG keyring
    keyring: Keyring,
}

// We implement this ourselves since Keyring doesn't implement PartialEq
// Hopefully matching Keyrings based on the KeyIDs they contain is good enough
impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        let gpg_key_match   = self.gpg_key == other.gpg_key;
        let keyring_match   = self.keyring.key_ids() == other.keyring.key_ids();
        let signature_match = self.signature == other.signature;

        gpg_key_match && keyring_match && signature_match
    }
}

impl Signature {
    pub fn new(signature: Bytes) -> Result<Self> {
        let gpg_key = get_gpg_key()?;

        let signature = Self::with_gpg_key(
            signature,
            gpg_key,
        )?;

        Ok(signature)
    }

    pub fn with_gpg_key(signature: Bytes, gpg_key: String) -> Result<Self> {
        let mut keyring = Keyring::new();
        let reader      = BufReader::new(gpg_key.as_bytes());

        keyring.append_keys_from_armoured(reader)?;

        let signature = Self {
            signature: signature,
            gpg_key:   gpg_key,
            keyring:   keyring,
        };

        Ok(signature)
    }

    pub fn check(&self, shasums: &Shasums) -> Result<()> {
        let shasums   = BufReader::new(shasums.content().as_bytes());
        let signature = self.signature.clone().reader();

        gpgrv::verify_detached(signature, shasums, &self.keyring)?;

        Ok(())
    }
}

// Read a file's content into a String
#[cfg(any(test, not(feature = "embed_gpg_key")))]
fn read_file_content(path: &PathBuf) -> Result<String> {
    let file         = File::open(&path)?;
    let mut reader   = BufReader::new(file);
    let mut contents = String::new();

    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

// Find the path where the GPG key should be stored.
#[cfg(not(feature = "embed_gpg_key"))]
fn get_gpg_key_path() -> Result<PathBuf> {
    // During tests we short circuit the path discovery to just take the
    // GPG key from the test-data directory.
    let path = if cfg!(test) {
        let test_data_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/gpg/",
        );

        let mut path = PathBuf::new();
        path.push(test_data_dir);
        path.push(HASHICORP_GPG_KEY_FILENAME);
        path
    }
    else {
        let mut path = dirs::data_dir()
            .ok_or_else(|| anyhow!("Couldn't find shared data directory"))?;

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
            let msg = format!(
                "GPG key file {} does not exist or it not a file.\n\
                 Check https://www.hashicorp.com/security to find the GPG key",
                path.display(),
            );

            return Err(anyhow!(msg));
        }

        path
    };

    Ok(path)
}

// Locate and read the GPG key.
#[cfg(not(feature = "embed_gpg_key"))]
fn get_gpg_key() -> Result<String> {
    let path     = get_gpg_key_path()?;
    let contents = read_file_content(&path)?;

    Ok(contents)
}

// Allow the wrap here, since this is for simplicity when toggling the
// embed_gpg_key feature.
#[cfg(feature = "embed_gpg_key")]
#[allow(clippy::unnecessary_wraps)]
fn get_gpg_key() -> Result<String> {
    let gpg_key = HASHICORP_GPG_KEY.to_string();

    Ok(gpg_key)
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
    fn test_signature_check_ok() {
        let gpg_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/gpg/",
        );

        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
        );

        let gpg_key_file_path = Path::new(&format!(
            "{}{}",
            gpg_path,
            HASHICORP_GPG_KEY_FILENAME,
        )).to_path_buf();

        let signature_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.15.1_SHA256SUMS.sig",
        )).to_path_buf();

        let gpg_key_content   = read_file_content(&gpg_key_file_path).unwrap();
        let signature_content = read_file_bytes(&signature_file_path).unwrap();
        let signature         = Signature::with_gpg_key(
            Bytes::from(signature_content),
            gpg_key_content,
        ).unwrap();

        let shasums_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.15.1_SHA256SUMS",
        )).to_path_buf();

        let shasums_content = read_file_content(&shasums_file_path).unwrap();
        let shasums         = Shasums::new(shasums_content);

        let res = signature.check(&shasums);

        assert!(res.is_ok())
    }

    #[test]
    fn test_signature_check_bad_gpg_key() {
        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
        );

        let signature_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.15.1_SHA256SUMS.sig",
        )).to_path_buf();

        let signature_content = read_file_bytes(&signature_file_path).unwrap();
        let signature         = Signature::with_gpg_key(
            Bytes::from(signature_content),
            "bad".into(),
        );

        assert_eq!(signature.unwrap_err().to_string(), "reading first line of key file")
    }

    #[test]
    fn test_signature_check_bad_signature() {
        let gpg_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/gpg/",
        );

        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
        );

        let gpg_key_file_path = Path::new(&format!(
            "{}{}",
            gpg_path,
            HASHICORP_GPG_KEY_FILENAME,
        )).to_path_buf();

        let signature_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.15.1_SHA256SUMS.sig",
        )).to_path_buf();

        let gpg_key_content   = read_file_content(&gpg_key_file_path).unwrap();
        let signature_content = read_file_bytes(&signature_file_path).unwrap();
        let signature         = Signature::with_gpg_key(
            Bytes::from(signature_content),
            gpg_key_content,
        ).unwrap();

        let shasums_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "test.txt",
        )).to_path_buf();

        let shasums_content = read_file_content(&shasums_file_path).unwrap();
        let shasums         = Shasums::new(shasums_content);

        let res = signature.check(&shasums);

        assert_eq!(
            res.unwrap_err().to_string(),
            "no valid signatures: [HintMismatch]",
        )
    }

    // This tests newer signatures against a known bad (compromised)
    // signature after HCSEC-2021-12.
    // https://discuss.hashicorp.com/t/hcsec-2021-12-codecov-security-event-and-hashicorp-gpg-key-exposure/23512
    #[test]
    fn test_signature_check_known_bad_signature() {
        let gpg_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/gpg/",
        );

        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
        );

        let gpg_key_file_path = Path::new(&format!(
            "{}{}",
            gpg_path,
            HASHICORP_GPG_KEY_FILENAME,
        )).to_path_buf();

        let signature_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.12.26_SHA256SUMS.sig",
        )).to_path_buf();

        let gpg_key_content   = read_file_content(&gpg_key_file_path).unwrap();
        let signature_content = read_file_bytes(&signature_file_path).unwrap();
        let signature         = Signature::with_gpg_key(
            Bytes::from(signature_content),
            gpg_key_content,
        ).unwrap();

        let shasums_file_path = Path::new(&format!(
            "{}{}",
            test_data_path,
            "terraform_0.12.26_SHA256SUMS",
        )).to_path_buf();

        let shasums_content = read_file_content(&shasums_file_path).unwrap();
        let shasums         = Shasums::new(shasums_content);

        let res = signature.check(&shasums);

        assert_eq!(
            res.unwrap_err().to_string(),
            "no valid signatures: [NoKey]",
        )
    }
}
