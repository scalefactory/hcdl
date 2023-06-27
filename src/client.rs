// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use crate::{
    error::ClientError,
    progressbar::ProgressBarBuilder,
    shasums::Shasums,
    signature::Signature,
    tmpfile::TmpFile,
};
use bytes::Bytes;
use reqwest::Response;
use std::io::prelude::*;
use std::io::BufWriter;
use url::Url;

/// Re-export of `build`.
pub mod build;

/// Re-export of `config`.
pub mod config;

/// Re-export of `product_version`.
pub mod product_version;

pub use config::ClientConfig;
use product_version::ProductVersion;

const RELEASES_API: &str = "https://api.releases.hashicorp.com/v1/releases";

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

/// A [`Client`] for downloading [HashiCorp](https://www.hashicorp.com)
/// products.
#[derive(Debug)]
pub struct Client {
    api_url: String,
    client:  reqwest::Client,
    config:  ClientConfig,
}

impl Client {
    /// Creates a new [`Client`] with the given [`ClientConfig`].
    ///
    /// # Errors
    ///
    /// Errors if failing to build the [`reqwest::Client`].
    pub fn new(config: ClientConfig) -> Result<Self, ClientError> {
        // Get a new reqwest client with our user-agent
        let client = reqwest::ClientBuilder::new()
            .gzip(true)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|_err| ClientError::ClientBuilder)?;

        let client = Self {
            api_url: RELEASES_API.to_string(),
            client:  client,
            config:  config,
        };

        Ok(client)
    }

    /// Checks the current version of the given `product` against the
    /// [HashiCorp](https://www.hashicorp.com) checkpoint API.
    ///
    /// # Errors
    ///
    /// Errors if:
    ///   - Failing to parse the created checkpoint URL
    ///   - Failing to get the product version
    ///   - Failing to create a [`crate::client::product_version::ProductVersion`]
    pub async fn check_version(
        &self,
        product: &str,
    ) -> Result<ProductVersion, ClientError> {
        let url = format!(
            "{api}/{product}/latest",
            api = self.api_url,
        );

        let url = Url::parse(&url)
            .map_err(|_err| ClientError::Url("check_version"))?;

        let resp = self.get(url)
            .await?
            .json::<ProductVersion>()
            .await
            .map_err(|_err| ClientError::ProductVersion)?;

        Ok(resp)
    }

    /// Downloads content from the given `url` to `tmpfile`.
    ///
    /// # Errors
    ///
    /// Errors if:
    ///   - Failing to make a request to the given `url`
    ///   - Failing to download the content from the given `url`
    ///   - Failing to write the downloaded content to the `tmpfile`
    pub async fn download(
        &self,
        url: Url,
        tmpfile: &mut TmpFile,
    ) -> Result<(), ClientError> {
        let file = tmpfile.handle()?;

        // Start the GET and attempt to get a content-length
        let mut resp   = self.get(url).await?;
        let total_size = resp.content_length();

        // Setup the progress display and wrap the file writer.
        let pb = ProgressBarBuilder::new()
            .no_color(self.config.no_color)
            .quiet(self.config.quiet)
            .size(total_size)
            .build();

        let writer = BufWriter::new(file);
        let mut writer = pb.wrap_write(writer);

        // Start downloading chunks.
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|_| ClientError::Chunk)?
        {
            // Write the chunk to the output file.
            writer.write_all(&chunk)?;
        }

        pb.finished();

        Ok(())
    }

    /// Perform an HTTP GET on the given `url`.
    async fn get(&self, url: Url) -> Result<Response, ClientError> {
        let resp = self.client
            .get(url.clone())
            .send()
            .await
            .map_err(|_err| ClientError::Get(url))?;

        Ok(resp)
    }

    /// Perform an HTTP GET on the given `url` and return the result as
    /// [`Bytes`].
    async fn get_bytes(&self, url: Url) -> Result<Bytes, ClientError> {
        let resp = self.get(url)
            .await?
            .bytes()
            .await
            .map_err(|_err| ClientError::GetBytes)?;

        Ok(resp)
    }

    /// Perform an HTTP GET on the given `url` and return the result as a
    /// `String`.
    async fn get_text(&self, url: Url) -> Result<String, ClientError> {
        let resp = self.get(url)
            .await?
            .text()
            .await
            .map_err(|_err| ClientError::GetText)?;

        Ok(resp)
    }

    /// Get the checksums for the given [`ProductVersion`] and return a new
    /// [`Shasums`].
    ///
    /// # Errors
    ///
    /// Errors when failing to get the shasum file.
    pub async fn get_shasums(
        &self,
        version: &ProductVersion,
    ) -> Result<Shasums, ClientError> {
        let url     = version.shasums_url();
        let shasums = self.get_text(url).await?;
        let shasums = Shasums::new(shasums);

        Ok(shasums)
    }

    /// Get the signature for the given [`ProductVersion`] and return a new
    /// [`Signature`].
    ///
    /// # Errors
    ///
    /// Errors if:
    ///   - Failing to get the shasums signature
    ///   - Failing to create a [`Signature`]
    pub async fn get_signature(
        &self,
        version: &ProductVersion,
    ) -> Result<Signature, ClientError> {
        let url       = version.shasums_signature_url();
        let signature = self.get_bytes(url).await?;
        let signature = Signature::new(signature)?;

        Ok(signature)
    }

    /// Get the [`ProductVersion`] for a given `product` and `version`.
    ///
    /// # Errors
    ///
    /// Errors if:
    ///   - Failing to get the version from the remote server
    ///   - Failing to deserialize the obtained version into a
    ///     [`ProductVersion`]
    pub async fn get_version(
        &self,
        product: &str,
        version: &str,
    ) -> Result<ProductVersion, ClientError> {
        let url = format!(
            "{api}/{product}/{version}",
            api = self.api_url,
        );

        let url = Url::parse(&url)
            .map_err(|_err| ClientError::Url("get_version"))?;

        let resp = self.get(url)
            .await?
            .json::<ProductVersion>()
            .await
            .map_err(|_err| ClientError::ProductVersion)?;

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
        format!("{TEST_DATA_DIR}{filename}")
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

        let mut server = mockito::Server::new_async().await;
        let data       = data_path("check_terraform.json");

        let _m = server.mock("GET", "/terraform/latest")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_file(&data)
            .create_async()
            .await;

        let mut client = Client::new(ClientConfig::default()).unwrap();
        client.api_url = server.url();

        let ret    = client.check_version("terraform").await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_bytes() {
        let mut server = mockito::Server::new_async().await;
        let server_url = server.url();
        let url        = Url::parse(&format!("{server_url}/test.txt")).unwrap();
        let expected   = "Test text\n";
        let data       = data_path("test.txt");

        let _m = server.mock("GET", "/test.txt")
            .with_status(200)
            .with_body_from_file(&data)
            .create_async()
            .await;

        let mut client = Client::new(ClientConfig::default()).unwrap();
        client.api_url = server.url();

        let ret = client.get_bytes(url).await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_signature() {
        let mut server = mockito::Server::new_async().await;
        let server_url = server.url();
        let data       = data_path("terraform_0.12.26_SHA256SUMS.sig");

        let _m = server.mock("GET", "/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig")
            .with_status(200)
            .with_body_from_file(&data)
            .create_async()
            .await;

        let version = ProductVersion {
            name:              "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums:       Url::parse(&format!("{server_url}/terraform/0.12.26/terraform_0.12.26_SHA256SUMS")).unwrap(),
            version:           "0.12.26".into(),
            builds:            vec![
                Build {
                    arch: "amd64".into(),
                    os:   "freebsd".into(),
                    url:  Url::parse(&format!("{server_url}/terraform/0.12.26/terraform_0.12.26_freebsd_amd64.zip")).unwrap(),
                },
                Build {
                    arch: "amd64".into(),
                    os:   "linux".into(),
                    url:  Url::parse(&format!("{server_url}/terraform/0.12.26/terraform_0.12.26_linux_amd64.zip")).unwrap(),
                },
            ],
            url_shasums_signatures: vec![
                Url::parse(&format!("{server_url}/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig")).unwrap(),
            ],
        };

        let gpg_key_path = format!("{}{}", GPG_DIR, "hashicorp.asc");
        let gpg_key      = read_file_bytes(&Path::new(&gpg_key_path).to_path_buf());
        let signature    = read_file_bytes(&Path::new(&data).to_path_buf());

        let expected = Signature::with_public_key(
            signature,
            ::std::str::from_utf8(&gpg_key).unwrap(),
        ).unwrap();

        let mut client = Client::new(ClientConfig::default()).unwrap();
        client.api_url = server.url();

        let ret = client.get_signature(&version).await.unwrap();

        assert_eq!(expected, ret)
    }

    #[tokio::test]
    async fn test_get_text() {
        let mut server = mockito::Server::new_async().await;
        let server_url = server.url();
        let url        = Url::parse(&format!("{server_url}/test.txt")).unwrap();
        let expected   = Bytes::from("Test text\n");
        let data       = data_path("test.txt");

        let _m = server.mock("GET", "/test.txt")
            .with_status(200)
            .with_body_from_file(&data)
            .create_async()
            .await;

        let mut client = Client::new(ClientConfig::default()).unwrap();
        client.api_url = server.url();

        let ret = client.get_text(url).await.unwrap();

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

        let mut server = mockito::Server::new_async().await;
        let data       = data_path("check_terraform.json");

        let _m = server.mock("GET", "/terraform/0.12.26")
            .with_body_from_file(&data)
            .with_body_from_file("test-data/check_terraform.json")
            .with_header("content-type", "application/json")
            .with_status(200)
            .create_async()
            .await;

        let mut client = Client::new(ClientConfig::default()).unwrap();
        client.api_url = server.url();

        let ret = client.get_version("terraform", "0.12.26").await.unwrap();

        assert_eq!(expected, ret)
    }
}
