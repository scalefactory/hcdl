// Messages output by other parts of the program
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Error;
use std::path::Path;

pub struct Messages {
    quiet: bool,
}

impl Messages {
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

    fn stderr(&self, msg: &str) {
        eprintln!("{msg}");
    }

    pub fn checksum_bad(&self, filename: &str) {
        let msg = format!("SHA256 of {filename} did not match.");

        self.stderr(&msg);
    }

    pub fn checksum_ok(&self, filename: &str) {
        let msg = format!("SHA256 of {filename} OK.");

        self.stdout(&msg);
    }

    pub fn downloading(&self, filename: &str) {
        let msg = format!("Downloading {filename}...");

        self.stdout(&msg);
    }

    pub fn download_only(&self, filename: &str) {
        let msg = format!("Download only mode, keeping {filename}.");

        self.stdout(&msg);
    }

    pub fn extracting_file(&self, filename: &Path, dest: &Path) {
        let msg = format!(
            "-> Extracting '{filename}' to '{dest}'...",
            filename = filename.display(),
            dest = dest.display(),
        );

        self.stdout(&msg);
    }

    pub fn find_build_failed(&self, os: &str, arch: &str) {
        let msg = format!("Could not find build for {os}-{arch}");

        self.stderr(&msg);
    }

    pub fn installation_failed(&self, error: &Error) {
        let msg = format!("Installation failed with error: {error}");

        self.stderr(&msg);
    }

    pub fn installation_successful(&self) {
        self.stdout("Installation successful.");
    }

    pub fn keep_zipfile(&self, filename: &str) {
        let msg = format!("Keeping zipfile {filename} in current directory.");

        self.stdout(&msg);
    }

    pub fn latest_version(&self, latest: &str) {
        let msg = format!("Latest version: {latest}");

        self.stdout(&msg);
    }

    pub fn list_products(&self, products: &[&str]) {
        let msg = format!(
            "Products: {products}",
            products = products.join(", "),
        );

        self.stdout(&msg);
    }

    pub fn os_mismatch(&self, os: &str, requested: &str) {
        let msg = format!(
            "Product downloaded for different OS, {os} != {requested}",
        );

        self.stdout(&msg);
    }

    pub fn signature_verification_failed(&self, error: &Error) {
        let msg = format!("Verification failed, error: {error}");

        self.stderr(&msg);
    }

    pub fn signature_verification_success(&self, signature: &str) {
        let msg = format!("Verified against {signature}.");

        self.stdout(&msg);
    }

    pub fn skipped_install(&self, filename: &str) {
        let msg = format!(
            "Skipping install and keeping zipfile '{filename}' in current \
             directory.",
        );

        self.stdout(&msg);
    }

    pub fn unzipping(&self, zipfile: &str, dest: &Path) {
        let msg = format!(
            "Unzipping contents of '{zipfile}' to '{dest}'",
            dest = dest.display(),
        );

        self.stdout(&msg);
    }

    pub fn verifying_signature(&self, shasums: &str) {
        let msg = format!(
            "Downloading and verifying signature of {shasums}...",
        );

        self.stdout(&msg);
    }
}
