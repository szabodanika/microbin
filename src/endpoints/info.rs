use crate::args::{Args, ARGS};
use crate::pasta::Pasta;
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "info.html")]
struct Info<'a> {
    args: &'a Args,
    pastas: &'a Vec<Pasta>,
    status: &'a String,
    version_string: &'a String,
    message: &'a String,
}

#[get("/info")]
pub async fn info(data: web::Data<AppState>) -> HttpResponse {
    // get access to the pasta collection
    let pastas = data.pastas.lock().unwrap();

    // todo status report more sophisticated
    let mut status = "OK";
    let mut message = "";

    if ARGS.public_path.to_string() == "" {
        status = "WARNING";
        message = "Warning: No public URL set with --public-path parameter. QR code and URL Copying functions have been disabled"
    }

    HttpResponse::Ok().content_type("text/html").body(
        Info {
            args: &ARGS,
            pastas: &pastas,
            status: &String::from(status),
            version_string: &String::from("1.2.0-20221107"),
            message: &String::from(message),
        }
        .render()
        .unwrap(),
    )
}
