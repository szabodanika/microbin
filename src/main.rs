extern crate core;

use crate::args::ARGS;
use crate::endpoints::{
    admin, auth_admin, auth_upload, create, edit, errors, file, guide, list,
    pasta as pasta_endpoint, qr, remove, static_resources,
};
use crate::pasta::Pasta;
use crate::util::db::read_all;
use crate::util::telemetry::start_telemetry_thread;
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
    pub mod animalnumbers;
    pub mod auth;
    pub mod db;
    pub mod db_json;
    pub mod db_sqlite;
    pub mod hashids;
    pub mod misc;
    pub mod syntaxhighlighter;
    pub mod telemetry;
    pub mod version;
}

pub mod endpoints {
    pub mod admin;
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

    match fs::create_dir_all(format!("./{}/public", ARGS.data_dir)) {
        Ok(dir) => dir,
        Err(error) => {
            log::error!(
                "Couldn't create data directory ./{}/attachments/: {:?}",
                ARGS.data_dir,
                error
            );
            panic!(
                "Couldn't create data directory ./{}/attachments/: {:?}",
                ARGS.data_dir, error
            );
        }
    };

    let data = web::Data::new(AppState {
        pastas: Mutex::new(read_all()),
    });

    if !ARGS.disable_telemetry {
        start_telemetry_thread();
    }

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::NormalizePath::trim())
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
            .service(web::resource("/upload").route(web::post().to(create::create)))
            .default_service(web::route().to(errors::not_found))
            .wrap(middleware::Logger::default())
            .service(remove::remove)
            .service(remove::post_remove)
            .service(list::list)
            .service(create::index_with_status)
            .wrap(Condition::new(
                ARGS.auth_basic_username.is_some()
                    && ARGS.auth_basic_username.as_ref().unwrap().trim() != "",
                HttpAuthentication::basic(util::auth::auth_validator),
            ))
    })
    .bind((ARGS.bind, ARGS.port))?
    .workers(ARGS.threads as usize)
    .run()
    .await
}
