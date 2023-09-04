use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::Pasta;
use crate::util::animalnumbers::to_u64;
use crate::util::db::update;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use askama::Template;
use futures::TryStreamExt;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Template)]
#[template(path = "upload.html", escape = "none")]
struct PastaTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
}

fn pastaresponse(
    data: web::Data<AppState>,
    id: web::Path<String>,
    password: String,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

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

    if found {
        if pastas[index].encrypt_server && password == *"" {
            return HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("/auth/{}", pastas[index].id_as_animals()),
                ))
                .finish();
        }

        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        let original_content = pastas[index].content.to_owned();

        // decrypt content temporarily
        if password != *"" && !original_content.is_empty() {
            let res = decrypt(&original_content, &password);
            if let Ok(..) = res {
                pastas[index]
                    .content
                    .replace_range(.., res.unwrap().as_str());
            } else {
                return HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/auth/{}/incorrect", pastas[index].id_as_animals()),
                    ))
                    .finish();
            }
        }

        // serve pasta in template
        let response = HttpResponse::Ok().content_type("text/html").body(
            PastaTemplate {
                pasta: &pastas[index],
                args: &ARGS,
            }
            .render()
            .unwrap(),
        );

        if pastas[index].content != original_content {
            pastas[index].content = original_content;
        }

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        return response;
    }

    // otherwise send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[post("/upload/{id}")]
pub async fn postpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    Ok(pastaresponse(data, id, password))
}

#[post("/p/{id}")]
pub async fn postshortpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    Ok(pastaresponse(data, id, password))
}

#[get("/upload/{id}")]
pub async fn getpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    pastaresponse(data, id, String::from(""))
}

#[get("/p/{id}")]
pub async fn getshortpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    pastaresponse(data, id, String::from(""))
}

fn urlresponse(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

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

    if found {
        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        // send redirect if it's a url pasta
        if pastas[index].pasta_type == "url" {
            let response = HttpResponse::Found()
                .append_header(("Location", String::from(&pastas[index].content)))
                .finish();

            // get current unix time in seconds
            let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => {
                    log::error!("SystemTime before UNIX EPOCH!");
                    0
                }
            } as i64;

            // update last read time
            pastas[index].last_read = timenow;

            // save the updated read count
            update(Some(&pastas), Some(&pastas[index]));

            return response;
        // send error if we're trying to open a non-url pasta as a redirect
        } else {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(ErrorTemplate { args: &ARGS }.render().unwrap());
        }
    }

    // otherwise send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/url/{id}")]
pub async fn redirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    urlresponse(data, id)
}

#[get("/u/{id}")]
pub async fn shortredirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    urlresponse(data, id)
}

#[get("/raw/{id}")]
pub async fn getrawpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

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

    if found {
        if pastas[index].encrypt_server {
            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("/auth_raw/{}", pastas[index].id_as_animals()),
                ))
                .finish());
        }

        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // send raw content of pasta
        let response = Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body(pastas[index].content.to_owned()));

        return response;
    }

    // otherwise send pasta not found error as raw text
    Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body(String::from("Upload not found! :-(")))
}

#[post("/raw/{id}")]
pub async fn postrawpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

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

    if found {
        if pastas[index].encrypt_server && password == *"" {
            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("/auth/{}", pastas[index].id_as_animals()),
                ))
                .finish());
        }

        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        let original_content = pastas[index].content.to_owned();

        // decrypt content temporarily
        if password != *"" {
            let res = decrypt(&original_content, &password);
            if res.is_ok() {
                pastas[index]
                    .content
                    .replace_range(.., res.unwrap().as_str());
            } else {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/auth/{}/incorrect", pastas[index].id_as_animals()),
                    ))
                    .finish());
            }
        }

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        // send raw content of pasta
        let response = Ok(HttpResponse::NotFound()
            .content_type("text/html")
            .body(pastas[index].content.to_owned()));

        if pastas[index].content != original_content {
            pastas[index].content = original_content;
        }

        return response;
    }

    // otherwise send pasta not found error as raw text
    Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body(String::from("Upload not found! :-(")))
}

fn decrypt(text_str: &str, key_str: &str) -> Result<String, magic_crypt::MagicCryptError> {
    let mc = new_magic_crypt!(key_str, 256);

    mc.decrypt_base64_to_string(text_str)
}
