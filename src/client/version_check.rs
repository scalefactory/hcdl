// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use serde::Deserialize;

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
