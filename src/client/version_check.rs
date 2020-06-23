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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_display() {
        let version = VersionCheck {
            alerts:                Vec::new(),
            current_changelog_url: "https://github.com/hashicorp/terraform/blob/v0.12.26/CHANGELOG.md".into(),
            current_download_url:  "https://releases.hashicorp.com/terraform/0.12.26/".into(),
            current_release:       1590599832,
            current_version:       "0.12.26".into(),
            product:               "terraform".into(),
            project_website:       "https://www.terraform.io".into(),
        };

        let expected = "terraform v0.12.26 from Wed, 27 May 2020 17:17:12 +0000";
        let ret      = format!("{}", version);

        assert_eq!(expected, ret)
    }
}
