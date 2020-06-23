// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use chrono::{
    TimeZone,
    Utc,
};
use serde::Deserialize;
use std::fmt;

// Represents a result from the checkpoint API
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct VersionCheck {
    pub alerts:                Vec<String>,
    pub current_changelog_url: String,
    pub current_download_url:  String,
    pub current_release:       u64,
    pub current_version:       String,
    pub product:               String,
    pub project_website:       String,
}

impl fmt::Display for VersionCheck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{product} v{version} from {datetime}",
            product=self.product,
            version=self.current_version,
            datetime=Utc.timestamp(self.current_release as i64, 0).to_rfc2822(),
        )
    }
}
