use actix_web::{web, HttpResponse, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/assets/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().content_type("text/plain").body("404 Not Found"),
    }
}

#[actix_web::get("/static/{_:.*}")]
async fn static_resources(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

#[actix_web::get("/robots.txt")]
async fn static_resources_robots() -> impl Responder {
    handle_embedded_file("robots.txt")
}

#[actix_web::get("/sitemap.xml")]
async fn static_resources_sitemap() -> impl Responder {
    handle_embedded_file("sitemap.xml")
}

#[actix_web::get("/favicon.ico")]
async fn static_resources_favicon() -> impl Responder {
    handle_embedded_file("favicon.ico")
}