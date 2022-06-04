use crate::args::{Args, ARGS};
use actix_web::{get, HttpResponse};
use askama::Template;
use std::marker::PhantomData;

#[derive(Template)]
#[template(path = "help.html")]
struct Help<'a> {
    args: &'a Args,
    _marker: PhantomData<&'a ()>,
}

#[get("/help")]
pub async fn help() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        Help {
            args: &ARGS,
            _marker: Default::default(),
        }
        .render()
        .unwrap(),
    )
}
