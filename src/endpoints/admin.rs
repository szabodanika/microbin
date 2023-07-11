use crate::args::{Args, ARGS};
use crate::pasta::Pasta;
use crate::util::misc::remove_expired;
use crate::util::version::{fetch_latest_version, Version, CURRENT_VERSION};
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use askama::Template;
use futures::TryStreamExt;

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate<'a> {
    pastas: &'a Vec<Pasta>,
    args: &'a Args,
    status: &'a String,
    version_string: &'a String,
    message: &'a String,
    update: &'a Option<Version>,
}

#[get("/admin")]
pub async fn get_admin() -> Result<HttpResponse, Error> {
    return Ok(HttpResponse::Found()
        .append_header(("Location", "/auth_admin"))
        .finish());
}

#[post("/admin")]
pub async fn post_admin(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut username = String::from("");
    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "username" {
            while let Some(chunk) = field.try_next().await? {
                username.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        } else if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    if username != ARGS.auth_admin_username || password != ARGS.auth_admin_password {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/auth_admin/incorrect"))
            .finish());
    }

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    // sort pastas in reverse-chronological order of creation time
    pastas.sort_by(|a, b| b.created.cmp(&a.created));

    // todo status report more sophisticated
    let mut status = "OK";
    let mut message = "";

    if ARGS.public_path.is_none() {
        status = "WARNING";
        message = "Warning: No public URL set with --public-path parameter. QR code and URL Copying functions have been disabled"
    }

    if ARGS.auth_admin_username == "admin" && ARGS.auth_admin_password == "m1cr0b1n" {
        status = "WARNING";
        message = "Warning: You are using the default admin login details. This is a security risk, please change them."
    }

    let update;

    if !ARGS.disable_update_checking {
        let latest_version_res = fetch_latest_version().await;
        if latest_version_res.is_ok() {
            let latest_version = latest_version_res.unwrap();
            if latest_version.newer_than_current() {
                update = Some(latest_version);
            } else {
                update = None;
            }
        } else {
            update = None;
        }
    } else {
        update = None;
    }

    Ok(HttpResponse::Ok().content_type("text/html").body(
        AdminTemplate {
            pastas: &pastas,
            args: &ARGS,
            status: &String::from(status),
            version_string: &format!("{}", CURRENT_VERSION.long_title),
            message: &String::from(message),
            update: &update,
        }
        .render()
        .unwrap(),
    ))
}
