// DISCLAIMER
// (c) 2024-05-27 overcuriousity - derived from the original Microbin Project by Daniel Szabo
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

pub static GIT_COMMIT: &str = env!("GIT_COMMIT_SHORT");

pub static CURRENT_VERSION: Version = Version {
    major: 1,
    minor: 1,
    patch: 4,
    title: Cow::Borrowed("1.1.4"),
    long_title: Cow::Borrowed("Version 1.1.4, Build 20241107"),
    description: Cow::Borrowed("This version includes bug fixes and smaller design enhancements."),
    date: Cow::Borrowed("2024-11-07"),
    update_type: Cow::Borrowed("beta"),
};
