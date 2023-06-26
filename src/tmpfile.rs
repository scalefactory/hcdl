// Handles a tmpfile for downloading
use super::error::TmpFileError;
use std::fs::OpenOptions;
use std::io::{
    self,
    BufWriter,
    Seek,
    SeekFrom,
};
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

/// Wrapper for a [`tempfile::NamedTempFile`].
pub struct TmpFile {
    tmpfile:  NamedTempFile,
    filename: String,
}

impl TmpFile {
    /// Make a new [`TmpFile`] for filename.
    ///
    /// # Errors
    ///
    /// Can error if unable to create a [`NamedTempFile`].
    pub fn new(filename: &str) -> Result<Self, TmpFileError> {
        let tmp = Self {
            filename: filename.to_owned(),
            tmpfile:  NamedTempFile::new()?,
        };

        Ok(tmp)
    }

    /// Return the tmpfile filename
    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Return a [`NamedTempFile`] handle that has been rewound to 0.
    ///
    /// # Errors
    ///
    /// Can error if seeking to the beginning of the `tmpfile` fails.
    pub fn handle(&mut self) -> Result<&mut NamedTempFile, TmpFileError> {
        self.tmpfile.seek(SeekFrom::Start(0))?;

        Ok(&mut self.tmpfile)
    }

    /// Persist the file into our current directory as self.filename
    ///
    /// # Errors
    ///
    /// Can error under various common IO issues such as:
    ///   - Failure to open file for writing
    ///   - Attempting to get the file handle for the `tmpfile`
    ///   - Issues while writing to the `tmpfile`
    pub fn persist(&mut self) -> Result<(), TmpFileError> {
        let dest        = Path::new(&self.filename);
        let mut options = OpenOptions::new();

        // Keep file around with -rw-r--r-- permissions.
        #[cfg(target_family = "unix")]
        options.mode(0o644);

        let writer = options
            .create(true)
            .write(true)
            .truncate(true)
            .open(dest)?;

        let mut writer = BufWriter::new(writer);
        let mut handle = self.handle()?;

        io::copy(&mut handle, &mut writer)?;

        Ok(())
    }
}
