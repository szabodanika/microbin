use crate::dbio::save_to_file;
use crate::util::animalnumbers::to_animal_names;
use crate::util::misc::is_valid_url;
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::{get, web, Error, HttpResponse, Responder};
use askama::Template;
use futures::TryStreamExt;
use rand::Rng;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    args: &'a ARGS,
}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(IndexTemplate { args: &ARGS }.render().unwrap())
}

pub async fn create(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    if ARGS.readonly {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }

    let mut pastas = data.pastas.lock().unwrap();

    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => {
            log::error!("SystemTime before UNIX EPOCH!");
            0
        }
    } as i64;

    let mut new_pasta = Pasta {
        id: rand::thread_rng().gen::<u16>() as u64,
        content: String::from("No Text Content"),
        file: String::from("no-file"),
        extension: String::from(""),
        private: false,
        editable: false,
        created: timenow,
        pasta_type: String::from(""),
        expiration: 0,
    };

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "editable" => {
                // while let Some(_chunk) = field.try_next().await? {}
                new_pasta.editable = true;
                continue;
            }
            "private" => {
                // while let Some(_chunk) = field.try_next().await? {}
                new_pasta.private = true;
                continue;
            }
            "expiration" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.expiration = match std::str::from_utf8(&chunk).unwrap() {
                        "1min" => timenow + 60,
                        "10min" => timenow + 60 * 10,
                        "1hour" => timenow + 60 * 60,
                        "24hour" => timenow + 60 * 60 * 24,
                        "1week" => timenow + 60 * 60 * 24 * 7,
                        "never" => 0,
                        _ => {
                            log::error!("{}", "Unexpected expiration time!");
                            0
                        }
                    };
                }

                continue;
            }
            "content" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.content = std::str::from_utf8(&chunk).unwrap().to_string();
                    new_pasta.pasta_type = if is_valid_url(new_pasta.content.as_str()) {
                        String::from("url")
                    } else {
                        String::from("text")
                    };
                }
                continue;
            }
            "syntax-highlight" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.extension = std::str::from_utf8(&chunk).unwrap().to_string();
                }
                continue;
            }
            "file" => {
                let content_disposition = field.content_disposition();

                let filename = match content_disposition.get_filename() {
                    Some("") => continue,
                    Some(filename) => filename.replace(' ', "_").to_string(),
                    None => continue,
                };

                std::fs::create_dir_all(format!("./pasta_data/{}", &new_pasta.id_as_animals()))
                    .unwrap();

                let filepath = format!("./pasta_data/{}/{}", &new_pasta.id_as_animals(), &filename);

                new_pasta.file = filename;

                let mut f = web::block(|| std::fs::File::create(filepath)).await??;

                while let Some(chunk) = field.try_next().await? {
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                new_pasta.pasta_type = String::from("text");
            }
            _ => {}
        }
    }

    let id = new_pasta.id;

    pastas.push(new_pasta);

    save_to_file(&pastas);

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/pasta/{}", to_animal_names(id))))
        .finish())
}
