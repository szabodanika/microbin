use actix_web::{web, HttpResponse, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/assets/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[actix_web::get("/static/{_:.*}")]
async fn static_resources(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

// #[derive(Template)]
// #[template(path = "water.css", escape = "none")]
// struct WaterCSS<'a> {
//     _marker: PhantomData<&'a ()>,
// }

// // #[derive(Template)]
// // #[template(path = "logo.png", escape = "none")]
// struct LogoPNG<'a> {
//     _marker: PhantomData<&'a ()>,
// }

// #[derive(Template)]
// #[template(path = "favicon.svg", escape = "none")]
// struct Favicon<'a> {
//     _marker: PhantomData<&'a ()>,
// }

// #[get("/static/{resource}")]
// pub async fn static_resources(resource_id: web::Path<String>) -> HttpResponse {
//     match resource_id.into_inner().as_str() {
//         "water.css" => HttpResponse::Ok().content_type("text/css").body(
//             WaterCSS {
//                 _marker: Default::default(),
//             }
//             .render()
//             .unwrap(),
//         ),
//         "logo.png" => HttpResponse::Ok()
//             .content_type("image/png")
//             .body(Ok(EmbedFile::open("templates/logo.png")?)),
//         "favicon.svg" => HttpResponse::Ok().content_type("image/svg+xml").body(
//             Favicon {
//                 _marker: Default::default(),
//             }
//             .render()
//             .unwrap(),
//         ),
//         _ => HttpResponse::NotFound().content_type("text/html").finish(),
//     }
// }
