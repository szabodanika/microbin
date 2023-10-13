use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use futures::TryStreamExt;

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::PastaFile;
use crate::util::animalnumbers::to_u64;
use crate::util::db::delete;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{decrypt, remove_expired};
use crate::AppState;
use askama::Template;
use std::fs;
use actix_web::error::ErrorInternalServerError;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            // if it's encrypted or read-only, it needs password to be deleted
            if pasta.encrypt_server || pasta.readonly {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/auth_remove_private/{}", pasta.id_as_animals()),
                    ))
                    .finish());
            }

            // remove the file itself
            if let Some(PastaFile { name, .. }) = &pasta.file {
                if fs::remove_file(format!(
                    "./{}/attachments/{}/{}",
                    ARGS.data_dir,
                    pasta.id_as_animals(),
                    name
                ))
                .is_err()
                {
                    log::error!("Failed to delete file {}!", name)
                }

                // and remove the containing directory
                if fs::remove_dir(format!(
                    "./{}/attachments/{}/",
                    ARGS.data_dir,
                    pasta.id_as_animals()
                ))
                .is_err()
                {
                    log::error!("Failed to delete directory {}!", name)
                }
            }

            // remove it from in-memory pasta list
            pastas.remove(i);

            if let Err(error) = delete(Some(&pastas), Some(id)) {
                log::error!("Failed to delete pasta with id {} => {}", id, error);
                return Err(ErrorInternalServerError("Database delete error"));
            }

            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/list", ARGS.public_path_as_str())))
                .finish());
        }
    }

    remove_expired(&mut pastas);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}

#[post("/remove/{id}")]
pub async fn post_remove(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password = std::str::from_utf8(&chunk).unwrap().to_string();
            }
        }
    }

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            if pastas[i].readonly || pastas[i].encrypt_server {
                if password != *"" {
                    let res = decrypt(pastas[i].content.to_owned().as_str(), &password);
                    if res.is_ok() {
                        // remove the file itself
                        if let Some(PastaFile { name, .. }) = &pasta.file {
                            if fs::remove_file(format!(
                                "./{}/attachments/{}/{}",
                                ARGS.data_dir,
                                pasta.id_as_animals(),
                                name
                            ))
                            .is_err()
                            {
                                log::error!("Failed to delete file {}!", name)
                            }

                            // and remove the containing directory
                            if fs::remove_dir(format!(
                                "./{}/attachments/{}/",
                                ARGS.data_dir,
                                pasta.id_as_animals()
                            ))
                            .is_err()
                            {
                                log::error!("Failed to delete directory {}!", name)
                            }
                        }

                        // remove it from in-memory pasta list
                        pastas.remove(i);
                        if let Err(error) = delete(Some(&pastas), Some(id)) {
                            log::error!("Failed to delete pasta with id {} => {}", id, error);
                            return Err(ErrorInternalServerError("Database delete error"));
                        }

                        return Ok(HttpResponse::Found()
                            .append_header((
                                "Location",
                                format!("{}/list", ARGS.public_path_as_str()),
                            ))
                            .finish());
                    } else {
                        return Ok(HttpResponse::Found()
                            .append_header((
                                "Location",
                                format!("/auth_remove_private/{}/incorrect", pasta.id_as_animals()),
                            ))
                            .finish());
                    }
                } else {
                    return Ok(HttpResponse::Found()
                        .append_header((
                            "Location",
                            format!("/auth_remove_private/{}/incorrect", pasta.id_as_animals()),
                        ))
                        .finish());
                }
            }

            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!(
                        "{}/upload/{}",
                        ARGS.public_path_as_str(),
                        pastas[i].id_as_animals()
                    ),
                ))
                .finish());
        }
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
