use actix_web::{get, web, HttpResponse};
use askama::Template;
use std::marker::PhantomData;

#[derive(Template)]
#[template(path = "water.css", escape = "none")]
struct WaterCSS<'a> {
    _marker: PhantomData<&'a ()>,
}

#[derive(Template)]
#[template(path = "favicon.svg", escape = "none")]
struct Favicon<'a> {
    _marker: PhantomData<&'a ()>,
}

#[get("/static/{resource}")]
pub async fn static_resources(resource_id: web::Path<String>) -> HttpResponse {
    match resource_id.into_inner().as_str() {
        "water.css" => HttpResponse::Ok().content_type("text/css").body(
            WaterCSS {
                _marker: Default::default(),
            }
            .render()
            .unwrap(),
        ),
        "favicon.svg" => HttpResponse::Ok().content_type("image/svg+xml").body(
            Favicon {
                _marker: Default::default(),
            }
            .render()
            .unwrap(),
        ),
        _ => HttpResponse::NotFound().content_type("text/html").finish(),
    }
}
