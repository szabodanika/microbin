// DISCLAIMER
// (c) 2024-05-27 Mario Stöckl - derived from the original Microbin Project by Daniel Szabo
use std::fs::File;
use std::path::PathBuf;

use crate::args::ARGS;
use crate::util::auth;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::util::{bip39words::to_u64, misc::decrypt_file};
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::http::header;
use actix_web::{get, post, web, Error, HttpResponse};

fn enc_file_path(data_dir: &str, id_as_words: &str, filename: &str) -> Option<String> {
    let new_path = format!("{}/attachments/{}/{}.enc", data_dir, id_as_words, filename);
    let legacy_path = format!("{}/attachments/{}/data.enc", data_dir, id_as_words);
    if std::path::Path::new(&new_path).exists() {
        Some(new_path)
    } else if std::path::Path::new(&legacy_path).exists() {
        Some(legacy_path)
    } else {
        None
    }
}

#[post("/secure_file/{id}")]
pub async fn post_secure_file(
    data: web::Data<AppState>,
    id: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            index = i;
            found = true;
            break;
        }
    }

    let password = auth::password_from_multipart(payload).await?;

    if found {
        let fname = query.get("fname").cloned();

        // Determine which file to serve
        let pasta_file = if let Some(ref fname) = fname {
            // Check primary file
            if pastas[index].file.as_ref().map(|f| f.name() == fname).unwrap_or(false) {
                pastas[index].file.as_ref()
            } else {
                // Check attachments
                pastas[index].attachments.as_ref()
                    .and_then(|a| a.iter().find(|f| f.name() == fname))
            }
        } else {
            pastas[index].file.as_ref()
        };

        if let Some(pasta_file) = pasta_file {
            let enc_path = match enc_file_path(
                &ARGS.data_dir,
                &pastas[index].id_as_words(),
                pasta_file.name(),
            ) {
                Some(p) => p,
                None => return Ok(HttpResponse::NotFound().finish()),
            };
            let file = File::open(&enc_path)?;
            let decrypted_data: Vec<u8> = decrypt_file(&password, &file)?;

            let content_type = mime_guess::from_path(pasta_file.name())
                .first_or_octet_stream()
                .to_string();

            return Ok(HttpResponse::Ok()
                .content_type(content_type)
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", pasta_file.name()),
                ))
                .body(decrypted_data));
        }
    }
    Ok(HttpResponse::NotFound().finish())
}

#[get("/file/{id}")]
pub async fn get_file(
    request: actix_web::HttpRequest,
    id: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();

    let id_intern = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id_intern {
            index = i;
            found = true;
            break;
        }
    }

    if found {
        if pastas[index].encrypt_server {
            let mut location = format!("/auth_file/{}", pastas[index].id_as_words());
            if let Some(qs) = request.uri().query() {
                if !qs.is_empty() {
                    location.push('?');
                    location.push_str(qs);
                }
            }
            return Ok(HttpResponse::Found()
                .append_header(("Location", location))
                .finish());
        }

        let fname = query.get("fname").cloned();
        let inline = query.get("preview").map(|v| v == "true").unwrap_or(false);

        // Determine which file to serve
        let pasta_file = if let Some(ref fname) = fname {
            if pastas[index].file.as_ref().map(|f| f.name() == fname).unwrap_or(false) {
                pastas[index].file.as_ref()
            } else {
                pastas[index].attachments.as_ref()
                    .and_then(|a| a.iter().find(|f| f.name() == fname))
            }
        } else {
            pastas[index].file.as_ref()
        };

        if let Some(pasta_file) = pasta_file {
            let file_path = PathBuf::from(format!(
                "{}/attachments/{}/{}",
                ARGS.data_dir,
                pastas[index].id_as_words(),
                pasta_file.name()
            ));

            let disposition_type = if inline {
                header::DispositionType::Inline
            } else {
                header::DispositionType::Attachment
            };

            let file_response = actix_files::NamedFile::open(file_path)?;
            let file_response = file_response.set_content_disposition(header::ContentDisposition {
                disposition: disposition_type,
                parameters: vec![header::DispositionParam::Filename(
                    pasta_file.name().to_string(),
                )],
            });
            return Ok(file_response.into_response(&request));
        }
    }

    Ok(HttpResponse::NotFound().finish())
}
