use crate::pasta::PastaFile;
use crate::util::animalnumbers::to_animal_names;
use crate::util::db::{insert, update};
use crate::util::hashids::to_hashids;
use crate::util::misc::{encrypt, encrypt_file, is_valid_url, remove_expired};
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use bytes::BytesMut;
use bytesize::ByteSize;
use futures::TryStreamExt;
use rand::Rng;
use serde::Serialize;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::endpoints::create::expiration_to_timestamp;

#[derive(Serialize)]
struct ApiPasta {
    id: String,
    content: String,
    pasta_type: String,
    expiration: String,
    created: String,
    read_count: u64,
    burn_after_reads: u64,
    private: bool,
    readonly: bool,
    editable: bool,
    encrypt_server: bool,
    encrypt_client: bool,
    has_file: bool,
    file_name: Option<String>,
    file_size: Option<String>,
    url: String,
    raw_url: String,
}

#[derive(Serialize)]
struct ApiCreateResponse {
    id: String,
    url: String,
    raw_url: String,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn pasta_to_api(pasta: &Pasta) -> ApiPasta {
    let slug = pasta.id_as_animals();
    let base = ARGS.public_path_as_str();
    ApiPasta {
        id: slug.clone(),
        content: if pasta.encrypt_server { String::from("[encrypted]") } else { pasta.content.clone() },
        pasta_type: pasta.pasta_type.clone(),
        expiration: pasta.expiration_as_string(),
        created: pasta.created_as_string(),
        read_count: pasta.read_count,
        burn_after_reads: pasta.burn_after_reads,
        private: pasta.private,
        readonly: pasta.readonly,
        editable: pasta.editable,
        encrypt_server: pasta.encrypt_server,
        encrypt_client: pasta.encrypt_client,
        has_file: pasta.file.is_some(),
        file_name: pasta.file.as_ref().map(|f| f.name().to_string()),
        file_size: pasta.file.as_ref().map(|f| f.size.to_string()),
        url: format!("{}/upload/{}", base, slug),
        raw_url: format!("{}/raw/{}", base, slug),
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn json_error(status: actix_web::http::StatusCode, msg: &str) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("application/json")
        .json(ApiError { error: msg.to_string() })
}

#[get("/api/list")]
pub async fn api_list(data: web::Data<AppState>) -> HttpResponse {
    if ARGS.no_listing {
        return json_error(actix_web::http::StatusCode::FORBIDDEN, "Listing is disabled");
    }

    let mut pastas = data.pastas.lock().unwrap();
    remove_expired(&mut pastas);
    pastas.sort_by(|a, b| b.created.cmp(&a.created));

    let result: Vec<ApiPasta> = pastas
        .iter()
        .filter(|p| !p.private)
        .map(pasta_to_api)
        .collect();

    HttpResponse::Ok().json(result)
}

#[get("/api/pasta/{id}")]
pub async fn api_get_pasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id_val = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    let index = pastas.iter().position(|p| p.id == id_val);

    match index {
        Some(i) => {
            if pastas[i].encrypt_server {
                return json_error(
                    actix_web::http::StatusCode::FORBIDDEN,
                    "This paste is encrypted. Use the web interface to decrypt it.",
                );
            }

            pastas[i].read_count += 1;
            pastas[i].last_read = now_secs();
            update(Some(&pastas), Some(&pastas[i]));

            let result = pasta_to_api(&pastas[i]);
            HttpResponse::Ok().json(result)
        }
        None => json_error(actix_web::http::StatusCode::NOT_FOUND, "Paste not found"),
    }
}

#[post("/api/create")]
pub async fn api_create(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();
    let timenow = now_secs();

    let mut new_pasta = Pasta {
        id: rand::thread_rng().gen::<u16>() as u64,
        content: String::from(""),
        file: None,
        extension: String::from(""),
        private: false,
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

    let mut plain_key = String::from("");
    let mut uploader_password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        let Some(field_name) = field.name() else {
            continue;
        };
        match field_name {
            "uploader_password" => {
                while let Some(chunk) = field.try_next().await? {
                    uploader_password
                        .push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
                }
            }
            "content" => {
                let mut buf = BytesMut::new();
                while let Some(chunk) = field.try_next().await? {
                    buf.extend_from_slice(&chunk);
                }
                if !buf.is_empty() {
                    new_pasta.content = String::from_utf8(buf.to_vec())
                        .unwrap_or_default();
                    new_pasta.pasta_type = if is_valid_url(new_pasta.content.as_str()) {
                        String::from("url")
                    } else {
                        String::from("text")
                    };
                }
            }
            "expiration" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.expiration =
                        expiration_to_timestamp(std::str::from_utf8(&chunk).unwrap(), timenow);
                }
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
                        _ => 0,
                    };
                }
            }
            "privacy" => {
                while let Some(chunk) = field.try_next().await? {
                    let privacy = std::str::from_utf8(&chunk).unwrap();
                    new_pasta.private = privacy != "public";
                    new_pasta.readonly = privacy == "readonly";
                    new_pasta.encrypt_server = privacy == "private";
                }
            }
            "plain_key" => {
                while let Some(chunk) = field.try_next().await? {
                    plain_key = std::str::from_utf8(&chunk).unwrap().to_string();
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

                let path = field.content_disposition().and_then(|cd| cd.get_filename());
                let path = match path {
                    Some("") => continue,
                    Some(p) => p,
                    None => continue,
                };

                let mut file = match PastaFile::from_unsanitized(path) {
                    Ok(f) => f,
                    Err(e) => {
                        log::warn!("Unsafe file name: {e:?}");
                        continue;
                    }
                };

                std::fs::create_dir_all(format!(
                    "{}/attachments/{}",
                    ARGS.data_dir,
                    &new_pasta.id_as_animals()
                ))
                .unwrap();

                let filepath = format!(
                    "{}/attachments/{}/{}",
                    ARGS.data_dir,
                    &new_pasta.id_as_animals(),
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
                        return Ok(json_error(
                            actix_web::http::StatusCode::BAD_REQUEST,
                            "File exceeded size limit",
                        ));
                    }
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                file.size = ByteSize::b(size as u64);
                new_pasta.file = Some(file);
                new_pasta.pasta_type = String::from("text");
            }
            _ => {}
        }
    }

    // Check uploader password if required
    if ARGS.readonly && ARGS.uploader_password.is_some() {
        if uploader_password.trim() != ARGS.uploader_password.as_ref().unwrap().trim() {
            return Ok(json_error(
                actix_web::http::StatusCode::UNAUTHORIZED,
                "Invalid uploader password",
            ));
        }
    }

    if new_pasta.content.is_empty() && new_pasta.file.is_none() {
        return Ok(json_error(
            actix_web::http::StatusCode::BAD_REQUEST,
            "Content or file is required",
        ));
    }

    let id = new_pasta.id;

    // Server-side encryption with plain key
    if new_pasta.encrypt_server && !new_pasta.readonly && !plain_key.is_empty() {
        if !new_pasta.content.is_empty() {
            new_pasta.content = encrypt(&new_pasta.content, &plain_key);
        }
        if new_pasta.file.is_some() {
            let filepath = format!(
                "{}/attachments/{}/{}",
                ARGS.data_dir,
                &new_pasta.id_as_animals(),
                &new_pasta.file.as_ref().unwrap().name()
            );
            encrypt_file(&plain_key, &filepath).expect("Failed to encrypt file");
        }
        if new_pasta.readonly {
            new_pasta.encrypted_key = Some(encrypt(id.to_string().as_str(), &plain_key));
        }
    }

    pastas.push(new_pasta);

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            insert(Some(&pastas), Some(pasta));
        }
    }

    let slug = if ARGS.hash_ids {
        to_hashids(id)
    } else {
        to_animal_names(id)
    };

    let base = ARGS.public_path_as_str();

    Ok(HttpResponse::Created().json(ApiCreateResponse {
        id: slug.clone(),
        url: format!("{}/upload/{}", base, slug),
        raw_url: format!("{}/raw/{}", base, slug),
    }))
}
