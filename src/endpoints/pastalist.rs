use std::borrow::Borrow;
use std::sync::Mutex;

use actix_web::{get, web, HttpResponse};
use askama::Template;
use lazy_static::__Deref;

use crate::args::{Args, ARGS};
use crate::pasta::Pasta;
use crate::util::dbio::DataStore;
use crate::util::misc::remove_expired;
use crate::AppState;

#[derive(Template)]
#[template(path = "pastalist.html")]
struct PastaListTemplate<'a> {
    pastas: &'a Vec<Pasta>,
    args: &'a Args,
}

#[get("/pastalist")]
pub async fn list(data: web::Data<Box<dyn DataStore + Send + Sync>>) -> HttpResponse {
    if ARGS.no_listing {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    data.remove_expired();
    let pastas = data.get_pastalist();
    HttpResponse::Ok().content_type("text/html").body(
        PastaListTemplate {
            pastas: &pastas,
            args: &ARGS,
        }
        .render()
        .unwrap(),
    )
}
