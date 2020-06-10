// install: Handle installation of product.
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use std::fs::{
    remove_file,
    File,
    OpenOptions,
};
use std::io;
use std::path::{
    Path,
    PathBuf,
};
use zip::{
    read::ZipFile,
    ZipArchive,
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

pub fn cleanup(filename: &str) -> Result<()> {
    remove_file(filename)?;

    Ok(())
}

fn extract_file(mut zipfile: &mut ZipFile, dest: &PathBuf) -> Result<()> {
    let name = zipfile.name();
    let dest = dest.join(name);

    let mut options = OpenOptions::new();

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

pub fn install(zipfile: &str, dest: &PathBuf) -> Result<()> {
    let path = Path::new(zipfile);
    let file = File::open(&path).expect("open zipfile");

    let mut zip    = ZipArchive::new(file).expect("new ziparchive");

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        extract_file(&mut file, &dest)?;
    }

    Ok(())
}
