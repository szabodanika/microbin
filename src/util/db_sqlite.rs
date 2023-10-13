use bytesize::ByteSize;
use rusqlite::{params, Connection, Error};

use crate::{args::ARGS, pasta::PastaFile, Pasta};

pub fn get_connection() -> Result<Connection, Error> {
    Connection::open(format!("{}/database.sqlite", ARGS.data_dir))
}

pub fn init_db() -> Result<(), Error> {
    let query_create_db =         "
        CREATE TABLE IF NOT EXISTS pasta (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            file_name TEXT,
            file_size INTEGER,
            extension TEXT NOT NULL,
            read_only INTEGER NOT NULL,
            private INTEGER NOT NULL,
            editable INTEGER NOT NULL,
            encrypt_server INTEGER NOT NULL,
            encrypt_client INTEGER NOT NULL,
            encrypted_key TEXT,
            created INTEGER NOT NULL,
            expiration INTEGER NOT NULL,
            last_read INTEGER NOT NULL,
            read_count INTEGER NOT NULL,
            burn_after_reads INTEGER NOT NULL,
            pasta_type TEXT NOT NULL
        );";
    let conn = get_connection()?;
    conn.execute(query_create_db, params![])?;
    match conn.close() {
        Ok(_) => Ok(()),
        Err(c) => Err(c.1)
    }
}

pub fn recreate_db() -> Result<(), Error> {
    let query_drop = "
        DROP TABLE IF EXISTS pasta;
        );";
    let conn = get_connection()?;
    conn.execute(query_drop, params![])?;
    match conn.close() {
        Ok(_) => { },
        Err(c) => return Err(c.1)
    }
    init_db()?;
    Ok(())
}

pub fn read_all() -> Result<Vec<Pasta>, Error> {
    select_all_from_db()
}

pub fn update_all(pastas: &[Pasta]) -> Result<(), Error> {
    rewrite_all_to_db(pastas)?;
    Ok(())
}

pub fn rewrite_all_to_db(pasta_data: &[Pasta]) -> Result<(), Error> {
    recreate_db()?;
    let conn = get_connection()?;


    for pasta in pasta_data.iter() {
        conn.execute(
            "INSERT INTO pasta (
                id,
                content,
                file_name,
                file_size,
                extension,
                private,
                read_only,
                editable,
                encrypt_server,
                encrypt_client,
                encrypted_key,
                created,
                expiration,
                last_read,
                read_count,
                burn_after_reads,
                pasta_type
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                pasta.id,
                pasta.content,
                pasta.file.as_ref().map_or("", |f| f.name.as_str()),
                pasta.file.as_ref().map_or(0, |f| f.size.as_u64()),
                pasta.extension,
                pasta.private as i32,
                pasta.readonly as i32,
                pasta.editable as i32,
                pasta.encrypt_server as i32,
                pasta.encrypt_client as i32,
                pasta.encrypted_key.as_deref(),
                pasta.created,
                pasta.expiration,
                pasta.last_read,
                pasta.read_count,
                pasta.burn_after_reads,
                pasta.pasta_type,
            ],
        )?;
    }
    match conn.close() {
        Ok(_) => Ok(()),
        Err(c) => Err(c.1)
    }
}

pub fn select_all_from_db() -> Result<Vec<Pasta>, Error> {
    let conn = get_connection()?;

    let mut stmt = conn
        .prepare("SELECT * FROM pasta ORDER BY created ASC")?;

    let pasta_iter = stmt
        .query_map([], |row| {
            Ok(Pasta {
                id: row.get(0)?,
                content: row.get(1)?,
                file: if let (Some(file_name), Some(file_size)) = (row.get(2)?, row.get(3)?) {
                    let file_size: u64 = file_size;
                    if file_name != "" && file_size != 0 {
                        Some(PastaFile {
                            name: file_name,
                            size: ByteSize::b(file_size),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                },
                extension: row.get(4)?,
                readonly: row.get(5)?,
                private: row.get(6)?,
                editable: row.get(7)?,
                encrypt_server: row.get(8)?,
                encrypt_client: row.get(9)?,
                encrypted_key: row.get(10)?,
                created: row.get(11)?,
                expiration: row.get(12)?,
                last_read: row.get(13)?,
                read_count: row.get(14)?,
                burn_after_reads: row.get(15)?,
                pasta_type: row.get(16)?,
            })
        })?;

    let pastas_result: Result<Vec<Pasta>, Error> = pasta_iter.collect();
    let pastas = match pastas_result {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    Ok(pastas)
}

pub fn insert(pasta: &Pasta) -> Result<(), Error> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO pasta (
                id,
                content,
                file_name,
                file_size,
                extension,
                read_only,
                private,
                editable,
                encrypt_server,
                encrypt_client,
                encrypted_key,
                created,
                expiration,
                last_read,
                read_count,
                burn_after_reads,
                pasta_type
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            pasta.id,
            pasta.content,
            pasta.file.as_ref().map_or("", |f| f.name.as_str()),
            pasta.file.as_ref().map_or(0, |f| f.size.as_u64()),
            pasta.extension,
            pasta.readonly as i32,
            pasta.private as i32,
            pasta.editable as i32,
            pasta.encrypt_server as i32,
            pasta.encrypt_client as i32,
            pasta.encrypted_key.as_deref(),
            pasta.created,
            pasta.expiration,
            pasta.last_read,
            pasta.read_count,
            pasta.burn_after_reads,
            pasta.pasta_type,
        ],
    )?;
    match conn.close() {
        Ok(_) => Ok(()),
        Err(c) => Err(c.1)
    }
}

pub fn update(pasta: &Pasta) -> Result<(), Error> {
    let conn = Connection::open(format!("{}/database.sqlite", ARGS.data_dir))
        .expect("Failed to open SQLite database!");

    conn.execute(
        "UPDATE pasta SET
            content = ?2,
            file_name = ?3,
            file_size = ?4,
            extension = ?5,
            read_only = ?6,
            private = ?7,
            editable = ?8,
            encrypt_server = ?9,
            encrypt_client = ?10,
            encrypted_key = ?11,
            created = ?12,
            expiration = ?13,
            last_read = ?14,
            read_count = ?15,
            burn_after_reads = ?16,
            pasta_type = ?17
        WHERE id = ?1;",
        params![
            pasta.id,
            pasta.content,
            pasta.file.as_ref().map_or("", |f| f.name.as_str()),
            pasta.file.as_ref().map_or(0, |f| f.size.as_u64()),
            pasta.extension,
            pasta.readonly as i32,
            pasta.private as i32,
            pasta.editable as i32,
            pasta.encrypt_server as i32,
            pasta.encrypt_client as i32,
            pasta.encrypted_key.as_deref(),
            pasta.created,
            pasta.expiration,
            pasta.last_read,
            pasta.read_count,
            pasta.burn_after_reads,
            pasta.pasta_type,
        ],
    )?;
    match conn.close() {
        Ok(_) => Ok(()),
        Err(c) => Err(c.1)
    }
}

pub fn delete_by_id(id: u64) -> Result<(), Error> {
    let conn = get_connection()?;

    conn.execute(
        "DELETE FROM pasta 
        WHERE id = ?1;",
        params![id],
    )?;
    match conn.close() {
        Ok(_) => Ok(()),
        Err(c) => Err(c.1)
    }
}
