use crate::args::{Args, ARGS};
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "auth_admin.html")]
struct AuthAdmin<'a> {
    args: &'a Args,
    status: String,
}

#[get("/auth_admin")]
pub async fn auth_admin() -> HttpResponse {
    return HttpResponse::Ok().content_type("text/html").body(
        AuthAdmin {
            args: &ARGS,
            status: String::from(""),
        }
        .render()
        .unwrap(),
    );
}

#[get("/auth_admin/{status}")]
pub async fn auth_admin_with_status(param: web::Path<String>) -> HttpResponse {
    let status = param.into_inner();

    return HttpResponse::Ok().content_type("text/html").body(
        AuthAdmin {
            args: &ARGS,
            status,
        }
        .render()
        .unwrap(),
    );
}
