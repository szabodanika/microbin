use crate::args::{Args, ARGS};
use crate::pasta::Pasta;
use crate::util::misc::{self, remove_expired};
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;
use qrcode_generator::QrCodeEcc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Template)]
#[template(path = "qr.html", escape = "none")]
struct QRTemplate<'a> {
    qr: &'a String,
    args: &'a Args,
}

#[get("/qr/{id}")]
pub async fn getqr(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // find the index of the pasta in the collection based on u64 id

    let svg: String = misc::string_to_qr_svg(
        format!("{}/pasta/{}", &ARGS.public_path, &*id.into_inner()).as_str(),
    );

    // serve qr code in template
    HttpResponse::Ok().content_type("text/html").body(
        QRTemplate {
            qr: &svg,
            args: &ARGS,
        }
        .render()
        .unwrap(),
    )
}
