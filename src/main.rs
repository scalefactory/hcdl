//! hcdl: Easily update Hashicorp tools
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use std::path::Path;
use std::process::exit;

mod cli;
mod client;
mod install;
mod products;
mod shasums;
mod signature;
mod tmpfile;
use tmpfile::TmpFile;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli::parse_args();

    // We don't need to do very much if we're listing products
    if matches.is_present("LIST_PRODUCTS") {
        println!(
            "Products: {products}",
            products=products::PRODUCTS_LIST.join(", "),
        );

        exit(0);
    };

    // Pull options from matches
    // Unwraps here should be fine as these are checked and have default
    // values.
    let build_version = matches.value_of("BUILD").unwrap();
    let product       = matches.value_of("PRODUCT").unwrap();

    let client = client::Client::new();

    let builds = if build_version == "latest" {
        let latest = client.check_version(product).await?;

        // Check only, no download.
        if matches.is_present("CHECK") {
            println!("Latest version: {latest}", latest=latest);

            exit(0);
        }

        let current_version = &latest.current_version;

        client.get_version(product, current_version).await?
    }
    else {
        client.get_version(product, build_version).await?
    };

    let arch  = matches.value_of("ARCH").unwrap();
    let os    = matches.value_of("OS").unwrap();
    let build = match builds.build(arch, os) {
        Some(build) => build,
        None        => {
            eprintln!(
                "Couldn't find build for {os}-{arch}",
                os=os,
                arch=arch,
            );

            exit(1);
        },
    };

    // Download SHASUMS file
    let shasums = client.get_shasums(&builds).await?;

    // Verify the SHASUMS file against its signature
    let no_sig = matches.is_present("NO_VERIFY_SIGNATURE");
    if !no_sig {
        println!(
            "Downloading and verifying signature of {shasums}...",
            shasums=builds.shasums,
        );

        // Download signature file
        let signature = client.get_signature(&builds).await?;

        match signature.check(&shasums) {
            Ok(_)  => {
                println!(
                    "  Verified against {signature}.",
                    signature=builds.shasums_signature,
                )
            },
            Err(e) => {
                eprintln!(
                    "  Verification against {shasums} failed.\nError: {error}\nExiting.",
                    shasums=builds.shasums,
                    error=e,
                );

                exit(1);
            },
        };
    }

    // Download the product
    let download_url = &build.url;
    let filename     = &build.filename;

    // Get a new tmpfile for the download.
    let mut tmpfile = TmpFile::new(&filename)?;

    println!("Downloading {filename}...", filename=&filename);
    client.download(&download_url, &mut tmpfile).await?;

    // Ensure the SHASUM is correct
    match shasums.check(&mut tmpfile)? {
        shasums::Checksum::OK => {
            println!("SHA256 of {filename} OK.", filename=filename);
        },
        shasums::Checksum::Bad => {
            println!(
                "SHA256 of {filename} did not match.",
                filename=filename,
            );

            exit(1);
        },
    };

    // If we're only downloading, just persist the file and we're done.
    if matches.is_present("DOWNLOAD_ONLY") {
        tmpfile.persist()?;

        exit(0);
    }

    // Continue to attempt installation
    // Try to get an install_dir
    let bin_dir = if let Some(dir) = matches.value_of("INSTALL_DIR") {
        // If a --install-dir was given, use that. We validated this in the
        // CLI so we know this is good.
        Path::new(dir).to_path_buf()
    }
    else {
        install::bin_dir()?
    };

    println!(
        "Unzipping '{product}' from '{zipfile}' to '{dest}'...",
        product=product,
        zipfile=filename,
        dest=bin_dir.display(),
    );

    let mut handle = tmpfile.handle()?;
    match install::install(&mut handle, &bin_dir) {
        Ok(_)  => println!("  Installation successful."),
        Err(e) => {
            eprintln!("  Installation failed with error: {error}", error=e);

            exit(1);
        }
    }

    if matches.is_present("KEEP") {
        tmpfile.persist()?;
    }

    Ok(())
}
