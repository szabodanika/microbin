extern crate core;

use actix_files as fs;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use askama::Template;
use clap::Parser;
use linkify::{LinkFinder, LinkKind};
use rand::Rng;
use regex::Regex;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::animalnumbers::{to_animal_names, to_u64};
use crate::pasta::{Pasta, PastaFormData};

mod animalnumbers;
mod pasta;

struct AppState {
    pastas: Mutex<Vec<Pasta>>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 8080)]
    port: u32,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[derive(Template)]
#[template(path = "pasta.html")]
struct PastaTemplate<'a> {
    pasta: &'a Pasta,
}

#[derive(Template)]
#[template(path = "pastalist.html")]
struct PastaListTemplate<'a> {
    pastas: &'a Vec<Pasta>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Found()
        .content_type("text/html")
        .body(IndexTemplate {}.render().unwrap())
}

#[post("/create")]
async fn create(data: web::Data<AppState>, pasta: web::Form<PastaFormData>) -> impl Responder {
    let mut pastas = data.pastas.lock().unwrap();

    let inner_pasta = pasta.into_inner();

    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    } as i64;

    let expiration = match inner_pasta.expiration.as_str() {
        "1min" => timenow + 60,
        "10min" => timenow + 60 * 10,
        "1hour" => timenow + 60 * 60,
        "24hour" => timenow + 60 * 60 * 24,
        "1week" => timenow + 60 * 60 * 24 * 7,
        "never" => 0,
        _ => panic!("Unexpected expiration time!"),
    };

    let pasta_type = if is_valid_url(inner_pasta.content.as_str()) {
        String::from("url")
    } else {
        String::from("text")
    };

    let new_pasta = Pasta {
        id: rand::thread_rng().gen::<u16>() as u64,
        content: inner_pasta.content,
        created: timenow,
        pasta_type,
        expiration,
    };

    let id = new_pasta.id;

    pastas.push(new_pasta);

    HttpResponse::Found()
        .append_header(("Location", format!("/pasta/{}", to_animal_names(id))))
        .finish()
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
                .body(PastaTemplate { pasta }.render().unwrap());
        }
    }

    HttpResponse::Found().body("Pasta not found! :-(")
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
                return HttpResponse::Found().body("This is not a valid URL. :-(");
            }
        }
    }

    HttpResponse::Found().body("Pasta not found! :-(")
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

    HttpResponse::Found().body("Pasta not found! :-(")
}

#[get("/pastalist")]
async fn list(data: web::Data<AppState>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    HttpResponse::Found()
        .content_type("text/html")
        .body(PastaListTemplate { pastas: &pastas }.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!(
        "{}",
        format!("Listening on http://127.0.0.1:{}", args.port.to_string())
    );

    let data = web::Data::new(AppState {
        pastas: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(index)
            .service(create)
            .service(getpasta)
            .service(redirecturl)
            .service(getrawpasta)
            .service(remove)
            .service(list)
            .service(fs::Files::new("/static", "./static"))
    })
    .bind(format!("127.0.0.1:{}", args.port.to_string()))?
    .run()
    .await
}

fn remove_expired(pastas: &mut Vec<Pasta>) {
    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    } as i64;

    pastas.retain(|p| p.expiration == 0 || p.expiration > timenow);
}

fn is_valid_url(url: &str) -> bool {
    let finder = LinkFinder::new();
    let spans: Vec<_> = finder.spans(url).collect();
    spans[0].as_str() == url && Some(&LinkKind::Url) == spans[0].kind()
}
