use crate::args::ARGS;
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use actix_web::{get, web, Error, HttpResponse};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use std::io::Seek;

#[get("/archive/{id}")]
pub async fn get_archive(
    id: web::Path<String>,
    data: web::Data<AppState>,
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
        let pasta = &pastas[index];

        // Create a temporary file for the zip
        let zip_file = tempfile::tempfile()?;
        let mut zip = zip::ZipWriter::new(zip_file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        // Helper to add file to zip
        let mut add_file_to_zip = |name: &str, path: PathBuf| -> Result<(), std::io::Error> {
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            Ok(())
        };

        // Add primary file
        if let Some(file) = &pasta.file {
            let file_path = if pasta.encrypt_server {
                // If encrypted, we download the encrypted data
                 format!(
                    "{}/attachments/{}/{}.enc",
                    ARGS.data_dir,
                    pasta.id_as_animals(),
                    file.name()
                )
            } else {
                 format!(
                    "{}/attachments/{}/{}",
                    ARGS.data_dir,
                    pasta.id_as_animals(),
                    file.name()
                )
            };
            
            // Check if .enc exists, if not try legacy data.enc for encrypted
             let mut final_path = PathBuf::from(&file_path);
             if pasta.encrypt_server && !final_path.exists() {
                  final_path = PathBuf::from(format!(
                    "{}/attachments/{}/data.enc",
                    ARGS.data_dir,
                    pasta.id_as_animals()
                ));
             }

            if final_path.exists() {
                 let filename = if pasta.encrypt_server {
                     format!("{}.enc", file.name())
                 } else {
                     file.name().to_string()
                 };
                add_file_to_zip(&filename, final_path)?;
            }
        }

        // Add attachments
        if let Some(attachments) = &pasta.attachments {
            for file in attachments {
                 let file_path = if pasta.encrypt_server {
                    format!(
                        "{}/attachments/{}/{}.enc",
                        ARGS.data_dir,
                        pasta.id_as_animals(),
                        file.name()
                    )
                } else {
                    format!(
                        "{}/attachments/{}/{}",
                        ARGS.data_dir,
                        pasta.id_as_animals(),
                        file.name()
                    )
                };
                 let final_path = PathBuf::from(&file_path);
                 if final_path.exists() {
                     let filename = if pasta.encrypt_server {
                         format!("{}.enc", file.name())
                     } else {
                         file.name().to_string()
                     };
                    add_file_to_zip(&filename, final_path)?;
                 }
            }
        }

        let mut zip_file = zip.finish().map_err(actix_web::error::ErrorInternalServerError)?;
        
        // Read the zip back into a buffer to send
        zip_file.seek(std::io::SeekFrom::Start(0))?;
        let mut buffer = Vec::new();
        zip_file.read_to_end(&mut buffer)?;

        return Ok(HttpResponse::Ok()
            .content_type("application/zip")
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}.zip\"", pasta.id_as_animals()),
            ))
            .body(buffer));
    }

    Ok(HttpResponse::NotFound().finish())
}
