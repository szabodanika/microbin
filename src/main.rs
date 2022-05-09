extern crate core;

use env_logger::Builder;
use std::io::Write;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_files;
use actix_multipart::Multipart;
use actix_web::dev::ServiceRequest;
use actix_web::middleware::Condition;
use actix_web::{error, get, middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use askama::Template;
use chrono::Local;
use clap::Parser;
use futures::TryStreamExt as _;
use lazy_static::lazy_static;
use linkify::{LinkFinder, LinkKind};
use log::LevelFilter;
use rand::Rng;
use std::fs;

use crate::animalnumbers::{to_animal_names, to_u64};
use crate::dbio::save_to_file;
use crate::pasta::Pasta;

mod animalnumbers;
mod dbio;
mod pasta;

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

struct AppState {
    pastas: Mutex<Vec<Pasta>>,
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 8080)]
    port: u32,

    #[clap(short, long, default_value_t = 1)]
    threads: u8,

    #[clap(long)]
    hide_header: bool,

    #[clap(long)]
    hide_footer: bool,

    #[clap(long)]
    pure_html: bool,

    #[clap(long)]
    no_listing: bool,

    #[clap(long)]
    auth_username: Option<String>,

    #[clap(long)]
    auth_password: Option<String>,
}

async fn auth_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    // check if username matches
    if credentials.user_id().as_ref() == ARGS.auth_username.as_ref().unwrap() {
        return match ARGS.auth_password.as_ref() {
            Some(cred_pass) => match credentials.password() {
                None => Err(error::ErrorBadRequest("Invalid login details.")),
                Some(arg_pass) => {
                    if arg_pass == cred_pass {
                        Ok(req)
                    } else {
                        Err(error::ErrorBadRequest("Invalid login details."))
                    }
                }
            },
            None => Ok(req),
        };
    } else {
        Err(error::ErrorBadRequest("Invalid login details."))
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    args: &'a Args,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate<'a> {
    args: &'a Args,
}

#[derive(Template)]
#[template(path = "pasta.html")]
struct PastaTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
}

#[derive(Template)]
#[template(path = "pastalist.html")]
struct PastaListTemplate<'a> {
    pastas: &'a Vec<Pasta>,
    args: &'a Args,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Found()
        .content_type("text/html")
        .body(IndexTemplate { args: &ARGS }.render().unwrap())
}

async fn not_found() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Found()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}

async fn create(data: web::Data<AppState>, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();

    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    } as i64;

    let mut new_pasta = Pasta {
        id: rand::thread_rng().gen::<u16>() as u64,
        content: String::from("No Text Content"),
        file: String::from("no-file"),
        created: timenow,
        pasta_type: String::from(""),
        expiration: 0,
    };

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "expiration" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.expiration = match std::str::from_utf8(&chunk).unwrap() {
                        "1min" => timenow + 60,
                        "10min" => timenow + 60 * 10,
                        "1hour" => timenow + 60 * 60,
                        "24hour" => timenow + 60 * 60 * 24,
                        "1week" => timenow + 60 * 60 * 24 * 7,
                        "never" => 0,
                        _ => panic!("Unexpected expiration time!"),
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

#[get("/pasta/{id}")]
async fn getpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = to_u64(&*id.into_inner());

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == id {
            return HttpResponse::Found()
                .content_type("text/html")
                .body(PastaTemplate { pasta, args: &ARGS }.render().unwrap());
        }
    }

    HttpResponse::Found()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/url/{id}")]
async fn redirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = to_u64(&*id.into_inner());

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == id {
            if pasta.pasta_type == "url" {
                return HttpResponse::Found()
                    .append_header(("Location", String::from(&pasta.content)))
                    .finish();
            } else {
                return HttpResponse::Found()
                    .content_type("text/html")
                    .body(ErrorTemplate { args: &ARGS }.render().unwrap());
            }
        }
    }

    HttpResponse::Found()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/raw/{id}")]
async fn getrawpasta(data: web::Data<AppState>, id: web::Path<String>) -> String {
    let mut pastas = data.pastas.lock().unwrap();

    let id = to_u64(&*id.into_inner());

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == id {
            return pasta.content.to_owned();
        }
    }

    String::from("Pasta not found! :-(")
}

#[get("/remove/{id}")]
async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = to_u64(&*id.into_inner());

    remove_expired(&mut pastas);

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            pastas.remove(i);
            return HttpResponse::Found()
                .append_header(("Location", "/pastalist"))
                .finish();
        }
    }

    HttpResponse::Found()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/pastalist")]
async fn list(data: web::Data<AppState>) -> HttpResponse {
    if ARGS.no_listing {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    HttpResponse::Found().content_type("text/html").body(
        PastaListTemplate {
            pastas: &pastas,
            args: &ARGS,
        }
        .render()
        .unwrap(),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();

    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    log::info!(
        "MicroBin starting on http://127.0.0.1:{}",
        args.port.to_string()
    );

    match std::fs::create_dir_all("./pasta_data") {
        Ok(dir) => dir,
        Err(error) => {
            log::error!("Couldn't create data directory ./pasta_data: {:?}", error);
            panic!("Couldn't create data directory ./pasta_data: {:?}", error);
        }
    };

    let data = web::Data::new(AppState {
        pastas: Mutex::new(dbio::load_from_file().unwrap()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::NormalizePath::trim())
            .service(index)
            .service(getpasta)
            .service(redirecturl)
            .service(getrawpasta)
            .service(actix_files::Files::new("/static", "./static"))
            .service(actix_files::Files::new("/file", "./pasta_data"))
            .service(web::resource("/upload").route(web::post().to(create)))
            .default_service(web::route().to(not_found))
            .wrap(middleware::Logger::default())
            .service(remove)
            .service(list)
            .wrap(Condition::new(
                args.auth_username.is_some(),
                HttpAuthentication::basic(auth_validator),
            ))
    })
    .bind(format!("0.0.0.0:{}", args.port.to_string()))?
    .workers(args.threads as usize)
    .run()
    .await
}

fn remove_expired(pastas: &mut Vec<Pasta>) {
    // get current time - this will be needed to check which pastas have expired
    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    } as i64;

    pastas.retain(|p| {
        // delete the files too
        if p.expiration < timenow {
            // remove the file itself
            fs::remove_file(format!("./pasta_data/{}/{}", p.id_as_animals(), p.file));
            // and remove the containing directory
            fs::remove_dir(format!("./pasta_data/{}/", p.id_as_animals()));
        };

        p.expiration == 0 || p.expiration > timenow
    });
}

fn is_valid_url(url: &str) -> bool {
    let finder = LinkFinder::new();
    let spans: Vec<_> = finder.spans(url).collect();
    spans[0].as_str() == url && Some(&LinkKind::Url) == spans[0].kind()
}
