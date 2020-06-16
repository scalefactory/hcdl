// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use serde::Deserialize;

// Represents a result from the checkpoint API
#[derive(Clone, Debug, Deserialize)]
pub struct VersionCheck {
    alerts:                   Vec<String>,
    current_changelog_url:    String,
    current_release:          u64,
    product:                  String,
    project_website:          String,
    pub current_download_url: String,
    pub current_version:      String,
}
