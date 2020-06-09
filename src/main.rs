//! hcdl: Easily update Hashicorp tools
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use std::path::Path;
use tokio;

mod cli;
mod client;
mod install;
mod shasums;
mod signature;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli::parse_args();

    // Pull options from matches
    let product     = matches.value_of("PRODUCT").unwrap();
    let arch        = matches.value_of("ARCH").unwrap();
    let os          = matches.value_of("OS").unwrap();
    let do_install  = matches.is_present("INSTALL");
    let install_dir = matches.value_of("INSTALL_DIR");

    let client = client::Client::new();

    let latest     = client.check_version(product).await?;
    let url_prefix = &latest.current_download_url;
    let info       = client.get_version(url_prefix).await?;

    let build = info.build(arch, os).unwrap();

    let download_url = &build.url;
    let filename     = &build.filename;

    println!("Downloading and verifying signature of {}...", info.shasums);

    // Download signature file
    let signature = client.get_signature(
        url_prefix,
        &info.shasums_signature,
    ).await?;

    // Download SHASUMS file
    let shasums = client.get_shasums(url_prefix, &info.shasums).await?;

    // Verify the SHASUMS file against its signature
    match signature.check(&shasums) {
        Ok(_)  => println!("  Verified against {}.", info.shasums_signature),
        Err(e) => {
            eprintln!(
                "  Verification against {} failed.\nError: {}\nExiting.",
                info.shasums,
                e,
            );

            ::std::process::exit(1);
        },
    };

    // Download the product
    println!("Downloading {}...", filename);
    client.download(download_url, filename).await?;

    // Ensure the SHASUM is correct
    match shasums.check(filename)? {
        shasums::Checksum::OK => {
            println!("SHA256 of {filename} OK.", filename=filename);
        },
        shasums::Checksum::Bad => {
            println!(
                "SHA256 of {filename} did not match.",
                filename=filename,
            );

            ::std::process::exit(1);
        },
    };

    if do_install {
        // Try to get an install_dir
        let bin_dir = if let Some(dir) = install_dir {
            // If a --install-dir was given, use that. We validated this in the
            // CLI so we know this is good.
            Path::new(dir).to_path_buf()
        }
        else {
            // If a --install-dir wasn't given, try to use the XDG executable
            // dir.
            match dirs::executable_dir() {
                Some(dir) => {
                    // We don't currently handle creating these directories.
                    if !dir.exists() {
                        eprintln!(
                            "'{dir}' does not exist, create it and try again",
                            dir=dir.display(),
                        );

                        ::std::process::exit(1);
                    }

                    dir
                },
                None => {
                    // If we get None, we're likely on Windows.
                    eprintln!("Could not find suitable install-dir.");
                    eprintln!("Consider passing --install-dir to manually specify");

                    ::std::process::exit(1);
                },
            }
        };

        println!(
            "Unzipping '{product}' from '{zipfile}' to '{dest}'...",
            product=product,
            zipfile=filename,
            dest=bin_dir.display(),
        );

        match install::install(filename, product, &bin_dir) {
            Ok(_)  => println!("  Installation successful."),
            Err(e) => {
                eprintln!("  Installation failed with error: {}", e);

                ::std::process::exit(1);
            }
        }
    }

    Ok(())
}
