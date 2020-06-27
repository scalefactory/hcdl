// Messages output by other parts of the program
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Error;
use std::path::PathBuf;

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
            println!("{}", msg);
        }
    }

    fn stderr(&self, msg: &str) {
        eprintln!("{}", msg);
    }

    pub fn checksum_bad(&self, filename: &str) {
        let msg = format!("SHA256 of {filename} did not match.", filename=filename);
        self.stderr(&msg);
    }

    pub fn checksum_ok(&self, filename: &str) {
        let msg = format!("SHA256 of {filename} OK.", filename=filename);
        self.stdout(&msg);
    }

    pub fn downloading(&self, filename: &str) {
        let msg = format!("Downloading {filename}...", filename=filename);
        self.stdout(&msg);
    }

    pub fn find_build_failed(&self, os: &str, arch: &str) {
        let msg = format!("Could not find build for {os}-{arch}", os=os, arch=arch);
        self.stderr(&msg);
    }

    pub fn installation_failed(&self, error: &Error) {
        let msg = format!("Installation failed with error: {error}", error=error);
        self.stderr(&msg);
    }

    pub fn installation_successful(&self) {
        self.stdout("Installation successful.");
    }

    pub fn keep_zipfile(&self, filename: &str) {
        let msg = format!(
            "Keeping zipfile {filename} in current directory.",
            filename=filename,
        );

        self.stdout(&msg);
    }

    pub fn latest_version(&self, latest: &str) {
        let msg = format!("Latest version: {latest}", latest=latest);
        self.stdout(&msg);
    }

    pub fn list_products(&self, products: &[&str]) {
        let msg = format!("Products: {products}", products=products.join(", "));
        self.stdout(&msg);
    }

    pub fn os_mismatch(&self, os: &str, requested: &str) {
        let msg = format!(
            "Product downloaded for different OS, {os} != {requested}",
            os=os,
            requested=requested,
        );

        self.stdout(&msg);
    }

    pub fn product_install(&self, product: &str, zipfile: &str, dest: &PathBuf) {
        let msg = format!(
            "Unzipping '{product}' from '{zipfile}' to '{dest}'",
            product=product,
            zipfile=zipfile,
            dest=dest.display(),
        );

        self.stdout(&msg);
    }

    pub fn signature_verification_failed(&self, error: &Error) {
        let msg = format!("Verification failed, error: {error}", error=error);
        self.stderr(&msg);
    }

    pub fn signature_verification_success(&self, signature: &str) {
        let msg = format!("Verified against {signature}.", signature=signature);
        self.stdout(&msg);
    }

    pub fn skipped_install(&self, filename: &str) {
        let msg = format!(
            "Skipping install and keeping zipfile '{filename}' in current directory.",
            filename=filename,
        );

        self.stdout(&msg);
    }

    pub fn verifying_signature(&self, shasums: &str) {
        let msg = format!(
            "Downloading and verifying signature of {shasums}...",
            shasums=shasums,
        );

        self.stdout(&msg);
    }
}
