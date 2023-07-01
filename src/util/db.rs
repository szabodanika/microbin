use crate::{args::ARGS, pasta::Pasta};

pub fn read_all() -> Vec<Pasta> {
    if ARGS.json_db {
        super::db_json::read_all()
    } else {
        super::db_sqlite::read_all()
    }
}

pub fn insert(pastas: Option<&Vec<Pasta>>, pasta: Option<&Pasta>) {
    if ARGS.json_db {
        super::db_json::update_all(pastas.expect("Called insert() without passing Pasta vector"));
    } else {
        super::db_sqlite::insert(pasta.expect("Called insert() without passing new Pasta"));
    }
}

pub fn update(pastas: Option<&Vec<Pasta>>, pasta: Option<&Pasta>) {
    if ARGS.json_db {
        super::db_json::update_all(pastas.expect("Called update() without passing Pasta vector"));
    } else {
        super::db_sqlite::update(pasta.expect("Called insert() without passing Pasta to update"));
    }
}

pub fn update_all(pastas: &Vec<Pasta>) {
    if ARGS.json_db {
        super::db_json::update_all(pastas);
    } else {
        super::db_sqlite::update_all(pastas);
    }
}

pub fn delete(pastas: Option<&Vec<Pasta>>, id: Option<u64>) {
    if ARGS.json_db {
        super::db_json::update_all(pastas.expect("Called delete() without passing Pasta vector"));
    } else {
        super::db_sqlite::delete_by_id(id.expect("Called delete() without passing Pasta id"));
    }
}
