// shasums: Handle checking of files against shasums
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::TmpFile;
use anyhow::{
    anyhow,
    Result,
};
use sha2::{
    Digest,
    Sha256,
};
use std::io;

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
    pub fn check(&self, tmpfile: &mut TmpFile) -> Result<Checksum> {
        let filename = &tmpfile.filename;

        let shasum = self.shasum(filename)
            .ok_or_else(|| anyhow!("Couldn't find shasum for {}", filename))?;

        let mut file   = tmpfile.handle()?;
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
        let shasum_entry: &str = self.shasums
            .lines()
            .filter(|l| l.ends_with(filename))
            .collect::<Vec<&str>>()
            .first()?;

        // Split the shasum from the filename
        let shasum = shasum_entry
            .split_whitespace()
            .collect::<Vec<&str>>()
            .first()?
            .to_string();

        // Return the shasum hex
        Some(shasum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::{
        self,
        BufReader,
    };

    // Copies the given content into the tmpfile handle.
    fn tmpfile_from_file(path: &str) -> TmpFile {
        let file        = File::open(&path).unwrap();
        let mut reader  = BufReader::new(file);
        let mut tmpfile = TmpFile::new(&path).unwrap();
        let mut handle  = tmpfile.handle().unwrap();

        io::copy(&mut reader, &mut handle).unwrap();

        tmpfile
    }

    #[test]
    fn test_check_bad_checksum() {
        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
            "shasums-check.txt"
        );

        let shasums_content = format!(
            "{shasum} {filename}",
            shasum = "bad",
            filename = test_data_path,
        );

        let mut tmpfile = tmpfile_from_file(&test_data_path);

        let shasums = Shasums::new(shasums_content.into());
        let res     = shasums.check(&mut tmpfile).unwrap();

        assert_eq!(Checksum::Bad, res);
    }

    #[test]
    fn test_check_bad_no_such_filename() {
        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
            "shasums-check.txt"
        );

        let shasums_content = format!(
            "{shasum} {filename}",
            shasum = "bad",
            filename = "nope",
        );

        let mut tmpfile = tmpfile_from_file(&test_data_path);

        let shasums = Shasums::new(shasums_content.into());
        let res     = shasums.check(&mut tmpfile);

        assert_eq!(
            res.unwrap_err().to_string(),
            format!("Couldn't find shasum for {}", test_data_path),
        );
    }

    #[test]
    fn test_check_ok() {
        let test_data_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/",
            "shasums-check.txt"
        );

        let shasums_content = format!(
            "{shasum} {filename}",
            shasum = "bd6abe380b9ffdca9375f1202b36e1c7b8ca3e8b5de4ae8582c0037949c30ce8",
            filename = test_data_path,
        );

        let mut tmpfile = tmpfile_from_file(&test_data_path);

        let shasums = Shasums::new(shasums_content.into());
        let res     = shasums.check(&mut tmpfile).unwrap();

        assert_eq!(Checksum::OK, res);
    }

    #[test]
    fn test_content() {
        let shasums_content = format!(
            "{shasum} {filename}",
            shasum = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03",
            filename = "test",
        );

        let shasums = Shasums::new(shasums_content.clone().into());

        assert_eq!(shasums_content, shasums.content())
    }

    #[test]
    fn test_shasum() {
        let shasums_content = format!(
            "{shasum} {filename}",
            shasum = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03",
            filename = "test",
        );

        let shasums  = Shasums::new(shasums_content.into());
        let expected = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
        let ret      = shasums.shasum("test").unwrap();

        assert_eq!(expected, ret)
    }
}
