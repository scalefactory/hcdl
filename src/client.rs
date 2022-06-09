// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use crate::progressbar::ProgressBarBuilder;
use crate::shasums::Shasums;
use crate::signature::Signature;
use crate::tmpfile::TmpFile;
use anyhow::Result;
use bytes::Bytes;
use reqwest::Response;
use std::io::prelude::*;
use url::Url;

#[cfg(test)]
use once_cell::sync::Lazy;

mod build;
mod product_version;
use product_version::ProductVersion;

#[cfg(not(test))]
const RELEASES_API: &str = "https://api.releases.hashicorp.com/v1/releases";

#[cfg(test)]
static RELEASES_API: Lazy<String> = Lazy::new(|| {
    let url = mockito::server_url();
    url
});

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub struct Client {
    client:   reqwest::Client,
    no_color: bool,
    quiet:    bool,
}

impl Client {
    // Return a new reqwest client with our user-agent
    pub fn new(quiet: bool, no_color: bool) -> Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .gzip(true)
            .user_agent(USER_AGENT)
            .build()?;

        let client = Self {
            client:   client,
            no_color: no_color,
            quiet:    quiet,
        };

        Ok(client)
    }

    // Version check the given product via the checkpoint API
    pub async fn check_version(&self, product: &str) -> Result<ProductVersion> {
        // We to_string here for the test scenario.
        #![allow(clippy::to_string_in_format_args)]
        let url = format!(
            "{api}/{product}/latest",
            api = RELEASES_API.to_string(),
            product = product,
        );

        let url = Url::parse(&url)?;

        let resp = self.get(url)
            .await?
            .json::<ProductVersion>()
            .await?;

        Ok(resp)
    }

    // Download from the given URL to the output file.
    pub async fn download(&self, url: Url, tmpfile: &mut TmpFile) -> Result<()> {
        let file = tmpfile.handle()?;

        // Start the GET and attempt to get a content-length
        let mut resp   = self.get(url).await?;
        let total_size = resp.content_length();

        // Setup the progress display and wrap the file writer.
        //let pb = self.progress_bar(total_size);
        let pb = ProgressBarBuilder::new()
            .no_color(self.no_color)
            .quiet(self.quiet)
            .size(total_size)
            .build();

        let mut writer = pb.wrap_write(file);

        // Start downloading chunks.
        while let Some(chunk) = resp.chunk().await? {
            // Write the chunk to the output file.
            writer.write_all(&chunk)?;
        }

        pb.finished();

        Ok(())
    }

    // Perform an HTTP GET on the given URL
    pub async fn get(&self, url: Url) -> Result<Response> {
        let resp = self.client
            .get(url)
            .send()
            .await?;

        Ok(resp)
    }

    // Perform an HTTP GET on the given URL and return the result as Bytes
    pub async fn get_bytes(&self, url: Url) -> Result<Bytes> {
        let resp = self.get(url)
            .await?
            .bytes()
            .await?;

        Ok(resp)
    }

    // Perform an HTTP GET on the given URL and return the result as a String
    pub async fn get_text(&self, url: Url) -> Result<String> {
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
        let shasums = self.get_text(url).await?;
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
        let signature = self.get_bytes(url).await?;
        let signature = Signature::new(signature)?;

        Ok(signature)
    }

    // Get the ProductVersion for a given product and version.
    pub async fn get_version(
        &self,
        product: &str,
        version: &str,
    ) -> Result<ProductVersion> {
        // We to_string here for the test scenario.
        #![allow(clippy::to_string_in_format_args)]
        let url = format!(
            "{api}/{product}/{version}",
            api = RELEASES_API.to_string(),
            product = product,
            version = version,
        );

        let url = Url::parse(&url)?;

        let resp = self.get(url)
            .await?
            .json::<ProductVersion>()
            .await?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::build::Build;
    use chrono::{
        DateTime,
        Utc,
    };
    use mockito::mock;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::{
        Path,
        PathBuf,
    };
    use std::str::FromStr;

    const GPG_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/gpg/",
    );

    const TEST_DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-data/",
    );

    // Builds up the path to the test file
    fn data_path(filename: &str) -> String {
        format!("{}{}", TEST_DATA_DIR, filename)
    }

    fn read_file_bytes(path: &PathBuf) -> Bytes {
        let file         = File::open(&path).unwrap();
        let mut reader   = BufReader::new(file);
        let mut contents = Vec::new();

        reader.read_to_end(&mut contents).unwrap();

        Bytes::from(contents)
    }

    #[tokio::test]
    async fn test_check_version_ok() {
        let expected = ProductVersion {
            name:        "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums: Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS").unwrap(),
            version:     "0.12.26".into(),
            builds:      vec![
                Build {
                    arch: "amd64".into(),
                    os:   "freebsd".into(),
                    url:  Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_freebsd_amd64.zip").unwrap(),
                },
                Build {
                    arch: "amd64".into(),
                    os:   "linux".into(),
                    url:  Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_linux_amd64.zip").unwrap(),
                },
            ],
            url_shasums_signatures: vec![
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig").unwrap(),
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.348FFC4C.sig").unwrap(),
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.72D7468F.sig").unwrap(),
            ],
        };

        let data = data_path("check_terraform.json");
        let _m   = mock("GET", "/terraform/latest")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_file(&data)
            .create();

        let client = Client::new(true, true).unwrap();
        let ret    = client.check_version("terraform").await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_bytes() {
        let server_url = mockito::server_url();
        let url        = Url::parse(&format!("{}/test.txt", server_url)).unwrap();
        let expected   = "Test text\n";
        let data       = data_path("test.txt");
        let _m         = mock("GET", "/test.txt")
            .with_status(200)
            .with_body_from_file(&data)
            .create();

        let client = Client::new(true, true).unwrap();
        let ret    = client.get_bytes(url).await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_signature() {
        let data = data_path("terraform_0.12.26_SHA256SUMS.sig");
        let _m   = mock("GET", "/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig")
            .with_status(200)
            .with_body_from_file(&data)
            .create();

        let version = ProductVersion {
            name:              "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums:       Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS").unwrap(),
            version:           "0.12.26".into(),
            builds:            vec![
                Build {
                    arch: "amd64".into(),
                    os:   "freebsd".into(),
                    url:  Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_freebsd_amd64.zip").unwrap(),
                },
                Build {
                    arch: "amd64".into(),
                    os:   "linux".into(),
                    url:  Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_linux_amd64.zip").unwrap(),
                },
            ],
            url_shasums_signatures: vec![
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig").unwrap(),
            ],
        };

        let gpg_key_path = format!("{}{}", GPG_DIR, "hashicorp.asc");
        let gpg_key      = read_file_bytes(&Path::new(&gpg_key_path).to_path_buf());
        let signature    = read_file_bytes(&Path::new(&data).to_path_buf());

        let expected = Signature::with_public_key(
            signature,
            ::std::str::from_utf8(&gpg_key).unwrap().to_string(),
        ).unwrap();

        let client = Client::new(true, true).unwrap();
        let ret    = client.get_signature(&version).await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_text() {
        let server_url = mockito::server_url();
        let url        = Url::parse(&format!("{}/test.txt", server_url)).unwrap();
        let expected   = Bytes::from("Test text\n");
        let data       = data_path("test.txt");
        let _m         = mock("GET", "/test.txt")
            .with_status(200)
            .with_body_from_file(&data)
            .create();

        let client = Client::new(true, true).unwrap();
        let ret    = client.get_text(url).await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_version() {
        let expected = ProductVersion {
            name:              "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums:       Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS").unwrap(),
            version:           "0.12.26".into(),
            builds:            vec![
                Build {
                    arch: "amd64".into(),
                    os:   "freebsd".into(),
                    url:  Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_freebsd_amd64.zip").unwrap(),
                },
                Build {
                    arch: "amd64".into(),
                    os:   "linux".into(),
                    url:  Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_linux_amd64.zip").unwrap(),
                },
            ],
            url_shasums_signatures: vec![
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig").unwrap(),
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.348FFC4C.sig").unwrap(),
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.72D7468F.sig").unwrap(),
            ],
        };

        let data = data_path("check_terraform.json");
        let _m   = mock("GET", "/terraform/0.12.26")
            .with_body_from_file(&data)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let client = Client::new(true, true).unwrap();
        let ret    = client.get_version("terraform", "0.12.26").await.unwrap();

        assert_eq!(expected, ret)
    }
}
