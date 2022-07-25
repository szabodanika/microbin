use std::borrow::{Cow, Borrow};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::sync::Mutex;
use std::ops::Deref;
use actix_web::web::Json;

use crate::Pasta;

pub trait DataStore {
    fn remove_expired(&self);
    fn edit(&self, id: u64, content: String);
    fn create(&self, pasta: Pasta);
    fn get_pasta(&self, id: u64) -> Option<Pasta>;
    fn get_pastalist(&self) -> Box<dyn Deref<Target=Vec<Pasta>> + '_>;
}


pub struct JsonStore {
    pub pastas: Mutex<Vec<Pasta>>
}

impl Default for JsonStore {
    fn default() -> Self {
        Self{pastas: Mutex::new(vec!())}
    }
}

impl DataStore for JsonStore {
    fn remove_expired(&self) {
        
    }

    fn edit(&self, id: u64, content: String) {
        todo!()
    }

    fn create(&self, pasta: Pasta) {
        let mut pastas = self.pastas.lock().unwrap();
        pastas.push(pasta);
        save_to_file(&pastas);
    }

    fn get_pasta(&self, id: u64) -> Option<Pasta> {
        let pastas = self.pastas.lock().unwrap();
        for pasta in pastas.iter() {
            if pasta.id == id {
                return Some(pasta.clone());
            }
        }
        None
    }

    fn get_pastalist(&self) -> Box<dyn Deref<Target=Vec<Pasta>> + '_>{
        return Box::new(self.pastas.lock().unwrap());
    }
}


static DATABASE_PATH: &'static str = "pasta_data/database.json";

pub fn save_to_file(pasta_data: &Vec<Pasta>) {
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

pub fn load_from_file() -> io::Result<Vec<Pasta>> {
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
