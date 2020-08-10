// crc32: Handle checking CRC32 of a given file against the expected CRC32.
#![forbid(unsafe_code)]
use anyhow::{
    anyhow,
    Result,
};
use crc32fast::Hasher;
use std::fs::File;
use std::io::{
    prelude::*,
    BufReader,
};
use std::path::Path;

// Buffer size, 1MiB
const BUFFER_SIZE: usize = 1_024 * 1_024;

// Check the given `path`'s CRC32 against the `expected` CRC32.
pub fn check<P>(path: P, expected: u32) -> Result<()>
where
    P: AsRef<Path>,
{
    let file       = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut buf    = [0; BUFFER_SIZE];
    let mut hasher = Hasher::new();

    // crc32fast doesn't have a Write implementation, so we have to handle the
    // updates manually instead of using io::copy.
    loop {
        match reader.read(&mut buf) {
            Ok(0)  => break,
            Ok(n)  => hasher.update(&buf[..n]),
            Err(e) => return Err(anyhow!(e)),
        }
    }

    let result = hasher.finalize();

    if result != expected {
        let msg = anyhow!(
            "Error CRC32: Expected: {:#10x}, Got: {:#10x}",
            expected,
            result,
        );

        return Err(msg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_err_bad_checksum() {
        let test_data = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/test.txt",
        );

        let expected = 0x00000000;
        let result   = check(&test_data, expected);

        assert!(result.is_err());
    }

    #[test]
    fn test_check_err_non_existant_file() {
        let test_data = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/doesntexist.txt",
        );

        let expected = 0x00000000;
        let result   = check(&test_data, expected);

        assert_eq!(
            result.unwrap_err().to_string(),
            "No such file or directory (os error 2)",
        );
    }

    #[test]
    fn test_check_ok() {
        let test_data = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/test.txt",
        );

        let expected = 0x891bc0e8;
        let result   = check(&test_data, expected);

        assert!(result.is_ok());
    }
}
