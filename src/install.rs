// install: Handle installation of product.
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::{
    anyhow,
    Result,
};
use std::fs::{
    self,
    OpenOptions,
};
use std::io::{
    self,
    Read,
    Seek,
};
use std::path::PathBuf;
use zip::{
    read::ZipFile,
    ZipArchive,
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

// Find a suitable bindir for installing to.
pub fn bin_dir() -> Result<PathBuf> {
    let dir = match dirs::executable_dir() {
        Some(dir) => {
            // Attempt to create the directory if it doesn't exist
            if !dir.exists() {
                fs::create_dir_all(&dir)?;
            }

            dir
        },
        None => {
            // If we get None, we're likely on Windows.
            let msg = concat!(
                "Could not find suitable install-dir.\n",
                "Consider passing --install-dir to manually specify",
            );

            return Err(anyhow!(msg))
        },
    };

    Ok(dir)
}

fn extract_file(mut zipfile: &mut ZipFile, dest: &PathBuf) -> Result<()> {
    let name        = zipfile.name();
    let dest        = dest.join(name);
    let mut options = OpenOptions::new();

    // On Unix type OSs we set the written file as executable.
    #[cfg(target_family = "unix")]
    options.mode(0o755);

    let mut writer = options
        .create(true)
        .write(true)
        .truncate(true)
        .open(&dest)?;

    io::copy(&mut zipfile, &mut writer)?;

    Ok(())
}

pub fn install<F>(zipfile: &mut F, dest: &PathBuf) -> Result<()>
where F: Read + Seek {
    let mut zip = ZipArchive::new(zipfile).expect("new ziparchive");

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        extract_file(&mut file, &dest)?;
    }

    Ok(())
}
