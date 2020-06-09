// hcdl: Easily update Hashicorp tools
use anyhow::Result;
use bytes::buf::BufExt;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use reqwest;
use serde::Deserialize;
use sha2::{
    Digest,
    Sha256,
};
use std::fs::{
    File,
    OpenOptions,
};
use std::io::{
    self,
    prelude::*,
    BufReader,
};
use std::path::{
    Path,
    PathBuf,
};
use tokio;
use zip::ZipArchive;

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

mod cli;

const HASHICORP_GPG_KEY: &'static str = include_str!("../gpg/hashicorp.asc");

// Checksum status
enum ChecksumResult {
    Good,
    Bad,
}

#[derive(Clone, Debug, Deserialize)]
struct Build {
    arch:     String,
    filename: String,
    name:     String,
    os:       String,
    url:      String,
    version:  String,
}

#[derive(Clone, Debug, Deserialize)]
struct ProductVersion {
    builds:            Vec<Build>,
    name:              String,
    shasums:           String,
    shasums_signature: String,
    version:           String,
}

#[derive(Clone, Debug, Deserialize)]
struct VersionCheck {
    alerts:                Vec<String>,
    current_changelog_url: String,
    current_download_url:  String,
    current_release:       u64,
    current_version:       String,
    product:               String,
    project_website:       String,
}

async fn get_version(url: &str) -> Result<ProductVersion> {
    let url = format!("{url}index.json", url=url);

    let resp = reqwest::get(&url)
        .await?
        .json::<ProductVersion>()
        .await?;

    Ok(resp)
}

fn get_build(builds: &[Build], arch: &str, os: &str) -> Option<Build> {
    let filtered: Vec<Build> = builds
        .iter()
        .filter(|b| b.arch == arch)
        .filter(|b| b.os == os)
        .map(|b| b.clone())
        .collect();

    if filtered.is_empty() {
        None
    }
    else {
        Some(filtered[0].to_owned())
    }
}

async fn get_shasums(url: &str) -> Result<String> {
    let shasums = reqwest::get(url)
        .await?
        .text()
        .await?;

    Ok(shasums)
}

// Get the shasum for the given filename
async fn get_shasum(shasums: &str, filename: &str) -> Result<String> {
    let shasum = shasums.lines()
        .filter(|l| l.ends_with(filename))
        .map(|l| {
            let sum: Vec<&str> = l.split_whitespace().collect();

            sum[0].to_string()
        })
        .collect::<Vec<String>>()
        .first()
        .unwrap()
        .to_owned();

    Ok(shasum)
}

async fn check_version(product: &str) -> Result<VersionCheck> {
    let url = format!(
        "https://checkpoint-api.hashicorp.com/v1/check/{product}",
        product=product,
    );

    let resp = reqwest::get(&url)
        .await?
        .json::<VersionCheck>()
        .await?;

    Ok(resp)
}

async fn download_file(url: &str, output: &str) -> Result<()> {
    println!("Downloading {output}...", output=output);

    // Attempt to create the output file.
    let path = Path::new(output);
    let mut file = File::create(&path)?;

    // Start the GET and attempt to get a content-length
    let mut resp = reqwest::get(url).await?;
    let total_size = resp.content_length();

    // Setup the progress display
    let pb = if let Some(size) = total_size {
        let style = ProgressStyle::default_bar()
            .template("{spinner:green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");

        let pb = ProgressBar::new(size);
        pb.set_style(style);

        pb
    }
    else {
        ProgressBar::new_spinner()
    };

    pb.set_message(output);

    // Start downloading chunks.
    while let Some(chunk) = resp.chunk().await? {
        // Write the chunk to the output file.
        file.write(&chunk)?;

        // Poke the progress indicator
        if total_size.is_some() {
            pb.inc(chunk.len() as u64);
        }
        else {
            pb.tick();
        }
    }

    Ok(())
}

fn check_sha256(shasum: &str, filename: &str) -> Result<ChecksumResult> {
    let path = Path::new(filename);
    let mut file = File::open(&path)?;
    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher)?;

    let hash = hasher.result();

    let res = if hex::encode(hash) == shasum {
        ChecksumResult::Good
    }
    else {
        ChecksumResult::Bad
    };

    Ok(res)
}

async fn check_shasum_sig(url: &str, content: &str) -> Result<()> {
    // Signature is a binary file
    let signature = reqwest::get(url)
        .await?
        .bytes()
        .await?;

    let mut keyring = gpgrv::Keyring::new();
    let gpg_key = BufReader::new(HASHICORP_GPG_KEY.as_bytes());

    // compat handles an Error returned by the Failure crate.
    match keyring.append_keys_from_armoured(gpg_key) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e.compat()),
    }?;

    // Readers for signature and content
    //let signature = BufReader::new(signature);
    let shasums   = BufReader::new(content.as_bytes());

    match gpgrv::verify_detached(signature.reader(), shasums, &keyring) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e.compat()),
    }?;

    println!("Signature of SHASUM256 file verified.");

    Ok(())
}

fn install(zipfile: &str, filename: &str, dest: &PathBuf) -> Result<()> {
    println!(
        "Unzipping '{filename}' from '{zipfile}' to '{dest}'",
        filename=filename,
        zipfile=zipfile,
        dest=dest.display(),
    );

    let path = Path::new(zipfile);
    let file = File::open(&path).expect("open zipfile");

    let mut zip    = ZipArchive::new(file).expect("new ziparchive");
    let mut wanted = zip.by_name(filename).expect("find zip contents");

    let dest = dest.join(filename);

    let mut options = OpenOptions::new();

    #[cfg(target_family = "unix")]
    options.mode(0o755);

    let mut writer = options
        .create(true)
        .write(true)
        .truncate(true)
        .open(&dest)?;

    io::copy(&mut wanted, &mut writer)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let bin_dir = match dirs::executable_dir() {
        Some(dir) => dir,
        None      => {
            eprintln!("Couldn't find local executable dir. Is $HOME broken?");
            ::std::process::exit(1);
        }
    };

    let matches = cli::parse_args();

    // Pull options from matches
    let product    = matches.value_of("PRODUCT").unwrap();
    let arch       = matches.value_of("ARCH").unwrap();
    let os         = matches.value_of("OS").unwrap();
    let do_install = matches.is_present("INSTALL");

    let latest     = check_version(product).await?;
    let url_prefix = &latest.current_download_url;
    let info       = get_version(url_prefix).await?;

    // println!("{:#?}", latest);
    // println!("{:#?}", info);

    let shasums_url = format!(
        "{prefix}{shasums}",
        prefix=url_prefix,
        shasums=info.shasums,
    );

    let shasums_sig = format!(
        "{prefix}{shasums_sig}",
        prefix=url_prefix,
        shasums_sig=info.shasums_signature,
    );

    let build = get_build(&info.builds, arch, os).unwrap();
    // println!("{:#?}", build);

    let download_url = &build.url;
    let filename     = &build.filename;

    // Download SHASUMS file
    let shasums = get_shasums(&shasums_url).await?;

    // Verify the SHASUMS file against its signature
    check_shasum_sig(&shasums_sig, &shasums).await?;

    // Get the specific SHASUM for the file we want to download
    let shasum = get_shasum(&shasums, filename).await?;

    // Download the product
    download_file(download_url, filename).await?;

    // Ensure the SHASUM is correct
    match check_sha256(&shasum, filename)? {
        ChecksumResult::Good => {
            println!("SHA256 of {filename} OK.", filename=filename);
        },
        ChecksumResult::Bad => {
            println!(
                "SHA256 of {filename} did not match {shasum}.",
                filename=filename,
                shasum=shasum,
            );

            ::std::process::exit(1);
        },
    };

    if do_install {
        install(filename, product, &bin_dir)?;
    }

    Ok(())
}
