// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use anyhow::Result;
use bytes::Bytes;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use reqwest;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use super::shasums::Shasums;
use super::signature::Signature;

mod build;
mod product_version;
use product_version::*;
mod version_check;
use version_check::*;

#[cfg(not(test))]
static CHECKPOINT_URL: &str = "https://checkpoint-api.hashicorp.com/v1/check";

#[cfg(test)]
use lazy_static::lazy_static;

#[cfg(test)]
lazy_static! {
    static ref CHECKPOINT_URL: Box<String> = {
        let url = mockito::server_url();
        Box::new(url)
    };
}

static PROGRESS_CHARS: &str = "#>-";

static PROGRESS_TEMPLATE: &str = concat!(
    "{spinner:green} ",
    "[{elapsed_precise}] ",
    "[{bar:40.cyan/blue}] ",
    "{bytes}/{total_bytes} ",
    "({eta})",
    " {msg}",
);

static RELEASES_URL: &str = "https://releases.hashicorp.com/";

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

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

    // Version check the given product via the checkpoint API
    pub async fn check_version(&self, product: &str) -> Result<VersionCheck> {
        // We to_string here for the test scenario.
        let url = format!(
            "{checkpoint}/{product}",
            checkpoint=CHECKPOINT_URL.to_string(),
            product=product,
        );

        let resp = self.get(&url)
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
        let mut resp   = self.get(url).await?;
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

    // Perform an HTTP GET on the given URL
    pub async fn get(&self, url: &str) -> Result<reqwest::Response> {
        let resp = self.client
            .get(url)
            .send()
            .await?;

        Ok(resp)
    }

    // Perform an HTTP GET on the given URL and return the result as Bytes
    pub async fn get_bytes(&self, url: &str) -> Result<Bytes> {
        let resp: Bytes = self.get(url)
            .await?
            .bytes()
            .await?;

        Ok(resp)
    }

    // Perform an HTTP GET on the given URL and return the result as a String
    pub async fn get_text(&self, url: &str) -> Result<String> {
        let resp = self.get(url)
            .await?
            .text()
            .await?;

        Ok(resp)
    }

    // Get the shasums for the given product version and return a new Shasums.
    pub async fn get_shasums(
        &self,
        version: &ProductVersion,
    ) -> Result<Shasums> {
        let url     = version.shasums_url();
        let shasums = self.get_text(&url).await?;
        let shasums = Shasums::new(shasums);

        Ok(shasums)
    }

    // Get the signature for the given ProductVersion and return a new
    // Signature.
    pub async fn get_signature(
        &self,
        version: &ProductVersion,
    ) -> Result<Signature> {
        let url       = version.shasums_signature_url();
        let signature = self.get_bytes(&url).await?;
        let signature = Signature::new(signature)?;

        Ok(signature)
    }

    // Get the ProductVersion for a given product and version.
    pub async fn get_version(&self, product: &str, version: &str) -> Result<ProductVersion> {
        let url = format!(
            "{releases_url}{product}/{version}/index.json",
            releases_url=RELEASES_URL,
            product=product,
            version=version,
        );

        let resp = self.get(&url)
            .await?
            .json::<ProductVersion>()
            .await?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use pretty_assertions::assert_eq;

    const TEST_DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-data/",
    );

    // Builds up the path to the test file
    fn data_path(filename: &str) -> String {
        format!("{}{}", TEST_DATA_DIR, filename)
    }

    #[tokio::test]
    async fn test_check_version_ok() {
        let expected = VersionCheck {
            alerts:                Vec::new(),
            current_changelog_url: "https://github.com/hashicorp/terraform/blob/v0.12.26/CHANGELOG.md".into(),
            current_download_url:  "https://releases.hashicorp.com/terraform/0.12.26/".into(),
            current_release:       1590599832,
            current_version:       "0.12.26".into(),
            product:               "terraform".into(),
            project_website:       "https://www.terraform.io".into(),
        };

        let data = data_path("check_terraform.http");
        let _m   = mock("GET", "/terraform")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_file(&data)
            .create();

        let client = Client::new();
        let ret    = client.check_version("terraform").await.unwrap();

        assert_eq!(expected, ret)
    }
}
