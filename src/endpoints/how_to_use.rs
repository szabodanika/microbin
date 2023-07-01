use crate::args::{Args, ARGS};
use actix_web::{get, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "how_to_use.html")]
struct Guide<'a> {
    args: &'a Args,
}

#[get("/guide")]
pub async fn how_to_use() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(Guide { args: &ARGS }.render().unwrap())
}
