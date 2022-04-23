use std::fmt;
use actix_web::cookie::time::macros::format_description;
use chrono::{Datelike, DateTime, NaiveDateTime, Timelike, Utc};
use serde::Deserialize;
use crate::to_animal_names;

pub struct Pasta {
	pub id: u64,
	pub content: String,
	pub created: i64,
	pub expiration: i64,
	pub pasta_type: String
}

#[derive(Deserialize)]
pub struct PastaFormData {
	pub content: String,
	pub expiration: String
}

impl Pasta {

	pub fn idAsAnimals(&self) -> String {
		to_animal_names(self.id)
	}

	pub fn createdAsString(&self) -> String {
		let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.created, 0), Utc);
		format!(
			"{}-{:02}-{:02} {}:{}",
			date.year(),
			date.month(),
			date.day(),
			date.hour(),
			date.minute(),
		)
	}

	pub fn expirationAsString(&self) -> String {
		let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.expiration, 0), Utc);
		format!(
			"{}-{:02}-{:02} {}:{}",
			date.year(),
			date.month(),
			date.day(),
			date.hour(),
			date.minute(),
		)
	}

}


impl fmt::Display for Pasta {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.content)
	}
}
