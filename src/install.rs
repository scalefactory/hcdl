// install: Handle installation of product.
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::crc32;
use super::error::InstallError;
use std::fs;
use std::io::{
    self,
    Read,
    Seek,
};
use std::path::{
    Path,
    PathBuf,
};
use tempfile::{
    NamedTempFile,
    TempPath,
};
use zip::{
    read::ZipFile,
    ZipArchive,
};

#[cfg(target_family = "unix")]
use std::fs::Permissions;

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

/// `bin_dir` finds a suitable executable directory to install a product to.
///
/// # Errors
///
/// Errors if:
///   - Failing to find a suitable executable directory
///   - Failing to create the executable directory if needed
pub fn bin_dir() -> Result<PathBuf, InstallError> {
    if let Some(dir) = dirs::executable_dir() {
        // Attempt to create the directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        Ok(dir)
    }
    else {
        // If we get None, we're likely on Windows.
        Err(InstallError::NoExecutableDir)
    }
}

/// Extracts a given `zipfile` to a temporary file under `dir`. Also checks
/// the CRC32 of the extracted file to make sure extraction was successful.
/// Returns a [`tempfile::TempPath`] which the caller is responsible for
/// persisting.
fn extract(mut zipfile: &mut ZipFile, dir: &Path) -> Result<TempPath, InstallError> {
    // Get a tempfile to extract to under the dest path
    let mut tmpfile = NamedTempFile::new_in(dir)?;

    // Extract our file
    io::copy(&mut zipfile, &mut tmpfile)?;

    // Closes the file, keeping only the path.
    let tmpfile = tmpfile.into_temp_path();

    // Get the file's expected CRC32 and check against what we wrote.
    let expected = zipfile.crc32();
    crc32::check(&tmpfile, expected)?;

    Ok(tmpfile)
}

/// Installs files from the given `zipfile` under the directory at `dir`.
///
/// # Errors
///
/// Can error if:
///   - Installation directory doesn't exist
///   - Failing to get a file index from the `zipfile`
///   - Failing to extract files from the `zipfile`
///   - Failing to persist the extracted file
///   - Failing to set file permissions on the extracted file
//
// type_complexity is allowed here, since attempting to make the suggested
// type alias results in other complains with no good compiler suggestions.
#[allow(clippy::type_complexity)]
pub fn install<F>(
    zipfile: &mut F,
    dir: &Path,
) -> Result<Vec<PathBuf>, InstallError>
where
    F: Read + Seek,
{
    if !dir.is_dir() {
        return Err(InstallError::NoInstallDir(dir.to_path_buf()));
    }

    let mut extracted_files = Vec::new();
    let mut zip = ZipArchive::new(zipfile)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        // Perform some sanitization on the filename.
        // We assume here that HashiCorp zips only ever have files at the root
        // of the zip file, this allows us to not care about their paths.
        let filename = file.name();

        // Attempt to get the basename of the filename
        let basename = Path::new(filename)
            .file_name()
            .ok_or_else(|| {
                InstallError::ZipFileBasename(filename.to_string())
            })?;

        // Finally get a pathbuf of the basename
        let filename = Path::new(basename).to_path_buf();

        // Extract the file
        let tmpfile = extract(&mut file, dir)?;

        // Persist the tmpfile to the real dest.
        let dest = dir.join(&filename);
        tmpfile.persist(&dest)?;

        // Set the permissions on the installed file.
        #[cfg(target_family = "unix")]
        if let Some(mode) = file.unix_mode() {
            fs::set_permissions(&dest, Permissions::from_mode(mode))?;
        }

        extracted_files.push(filename);
    }

    Ok(extracted_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn test_install_dir_not_dir() {
        let test_file = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-data/test.txt",
        );

        // This should really be mocked, but for now we have a real file we
        // can open from the test-data.
        let mut file   = File::open(&test_file).unwrap();
        let dest       = Path::new(test_file).to_path_buf();

        let res = install(&mut file, &dest);

        assert!(res.is_err());
    }
}
