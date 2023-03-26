use bytesize::ByteSize;
use chrono::{Datelike, Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::args::ARGS;
use crate::util::animalnumbers::to_animal_names;
use crate::util::hashids::to_hashids;
use crate::util::syntaxhighlighter::html_highlight;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct PastaFile {
    pub name: String,
    pub size: ByteSize,
}

impl PastaFile {
    pub fn from_unsanitized(path: &str) -> Result<Self, &'static str> {
        let path = Path::new(path);
        let name = path.file_name().ok_or("Path did not contain a file name")?;
        let name = name.to_string_lossy().replace(' ', "_");
        Ok(Self {
            name,
            size: ByteSize::b(0),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize)]
pub struct Pasta {
    pub id: u64,
    pub content: String,
    pub file: Option<PastaFile>,
    pub extension: String,
    pub private: bool,
    pub editable: bool,
    pub created: i64,
    pub expiration: i64,
    pub last_read: i64,
    pub read_count: u64,
    pub burn_after_reads: u64,
    pub pasta_type: String,
}

impl Pasta {
    pub fn id_as_animals(&self) -> String {
        if ARGS.hash_ids {
            to_hashids(self.id)
        } else {
            to_animal_names(self.id)
        }
    }

    pub fn created_as_string(&self) -> String {
        let date = Local.timestamp(self.created, 0);
        format!(
            "{:02}-{:02} {:02}:{:02}",
            date.month(),
            date.day(),
            date.hour(),
            date.minute(),
        )
    }

    pub fn expiration_as_string(&self) -> String {
        if self.expiration == 0 {
            String::from("Never")
        } else {
            let date = Local.timestamp(self.expiration, 0);
            format!(
                "{:02}-{:02} {:02}:{:02}",
                date.month(),
                date.day(),
                date.hour(),
                date.minute(),
            )
        }
    }

    pub fn last_read_time_ago_as_string(&self) -> String {
        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // get seconds since last read and convert it to days
        let days = ((timenow - self.last_read) / 86400) as u16;
        if days > 1 {
            return format!("{} days ago", days);
        };

        // it's less than 1 day, let's do hours then
        let hours = ((timenow - self.last_read) / 3600) as u16;
        if hours > 1 {
            return format!("{} hours ago", hours);
        };

        // it's less than 1 hour, let's do minutes then
        let minutes = ((timenow - self.last_read) / 60) as u16;
        if minutes > 1 {
            return format!("{} minutes ago", minutes);
        };

        // it's less than 1 minute, let's do seconds then
        let seconds = (timenow - self.last_read) as u16;
        if seconds > 1 {
            return format!("{} seconds ago", seconds);
        };

        // it's less than 1 second?????
        String::from("just now")
    }

    pub fn last_read_days_ago(&self) -> u16 {
        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // get seconds since last read and convert it to days
        ((timenow - self.last_read) / 86400) as u16
    }

    pub fn content_syntax_highlighted(&self) -> String {
        html_highlight(&self.content, &self.extension)
    }

    pub fn content_not_highlighted(&self) -> String {
        html_highlight(&self.content, "txt")
    }

    pub fn content_escaped(&self) -> String {
        html_escape::encode_text(&self.content.replace('`', "\\`").replace('$', "\\$")).to_string()
    }
}

impl fmt::Display for Pasta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
