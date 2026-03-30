// DISCLAIMER
// (c) 2024-05-27 Mario Stöckl - derived from the original Microbin Project by Daniel Szabo
use bytesize::ByteSize;
use rusqlite::{params, Connection};

use crate::{args::ARGS, pasta::PastaFile, Pasta};

const CREATE_TABLE_SQL: &str = "
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
        pasta_type TEXT NOT NULL,
        attachments TEXT
    );";

fn open_db() -> Connection {
    Connection::open(format!("{}/database.sqlite", ARGS.data_dir))
        .expect("Failed to open SQLite database!")
}

/// Ensure the attachments column exists — no-op if already present.
fn migrate_attachments_column(conn: &Connection) {
    let _ = conn.execute("ALTER TABLE pasta ADD COLUMN attachments TEXT", params![]);
}

pub fn read_all() -> Vec<Pasta> {
    select_all_from_db()
}

pub fn update_all(pastas: &[Pasta]) {
    rewrite_all_to_db(pastas);
}

pub fn rewrite_all_to_db(pasta_data: &[Pasta]) {
    let conn = open_db();

    conn.execute("DROP TABLE IF EXISTS pasta;", params![])
        .expect("Failed to drop SQLite table for Pasta!");

    conn.execute(CREATE_TABLE_SQL, params![])
        .expect("Failed to create SQLite table for Pasta!");

    for pasta in pasta_data.iter() {
        let attachments_json = pasta
            .attachments
            .as_ref()
            .filter(|a| !a.is_empty())
            .map(|a| serde_json::to_string(a).unwrap_or_default());

        conn.execute(
            "INSERT INTO pasta (
                id, content, file_name, file_size, extension, private,
                read_only, editable, encrypt_server, encrypt_client,
                encrypted_key, created, expiration, last_read,
                read_count, burn_after_reads, pasta_type, attachments
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
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
                attachments_json,
            ],
        )
        .expect("Failed to insert pasta.");
    }
}

pub fn select_all_from_db() -> Vec<Pasta> {
    let conn = open_db();

    conn.execute(CREATE_TABLE_SQL, params![])
        .expect("Failed to create SQLite table for Pasta!");

    migrate_attachments_column(&conn);

    let mut stmt = conn
        .prepare("SELECT * FROM pasta ORDER BY created ASC")
        .expect("Failed to prepare SQL statement to load pastas");

    let pasta_iter = stmt
        .query_map([], |row| {
            let attachments: Option<Vec<PastaFile>> = row
                .get::<_, Option<String>>(17)?
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(Pasta {
                id: row.get(0)?,
                content: row.get(1)?,
                file: if let (Some(file_name), Some(file_size)) =
                    (row.get::<_, Option<String>>(2)?, row.get::<_, Option<u64>>(3)?)
                {
                    if !file_name.is_empty() && file_size != 0 {
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
                attachments,
            })
        })
        .expect("Failed to select Pastas from SQLite database.");

    pasta_iter
        .map(|r| r.expect("Failed to get pasta"))
        .collect::<Vec<Pasta>>()
}

pub fn insert(pasta: &Pasta) {
    let conn = open_db();

    conn.execute(CREATE_TABLE_SQL, params![])
        .expect("Failed to create SQLite table for Pasta!");

    migrate_attachments_column(&conn);

    let attachments_json = pasta
        .attachments
        .as_ref()
        .filter(|a| !a.is_empty())
        .map(|a| serde_json::to_string(a).unwrap_or_default());

    conn.execute(
        "INSERT INTO pasta (
            id, content, file_name, file_size, extension,
            read_only, private, editable, encrypt_server, encrypt_client,
            encrypted_key, created, expiration, last_read,
            read_count, burn_after_reads, pasta_type, attachments
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
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
            attachments_json,
        ],
    )
    .expect("Failed to insert pasta.");
}

pub fn update(pasta: &Pasta) {
    let conn = open_db();

    migrate_attachments_column(&conn);

    let attachments_json = pasta
        .attachments
        .as_ref()
        .filter(|a| !a.is_empty())
        .map(|a| serde_json::to_string(a).unwrap_or_default());

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
            pasta_type = ?17,
            attachments = ?18
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
            attachments_json,
        ],
    )
    .expect("Failed to update pasta.");
}

pub fn delete_by_id(id: u64) {
    let conn = open_db();

    conn.execute(
        "DELETE FROM pasta WHERE id = ?1;",
        params![id],
    )
    .expect("Failed to delete pasta.");
}
