use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};

use crate::Pasta;

static DATABASE_PATH: &str = "pasta_data/database.json";

pub fn read_all() -> Vec<Pasta> {
    load_from_file().expect("Failed to load pastas from JSON")
}

pub fn update_all(pastas: &Vec<Pasta>) {
    save_to_file(pastas);
}

fn save_to_file(pasta_data: &Vec<Pasta>) {
    let mut file = File::create(DATABASE_PATH);
    match file {
        Ok(_) => {
            let writer = BufWriter::new(file.unwrap());
            serde_json::to_writer(writer, &pasta_data).expect("Failed to create JSON writer");
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

fn load_from_file() -> io::Result<Vec<Pasta>> {
    let file = File::open(DATABASE_PATH);
    match file {
        Ok(_) => {
            let reader = BufReader::new(file.unwrap());
            let data: Vec<Pasta> = match serde_json::from_reader(reader) {
                Ok(t) => t,
                _ => Vec::new(),
            };
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
