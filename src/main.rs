extern crate core;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_files::NamedFile;
use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, post, Responder, Result, web};
use actix_web::web::Data;
use askama::Template;
use rand::Rng;

use crate::animalnumbers::{to_animal_names, to_u64};
use crate::pasta::{Pasta, PastaFormData};

mod pasta;
mod animalnumbers;

struct AppState {
	pastas: Mutex<Vec<Pasta>>,
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
	HttpResponse::Found().content_type("text/html").body(IndexTemplate {}.render().unwrap())
}

#[post("/create")]
async fn create(data: web::Data<AppState>, pasta: web::Form<PastaFormData>) -> impl Responder {
	let mut pastas = data.pastas.lock().unwrap();

	let mut innerPasta = pasta.into_inner();

	let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => n.as_secs(),
		Err(_) => panic!("SystemTime before UNIX EPOCH!"),
	} as i64;

	let expiration = match innerPasta.expiration.as_str() {
		"firstread" => 1,
		"10min" => timenow + 60 * 10,
		"1hour" => timenow + 60 * 60,
		"24hour" => timenow + 60 * 60 * 24,
		"1week" => timenow + 60 * 60 * 24 * 7,
		"never" => 0,
		_ => panic!("Unexpected expiration time!")
	};

	let mut newPasta = Pasta {
		id: rand::thread_rng().gen::<u16>() as u64,
		content: innerPasta.content,
		created: timenow,
		expiration,
	};

	let id = newPasta.id;

	pastas.push(newPasta);

	HttpResponse::Found().append_header(("Location", format!("/pasta/{}", to_animal_names(id)))).finish()
}

#[get("/pasta/{id}")]
async fn getpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
	let pastas = data.pastas.lock().unwrap();
	let id = to_u64(&*id.into_inner());

	for pasta in pastas.iter() {
		if pasta.id == id {
			return HttpResponse::Found().content_type("text/html").body(PastaTemplate { pasta }.render().unwrap());
		}
	}

	HttpResponse::Found().body("Pasta not found! :-(")
}

#[get("/rawpasta/{id}")]
async fn getrawpasta(data: web::Data<AppState>, id: web::Path<String>) -> String {
	let pastas = data.pastas.lock().unwrap();
	let id = to_u64(&*id.into_inner());

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

	for (i, pasta) in pastas.iter().enumerate() {
		if pasta.id == id {
			pastas.remove(i);
			return HttpResponse::Found().append_header(("Location", "/pastalist")).finish();
		}
	}

	HttpResponse::Found().body("Pasta not found! :-(")
}

#[get("/pastalist")]
async fn list(data: web::Data<AppState>) -> HttpResponse {
	let mut pastas = data.pastas.lock().unwrap();

	HttpResponse::Found().content_type("text/html").body(PastaListTemplate { pastas: &pastas }.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let data = web::Data::new(AppState {
		pastas: Mutex::new(Vec::new()),
	});

	HttpServer::new(move || App::new().app_data(data.clone()).service(index).service(create).service(getpasta).service(getrawpasta).service(remove).service(list)
	).bind("127.0.0.1:8080")?.run().await
}
