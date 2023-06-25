// signature: Check GPG signatures
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::shasums::Shasums;
use anyhow::{
    anyhow,
    Result,
};
use bytes::{
    Buf,
    Bytes,
};
use pgp::composed::{
    Deserializable,
    StandaloneSignature,
};
use pgp::composed::signed_key::public::SignedPublicKey;
use std::io::BufReader;
use std::io::Cursor;

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

/// Handle checking `signature` against `public_key`.
#[derive(Debug)]
pub struct Signature {
    // The public key
    public_key: SignedPublicKey,

    // This is the signature of the shasums file.
    signature: StandaloneSignature,
}

// Only used by client::test_get_signature.
#[cfg(test)]
impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        let public_key_match = self.public_key == other.public_key;
        let signature_match = self.signature.signature == other.signature.signature;

        public_key_match && signature_match
    }
}

impl Signature {
    /// Create a new [`Signature`] handler from the given `signature`.
    ///
    /// # Errors
    ///
    /// Can error if failing to get the public key.
    pub fn new(signature: Bytes) -> Result<Self> {
        let public_key = get_public_key()?;

        let signature = Self::with_public_key(
            signature,
            &public_key,
        )?;

        Ok(signature)
    }

    /// Create a new [`Signature`] handler from the given `signature` and
    /// `public_key`.
    ///
    /// # Errors
    ///
    /// Can error if failing to read the public key or the signature.
    pub fn with_public_key(signature: Bytes, public_key: &str) -> Result<Self> {
        let mut cursor = Cursor::new(public_key.as_bytes());
        let public_key = SignedPublicKey::from_armor_single(&mut cursor)?;
        let reader = BufReader::new(signature.reader());
        let signature = StandaloneSignature::from_bytes(reader)?;

        let signature = Self {
            signature:  signature,
            public_key: public_key.0,
        };

        Ok(signature)
    }

    /// Check the given [`Shasums`] content against the [`Signature`].
    ///
    /// # Errors
    ///
    /// Will return an error if unable to verify the signature against the
    /// public key or any of its subkeys.
    pub fn check(&self, shasums: &Shasums) -> Result<()> {
        let shasums = shasums.content().as_bytes();

        // We have to check the signature against all public subkeys and the
        // overall public key.
        for subkey in &self.public_key.public_subkeys {
            match self.signature.verify(&subkey, shasums) {
                Err(_) => continue,
                Ok(()) => return Ok(()),
            }
        }

        // One last attempt, check against the main public key.
        match self.signature.verify(&self.public_key, shasums) {
            Err(_) => Err(anyhow!("Couldn't verify signature")),
            Ok(()) => Ok(()),
        }
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
fn get_public_key_path() -> Result<PathBuf> {
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
                "Data directory {dir} does not exist or is not a directory",
                dir = path.display(),
            );

            return Err(msg);
        }

        path = path.join(env!("CARGO_PKG_NAME"));
        path = path.join(HASHICORP_GPG_KEY_FILENAME);

        // Ensure that the GPG key exists
        if !path.exists() || !path.is_file() {
            let msg = format!(
                "GPG key file {path} does not exist or it not a file.\n\
                 Check https://www.hashicorp.com/security to find the GPG key",
                path = path.display(),
            );

            return Err(anyhow!(msg));
        }

        path
    };

    Ok(path)
}

// Locate and read the GPG key.
#[cfg(not(feature = "embed_gpg_key"))]
fn get_public_key() -> Result<String> {
    let path       = get_public_key_path()?;
    let public_key = read_file_content(&path)?;

    Ok(public_key)
}

// Allow the wrap here, since this is for simplicity when toggling the
// embed_gpg_key feature.
#[cfg(feature = "embed_gpg_key")]
#[allow(clippy::unnecessary_wraps)]
fn get_public_key() -> Result<String> {
    let public_key = HASHICORP_GPG_KEY.to_string();

    Ok(public_key)
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
        let signature         = Signature::with_public_key(
            Bytes::from(signature_content),
            &gpg_key_content,
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
    fn test_signature_check_bad_public_key() {
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
        let signature         = Signature::with_public_key(
            Bytes::from(signature_content),
            "bad".into(),
        );

        assert_eq!(
            signature.unwrap_err().to_string(),
            "io error: Custom { kind: Interrupted, error: \"incomplete parse\" }",
        )
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
        let signature         = Signature::with_public_key(
            Bytes::from(signature_content),
            &gpg_key_content,
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
            "Couldn't verify signature",
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
        let signature         = Signature::with_public_key(
            Bytes::from(signature_content),
            &gpg_key_content,
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
            "Couldn't verify signature",
        )
    }
}
