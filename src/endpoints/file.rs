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
    // Resolve ID and read the request body BEFORE locking — body reading is
    // async I/O and must not hold the shared-state mutex.
    let id_intern = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let password = auth::password_from_multipart(payload).await?;

    // Collect the enc path and filename under the lock, then drop it before
    // any blocking filesystem I/O (File::open + decrypt_file reads the file).
    let file_info: Option<(String, String, String)> = {
        let mut pastas = data.pastas.lock().unwrap();
        remove_expired(&mut pastas);

        let fname = query.get("fname").cloned();

        pastas.iter().find(|p| p.id == id_intern).and_then(|pasta| {
            let pasta_file = if let Some(ref fname) = fname {
                if pasta.file.as_ref().map(|f| f.name() == fname).unwrap_or(false) {
                    pasta.file.as_ref()
                } else {
                    pasta.attachments.as_ref()
                        .and_then(|a| a.iter().find(|f| f.name() == fname))
                }
            } else {
                pasta.file.as_ref()
            };

            pasta_file.and_then(|pf| {
                enc_file_path(&ARGS.data_dir, &pasta.id_as_words(), pf.name()).map(|enc_path| {
                    let content_type = mime_guess::from_path(pf.name())
                        .first_or_octet_stream()
                        .to_string();
                    (enc_path, pf.name().to_string(), content_type)
                })
            })
        })
    }; // lock dropped here

    match file_info {
        None => Ok(HttpResponse::NotFound().finish()),
        Some((enc_path, filename, content_type)) => {
            let file = File::open(&enc_path)?;
            let decrypted_data: Vec<u8> = decrypt_file(&password, &file)?;
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", filename),
                ))
                .body(decrypted_data))
        }
    }
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
        let preview_requested = query.get("preview").map(|v| v == "true").unwrap_or(false);

        // Collect path, filename, and disposition info under the lock, then
        // drop it before the blocking NamedFile::open syscall.
        let file_info: Option<(PathBuf, String, bool)> = {
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

            pasta_file.map(|pf| {
                let file_path = PathBuf::from(format!(
                    "{}/attachments/{}/{}",
                    ARGS.data_dir,
                    pastas[index].id_as_words(),
                    pf.name()
                ));
                // Only allow inline for raster image/* and video/* — SVG and
                // arbitrary content served inline is a stored-XSS vector.
                let ct = mime_guess::from_path(pf.name()).first_or_octet_stream();
                let use_inline = preview_requested
                    && ((ct.type_().as_str() == "image"
                        && ct.subtype().as_str() != "svg+xml"
                        && ct.subtype().as_str() != "svg")
                        || ct.type_().as_str() == "video");
                (file_path, pf.name().to_string(), use_inline)
            })
        };
        drop(pastas); // release lock before I/O

        if let Some((file_path, filename, use_inline)) = file_info {
            let disposition_type = if use_inline {
                header::DispositionType::Inline
            } else {
                header::DispositionType::Attachment
            };
            let file_response = actix_files::NamedFile::open(file_path)?;
            let file_response = file_response.set_content_disposition(header::ContentDisposition {
                disposition: disposition_type,
                parameters: vec![header::DispositionParam::Filename(filename)],
            });
            let mut response = file_response.into_response(&request);
            response.headers_mut().insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            return Ok(response);
        }
    }

    Ok(HttpResponse::NotFound().finish())
}
