// Library errors.

use std::path::PathBuf;
use thiserror::Error;

/// Errors encountered in the [`client`] module.
#[derive(Debug, Error)]
pub enum ClientError {
    /// Returned when encountering an error building the [`Client`].
    #[error("couldn't build http client")]
    ClientBuilder,

    /// Returned if there's an error downloading a chunk of content.
    #[error("couldn't download chunk of content")]
    Chunk,

    /// Returned when there's an error getting a [`Url`].
    #[error("couldn't get url '{0}'")]
    Get(url::Url),

    /// Returned if there's an error getting the [`Bytes`] from a GET request.
    #[error("couldn't get bytes from get request")]
    GetBytes,

    /// Returned if there's an error getting a [`String`] from a GET request.
    #[error("couldn't get text from get request")]
    GetText,

    /// Returned if there's an IO error while downloading content.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Returned if there's an error parsing the [`ProductVersion`].
    #[error("couldn't parse product version")]
    ProductVersion,

    /// Returned when there's an error getting a [`Signature`] for the
    /// [`ProductVersion`].
    #[error(transparent)]
    Signature(#[from] SignatureError),

    /// Returned if there's [`TmpFile`] error while downloading content.
    #[error(transparent)]
    TmpFile(#[from] TmpFileError),

    /// Returned if there's an error parsing a [`Url`].
    #[error("couldn't parse {0} url")]
    Url(&'static str),
}

/// Errors encountered in the [`crc32`] module.
#[derive(Debug, Error)]
pub enum Crc32Error {
    /// Returned if there's an IO error while calculating the CRC32 for the
    /// extracted file.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Returned if there's a problem with the calculated CRC32 for the
    /// extracted file.
    #[error("unexpected crc32: {0:#010x}, wanted: {1:#010x}")]
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

/// Errors encountered in the [`shasums`] module.
#[derive(Debug, Error)]
pub enum ShasumsError {
    /// Returned if there's an error while calculating the shasum for the file.
    #[error("io error while hashing file")]
    Hashing,

    /// Returned when the shasum for a file could not be found.
    #[error("couldn't find shasum for {0}")]
    NoShasumForFile(String),

    /// Returned if there's a [`TmpFileError`] while hashing the file.
    #[error(transparent)]
    TmpFile(#[from] TmpFileError),
}

/// Errors encountered in the [`signature`] module.
#[derive(Debug, Error)]
pub enum SignatureError {
    /// Returned if the GPG key path does not exist or is not a file.
    #[error("gpg key file '{0}' does not exist or is not a file")]
    GpgKey(PathBuf),

    /// Returned when there's an IO error dealing with signature data.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Returned if there's no XDG shared data directory returned.
    #[error("couldn't find shared data directory")]
    NoSharedDataDir,

    /// Returned when the XDG shared data path returned does not exist or is
    /// not a directory.
    #[error("data directory '{0}' does not exist or is not a directory")]
    NoSharedDataDirExists(PathBuf),

    /// Returned when the shasum signatures couldn't be verified.
    #[error(transparent)]
    Pgp(#[from] pgp::errors::Error),

    /// Returned when the signature couldn't be verified.
    #[error("couldn't verify signature")]
    Verification,
}

/// Errors encountered in the [`tmpfile`] module.
#[derive(Debug, Error)]
pub enum TmpFileError {
    /// Returned if IO errors are encountered.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
