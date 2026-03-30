// DISCLAIMER
// (c) 2024-05-27 overcuriousity - derived from the original Microbin Project by Daniel Szabo
use crate::args::{Args, ARGS};
use actix_web::{get, HttpResponse};
use askama::Template;


#[derive(Template)]
#[template(path = "guide.html")]
struct Guide<'a> {
    args: &'a Args,
}



#[get("/guide")]
pub async fn guide() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(Guide { args: &ARGS }.render().unwrap())
}
