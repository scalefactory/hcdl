// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use serde::Deserialize;
use url::Url;

// Represents a single build of a HashiCorp product
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Build {
    pub arch: String,
    pub os:   String,
    pub url:  Url,
}
