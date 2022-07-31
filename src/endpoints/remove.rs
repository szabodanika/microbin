use actix_web::{get, web, HttpResponse};

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::PastaFile;
use crate::util::animalnumbers::to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use askama::Template;
use std::fs;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    if ARGS.readonly {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut pastas = data.pastas.lock().unwrap();

    let id = to_u64(&*id.into_inner()).unwrap_or(0);

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            // remove the file itself
            if let Some(PastaFile { name, .. }) = &pasta.file {
                if fs::remove_file(format!("./pasta_data/{}/{}", pasta.id_as_animals(), name))
                    .is_err()
                {
                    log::error!("Failed to delete file {}!", name)
                }

                // and remove the containing directory
                if fs::remove_dir(format!("./pasta_data/{}/", pasta.id_as_animals())).is_err() {
                    log::error!("Failed to delete directory {}!", name)
                }
            }
            // remove it from in-memory pasta list
            pastas.remove(i);
            return HttpResponse::Found()
                .append_header(("Location", "/pastalist"))
                .finish();
        }
    }

    remove_expired(&mut pastas);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}
