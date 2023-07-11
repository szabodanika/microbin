use crate::args::Args;
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::db::update;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{decrypt, encrypt, remove_expired};
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use askama::Template;
use futures::TryStreamExt;

#[derive(Template)]
#[template(path = "edit.html", escape = "none")]
struct EditTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
    path: &'a String,
    status: &'a String,
}

#[get("/edit/{id}")]
pub async fn get_edit(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == id {
            if !pasta.editable {
                return HttpResponse::Found()
                    .append_header(("Location", format!("{}/", ARGS.public_path_as_str())))
                    .finish();
            }

            if pasta.encrypt_server {
                return HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/auth_edit_private/{}", pasta.id_as_animals()),
                    ))
                    .finish();
            }

            return HttpResponse::Ok().content_type("text/html").body(
                EditTemplate {
                    pasta,
                    args: &ARGS,
                    path: &String::from("edit"),
                    status: &String::from(""),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/edit/{id}/{status}")]
pub async fn get_edit_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == intern_id {
            if !pasta.editable {
                return HttpResponse::Found()
                    .append_header(("Location", format!("{}/", ARGS.public_path_as_str())))
                    .finish();
            }

            if pasta.encrypt_server {
                return HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/auth_edit_private/{}", pasta.id_as_animals()),
                    ))
                    .finish();
            }

            return HttpResponse::Ok().content_type("text/html").body(
                EditTemplate {
                    pasta,
                    args: &ARGS,
                    path: &String::from("edit"),
                    status: &status,
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[post("/edit_private/{id}")]
pub async fn post_edit_private(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    // find the index of the pasta in the collection based on u64 id
    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            index = i;
            found = true;
            break;
        }
    }

    if found && !pastas[index].encrypt_client {
        let original_content = pastas[index].content.to_owned();

        // decrypt content temporarily
        if password != *"" {
            let res = decrypt(&original_content, &password);
            if res.is_ok() {
                pastas[index]
                    .content
                    .replace_range(.., res.unwrap().as_str());
                // save pasta in database
                update(Some(&pastas), Some(&pastas[index]));
            } else {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!(
                            "/auth_edit_private/{}/incorrect",
                            pastas[index].id_as_animals()
                        ),
                    ))
                    .finish());
            }
        }

        // serve pasta in template
        let response = HttpResponse::Ok().content_type("text/html").body(
            EditTemplate {
                pasta: &pastas[index],
                args: &ARGS,
                path: &String::from("submit_edit_private"),
                status: &String::from(""),
            }
            .render()
            .unwrap(),
        );

        if pastas[index].content != original_content {
            pastas[index].content = original_content;
        }

        return Ok(response);
    }
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}

#[post("/submit_edit_private/{id}")]
pub async fn post_submit_edit_private(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let mut password = String::from("");
    let mut new_content = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "content" {
            while let Some(chunk) = field.try_next().await? {
                new_content.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password = std::str::from_utf8(&chunk).unwrap().to_string();
            }
        }
    }

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    // find the index of the pasta in the collection based on u64 id
    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            index = i;
            found = true;
            break;
        }
    }

    if found && pastas[index].editable && !pastas[index].encrypt_client {
        if pastas[index].readonly {
            let res = decrypt(pastas[index].encrypted_key.as_ref().unwrap(), &password);
            if res.is_ok() {
                pastas[index]
                    .content
                    .replace_range(.., &encrypt(&new_content, &password));
            } else {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/edit/{}/incorrect", pastas[index].id_as_animals()),
                    ))
                    .finish());
            }
        } else if pastas[index].private {
            let res = decrypt(&pastas[index].content, &password);
            if res.is_ok() {
                pastas[index]
                    .content
                    .replace_range(.., &encrypt(&new_content, &password));
                // save pasta in database
                update(Some(&pastas), Some(&pastas[index]));
            } else {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!(
                            "/auth_edit_private/{}/incorrect",
                            pastas[index].id_as_animals()
                        ),
                    ))
                    .finish());
            }
        }

        return Ok(HttpResponse::Found()
            .append_header((
                "Location",
                format!("/auth/{}/success", pastas[index].id_as_animals()),
            ))
            .finish());
    }
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}

#[post("/edit/{id}")]
pub async fn post_edit(
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

    let mut new_content = String::from("");
    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "content" {
            while let Some(chunk) = field.try_next().await? {
                new_content.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password = std::str::from_utf8(&chunk).unwrap().to_string();
            }
        }
    }

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            if pasta.editable && !pasta.encrypt_client {
                if pastas[i].readonly || pastas[i].encrypt_server {
                    if password != *"" {
                        let res = decrypt(pastas[i].encrypted_key.as_ref().unwrap(), &password);
                        if res.is_ok() {
                            pastas[i].content.replace_range(.., &new_content);
                            // save pasta in database
                            update(Some(&pastas), Some(&pastas[i]));
                        } else {
                            return Ok(HttpResponse::Found()
                                .append_header((
                                    "Location",
                                    format!("/edit/{}/incorrect", pasta.id_as_animals()),
                                ))
                                .finish());
                        }
                    } else {
                        return Ok(HttpResponse::Found()
                            .append_header((
                                "Location",
                                format!("/edit/{}/incorrect", pasta.id_as_animals()),
                            ))
                            .finish());
                    }
                } else {
                    pastas[i].content.replace_range(.., &new_content);
                    // save pasta in database
                    update(Some(&pastas), Some(&pastas[i]));
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
            } else {
                break;
            }
        }
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
