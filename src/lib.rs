//! hcdl: Easily update Hashicorp tools
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::redundant_field_names)]

/// Client for downloading products.
pub mod client;

/// Handle checking the CRC32 of files extracted from zipfiles.
pub mod crc32;

/// Handle extracting and installing downloaded product.
pub mod install;

/// List of [HashiCorp](https://www.hashicorp.com/) products.
pub mod products;

/// Handle drawing progress bars during download and install.
pub mod progressbar;

/// Handle file checksums.
pub mod shasums;

/// Handles for checking file signatures.
pub mod signature;

/// Wrapper for handling a tempfile.
pub mod tmpfile;
