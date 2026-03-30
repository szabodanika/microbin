// DISCLAIMER
// (c) 2024-05-27 Mario Stöckl - derived from the original Microbin Project by Daniel Szabo
use crate::args::ARGS;
use crate::util::bip39words::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use actix_web::{get, web, Error, HttpResponse};
use std::io::Write;

#[get("/archive/{id}")]
pub async fn get_archive(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();

    let id_intern = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    let mut index: usize = 0;
    let mut found = false;
    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id_intern {
            index = i;
            found = true;
            break;
        }
    }

    if !found {
        return Ok(HttpResponse::NotFound().finish());
    }

    if pastas[index].encrypt_server {
        return Ok(HttpResponse::Found()
            .append_header((
                "Location",
                format!("/auth_file/{}", pastas[index].id_as_words()),
            ))
            .finish());
    }

    // Collect all files to zip
    let mut file_names: Vec<String> = Vec::new();
    if let Some(ref f) = pastas[index].file {
        file_names.push(f.name().to_owned());
    }
    if let Some(ref attachments) = pastas[index].attachments {
        for a in attachments {
            file_names.push(a.name().to_owned());
        }
    }

    if file_names.is_empty() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let id_words = pastas[index].id_as_words();
    let data_dir = ARGS.data_dir.clone();

    let zip_bytes = web::block(move || -> Result<Vec<u8>, std::io::Error> {
        let buf = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for name in &file_names {
            let file_path = format!("{}/attachments/{}/{}", data_dir, id_words, name);
            let file_data = std::fs::read(&file_path)?;
            zip.start_file(name, options)?;
            zip.write_all(&file_data)?;
        }

        let result = zip.finish()?;
        Ok(result.into_inner())
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let archive_name = format!("{}.zip", pastas[index].id_as_words());
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", archive_name),
        ))
        .body(zip_bytes))
}
