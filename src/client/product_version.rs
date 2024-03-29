// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use chrono::{
    DateTime,
    Utc,
};
use serde::{
    de,
    Deserialize,
    Deserializer,
};
use std::fmt;
use std::str::FromStr;
use url::Url;

pub use super::build::Build;

/// Represents a single version of a [HashiCorp](https://hashicorp.io) product.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ProductVersion {
    /// A `Vec` of [`Build`] for the [`ProductVersion`].
    pub builds: Vec<Build>,

    /// The name of the product.
    pub name: String,

    /// The [`Url`] that the product shasums can be found at.
    pub url_shasums: Url,

    /// The [`Url`] that the product's shasums signature can be found at.
    pub url_shasums_signatures: Vec<Url>,

    /// The version number of the product.
    pub version: String,

    /// A timestamp representing when the product version was created.
    #[serde(deserialize_with = "deserialize_from_str")]
    pub timestamp_created: DateTime<Utc>,

    /// A timestamp representing when the product version was updated.
    #[serde(deserialize_with = "deserialize_from_str")]
    pub timestamp_updated: DateTime<Utc>,
}

fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: fmt::Display,
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

impl ProductVersion {
    /// Pull a specific [`Build`] out of the [`ProductVersion`] `builds`.
    #[must_use]
    pub fn build(&self, arch: &str, os: &str) -> Option<&Build> {
        let filtered: Vec<&Build> = self.builds
            .iter()
            .filter(|b| b.arch == arch && b.os == os)
            .collect();

        if filtered.is_empty() {
            None
        }
        else {
            Some(filtered[0])
        }
    }

    /// Create and return the shasums signature URL.
    ///
    /// # Panics
    ///
    /// This function could panic if there are no signature URLs returned by
    /// the API.
    #[must_use]
    pub fn shasums_signature_url(&self) -> Url {
        self.url_shasums_signatures.first().unwrap().clone()
    }

    /// Create and return the shasums URL.
    #[must_use]
    pub fn shasums_url(&self) -> Url {
        self.url_shasums.clone()
    }
}

impl fmt::Display for ProductVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{product} v{version} from {datetime}",
            product = self.name,
            version = self.version,
            datetime = self.timestamp_updated,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build() {
        let version = ProductVersion {
            name:              "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums:       Url::parse("https://test.example.org/terraform_0.12.26_SHA256SUMS").unwrap(),
            version:           "0.12.26".into(),
            builds:            vec![
                Build {
                    arch: "amd64".into(),
                    os:   "freebsd".into(),
                    url:  Url::parse("https://test.example.org/terraform_0.12.26_freebsd_amd64.zip").unwrap(),
                },
                Build {
                    arch: "amd64".into(),
                    os:   "linux".into(),
                    url:  Url::parse("https://test.example.org/terraform_0.12.26_linux_amd64.zip").unwrap(),
                },
            ],
            url_shasums_signatures: vec![
                Url::parse("https://test.example.org/terraform_0.12.26_SHA256SUMS.sig").unwrap(),
            ],
        };

        let expected = Build {
            arch: "amd64".into(),
            os:   "freebsd".into(),
            url:  Url::parse("https://test.example.org/terraform_0.12.26_freebsd_amd64.zip").unwrap(),
        };

        let build = version.build("amd64", "freebsd").unwrap();

        assert_eq!(build, &expected)
    }

    #[test]
    fn test_shasums_url() {
        let version = ProductVersion {
            builds:            vec![],
            name:              "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums:       Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS").unwrap(),
            version:           "0.12.26".into(),
            url_shasums_signatures: vec![
                Url::parse("https://test.example.org/terraform_0.12.26_SHA256SUMS.sig").unwrap(),
            ],
        };

        let expected = Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS").unwrap();
        let url      = version.shasums_url();

        assert_eq!(url, expected)
    }

    #[test]
    fn test_shasums_signature_url() {
        let version = ProductVersion {
            builds:            vec![],
            name:              "terraform".into(),
            timestamp_created: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            timestamp_updated: DateTime::<Utc>::from_str("2020-05-27T16:55:35.000Z").unwrap(),
            url_shasums:       Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.  ↪ 12.26_SHA256SUMS").unwrap(),
            version:           "0.12.26".into(),
            url_shasums_signatures: vec![
                Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig").unwrap(),
            ],
        };

        let expected = Url::parse("https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig").unwrap();
        let url      = version.shasums_signature_url();

        assert_eq!(url, expected)
    }
}
