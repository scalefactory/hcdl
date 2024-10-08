//! hcdl: Easily update Hashicorp tools
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use hcdl::{
    client,
    install,
    shasums,
};
use hcdl::tmpfile::TmpFile;
use std::path::PathBuf;
use std::process::exit;

mod cli;
mod messages;
mod products;

use messages::Messages;

#[cfg(feature = "shell_completion")]
use clap_complete::Shell;

const LATEST: &str = "latest";

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<()> {
    let matches  = cli::parse_args();

    #[cfg(feature = "shell_completion")]
    // Generate completions if requested
    {
        if matches.contains_id("COMPLETIONS") {
            // This was validated during CLI parse.
            let shell = matches.get_one::<Shell>("COMPLETIONS").unwrap();
            cli::gen_completions(*shell);

            exit(0);
        }
    }

    let is_quiet = matches.get_flag("QUIET");
    let no_color = cli::no_color();
    let messages = Messages::new(is_quiet);

    // We don't need to do very much if we're listing products
    if matches.get_flag("LIST_PRODUCTS") {
        messages.list_products(products::PRODUCTS_LIST);

        exit(0);
    };

    // Pull options from matches
    // Unwraps here should be fine as these are checked and have default
    // values.
    let build_version = matches.get_one::<String>("BUILD").unwrap();
    let product       = matches.get_one::<String>("PRODUCT").unwrap();

    let client_config = client::ClientConfig::new()
        .no_color(no_color)
        .quiet(is_quiet);

    let client = client::Client::new(client_config)?;

    let builds = if build_version.to_lowercase() == LATEST {
        let latest = client.check_version(product).await?;

        messages.latest_version(&latest.to_string());

        // Check only, no download.
        if matches.get_flag("CHECK") {
            exit(0);
        }

        client.get_version(product, &latest.version).await?
    }
    else {
        client.get_version(product, build_version).await?
    };

    let arch = matches.get_one::<String>("ARCH").unwrap();
    let os   = matches.get_one::<String>("OS").unwrap();

    let Some(build) = builds.build(arch, os) else {
        messages.find_build_failed(os, arch);

        exit(1);
    };

    // Download SHASUMS file
    let shasums = client.get_shasums(&builds).await?;

    // Verify the SHASUMS file against its signature
    let no_sig = matches.get_flag("NO_VERIFY_SIGNATURE");
    if !no_sig {
        let shasums_filename = builds.url_shasums
            .path_segments()
            .unwrap()
            .last()
            .unwrap();
        messages.verifying_signature(shasums_filename);

        // Download signature file
        let signature = client.get_signature(&builds).await?;

        match signature.check(&shasums) {
            Ok(()) => {
                let url = builds.shasums_signature_url();
                let signature_filename = url
                    .path_segments()
                    .unwrap()
                    .last()
                    .unwrap();

                messages.signature_verification_success(signature_filename);
            },
            Err(e) => {
                messages.signature_verification_failed(&e);

                exit(1);
            },
        };
    }

    // Download the product
    let download_url = &build.url;
    let filename     = download_url
        .path_segments()
        .unwrap()
        .last()
        .unwrap();

    // Get a new tmpfile for the download.
    let mut tmpfile = TmpFile::new(filename)?;

    messages.downloading(filename);
    client.download(download_url.clone(), &mut tmpfile).await?;

    // Ensure the SHASUM is correct
    match shasums.check(&mut tmpfile)? {
        shasums::Checksum::OK  => messages.checksum_ok(filename),
        shasums::Checksum::Bad => {
            messages.checksum_bad(filename);

            exit(1);
        },
    };

    // If we're DOWNLOAD_ONLY (implies KEEP), just persist the file and
    // we're done.
    if matches.get_flag("DOWNLOAD_ONLY") {
        messages.download_only(filename);

        tmpfile.persist()?;

        exit(0);
    }

    // Work out if what we downloaded is installable. This is a crude check to
    // see if the OS we asked for matches what we were built for.
    let installable = os == cli::DEFAULT_OS;
    if !installable {
        messages.os_mismatch(cli::DEFAULT_OS, os);
        messages.skipped_install(filename);

        tmpfile.persist()?;

        exit(0);
    }

    // Continue to attempt installation
    // Try to get an install_dir
    let bin_dir = if let Some(dir) = matches.get_one::<PathBuf>("INSTALL_DIR") {
        // If a --install-dir was given, use that. We validated this in the
        // CLI so we know this is good.
        dir.clone()
    }
    else {
        install::bin_dir()?
    };

    messages.unzipping(filename, &bin_dir);

    let mut zip_handle = tmpfile.handle()?;

    match install::install(&mut zip_handle, &bin_dir) {
        Ok(extracted_files) => {
            for file in extracted_files {
                messages.extracted_file(&file, &bin_dir);
            }

            messages.installation_successful();
        },
        Err(e) => {
            messages.installation_failed(&e);

            exit(1);
        }
    }

    if matches.get_flag("KEEP") {
        messages.keep_zipfile(filename);

        tmpfile.persist()?;
    }

    Ok(())
}
