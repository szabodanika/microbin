use actix_web::{get, web, HttpResponse};
use askama::Template;

use crate::args::{Args, ARGS};
use crate::pasta::Pasta;
use crate::util::misc::remove_expired;
use crate::AppState;

#[derive(Template)]
#[template(path = "pastalist.html")]
struct PastaListTemplate<'a> {
    pastas: &'a Vec<Pasta>,
    args: &'a Args,
}

#[get("/pastalist")]
pub async fn list(data: web::Data<AppState>) -> HttpResponse {
    if ARGS.no_listing {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    HttpResponse::Ok().content_type("text/html").body(
        PastaListTemplate {
            pastas: &pastas,
            args: &ARGS,
        }
        .render()
        .unwrap(),
    )
}
