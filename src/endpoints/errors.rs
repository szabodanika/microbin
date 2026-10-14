// use actix_web::{Error, HttpResponse, error::ErrorNotFound};
use actix_web::{Error, HttpResponse};
use askama::Template;

use crate::args::{Args, ARGS};

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub args: &'a Args,
}

pub async fn not_found() -> Result<HttpResponse, Error> {
    // This is probably the intended way but I can't set Content-Type here...
    // Err(ErrorNotFound(ErrorTemplate { args: &ARGS }.render().unwrap()))
    // This looks absolutely weird but in a sense it succeeds in matching nothing probably...?
    Ok(HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
