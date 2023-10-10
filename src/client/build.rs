// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use serde::Deserialize;
use url::Url;

/// Represents a single build of a [HashiCorp](https://hashicorp.io) product.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Build {
    /// The `arch` of the build.
    pub arch: String,

    /// The `os` of the build.
    pub os: String,

    /// The [`Url`] where the build can be found.
    pub url: Url,
}
