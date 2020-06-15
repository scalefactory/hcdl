// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Build {
    pub arch:     String,
    pub filename: String,
    pub name:     String,
    pub os:       String,
    pub url:      String,
    pub version:  String,
}
