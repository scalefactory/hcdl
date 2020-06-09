// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use reqwest;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use super::shasums::Shasums;
use super::signature::Signature;

static CHECKPOINT_URL: &str = "https://checkpoint-api.hashicorp.com/v1/check/";

static PROGRESS_CHARS: &str = "#>-";

static PROGRESS_TEMPLATE: &str = concat!(
    "{spinner:green} ",
    "[{elapsed_precise}] ",
    "[{bar:40.cyan/blue}] ",
    "{bytes}/{total_bytes} ",
    "({eta})",
    " {msg}",
);

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

#[derive(Clone, Debug, Deserialize)]
pub struct Build {
    arch:         String,
    name:         String,
    os:           String,
    version:      String,
    pub filename: String,
    pub url:      String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProductVersion {
    builds:                Vec<Build>,
    name:                  String,
    version:               String,
    pub shasums:           String,
    pub shasums_signature: String,
}

impl ProductVersion {
    // Pull a specific build out of the product version builds.
    pub fn build(&self, arch: &str, os: &str) -> Option<Build> {
        let filtered: Vec<Build> = self.builds
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
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionCheck {
    alerts:                   Vec<String>,
    current_changelog_url:    String,
    current_release:          u64,
    current_version:          String,
    product:                  String,
    project_website:          String,
    pub current_download_url: String,
}

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    // Return a new reqwest client with our user-agent
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .gzip(true)
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        Self {
            client: client,
        }
    }

    // Version check the given product
    pub async fn check_version(&self, product: &str) -> Result<VersionCheck> {
        let url = format!(
            "{checkpoint}{product}",
            checkpoint=CHECKPOINT_URL,
            product=product,
        );

        let resp = self.client
            .get(&url)
            .send()
            .await?
            .json::<VersionCheck>()
            .await?;

        Ok(resp)
    }

    // Download from the given URL to the output file.
    pub async fn download(&self, url: &str, output: &str) -> Result<()> {
        // Attempt to create the output file.
        let path     = Path::new(output);
        let mut file = File::create(&path)?;

        // Start the GET and attempt to get a content-length
        let mut resp = self.client
            .get(url)
            .send()
            .await?;
        let total_size = resp.content_length();

        // Setup the progress display
        let pb = if let Some(size) = total_size {
            // If we know the total size, setup a nice bar
            let style = ProgressStyle::default_bar()
                .template(PROGRESS_TEMPLATE)
                .progress_chars(PROGRESS_CHARS);

            let pb = ProgressBar::new(size);
            pb.set_style(style);

            pb
        }
        else {
            // Otherwise, just a simple spinner
            ProgressBar::new_spinner()
        };

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

        pb.finish_with_message("done.");

        Ok(())
    }

    pub async fn get_shasums(
        &self,
        url: &str,
        filename: &str,
    ) -> Result<Shasums> {
        let url = format!("{}{}", url, filename);

        let shasums = self.client
            .get(&url)
            .send()
            .await?
            .text()
            .await?;

        let shasums = Shasums::new(shasums);

        Ok(shasums)
    }

    pub async fn get_signature(
        &self,
        url: &str,
        filename: &str,
    ) -> Result<Signature> {
        let url = format!("{}{}", url, filename);

        let signature = self.client
            .get(&url)
            .send()
            .await?
            .bytes()
            .await?;

        let signature = Signature::new(signature);

        Ok(signature)
    }

    pub async fn get_version(&self, url: &str) -> Result<ProductVersion> {
        let url = format!("{url}index.json", url=url);

        let resp = self.client
            .get(&url)
            .send()
            .await?
            .json::<ProductVersion>()
            .await?;

        Ok(resp)
    }
}
