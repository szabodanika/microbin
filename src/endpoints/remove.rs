use actix_web::{get, web, HttpResponse};

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::{Pasta, PastaFile};
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use askama::Template;
use std::fs;

#[get("/remove/{slug}")]
pub async fn remove(data: web::Data<AppState>, slug: web::Path<String>) -> HttpResponse {
    if ARGS.readonly {
        return HttpResponse::Found()
            .append_header(("Location", format!("{}/", ARGS.public_path)))
            .finish();
    }

    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&slug).unwrap_or(0)
    } else {
        to_u64(&slug).unwrap_or(0)
    };

    let slug = slug.as_ref();
    for (i, pasta) in pastas.iter().enumerate() {
        match pasta.slug {
            Some(ref s) if ARGS.slugs && s == slug => {
                // remove the file itself
                remove_file(pasta);

                // remove it from in-memory pasta list
                pastas.remove(i);
                return HttpResponse::Found()
                    .append_header(("Location", format!("{}/pastalist", ARGS.public_path)))
                    .finish();
            }
            None if pasta.id == id => {
                // remove the file itself
                remove_file(pasta);

                // remove it from in-memory pasta list
                pastas.remove(i);
                return HttpResponse::Found()
                    .append_header(("Location", format!("{}/pastalist", ARGS.public_path)))
                    .finish();
            }
            _ => {
                continue;
            }
        }
    }

    remove_expired(&mut pastas);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

fn remove_file(pasta: &Pasta) {
    if let Some(PastaFile { name, .. }) = &pasta.file {
        if fs::remove_file(format!(
            "./pasta_data/public/{}/{}",
            pasta.id_as_animals(),
            name
        ))
        .is_err()
        {
            log::error!("Failed to delete file {}!", name)
        }

        // and remove the containing directory
        if fs::remove_dir(format!("./pasta_data/public/{}/", pasta.id_as_animals())).is_err() {
            log::error!("Failed to delete directory {}!", name)
        }
    }
}
