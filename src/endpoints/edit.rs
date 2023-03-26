use crate::args::Args;
use crate::dbio::save_to_file;
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use askama::Template;
use futures::TryStreamExt;

#[derive(Template)]
#[template(path = "edit.html", escape = "none")]
struct EditTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
}

#[get("/edit/{id}")]
pub async fn get_edit(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == id {
            if !pasta.editable {
                return HttpResponse::Found()
                    .append_header(("Location", format!("{}/", ARGS.public_path)))
                    .finish();
            }
            return HttpResponse::Ok()
                .content_type("text/html")
                .body(EditTemplate { pasta, args: &ARGS }.render().unwrap());
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[post("/edit/{id}")]
pub async fn post_edit(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    if ARGS.readonly {
        return Ok(HttpResponse::Found()
            .append_header(("Location", format!("{}/", ARGS.public_path)))
            .finish());
    }

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let mut new_content = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "content" {
            while let Some(chunk) = field.try_next().await? {
                new_content.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            if pasta.editable {
                pastas[i].content.replace_range(.., &new_content);
                save_to_file(&pastas);

                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("{}/pasta/{}", ARGS.public_path, pastas[i].id_as_animals()),
                    ))
                    .finish());
            } else {
                break;
            }
        }
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
