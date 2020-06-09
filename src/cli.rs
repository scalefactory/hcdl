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

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        // Flags
        .arg(
            Arg::with_name("INSTALL")
                .env("HCDL_INSTALL")
                .hide_env_values(true)
                .long("install")
                .short("i")
                .help("Unzip and install the downloaded binary")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("VERIFY_SHASUM")
                .env("HCDL_VERIFY_SHASUM")
                .hide_env_values(true)
                .long("verify-shasum")
                .short("s")
                .help("Verify the SHASUM of the downloaded zip file")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("VERIFY_SIGNATURE")
                .env("HCDL_VERIFY_SIGNATURE")
                .hide_env_values(true)
                .long("verify-signature")
                .short("S")
                .help("Verify GPG signature of SHASUMS file")
                .takes_value(false)
        )
        // Options
        .arg(
            Arg::with_name("ARCH")
                .env("HCDL_ARCH")
                .hide_env_values(true)
                .long("arch")
                .short("a")
                .help("Specify product architecture to download")
                .default_value(DEFAULT_ARCH)
                .possible_values(VALID_ARCH)
        )
        .arg(
            Arg::with_name("OS")
                .env("HCDL_OS")
                .hide_env_values(true)
                .long("os")
                .short("o")
                .help("Specify product OS family to download")
                .default_value(DEFAULT_OS)
                .possible_values(VALID_OS)
        )
        // Positional
        .arg(
            Arg::with_name("PRODUCT")
                .env("HCDL_PRODUCT")
                .hide_env_values(true)
                .help("Name of the Hashicorp product to download")
                .index(1)
                .required(true)
                .takes_value(true)
        )
}

pub fn parse_args<'a>() -> ArgMatches<'a> {
    create_app().get_matches()
}
