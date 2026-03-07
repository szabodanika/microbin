use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;

use crate::util::animalnumbers::to_u64;
use crate::util::auth;
use crate::util::db::delete;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{decrypt, remove_expired};
use crate::AppState;
use askama::Template;
use std::fs;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            // if it's encrypted or read-only, it needs password to be deleted
            // OR if it is not editable (public immutable), it needs admin password to be deleted
            if pasta.encrypt_server || pasta.readonly || !pasta.editable {
                return HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("{}/auth_remove_private/{}", ARGS.public_path_as_str(), pasta.id_as_animals()),
                    ))
                    .finish();
            }

            // remove the directory and all its contents
            if fs::remove_dir_all(format!(
                "{}/attachments/{}/",
                ARGS.data_dir,
                pasta.id_as_animals()
            ))
            .is_err()
            {
                log::error!("Failed to delete directory for {}!", pasta.id_as_animals())
            }

            // remove it from in-memory pasta list
            pastas.remove(i);

            delete(Some(&pastas), Some(id));

            return HttpResponse::Found()
                .append_header(("Location", format!("{}/list", ARGS.public_path_as_str())))
                .finish();
        }
    }

    remove_expired(&mut pastas);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[post("/remove/{id}")]
pub async fn post_remove(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let password = auth::password_from_multipart(payload).await?;

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            if pastas[i].readonly || pastas[i].encrypt_server || !pastas[i].editable {
                if password != *"" {
                    let mut is_password_correct = false;

                    if password == ARGS.auth_admin_password {
                        is_password_correct = true;
                    }

                    // if it is read-only, the content is not encrypted, but the key is
                    if !is_password_correct && pastas[i].readonly {
                        if let Some(ref encrypted_key) = pastas[i].encrypted_key {
                            let res = decrypt(encrypted_key, &password);
                            if let Ok(decrypted_key) = res {
                                if decrypted_key == id.to_string() {
                                    is_password_correct = true;
                                }
                            }
                        }
                    } else if !is_password_correct && pastas[i].encrypt_server {
                        // if it is not read-only, the content is encrypted
                        let res = decrypt(pastas[i].content.to_owned().as_str(), &password);
                        if res.is_ok() {
                            is_password_correct = true;
                        }
                    }

                    if is_password_correct {
                // remove the directory and all its contents
                if fs::remove_dir_all(format!(
                    "{}/attachments/{}/",
                    ARGS.data_dir,
                    pasta.id_as_animals()
                ))
                .is_err()
                {
                    log::error!("Failed to delete directory for {}!", pasta.id_as_animals())
                }

                        // remove it from in-memory pasta list
                        pastas.remove(i);

                        delete(Some(&pastas), Some(id));

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
                                format!("{}/auth_remove_private/{}/incorrect", ARGS.public_path_as_str(), pasta.id_as_animals()),
                            ))
                            .finish());
                    }
                } else {
                    return Ok(HttpResponse::Found()
                        .append_header((
                            "Location",
                            format!("{}/auth_remove_private/{}/incorrect", ARGS.public_path_as_str(), pasta.id_as_animals()),
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
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
