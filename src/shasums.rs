// shasums: Handle checking of files against shasums
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::{
    anyhow,
    Result,
};
use sha2::{
    Digest,
    Sha256,
};
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Checksum {
    OK,
    Bad,
}

pub struct Shasums {
    shasums: String,
}

impl Shasums {
    pub fn new(shasums: String) -> Self {
        Self {
            shasums: shasums,
        }
    }

    // Check the shasum of the specified file
    pub fn check(&self, filename: &str) -> Result<Checksum> {
        let shasum = match self.shasum(filename) {
            Some(shasum) => Ok(shasum),
            None         => {
                Err(anyhow!("Couldn't find shasum for {}", filename))
            },
        }?;

        let path       = Path::new(filename);
        let mut file   = File::open(&path)?;
        let mut hasher = Sha256::new();

        io::copy(&mut file, &mut hasher)?;

        let hash = hasher.finalize();

        let res = if hex::encode(hash) == shasum {
            Checksum::OK
        }
        else {
            Checksum::Bad
        };

        Ok(res)
    }

    // Return a reference to the shasums
    pub fn content(&self) -> &str {
        &self.shasums
    }

    // Return the shasum for the specified filename
    fn shasum(&self, filename: &str) -> Option<String> {
        // Filter the shasum list down to the filename we're interested in
        let shasum: Vec<&str> = self.shasums
            .lines()
            .filter(|l| l.ends_with(filename))
            .collect();

        // Our list should only have a single thing in it now, try to take it
        let shasum = match shasum.first() {
            Some(sum) => sum,
            None      => return None,
        };

        // Split the shasum from the filename
        let shasum = shasum.split_whitespace()
            .collect::<Vec<&str>>()[0]
            .to_string();

        // Return the shasum hex
        Some(shasum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_check() {
        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
            "shasums-check.txt"
        );

        let shasums_content = format!(
            "{shasum} {filename}",
            shasum="bd6abe380b9ffdca9375f1202b36e1c7b8ca3e8b5de4ae8582c0037949c30ce8",
            filename=test_data_path,
        );

        let shasums = Shasums::new(shasums_content.into());
        let res     = shasums.check(&test_data_path).unwrap();

        assert_eq!(Checksum::OK, res);
    }

    #[test]
    fn test_content() {
        let shasums_content = indoc!("
            5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03 test
        ");

        let shasums = Shasums::new(shasums_content.into());

        assert_eq!(shasums_content, shasums.content())
    }

    #[test]
    fn test_shasum() {
        let shasums_content = indoc!("
            5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03 test
        ");

        let shasums  = Shasums::new(shasums_content.into());
        let expected = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
        let ret      = shasums.shasum("test").unwrap();

        assert_eq!(expected, ret)
    }
}
