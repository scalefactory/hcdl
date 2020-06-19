// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use serde::Deserialize;
use super::build::*;

static RELEASES_URL: &str = "https://releases.hashicorp.com/";

// Represents a single version of a HashiCorp product
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ProductVersion {
    pub builds:            Vec<Build>,
    pub name:              String,
    pub shasums:           String,
    pub shasums_signature: String,
    pub version:           String,
}

impl ProductVersion {
    // Pull a specific build out of the product version builds.
    pub fn build(&self, arch: &str, os: &str) -> Option<Build> {
        let filtered: Vec<Build> = self.builds
            .iter()
            .filter(|b| b.arch == arch && b.os == os)
            .map(|b| b.clone())
            .collect();

        if filtered.is_empty() {
            None
        }
        else {
            Some(filtered[0].to_owned())
        }
    }

    // Create and return the shasums signature URL.
    pub fn shasums_signature_url(&self) -> String {
        format!(
            "{releases_url}{product}/{version}/{filename}",
            releases_url=RELEASES_URL,
            product=self.name,
            version=self.version,
            filename=self.shasums_signature,
        )
    }

    // Create and return the shasums URL.
    pub fn shasums_url(&self) -> String {
        format!(
            "{releases_url}{product}/{version}/{filename}",
            releases_url=RELEASES_URL,
            product=self.name,
            version=self.version,
            filename=self.shasums,
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

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

        assert_eq!(build, expected)
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
