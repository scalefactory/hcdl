// cli: Handle command line parsing
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::products::PRODUCTS_LIST;
use clap::{
    crate_description,
    crate_name,
    crate_version,
    Arg,
    ArgAction,
    ArgMatches,
    ColorChoice,
    Command,
};
use clap::builder::PossibleValuesParser;
use std::env;
use std::path::{
    Path,
    PathBuf,
};

#[cfg(feature = "shell_completion")]
use clap::builder::EnumValueParser;

#[cfg(feature = "shell_completion")]
use clap_complete::generate;

#[cfg(feature = "shell_completion")]
use clap_complete::Shell;

#[cfg(feature = "shell_completion")]
use std::io;

#[cfg(target_arch = "arm")]
pub const DEFAULT_ARCH: &str = "arm";

#[cfg(target_arch = "x86")]
pub const DEFAULT_ARCH: &str = "386";

#[cfg(target_arch = "x86_64")]
pub const DEFAULT_ARCH: &str = "amd64";

const VALID_ARCH: &[&str] = &[
    "386",
    "amd64",
    "arm",
];

#[cfg(target_os = "freebsd")]
pub const DEFAULT_OS: &str = "freebsd";

#[cfg(target_os = "linux")]
pub const DEFAULT_OS: &str = "linux";

#[cfg(target_os = "macos")]
pub const DEFAULT_OS: &str = "darwin";

#[cfg(target_os = "openbsd")]
pub const DEFAULT_OS: &str = "openbsd";

#[cfg(target_os = "solaris")]
pub const DEFAULT_OS: &str = "solaris";

#[cfg(target_os = "windows")]
pub const DEFAULT_OS: &str = "windows";

const VALID_OS: &[&str] = &[
    "darwin",
    "freebsd",
    "linux",
    "openbsd",
    "solaris",
    "windows",
];

const DEFAULT_VERSION: &str = "latest";
const NO_COLOR: &str = "NO_COLOR";

// Checks the environment to see if NO_COLOR is in use.
pub fn no_color() -> bool {
    env::var_os(NO_COLOR).is_some()
}

// Ensure that the installation dir exists and is a directory.
fn is_valid_install_dir(s: &str) -> Result<PathBuf, String> {
    let path = Path::new(&s);

    if !path.exists() {
        return Err("install-dir does not exist".into());
    }

    if !path.is_dir() {
        return Err("install-dir is not a directory".into());
    }

    Ok(path.to_path_buf())
}

fn create_app() -> Command {
    let app = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::new("ARCH")
                .action(ArgAction::Set)
                .default_value(DEFAULT_ARCH)
                .help("Specify product architecture to download.")
                .long("arch")
                .short('a')
                .value_parser(PossibleValuesParser::new(VALID_ARCH))
        )
        .arg(
            Arg::new("BUILD")
                .action(ArgAction::Set)
                .default_value(DEFAULT_VERSION)
                .help("Specify product build version to download.")
                .long("build")
                .short('b')
                .value_name("VERSION")
        )
        .arg(
            Arg::new("CHECK")
                .action(ArgAction::SetTrue)
                .help("Check for the latest version and exit without downloading.")
                .long("check")
                .conflicts_with_all([
                    "BUILD",
                    "QUIET",
                ])
        );

    #[cfg(feature = "shell_completion")]
    let app = app.arg(
            Arg::new("COMPLETIONS")
                .action(ArgAction::Set)
                .help("Generate shell completions for the given shell")
                .long("completions")
                .value_name("SHELL")
                .value_parser(EnumValueParser::<Shell>::new())
        );

    let app = app
        .arg(
            Arg::new("DOWNLOAD_ONLY")
                .action(ArgAction::SetTrue)
                .help("Only download the product, do not install it. Implies --keep.")
                .long("download-only")
                .short('D')
        )
        .arg(
            Arg::new("INSTALL_DIR")
                .action(ArgAction::Set)
                .help("Specify directory to install product to.")
                .long("install-dir")
                .short('d')
                .value_name("DIR")
                .value_parser(is_valid_install_dir)
        )
        .arg(
            Arg::new("KEEP")
                .action(ArgAction::SetTrue)
                .help("Keep downloaded zipfile after install.")
                .long("keep")
                .short('k')
        )
        .arg(
            Arg::new("LIST_PRODUCTS")
                .action(ArgAction::SetTrue)
                .help("List all available HashiCorp products.")
                .long("list-products")
                .short('l')
        )
        .arg(
            Arg::new("NO_VERIFY_SIGNATURE")
                .action(ArgAction::SetTrue)
                .help("Disable GPG signature verification.")
                .long("no-verify-signature")
        )
        .arg(
            Arg::new("OS")
                .action(ArgAction::Set)
                .default_value(DEFAULT_OS)
                .help("Specify product OS family to download.")
                .long("os")
                .short('o')
                .value_parser(PossibleValuesParser::new(VALID_OS))
        )
        .arg(
            Arg::new("QUIET")
                .action(ArgAction::SetTrue)
                .help("Silence all non-error output")
                .long("quiet")
                .short('q')
        )
        // Positional
        .arg(
            Arg::new("PRODUCT")
                .action(ArgAction::Set)
                .help("Name of the Hashicorp product to download.")
                .index(1)
                .value_parser(PossibleValuesParser::new(PRODUCTS_LIST))
                .required_unless_present_any([
                    "COMPLETIONS",
                    "LIST_PRODUCTS",
                ])
        );

    if no_color() {
        app.color(ColorChoice::Never)
    }
    else {
        app
    }
}

pub fn parse_args() -> ArgMatches {
    create_app().get_matches()
}

#[cfg(feature = "shell_completion")]
pub fn gen_completions(shell: &Shell) {
    let mut app = create_app();

    generate(shell.to_owned(), &mut app, crate_name!(), &mut io::stdout());
}
