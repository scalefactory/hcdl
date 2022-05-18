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
use super::build::Build;
use url::Url;

// Represents a single version of a HashiCorp product
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ProductVersion {
    pub builds:                 Vec<Build>,
    pub license_class:          String,
    pub name:                   String,
    //pub status:                 Status,
    pub url_shasums:            Url,
    pub url_shasums_signatures: Vec<Url>,
    pub url_source_repository:  Url,
    pub version:                String,

    #[serde(deserialize_with = "deserialize_from_str")]
    pub timestamp_created: DateTime<Utc>,

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
    // Pull a specific build out of the product version builds.
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

    // Create and return the shasums signature URL.
    pub fn shasums_signature_url(&self) -> Url {
        self.url_shasums_signatures.first().unwrap().to_owned()
    }

    // Create and return the shasums URL.
    pub fn shasums_url(&self) -> Url {
        self.url_shasums.clone()
    }
}

impl fmt::Display for ProductVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "${product} v{version} from {datetime}",
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
            shasums:           "terraform_0.12.26_SHA256SUMS".into(),
            shasums_signature: "terraform_0.12.26_SHA256SUMS.sig".into(),
            version:           "0.12.26".into(),
            builds:            vec![
                Build {
                    arch:     "amd64".into(),
                    filename: "terraform_0.12.26_freebsd_amd64.zip".into(),
                    name:     "terraform".into(),
                    os:       "freebsd".into(),
                    url:      "".into(),
                    version:  "0.12.26".into(),
                },
                Build {
                    arch:     "amd64".into(),
                    filename: "terraform_0.12.26_linux_amd64.zip".into(),
                    name:     "terraform".into(),
                    os:       "linux".into(),
                    url:      "".into(),
                    version:  "0.12.26".into(),
                },
            ],
        };

        let expected = Build {
            arch:     "amd64".into(),
            filename: "terraform_0.12.26_freebsd_amd64.zip".into(),
            name:     "terraform".into(),
            os:       "freebsd".into(),
            url:      "".into(),
            version:  "0.12.26".into(),
        };

        let build = version.build("amd64", "freebsd").unwrap();

        assert_eq!(build, &expected)
    }

    #[test]
    fn test_shasums_url() {
        let version = ProductVersion {
            builds:            vec![],
            name:              "terraform".into(),
            shasums:           "terraform_0.12.26_SHA256SUMS".into(),
            shasums_signature: "terraform_0.12.26_SHA256SUMS.sig".into(),
            version:           "0.12.26".into(),
        };

        let expected = "https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS";
        let url      = version.shasums_url();

        assert_eq!(url, expected)
    }

    #[test]
    fn test_shasums_signature_url() {
        let version = ProductVersion {
            builds:            vec![],
            name:              "terraform".into(),
            shasums:           "terraform_0.12.26_SHA256SUMS".into(),
            shasums_signature: "terraform_0.12.26_SHA256SUMS.sig".into(),
            version:           "0.12.26".into(),
        };

        let expected = "https://releases.hashicorp.com/terraform/0.12.26/terraform_0.12.26_SHA256SUMS.sig";
        let url      = version.shasums_signature_url();

        assert_eq!(url, expected)
    }
}
