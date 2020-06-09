// install: Handle installation of product.
use anyhow::Result;
use std::fs::{
    File,
    OpenOptions,
};
use std::io;
use std::path::{
    Path,
    PathBuf,
};
use zip::ZipArchive;

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

pub fn install(zipfile: &str, filename: &str, dest: &PathBuf) -> Result<()> {
    let path = Path::new(zipfile);
    let file = File::open(&path).expect("open zipfile");

    let mut zip    = ZipArchive::new(file).expect("new ziparchive");
    let mut wanted = zip.by_name(filename).expect("find zip contents");

    let dest = dest.join(filename);

    let mut options = OpenOptions::new();

    #[cfg(target_family = "unix")]
    options.mode(0o755);

    let mut writer = options
        .create(true)
        .write(true)
        .truncate(true)
        .open(&dest)?;

    io::copy(&mut wanted, &mut writer)?;

    Ok(())
}
