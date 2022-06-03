use std::fmt;

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::util::animalnumbers::to_animal_names;
use crate::util::syntaxhighlighter::html_highlight;

#[derive(Serialize, Deserialize)]
pub struct Pasta {
    pub id: u64,
    pub content: String,
    pub file: String,
    pub extension: String,
    pub private: bool,
    pub editable: bool,
    pub created: i64,
    pub expiration: i64,
    pub pasta_type: String,
}

impl Pasta {
    pub fn id_as_animals(&self) -> String {
        to_animal_names(self.id)
    }

    pub fn created_as_string(&self) -> String {
        let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.created, 0), Utc);
        format!(
            "{:02}-{:02} {}:{}",
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
            let date =
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.expiration, 0), Utc);
            format!(
                "{:02}-{:02} {}:{}",
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
}

impl fmt::Display for Pasta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
