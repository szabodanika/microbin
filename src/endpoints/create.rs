use crate::dbio::save_to_file;
use crate::pasta::PastaFile;
use crate::util::animalnumbers::to_animal_names;
use crate::util::hashids::to_hashids;
use crate::util::misc::is_valid_url;
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::{get, web, Error, HttpResponse, Responder};
use askama::Template;
use bytesize::ByteSize;
use futures::TryStreamExt;
use log::warn;
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
            .append_header(("Location", format!("{}/", ARGS.public_path)))
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
        file: None,
        extension: String::from(""),
        private: false,
        editable: false,
        created: timenow,
        read_count: 0,
        burn_after_reads: 0,
        last_read: timenow,
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
                        "never" => {
                            if ARGS.no_eternal_pasta {
                                timenow + 60 * 60 * 24 * 7
                            } else {
                                0
                            }
                        }
                        _ => {
                            log::error!("{}", "Unexpected expiration time!");
                            timenow + 60 * 60 * 24 * 7
                        }
                    };
                }

                continue;
            }
            "burn_after" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.burn_after_reads = match std::str::from_utf8(&chunk).unwrap() {
                        // give an extra read because the user will be redirected to the pasta page automatically
                        "1" => 2,
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

                continue;
            }
            "content" => {
                let mut content = String::from("");
                while let Some(chunk) = field.try_next().await? {
                    content.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
                }
                if content.len() > 0 {
                    new_pasta.content = content;

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
                if ARGS.no_file_upload {
                    continue;
                }

                let path = field.content_disposition().get_filename();

                let path = match path {
                    Some("") => continue,
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

                std::fs::create_dir_all(format!(
                    "./pasta_data/public/{}",
                    &new_pasta.id_as_animals()
                ))
                .unwrap();

                let filepath = format!(
                    "./pasta_data/public/{}/{}",
                    &new_pasta.id_as_animals(),
                    &file.name()
                );

                let mut f = web::block(|| std::fs::File::create(filepath)).await??;
                let mut size = 0;
                while let Some(chunk) = field.try_next().await? {
                    size += chunk.len();
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                file.size = ByteSize::b(size as u64);

                new_pasta.file = Some(file);
                new_pasta.pasta_type = String::from("text");
            }
            field => {
                log::error!("Unexpected multipart field:  {}", field);
            }
        }
    }

    let id = new_pasta.id;

    pastas.push(new_pasta);

    save_to_file(&pastas);

    let slug = if ARGS.hash_ids {
        to_hashids(id)
    } else {
        to_animal_names(id)
    };
    Ok(HttpResponse::Found()
        .append_header(("Location", format!("{}/pasta/{}", ARGS.public_path, slug)))
        .finish())
}
