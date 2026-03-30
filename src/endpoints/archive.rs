// DISCLAIMER
// (c) 2024-05-27 overcuriousity - derived from the original Microbin Project by Daniel Szabo
use crate::args::ARGS;
use crate::util::bip39words::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{remove_expired, resolve_attachment_id};
use crate::AppState;
use actix_web::{get, web, Error, HttpResponse};
use std::io::Write;

#[get("/archive/{id}")]
pub async fn get_archive(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    // Collect needed metadata under the lock, then drop it before blocking I/O
    let (file_names, attachment_id, id_words) = {
        let mut pastas = data.pastas.lock().unwrap();

        let id_intern = if ARGS.hash_ids {
            hashid_to_u64(&id).unwrap_or(0)
        } else {
            to_u64(&id.into_inner()).unwrap_or(0)
        };

        remove_expired(&mut pastas);

        let pasta = pastas.iter().find(|p| p.id == id_intern);
        match pasta {
            None => return Ok(HttpResponse::NotFound().finish()),
            Some(p) if p.encrypt_server => {
                // Archive download for server-encrypted pastas is not supported:
                // there is no auth flow that ends in an authenticated ZIP download.
                // Return 403 rather than redirecting to /auth_file which only handles
                // single-file decryption.
                return Ok(HttpResponse::Forbidden().finish());
            }
            Some(p) => {
                let mut names: Vec<String> = Vec::new();
                let mut total_bytes: u64 = 0;
                if let Some(ref f) = p.file {
                    names.push(f.name().to_owned());
                    total_bytes = total_bytes.saturating_add(f.size.as_u64());
                }
                if let Some(ref attachments) = p.attachments {
                    for a in attachments {
                        names.push(a.name().to_owned());
                        total_bytes = total_bytes.saturating_add(a.size.as_u64());
                    }
                }
                if names.is_empty() {
                    return Ok(HttpResponse::NotFound().finish());
                }
                // Enforce a total archive size cap — ZIP is built in memory, so
                // reject before blocking I/O to avoid OOM on large sets.
                // Use a dedicated 512 MiB ceiling, further bounded by the configured
                // per-file limit, so the in-memory buffer stays manageable.
                const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
                let cap = (ARGS.max_file_size_unencrypted_mb as u64)
                    .saturating_mul(1024 * 1024)
                    .min(MAX_ARCHIVE_BYTES);
                if total_bytes > cap {
                    return Ok(HttpResponse::PayloadTooLarge().finish());
                }
                (names, resolve_attachment_id(p.id), p.id_as_words())
            }
        }
    }; // lock dropped here

    let data_dir = ARGS.data_dir.clone();
    let archive_name = format!("{}.zip", id_words);
    let id_words_closure = attachment_id;

    let zip_bytes = web::block(move || -> Result<Vec<u8>, std::io::Error> {
        let buf = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for name in &file_names {
            let file_path = format!("{}/attachments/{}/{}", data_dir, id_words_closure, name);
            let file_data = std::fs::read(&file_path)?;
            // Sanitize the entry name: strip path separators so unzip tools
            // cannot write outside the target directory (zip-slip).
            let entry_name = name
                .replace(['/', '\\'], "_")
                .trim_start_matches('.')
                .to_string();
            let entry_name = if entry_name.is_empty() { "file".to_string() } else { entry_name };
            zip.start_file(entry_name, options)?;
            zip.write_all(&file_data)?;
        }

        let result = zip.finish()?;
        Ok(result.into_inner())
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", archive_name),
        ))
        .body(zip_bytes))
}
