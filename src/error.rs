// Library errors.

use std::path::PathBuf;
use thiserror::Error;

/// Errors encountered in the [`crc32`] module.
#[derive(Debug, Error)]
pub enum Crc32Error {
    /// Returned if there's an IO error while calculating the CRC32 for the
    /// extracted file.
    #[error("io error")]
    IoError(#[from] std::io::Error),

    /// Returned if there's a problem with the calculated CRC32 for the
    /// extracted file.
    #[error("unexpected crc32. expected: {0:#010x}, got: {1:#010x}")]
    UnexpectedCrc32(u32, u32),
}

/// Errors encountered in the [`install`] module.
#[derive(Debug, Error)]
pub enum InstallError {
    /// Returned if there's an error while calculating the CRC32 for the
    /// installed file.
    #[error("crc32 error")]
    Crc32(#[from] Crc32Error),

    // Could not find suitable install-dir. Consider passing --install-dir to
    // manually specify.
    /// Returned if there's no executable directory.
    #[error("no executable dir")]
    NoExecutableDir,

    /// Returned if the installation path is not a directory.
    #[error("install: destination '{0}' is not a directory")]
    NoInstallDir(PathBuf),

    /// Returned if there's an error while persisting the tempfile to it's
    /// proper destination.
    #[error("error persisting file")]
    PathPersist(#[from] tempfile::PathPersistError),

    /// Returned if there's an error while setting the installed file's
    /// permissions.
    #[error("set permissions error")]
    SetPermissions(#[from] std::io::Error),

    /// Returned if there's an error while getting the zip file basename.
    #[error("couldn't get zip file basename from '{0}'")]
    ZipFileBasename(String),

    /// Returned if there's an error while getting the zip file index.
    #[error("zip index error")]
    ZipIndex(#[from] zip::result::ZipError),
}
