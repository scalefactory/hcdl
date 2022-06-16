// cli: Handle command line parsing
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use super::products::PRODUCTS_LIST;
use clap::{
    crate_description,
    crate_name,
    crate_version,
    Arg,
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

fn create_app<'a>() -> Command<'a> {
    let mut app = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        // Flags
        .arg(
            Arg::new("CHECK")
                .long("check")
                .help("Check for the latest version and exit without downloading.")
                .takes_value(false)
                .conflicts_with_all(&[
                    "BUILD",
                    "QUIET",
                ])
        )
        .arg(
            Arg::new("DOWNLOAD_ONLY")
                .long("download-only")
                .short('D')
                .help("Only download the product, do not install it. Implies --keep.")
                .takes_value(false)
        )
        .arg(
            Arg::new("KEEP")
                .long("keep")
                .short('k')
                .help("Keep downloaded zipfile after install.")
                .takes_value(false)
        )
        .arg(
            Arg::new("LIST_PRODUCTS")
                .long("list-products")
                .short('l')
                .help("List all available HashiCorp products.")
                .takes_value(false)
        )
        .arg(
            Arg::new("NO_VERIFY_SIGNATURE")
                .long("no-verify-signature")
                .help("Disable GPG signature verification.")
                .takes_value(false)
        )
        .arg(
            Arg::new("QUIET")
                .long("quiet")
                .short('q')
                .help("Silence all non-error output")
                .takes_value(false)
        )
        // Options
        .arg(
            Arg::new("ARCH")
                .long("arch")
                .short('a')
                .help("Specify product architecture to download.")
                .default_value(DEFAULT_ARCH)
                .value_parser(PossibleValuesParser::new(VALID_ARCH))
        )
        .arg(
            Arg::new("BUILD")
                .long("build")
                .short('b')
                .help("Specify product build version to download.")
                .default_value(DEFAULT_VERSION)
                .value_name("VERSION")
        )
        .arg(
            Arg::new("INSTALL_DIR")
                .long("install-dir")
                .short('d')
                .help("Specify directory to install product to.")
                .takes_value(true)
                .value_name("DIR")
                .value_parser(is_valid_install_dir)
        )
        .arg(
            Arg::new("OS")
                .long("os")
                .short('o')
                .help("Specify product OS family to download.")
                .default_value(DEFAULT_OS)
                .value_parser(PossibleValuesParser::new(VALID_OS))
        )
        // Positional
        .arg(
            Arg::new("PRODUCT")
                .help("Name of the Hashicorp product to download.")
                .index(1)
                .takes_value(true)
                .value_parser(PossibleValuesParser::new(PRODUCTS_LIST))
                .required_unless_present_any(&[
                    "COMPLETIONS",
                    "LIST_PRODUCTS",
                ])
        );

    if no_color() {
        app = app.color(ColorChoice::Never);
    }

    #[cfg(feature = "shell_completion")]
    {
        app = app.arg(
            Arg::new("COMPLETIONS")
                .long("completions")
                .help("Generate shell completions for the given shell")
                .takes_value(true)
                .value_name("SHELL")
                .value_parser(EnumValueParser::<Shell>::new())
        );
    }


    app
}

pub fn parse_args() -> ArgMatches {
    create_app().get_matches()
}

#[cfg(feature = "shell_completion")]
pub fn gen_completions(shell: &Shell) {
    let mut app = create_app();

    generate(shell.to_owned(), &mut app, crate_name!(), &mut io::stdout());
}
