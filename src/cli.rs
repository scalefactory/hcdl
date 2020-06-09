// cli: Handle command line parsing
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use clap::{
    crate_authors,
    crate_description,
    crate_name,
    crate_version,
    App,
    Arg,
    ArgMatches,
};
use std::path::Path;
use super::products::PRODUCTS_LIST;

#[cfg(target_arch = "arm")]
const DEFAULT_ARCH: &'static str = "arm";

#[cfg(target_arch = "x86")]
const DEFAULT_ARCH: &'static str = "386";

#[cfg(target_arch = "x86_64")]
const DEFAULT_ARCH: &'static str = "amd64";

const VALID_ARCH: &[&str] = &[
    "386",
    "amd64",
    "arm",
];

#[cfg(target_os = "freebsd")]
const DEFAULT_OS: &'static str = "freebsd";

#[cfg(target_os = "linux")]
const DEFAULT_OS: &'static str = "linux";

#[cfg(target_os = "mac_os")]
const DEFAULT_OS: &'static str = "darwin";

#[cfg(target_os = "openbsd")]
const DEFAULT_OS: &'static str = "openbsd";

#[cfg(target_os = "solaris")]
const DEFAULT_OS: &'static str = "solaris";

#[cfg(target_os = "windows")]
const DEFAULT_OS: &'static str = "windows";

const VALID_OS: &[&str] = &[
    "darwin",
    "freebsd",
    "linux",
    "openbsd",
    "solaris",
    "windows",
];

fn is_valid_install_dir(s: String) -> Result<(), String> {
    let path = Path::new(&s);

    if !path.exists() {
        return Err("install-dir does not exist".into());
    }

    if !path.is_dir() {
        return Err("install-dir is not a directory".into());
    }

    Ok(())
}

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        // Flags
        .arg(
            Arg::with_name("CLEANUP")
                .long("cleanup")
                .short("c")
                .help("Clean up downloaded zip file after install")
                .takes_value(false)
                .requires("INSTALL")
        )
        .arg(
            Arg::with_name("INSTALL")
                .long("install")
                .short("i")
                .help("Unzip and install the downloaded binary")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("LIST_PRODUCTS")
                .long("list-products")
                .short("l")
                .help("List all available HashiCorp products")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("NO_VERIFY_SIGNATURE")
                .long("no-verify-signature")
                .help("Disable GPG signature verification")
                .takes_value(false)
        )
        // Options
        .arg(
            Arg::with_name("ARCH")
                .long("arch")
                .short("a")
                .help("Specify product architecture to download")
                .default_value(DEFAULT_ARCH)
                .possible_values(VALID_ARCH)
        )
        .arg(
            Arg::with_name("INSTALL_DIR")
                .long("install-dir")
                .short("d")
                .help("Specify directory to install product to")
                .takes_value(true)
                .value_name("DIR")
                .requires("INSTALL")
                .validator(is_valid_install_dir)
        )
        .arg(
            Arg::with_name("OS")
                .long("os")
                .short("o")
                .help("Specify product OS family to download")
                .default_value(DEFAULT_OS)
                .possible_values(VALID_OS)
        )
        // Positional
        .arg(
            Arg::with_name("PRODUCT")
                .help("Name of the Hashicorp product to download")
                .index(1)
                .required_unless("LIST_PRODUCTS")
                .takes_value(true)
                .possible_values(PRODUCTS_LIST)
        )
}

pub fn parse_args<'a>() -> ArgMatches<'a> {
    create_app().get_matches()
}
