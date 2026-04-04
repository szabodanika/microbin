// DISCLAIMER
// (c) 2024-05-27 overcuriousity - derived from the original Microbin Project by Daniel Szabo
extern crate core;

use crate::args::ARGS;
use crate::endpoints::{
    admin, api, archive, auth_admin, auth_upload, create, edit, errors, file, guide, list,
    pasta as pasta_endpoint, qr, remove, static_resources,
};
use crate::pasta::Pasta;
use crate::util::db::read_all;
use actix_web::middleware::Condition;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::fs;
use std::io::Write;
use std::sync::Mutex;

pub mod args;
pub mod pasta;

pub mod util {
    pub mod bip39words;
    pub mod auth;
    pub mod db;
    pub mod db_json;
    #[cfg(feature = "default")]
    pub mod db_sqlite;
    pub mod hashids;
    pub mod misc;
    pub mod syntaxhighlighter;
    pub mod version;
}

pub mod endpoints {
    pub mod admin;
    pub mod api;
    pub mod archive;
    pub mod auth_admin;
    pub mod auth_upload;
    pub mod create;
    pub mod edit;
    pub mod errors;
    pub mod file;
    pub mod guide;
    pub mod list;
    pub mod pasta;
    pub mod qr;
    pub mod remove;
    pub mod static_resources;
}

pub struct AppState {
    pub pastas: Mutex<Vec<Pasta>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    log::info!(
        "MicroBin starting on http://{}:{}",
        ARGS.bind.to_string(),
        ARGS.port.to_string()
    );

    match fs::create_dir_all(format!("{}/public", ARGS.data_dir)) {
        Ok(dir) => dir,
        Err(error) => {
            log::error!(
                "Couldn't create data directory {}/attachments/: {:?}",
                ARGS.data_dir,
                error
            );
            panic!(
                "Couldn't create data directory {}/attachments/: {:?}",
                ARGS.data_dir, error
            );
        }
    };

    let data = web::Data::new(AppState {
        pastas: Mutex::new(read_all()),
    });

    let api_key_set = ARGS.api_key.as_deref().map(|k| !k.trim().is_empty()).unwrap_or(false);
    let basic_auth_set = ARGS.auth_basic_username.as_deref().map(|u| !u.trim().is_empty()).unwrap_or(false);
    if !api_key_set && !basic_auth_set {
        log::warn!("API is accessible without authentication. Set BITVAULT_API_KEY to require a bearer token.");
    }

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::new("%{r}a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
            // These endpoints are always open — register before the auth-wrapped scopes.
            .service(web::resource("/openapi.yaml").route(web::get().to(api::openapi_spec)))
            .service(web::resource("/docs").route(web::get().to(api::swagger_docs)))
            .service(web::resource("/api/v1/health").route(web::get().to(api::health)))
            .service(
                web::scope("/api/v1")
                    .app_data(
                        web::JsonConfig::default()
                            .error_handler(api::json_error_handler),
                    )
                    .wrap(Condition::new(
                        ARGS.auth_basic_username.is_some()
                            && ARGS.auth_basic_username.as_ref().unwrap().trim() != "",
                        HttpAuthentication::basic(util::auth::api_auth_validator),
                    ))
                    .route("/paste",      web::post().to(api::create_paste))
                    .route("/paste/{id}", web::get().to(api::get_paste))
                    .route("/paste/{id}", web::delete().to(api::delete_paste))
                    .route("/paste/{id}", web::patch().to(api::update_paste))
                    .route("/pastes",     web::get().to(api::list_pastes))
                    .default_service(web::route().to(api::not_found)),
            )
            .service(
                web::scope("")
                    .wrap(Condition::new(
                        ARGS.auth_basic_username.is_some()
                            && ARGS.auth_basic_username.as_ref().unwrap().trim() != "",
                        HttpAuthentication::basic(util::auth::auth_validator),
                    ))
                    .service(create::index)
                    .service(guide::guide)
                    .service(auth_admin::auth_admin)
                    .service(auth_upload::auth_file_with_status)
                    .service(auth_admin::auth_admin_with_status)
                    .service(auth_upload::auth_upload_with_status)
                    .service(auth_upload::auth_raw_pasta_with_status)
                    .service(auth_upload::auth_edit_private_with_status)
                    .service(auth_upload::auth_remove_private_with_status)
                    .service(auth_upload::auth_file)
                    .service(auth_upload::auth_upload)
                    .service(auth_upload::auth_raw_pasta)
                    .service(auth_upload::auth_edit_private)
                    .service(auth_upload::auth_remove_private)
                    .service(pasta_endpoint::getpasta)
                    .service(pasta_endpoint::postpasta)
                    .service(pasta_endpoint::getshortpasta)
                    .service(pasta_endpoint::postshortpasta)
                    .service(pasta_endpoint::getrawpasta)
                    .service(pasta_endpoint::postrawpasta)
                    .service(pasta_endpoint::redirecturl)
                    .service(pasta_endpoint::shortredirecturl)
                    .service(edit::get_edit)
                    .service(edit::get_edit_with_status)
                    .service(edit::post_edit)
                    .service(edit::post_edit_private)
                    .service(edit::post_submit_edit_private)
                    .service(admin::get_admin)
                    .service(admin::post_admin)
                    .service(static_resources::static_resources)
                    .service(qr::getqr)
                    .service(file::get_file)
                    .service(file::post_secure_file)
                    .service(archive::get_archive)
                    .service(web::resource("/upload").route(web::post().to(create::create)))
                    .service(remove::remove)
                    .service(remove::post_remove)
                    .service(list::list)
                    .service(create::index_with_status)
                    .default_service(web::route().to(errors::not_found)),
            )
    })
    .bind((ARGS.bind, ARGS.port))?
    .workers(ARGS.threads as usize)
    .run()
    .await
}
