use std::fs::{self, File};
use std::path::PathBuf;

use crate::args::ARGS;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::util::{animalnumbers::to_u64, misc::decrypt_file};
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use futures::TryStreamExt;

#[post("/secure_file/{id}")]
pub async fn post_secure_file(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    // find the index of the pasta in the collection based on u64 id
    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            index = i;
            found = true;
            break;
        }
    }

    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password.push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
            }
        }
    }

    if found {
        if let Some(ref pasta_file) = pastas[index].file {
            let file = File::open(format!(
                "./{}/attachments/{}/data.enc",
                ARGS.data_dir,
                pastas[index].id_as_animals()
            ))?;

            let decrypted_data: Vec<u8> = decrypt_file(&password, &file)?;

            // Set the content type based on the file extension
            let content_type = mime_guess::from_path(&pasta_file.name)
                .first_or_octet_stream()
                .to_string();

            // Create a response with the decrypted data
            let response = HttpResponse::Ok()
                .content_type(content_type)
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", pasta_file.name()),
                ))
                .body(decrypted_data);
            return Ok(response);
        }
    }
    Ok(HttpResponse::NotFound().finish())
}

#[get("/file/{id}")]
pub async fn get_file(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id_intern = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    // find the index of the pasta in the collection based on u64 id
    let mut index: usize = 0;
    let mut found: bool = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id_intern {
            index = i;
            found = true;
            break;
        }
    }

    if found {
        if let Some(ref pasta_file) = pastas[index].file {
            if pastas[index].encrypt_server {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("/auth_file/{}", pastas[index].id_as_animals()),
                    ))
                    .finish());
            }

            // Construct the path to the file
            let file_path = format!(
                "./{}/attachments/{}/{}",
                ARGS.data_dir,
                pastas[index].id_as_animals(),
                pasta_file.name()
            );
            let file_path = PathBuf::from(file_path);

            // Read the contents of the file into memory
            // let mut file_content = Vec::new();
            // let mut file = File::open(&file_path)?;
            // file.read_exact(&mut file_content)?;

            let file_contents = fs::read(&file_path)?;

            // Set the content type based on the file extension
            let content_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            // Create an HttpResponse object with the file contents as the response body
            let response = HttpResponse::Ok()
                .content_type(content_type)
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", pasta_file.name()),
                ))
                .body(file_contents);

            return Ok(response);
        }
    }

    Ok(HttpResponse::NotFound().finish())
}
