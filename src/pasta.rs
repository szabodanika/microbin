use bytesize::ByteSize;
use chrono::{Datelike, Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

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

    pub fn content_syntax_highlighted(&self) -> String {
        html_highlight(&self.content, &self.extension)
    }

    pub fn content_not_highlighted(&self) -> String {
        html_highlight(&self.content, "txt")
    }

    pub fn content_escaped(&self) -> String {
        self.content.replace("`", "\\`").replace("$", "\\$")
    }
}

impl fmt::Display for Pasta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
