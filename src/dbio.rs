use std::fs::File;
use std::io::{BufReader, BufWriter, Error};
use std::{fmt, io};

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use log::log;

use crate::{to_animal_names, Pasta};

static DATABASE_PATH: &'static str = "pasta_data/database.json";

pub fn save_to_file(pasta_data: &Vec<Pasta>) {
    let mut file = File::create(DATABASE_PATH);
    match file {
        Ok(_) => {
            let mut writer = BufWriter::new(file.unwrap());
            serde_json::to_writer(&mut writer, &pasta_data);
        }
        Err(_) => {
            log::info!("Database file {} not found!", DATABASE_PATH);
            file = File::create(DATABASE_PATH);
            match file {
                Ok(_) => {
                    log::info!("Database file {} created.", DATABASE_PATH);
                    save_to_file(pasta_data);
                }
                Err(err) => {
                    log::error!(
                        "Failed to create database file {}: {}!",
                        &DATABASE_PATH,
                        &err
                    );
                    panic!("Failed to create database file {}: {}!", DATABASE_PATH, err)
                }
            }
        }
    }
}

pub fn load_from_file() -> io::Result<Vec<Pasta>> {
    let mut file = File::open(DATABASE_PATH);
    match file {
        Ok(_) => {
            let mut reader = BufReader::new(file.unwrap());
            let data: Vec<Pasta> = serde_json::from_reader(&mut reader).unwrap();
            Ok(data)
        }
        Err(_) => {
            log::info!("Database file {} not found!", DATABASE_PATH);
            save_to_file(&Vec::<Pasta>::new());

            log::info!("Database file {} created.", DATABASE_PATH);
            load_from_file()
        }
    }
}
