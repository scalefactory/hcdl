// HTTP client
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

        let hash = hasher.result();

        let res = if hex::encode(hash) == shasum {
            Checksum::OK
        }
        else {
            Checksum::Bad
        };

        Ok(res)
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
