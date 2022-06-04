use actix_web::{get, web, HttpResponse};

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use askama::Template;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    if ARGS.readonly {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut pastas = data.pastas.lock().unwrap();

    let id = to_u64(&*id.into_inner()).unwrap_or(0);

    remove_expired(&mut pastas);

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            pastas.remove(i);
            return HttpResponse::Found()
                .append_header(("Location", "/pastalist"))
                .finish();
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}
