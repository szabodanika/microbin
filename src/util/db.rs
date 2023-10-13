use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use crate::{args::ARGS, pasta::Pasta};

pub fn read_all() -> Result<Vec<Pasta>, DbError> {
    let pastas = match ARGS.json_db {
        true => super::db_json::read_all(),
        false => super::db_sqlite::read_all()?
    };
    Ok(pastas)
}

pub fn insert(pastas: Option<&Vec<Pasta>>, pasta: Option<&Pasta>) -> Result<(), DbError> {
    if ARGS.json_db {
        super::db_json::update_all(pastas.expect("Called insert() without passing Pasta vector"));
    } else {
        super::db_sqlite::insert(pasta.expect("Called insert() without passing new Pasta"))?;
    }
    Ok(())
}

pub fn update(pastas: Option<&Vec<Pasta>>, pasta: Option<&Pasta>) -> Result<(), DbError> {
    if ARGS.json_db {
        super::db_json::update_all(pastas.expect("Called update() without passing Pasta vector"));
    } else {
        super::db_sqlite::update(pasta.expect("Called insert() without passing Pasta to update"))?;
    }
    Ok(())
}

pub fn update_all(pastas: &Vec<Pasta>) -> Result<(), DbError> {
    if ARGS.json_db {
        super::db_json::update_all(pastas);
    } else {
        super::db_sqlite::update_all(pastas)?;
    }
    Ok(())
}

pub fn delete(pastas: Option<&Vec<Pasta>>, id: Option<u64>) -> Result<(), DbError> {
    if ARGS.json_db {
        super::db_json::update_all(pastas.expect("Called delete() without passing Pasta vector"));
    } else {
        super::db_sqlite::delete_by_id(id.expect("Called delete() without passing Pasta id"))?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct DbError {
    sql_error: Option<Box<rusqlite::Error>>
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.sql_error.as_ref() {
            None => write!(f, "{}", "Unknown database error occoured"),
            Some(e) => write!(f, "SQL database error => {}", e)
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.sql_error.as_ref().map(|e| &**e as &(dyn Error + 'static))
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(value: rusqlite::Error) -> Self {
        DbError {sql_error: Some(Box::new(value))}
    }
}