// DISCLAIMER
// (c) 2024-05-27 Mario Stöckl - derived from the original Microbin Project by Daniel Szabo
use crate::args::EXPIRATION_OPTIONS;
use crate::pasta::PastaFile;
use crate::util::bip39words::to_bip39_words;
use crate::util::db::insert;
use crate::util::hashids::to_hashids;
use crate::util::misc::{encrypt, encrypt_file, is_valid_url};
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::error::ErrorBadRequest;
use actix_web::{get, web, Error, HttpResponse, Responder};
use askama::Template;

use bytesize::ByteSize;
use futures::TryStreamExt;
use log::warn;

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};



#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    args: &'a ARGS,
    status: String,
    max_expiry_index: usize,
    default_privacy_value: String,
}



#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(
        IndexTemplate {
            args: &ARGS,
            status: String::from(""),
            max_expiry_index: ARGS.max_expiry_index(),
            default_privacy_value: ARGS.default_privacy.clone().unwrap_or_else(|| String::from("unlisted")),
        }
        .render()
        .unwrap(),
    )
}

#[get("/{status}")]
pub async fn index_with_status(param: web::Path<String>) -> HttpResponse {
    let status = param.into_inner();

    return HttpResponse::Ok().content_type("text/html").body(
        IndexTemplate {
            args: &ARGS,
            status,
            max_expiry_index: ARGS.max_expiry_index(),
            default_privacy_value: ARGS.default_privacy.clone().unwrap_or_else(|| String::from("unlisted")),
        }
        .render()
        .unwrap(),
    );
}

pub fn expiration_to_timestamp(expiration: &str, timenow: i64) -> i64 {
    match expiration {
        "1min" => timenow + 60,
        "10min" => timenow + 60 * 10,
        "1hour" => timenow + 60 * 60,
        "24hour" => timenow + 60 * 60 * 24,
        "3days" => timenow + 60 * 60 * 24 * 3,
        "1week" => timenow + 60 * 60 * 24 * 7,
        "1month" => timenow + 60 * 60 * 24 * 30,
        "6months" => timenow + 60 * 60 * 24 * 183,
        "1year" => timenow + 60 * 60 * 24 * 365,
        "2years" => timenow + 60 * 60 * 24 * 365 * 2,
        "4years" => timenow + 60 * 60 * 24 * 365 * 4,
        "8years" => timenow + 60 * 60 * 24 * 365 * 8,
        "16years" => timenow + 60 * 60 * 24 * 365 * 16,
        "never" => {
            if ARGS.eternal_pasta {
                0
            } else {
                timenow + 60 * 60 * 24 * 365 * 100  // 100 years in the future
            }
        }
        _ => {
            log::error!("{}", "Unexpected expiration time!");
            timenow + 60 * 60 * 24 * 7
        }
    }
}

pub fn is_valid_expiration(expiration: &str) -> bool {
    let max_index = ARGS.max_expiry_index();
    if let Some(idx) = EXPIRATION_OPTIONS.iter().position(|&o| o == expiration) {
        idx <= max_index
    } else {
        false
    }
}

pub async fn create(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => {
            log::error!("SystemTime before UNIX EPOCH!");
            0
        }
    } as i64;

    let mut new_pasta = Pasta {
        id: rand::random_range(0..=8589934591),
        content: String::from(""),
        file: None,
        attachments: None,
        extension: String::from(""),
        private: true, 
        readonly: false,
        editable: ARGS.editable,
        encrypt_server: false,
        encrypted_key: Some(String::from("")),
        encrypt_client: false,
        created: timenow,
        read_count: 0,
        burn_after_reads: 0,
        last_read: timenow,
        pasta_type: String::from(""),
        expiration: expiration_to_timestamp(&ARGS.default_expiry, timenow),
    };

    let mut random_key: String = String::from("");
    let mut plain_key: String = String::from("");
    let mut uploader_password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        let field_name = field.name();
        match field_name {
            Some(name) => {
                match name {
                    "uploader_password" => {
                        while let Some(chunk) = field.try_next().await? {
                            uploader_password
                                .push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
                        }
                    }
                    "random_key" => {
                        while let Some(chunk) = field.try_next().await? {
                            random_key = std::str::from_utf8(&chunk).unwrap().to_string();
                        }
                    }
                    "privacy" => {
                        while let Some(chunk) = field.try_next().await? {
                            let privacy = std::str::from_utf8(&chunk).unwrap();
                            new_pasta.private = match privacy {
                                "public" => false,
                                _ => true,
                            };
                            new_pasta.readonly = match privacy {
                                "readonly" => true,
                                _ => false,
                            };
                            new_pasta.encrypt_client = match privacy {
                                "secret" => true,
                                _ => false,
                            };
                            new_pasta.encrypt_server = match privacy {
                                "private" => true,
                                "secret" => true,
                                _ => false,
                            };
                        }
                    }
                    "plain_key" => {
                        while let Some(chunk) = field.try_next().await? {
                            plain_key = std::str::from_utf8(&chunk).unwrap().to_string();
                        }
                    }
                    "encrypted_random_key" => {
                        while let Some(chunk) = field.try_next().await? {
                            new_pasta.encrypted_key =
                                Some(std::str::from_utf8(&chunk).unwrap().to_string());
                        }
                    }
                    "expiration" => {
                        let mut expiration_buf = String::new();
                        while let Some(chunk) = field.try_next().await? {
                            expiration_buf
                                .push_str(std::str::from_utf8(&chunk).unwrap());
                        }
                        if !is_valid_expiration(&expiration_buf) {
                            return Err(ErrorBadRequest("Expiration exceeds server maximum."));
                        }
                        new_pasta.expiration =
                            expiration_to_timestamp(&expiration_buf, timenow);
                    }
                    "burn_after" => {
                        while let Some(chunk) = field.try_next().await? {
                            new_pasta.burn_after_reads = match std::str::from_utf8(&chunk).unwrap() {
                                "1" => 1,
                                "10" => 10,
                                "100" => 100,
                                "1000" => 1000,
                                "10000" => 10000,
                                "0" => 0,
                                _ => {
                                    log::error!("{}", "Unexpected burn after value!");
                                    0
                                }
                            };
                        }
                    }
                    "content" => {
                        let mut content = String::from("");
                        while let Some(chunk) = field.try_next().await? {
                            content.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
                        }
                        if !content.is_empty() {
                            new_pasta.content = content;
        
                            new_pasta.pasta_type = if is_valid_url(new_pasta.content.as_str()) {
                                String::from("url")
                            } else {
                                String::from("text")
                            };
                        }
                    }
                    "syntax_highlight" => {
                        while let Some(chunk) = field.try_next().await? {
                            new_pasta.extension = std::str::from_utf8(&chunk).unwrap().to_string();
                        }
                    }
                    "file" => {
                        if ARGS.no_file_upload {
                            continue;
                        }
        
                        let path = field
                            .content_disposition()
                            .and_then(|cd| cd.get_filename())
                            .map(|f| f.to_owned());
        
                        let path = match path {
                            Some(ref p) if p.is_empty() => continue,
                            Some(p) => p,
                            None => continue,
                        };
        
                        let mut file = match PastaFile::from_unsanitized(&path) {
                            Ok(f) => f,
                            Err(e) => {
                                warn!("Unsafe file name: {e:?}");
                                continue;
                            }
                        };

                        // Deduplicate: if a file with the same sanitized name was
                        // already added, auto-rename to avoid overwriting it on disk.
                        {
                            let existing: Vec<&str> = new_pasta.file.iter().map(|f| f.name())
                                .chain(new_pasta.attachments.iter().flatten().map(|f| f.name()))
                                .collect();
                            if existing.contains(&file.name()) {
                                let name = file.name.clone();
                                let (stem, ext) = match name.rfind('.') {
                                    Some(pos) => (&name[..pos], &name[pos..]),
                                    None => (name.as_str(), ""),
                                };
                                let mut counter = 1u32;
                                loop {
                                    let candidate = format!("{} ({}){}", stem, counter, ext);
                                    if !existing.contains(&candidate.as_str()) {
                                        file.name = candidate;
                                        break;
                                    }
                                    counter += 1;
                                }
                            }
                        }

                        std::fs::create_dir_all(format!(
                            "{}/attachments/{}",
                            ARGS.data_dir,
                            &new_pasta.id_as_words()
                        ))
                        .unwrap();
        
                        let filepath = format!(
                            "{}/attachments/{}/{}",
                            ARGS.data_dir,
                            &new_pasta.id_as_words(),
                            &file.name()
                        );
        
                        let mut f = web::block(|| std::fs::File::create(filepath)).await??;
                        let mut size = 0;
                        while let Some(chunk) = field.try_next().await? {
                            size += chunk.len();
                            if (new_pasta.encrypt_server
                                && size > ARGS.max_file_size_encrypted_mb * 1024 * 1024)
                                || size > ARGS.max_file_size_unencrypted_mb * 1024 * 1024
                            {
                                return Err(ErrorBadRequest("File exceeded size limit."));
                            }
                            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                        }
        
                        file.size = ByteSize::b(size as u64);
        
                        if new_pasta.file.is_none() {
                            new_pasta.file = Some(file);
                        } else {
                            new_pasta.attachments
                                .get_or_insert_with(Vec::new)
                                .push(file);
                        }
                        new_pasta.pasta_type = String::from("text");
                    }
                    unknown_field => {
                        log::error!("Unexpected multipart field: {:?}", unknown_field);
                    }
                }
            }
            None => {
                log::error!("Field name is None");
                continue;
            }
        }
    }

    if ARGS.readonly && ARGS.uploader_password.is_some() {
        if uploader_password != ARGS.uploader_password.as_ref().unwrap().to_owned() {
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/incorrect"))
                .finish());
        }
    }

    // Perform all encryption before acquiring the lock — crypto and file I/O
    // only touch new_pasta (local) and the filesystem, not shared state.
    let id = new_pasta.id;

    if plain_key != *"" && new_pasta.readonly {
        new_pasta.encrypted_key = Some(encrypt(id.to_string().as_str(), &plain_key));
    }

    if new_pasta.encrypt_server && !new_pasta.readonly && new_pasta.content != *"" {
        if new_pasta.encrypt_client {
            new_pasta.content = encrypt(&new_pasta.content, &random_key);
        } else {
            new_pasta.content = encrypt(&new_pasta.content, &plain_key);
        }
    }

    if new_pasta.file.is_some() && new_pasta.encrypt_server && !new_pasta.readonly {
        let filepath = format!(
            "{}/attachments/{}/{}",
            ARGS.data_dir,
            &new_pasta.id_as_words(),
            &new_pasta.file.as_ref().unwrap().name()
        );
        if new_pasta.encrypt_client {
            encrypt_file(&random_key, &filepath).expect("Failed to encrypt file with random key")
        } else {
            encrypt_file(&plain_key, &filepath).expect("Failed to encrypt file with plain key")
        }
    }

    if new_pasta.encrypt_server && !new_pasta.readonly {
        if let Some(ref attachments) = new_pasta.attachments {
            for attachment in attachments {
                let filepath = format!(
                    "{}/attachments/{}/{}",
                    ARGS.data_dir,
                    &new_pasta.id_as_words(),
                    attachment.name()
                );
                if new_pasta.encrypt_client {
                    encrypt_file(&random_key, &filepath)
                        .expect("Failed to encrypt attachment with random key");
                } else {
                    encrypt_file(&plain_key, &filepath)
                        .expect("Failed to encrypt attachment with plain key");
                }
            }
        }
    }

    let encrypt_server = new_pasta.encrypt_server;

    // Now acquire the lock — only needed to mutate shared in-memory state and DB
    let mut pastas = data.pastas.lock().unwrap();

    pastas.push(new_pasta);

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            insert(Some(&pastas), Some(pasta));
        }
    }

    let slug = if ARGS.hash_ids {
        to_hashids(id)
    } else {
        to_bip39_words(id)
    };

    if encrypt_server {
        Ok(HttpResponse::Found()
            .append_header(("Location", format!("/auth/{}/success", slug)))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .append_header((
                "Location",
                format!("{}/upload/{}", ARGS.public_path_as_str(), slug),
            ))
            .finish())
    }
}
