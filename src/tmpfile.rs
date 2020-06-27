// Handles a tmpfile for downloading
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::{
    self,
    Seek,
    SeekFrom,
};
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

pub struct TmpFile {
    tmpfile:      NamedTempFile,
    pub filename: String,
}

impl TmpFile {
    // Make a new TmpFile for filename
    pub fn new(filename: &str) -> Result<Self> {
        let tmp = Self {
            filename: filename.to_owned(),
            tmpfile:  NamedTempFile::new()?,
        };

        Ok(tmp)
    }

    // Return a handle that has been rewound to 0
    pub fn handle(&mut self) -> Result<&mut NamedTempFile> {
        self.tmpfile.seek(SeekFrom::Start(0))?;

        Ok(&mut self.tmpfile)
    }

    // Persist the file into our current directory as self.filename
    pub fn persist(&mut self) -> Result<()> {
        let dest        = Path::new(&self.filename);
        let mut options = OpenOptions::new();

        // Keep file around with -rw-r--r-- permissions.
        #[cfg(target_family = "unix")]
        options.mode(0o644);

        let mut writer = options
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dest)?;

        let mut handle = self.handle()?;

        io::copy(&mut handle, &mut writer)?;

        Ok(())
    }
}
