use std::borrow::Cow;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub title: Cow<'static, str>,
    pub long_title: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub date: Cow<'static, str>,
    pub update_type: Cow<'static, str>,
}

lazy_static! {
    pub static ref CURRENT_VERSION: Version = Version {
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        title: Cow::Borrowed(env!("CARGO_PKG_VERSION")),
        long_title: Cow::Owned(format!("Version {}", env!("CARGO_PKG_VERSION"))),
        description: Cow::Borrowed(""),
        date: Cow::Borrowed(""),
        update_type: Cow::Borrowed("release"),
    };
}

impl Version {
    pub fn newer_than(&self, other: &Version) -> bool {
        if self.major != other.major {
            self.major > other.major
        } else if self.minor != other.minor {
            self.minor > other.minor
        } else {
            self.patch > other.patch
        }
    }

    pub fn newer_than_current(&self) -> bool {
        self.newer_than(&CURRENT_VERSION)
    }
}

pub async fn fetch_latest_version() -> Result<Version, reqwest::Error> {
    let url = "https://api.microbin.eu/version/";
    let http_client = crate::util::http_client::new_async();
    let response = http_client.get(url).send().await?;
    let version = response.json::<Version>().await?;

    Ok(version)
}
