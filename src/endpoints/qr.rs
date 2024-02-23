use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::Pasta;
use crate::util::hashids::alias_comparator;
use crate::util::misc::{self, remove_expired};
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "qr.html", escape = "none")]
struct QRTemplate<'a> {
    qr: &'a String,
    pasta: &'a Pasta,
    args: &'a Args,
}

#[get("/qr/{id}")]
pub async fn getqr(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let comparator = alias_comparator(id.as_str());

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    // find the index of the pasta in the collection based on u64 id
    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if comparator(pasta) {
            index = i;
            found = true;
            break;
        }
    }

    if found {
        // generate the QR code as an SVG - if its a file or text pastas, this will point to the /upload endpoint, otherwise to the /url endpoint, essentially directly taking the user to the url stored in the pasta
        let svg: String = match pastas[index].pasta_type.as_str() {
            "url" => misc::string_to_qr_svg(
                format!("{}/url/{}", &ARGS.public_path_as_str(), &id).as_str(),
            ),
            _ => misc::string_to_qr_svg(
                format!("{}/upload/{}", &ARGS.public_path_as_str(), &id).as_str(),
            ),
        };

        // serve qr code in template
        return HttpResponse::Ok().content_type("text/html").body(
            QRTemplate {
                qr: &svg,
                pasta: &pastas[index],
                args: &ARGS,
            }
            .render()
            .unwrap(),
        );
    }

    // otherwise
    // send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}
