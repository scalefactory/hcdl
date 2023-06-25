// Messages output by other parts of the program
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Error;
use std::path::Path;

/// Handler for the various message we need to output.
pub struct Messages {
    quiet: bool,
}

impl Messages {
    /// Crates a new [`Messages`] handler. If `quiet` is set to `true`, no
    /// output will be given.
    #[must_use]
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet: quiet,
        }
    }

    fn stdout(&self, msg: &str) {
        if !self.quiet {
            println!("{msg}");
        }
    }

    #[allow(clippy::unused_self)]
    fn stderr(&self, msg: &str) {
        eprintln!("{msg}");
    }

    /// Output when the checksum of the file is bad.
    pub fn checksum_bad(&self, filename: &str) {
        let msg = format!("SHA256 of {filename} did not match.");

        self.stderr(&msg);
    }

    /// Output when the checksum of the file is good.
    pub fn checksum_ok(&self, filename: &str) {
        let msg = format!("SHA256 of {filename} OK.");

        self.stdout(&msg);
    }

    /// Output when the download of a file is starting.
    pub fn downloading(&self, filename: &str) {
        let msg = format!("Downloading {filename}...");

        self.stdout(&msg);
    }

    /// Output when download only mode is used to indicate the downloaded file
    /// will not be deleted.
    pub fn download_only(&self, filename: &str) {
        let msg = format!("Download only mode, keeping {filename}.");

        self.stdout(&msg);
    }

    /// Output when a file is being extracted.
    pub fn extracting_file(&self, filename: &Path, dest: &Path) {
        let msg = format!(
            "-> Extracting '{filename}' to '{dest}'...",
            filename = filename.display(),
            dest = dest.display(),
        );

        self.stdout(&msg);
    }

    /// Output when we can't find a product build for the specified OS and
    /// architecture.
    pub fn find_build_failed(&self, os: &str, arch: &str) {
        let msg = format!("Could not find build for {os}-{arch}");

        self.stderr(&msg);
    }

    /// Output when a product installation has failed.
    pub fn installation_failed(&self, error: &Error) {
        let msg = format!("Installation failed with error: {error}");

        self.stderr(&msg);
    }

    /// Output when a product installation was successful.
    pub fn installation_successful(&self) {
        self.stdout("Installation successful.");
    }

    /// Output when a zipfile has been kept instead of being deleted.
    pub fn keep_zipfile(&self, filename: &str) {
        let msg = format!("Keeping zipfile {filename} in current directory.");

        self.stdout(&msg);
    }

    /// Output when checking for the latest product version.
    pub fn latest_version(&self, latest: &str) {
        let msg = format!("Latest version: {latest}");

        self.stdout(&msg);
    }

    /// Output when the product list was requested.
    pub fn list_products(&self, products: &[&str]) {
        let msg = format!(
            "Products: {products}",
            products = products.join(", "),
        );

        self.stdout(&msg);
    }

    /// Output when an installation is attempted for a product OS that doesn't
    /// match the current OS.
    pub fn os_mismatch(&self, os: &str, requested: &str) {
        let msg = format!(
            "Product downloaded for different OS, {os} != {requested}",
        );

        self.stdout(&msg);
    }

    /// Output when signature verification has failed.
    pub fn signature_verification_failed(&self, error: &Error) {
        let msg = format!("Verification failed, error: {error}");

        self.stderr(&msg);
    }

    /// Output when signature verification is successful.
    pub fn signature_verification_success(&self, signature: &str) {
        let msg = format!("Verified against {signature}.");

        self.stdout(&msg);
    }

    /// Output when installation of the product is skipped.
    pub fn skipped_install(&self, filename: &str) {
        let msg = format!(
            "Skipping install and keeping zipfile '{filename}' in current \
             directory.",
        );

        self.stdout(&msg);
    }

    /// Output when content is being unzipped.
    pub fn unzipping(&self, zipfile: &str, dest: &Path) {
        let msg = format!(
            "Unzipping contents of '{zipfile}' to '{dest}'",
            dest = dest.display(),
        );

        self.stdout(&msg);
    }

    /// Output when a signature is being verified.
    pub fn verifying_signature(&self, shasums: &str) {
        let msg = format!(
            "Downloading and verifying signature of {shasums}...",
        );

        self.stdout(&msg);
    }
}
