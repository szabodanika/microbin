use actix_web::{Error, HttpResponse};
use askama::Template;

use crate::args::{Args, ARGS};

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub args: &'a Args,
}

pub async fn not_found() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
