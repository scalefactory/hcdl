// Messages output by other parts of the program
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Error;
use std::path::PathBuf;

pub fn checksum_bad(filename: &str) {
    println!("SHA256 of {filename} did not match.", filename=filename);
}

pub fn checksum_ok(filename: &str) {
    println!("SHA256 of {filename} OK.", filename=filename);
}

pub fn downloading(filename: &str) {
    println!("Downloading {filename}...", filename=filename);
}

pub fn find_build_failed(os: &str, arch: &str) {
    eprintln!("Could not find build for {os}-{arch}", os=os, arch=arch);
}

pub fn installation_failed(error: &Error) {
    eprintln!("Installation failed with error: {error}", error=error);
}

pub fn installation_successful() {
    println!("Installation successful.");
}

pub fn keep_zipfile(filename: &str) {
    println!(
        "Keeping zipfile {filename} in current directory.",
        filename=filename,
    );
}

pub fn latest_version(latest: &str) {
    println!("Latest version: {latest}", latest=latest);
}

pub fn list_products(products: &[&str]) {
    println!("Products: {products}", products=products.join(", "));
}

pub fn os_mismatch(os: &str, requested: &str) {
    println!(
        "Product downloaded for different OS, {os} != {requested}",
        os=os,
        requested=requested,
    );
}

pub fn product_install(product: &str, zipfile: &str, dest: &PathBuf) {
    println!(
        "Unzipping '{product}' from '{zipfile}' to '{dest}'",
        product=product,
        zipfile=zipfile,
        dest=dest.display(),
    );
}

pub fn signature_verification_failed(error: &Error) {
    eprintln!("Verification failed, error: {error}", error=error);
}

pub fn signature_verification_success(signature: &str) {
    println!("Verified against {signature}.", signature=signature);
}

pub fn skipped_install(filename: &str) {
    println!(
        "Skipping install and keeping zipfile '{filename}' in current directory.",
        filename=filename,
    );
}

pub fn verifying_signature(shasums: &str) {
    println!(
        "Downloading and verifying signature of {shasums}...",
        shasums=shasums,
    );
}
