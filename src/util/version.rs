extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::borrow::Cow;

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

pub static CURRENT_VERSION: Version = Version {
    major: 2,
    minor: 0,
    patch: 4,
    title: Cow::Borrowed("2.0.4"),
    long_title: Cow::Borrowed("Version 2.0.4, Build 20230711"),
    description: Cow::Borrowed("This version includes bug fixes and performance improvements."),
    date: Cow::Borrowed("2023-07-11"),
    update_type: Cow::Borrowed("beta"),
};

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
    let response = reqwest::get(url).await?;
    let version = response.json::<Version>().await?;

    Ok(version)
}
