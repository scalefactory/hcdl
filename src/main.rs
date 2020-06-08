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
use std::path::Path;
use tokio;
use zip::ZipArchive;

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

// conditionally compiled OS names
#[cfg(target_os = "freebsd")]
const OS: &'static str = "freebsd";

#[cfg(target_os = "linux")]
const OS: &'static str = "linux";

#[cfg(target_os = "mac_os")]
const OS: &'static str = "darwin";

#[cfg(target_os = "openbsd")]
const OS: &'static str = "openbsd";

#[cfg(target_os = "solaris")]
const OS: &'static str = "solaris";

#[cfg(target_os = "windows")]
const OS: &'static str = "windows";

// Conditional architectures
#[cfg(target_arch = "arm")]
const ARCH: &'static str = "arm";

#[cfg(target_arch = "x86")]
const ARCH: &'static str = "386";

#[cfg(target_arch = "x86_64")]
const ARCH: &'static str = "amd64";

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

fn unzip(zipfile: &str, filename: &str, dest: &str) -> Result<()> {
    println!(
        "Unzipping '{filename}' from '{zipfile}' to '{dest}'",
        filename=filename,
        zipfile=zipfile,
        dest=dest,
    );

    let path = Path::new(zipfile);
    let file = File::open(&path).expect("open zipfile");

    let mut zip    = ZipArchive::new(file).expect("new ziparchive");
    let mut wanted = zip.by_name(filename).expect("find zip contents");

    let dest = Path::new(dest);
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
    let latest     = check_version("terraform").await?;
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

    let build = get_build(&info.builds, ARCH, OS).unwrap();
    // println!("{:#?}", build);

    let download_url = &build.url;
    let filename     = &build.filename;

    download_file(download_url, filename).await?;
    let shasums = get_shasums(&shasums_url).await?;
    check_shasum_sig(&shasums_sig, &shasums).await?;
    let shasum = get_shasum(&shasums, filename).await?;

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

    Ok(())
}
