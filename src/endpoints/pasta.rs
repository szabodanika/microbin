use crate::args::{Args, ARGS};
use crate::dbio::save_to_file;
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::Pasta;
use crate::util::misc::remove_expired;
use crate::AppState;

use actix_web::{get, web, HttpResponse};
use askama::Template;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Template)]
#[template(path = "pasta.html", escape = "none")]
struct PastaTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
}

fn pastaresponse(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    // find the index of the pasta in the collection based on u64 id
    let (index, found) = Pasta::get_index(id.as_ref(), &mut pastas);

    if found {
        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        save_to_file(&pastas);

        // serve pasta in template
        let response = HttpResponse::Ok().content_type("text/html").body(
            PastaTemplate {
                pasta: &pastas[index],
                args: &ARGS,
            }
            .render()
            .unwrap(),
        );

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        return response;
    }

    // otherwise
    // send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/pasta/{id}")]
pub async fn getpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    pastaresponse(data, id)
}

#[get("/p/{id}")]
pub async fn getshortpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    pastaresponse(data, id)
}

fn urlresponse(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let (index, found) = Pasta::get_index(id.as_ref(), &mut pastas);

    if found {
        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        save_to_file(&pastas);

        // send redirect if it's a url pasta
        if pastas[index].pasta_type == "url" {
            let response = HttpResponse::Found()
                .append_header(("Location", String::from(&pastas[index].content)))
                .finish();

            // get current unix time in seconds
            let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => {
                    log::error!("SystemTime before UNIX EPOCH!");
                    0
                }
            } as i64;

            // update last read time
            pastas[index].last_read = timenow;

            return response;
        // send error if we're trying to open a non-url pasta as a redirect
        } else {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(ErrorTemplate { args: &ARGS }.render().unwrap());
        }
    }

    // otherwise
    // send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}


#[get("/url/{id}")]
pub async fn redirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    urlresponse(data, id)
}

#[get("/u/{id}")]
pub async fn shortredirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    urlresponse(data, id)
}

#[get("/raw/{id}")]
pub async fn getrawpasta(data: web::Data<AppState>, id: web::Path<String>) -> String {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let (index, found) = Pasta::get_index(id.as_ref(), &mut pastas);

    if found {
        // increment read count
        pastas[index].read_count += 1;

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // save the updated read count
        save_to_file(&pastas);

        // send raw content of pasta
        return pastas[index].content.to_owned();
    }

    // otherwise
    // send pasta not found error as raw text
    String::from("Pasta not found! :-(")
}
