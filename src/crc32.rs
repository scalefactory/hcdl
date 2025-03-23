// crc32: Handle checking CRC32 of a given file against the expected CRC32.
#![forbid(unsafe_code)]
use crate::error::Crc32Error;
use crc32fast::Hasher;
use std::fs::File;
use std::io::{
    prelude::*,
    BufReader,
};
use std::path::Path;

// Buffer size, 16KiB
const BUFFER_SIZE: usize = 16 * 1_024;

/// Check the given `path`'s CRC32 against the `expected` CRC32.
///
/// # Errors
///
/// Errors if:
///   - Failing to open the given `path`
///   - Failing to read from the given `path`
///   - If the CRC32 of the given `path` doesn't match the `expected` value
///
/// # Examples
///
/// Check that the CRC32 of the crate's test-data file is correct.
///
/// ```
/// use hcdl::crc32;
///
/// let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/test.txt");
/// let result = crc32::check(path, 0x891bc0e8);
///
/// assert!(result.is_ok());
/// ```
///
/// The CRC32 was not what was expected.
///
/// ```
/// use hcdl::{
///     crc32,
///     error::Crc32Error,
/// };
///
/// let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/test.txt");
/// let result = crc32::check(path, 0x12345678);
///
/// assert!(matches!(result.unwrap_err(), Crc32Error::UnexpectedCrc32(_, _)));
/// ```
pub fn check<P>(path: P, expected: u32) -> Result<(), Crc32Error>
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
        match reader.read(&mut buf)? {
            0 => break,
            n => hasher.update(&buf[..n]),
        }
    }

    let result = hasher.finalize();

    if result != expected {
        return Err(Crc32Error::UnexpectedCrc32(result, expected));
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

        assert_eq!(
            result.unwrap_err().to_string(),
            "unexpected crc32: 0x891bc0e8, wanted: 0x00000000",
        );
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
