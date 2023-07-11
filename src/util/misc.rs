use crate::args::ARGS;
use linkify::{LinkFinder, LinkKind};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use qrcode_generator::QrCodeEcc;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Pasta;

use super::db::delete;

pub fn remove_expired(pastas: &mut Vec<Pasta>) {
    // get current time - this will be needed to check which pastas have expired
    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => {
            log::error!("SystemTime before UNIX EPOCH!");
            0
        }
    } as i64;

    pastas.retain(|p| {
        // keep if:
        //  expiration is `never` or not reached
        //  AND
        //  read count is less than burn limit, or no limit set
        //  AND
        //  has been read in the last N days where N is the arg --gc-days OR N is 0 (no GC)
        if (p.expiration == 0 || p.expiration > timenow)
            && (p.read_count < p.burn_after_reads || p.burn_after_reads == 0)
            && (p.last_read_days_ago() < ARGS.gc_days || ARGS.gc_days == 0)
        {
            // keep
            true
        } else {
            // remove from database
            delete(None, Some(p.id));

            // remove the file itself
            if let Some(file) = &p.file {
                if fs::remove_file(format!(
                    "./{}/attachments/{}/{}",
                    ARGS.data_dir,
                    p.id_as_animals(),
                    file.name()
                ))
                .is_err()
                {
                    log::error!("Failed to delete file {}!", file.name())
                }

                // and remove the containing directory
                if fs::remove_dir(format!(
                    "./{}/attachments/{}/",
                    ARGS.data_dir,
                    p.id_as_animals()
                ))
                .is_err()
                {
                    log::error!("Failed to delete directory {}!", file.name())
                }
            }
            false
        }
    });
}

pub fn string_to_qr_svg(str: &str) -> String {
    qrcode_generator::to_svg_to_string(str, QrCodeEcc::Low, 256, None::<&str>).unwrap()
}

pub fn is_valid_url(url: &str) -> bool {
    let finder = LinkFinder::new();
    let spans: Vec<_> = finder.spans(url).collect();
    spans[0].as_str() == url && Some(&LinkKind::Url) == spans[0].kind()
}

pub fn encrypt(text_str: &str, key_str: &str) -> String {
    if text_str.is_empty() {
        return String::from("");
    }

    let mc = new_magic_crypt!(key_str, 256);

    mc.encrypt_str_to_base64(text_str)
}

pub fn decrypt(text_str: &str, key_str: &str) -> Result<String, magic_crypt::MagicCryptError> {
    if text_str.is_empty() {
        return Ok(String::from(""));
    }

    let mc = new_magic_crypt!(key_str, 256);

    mc.decrypt_base64_to_string(text_str)
}

pub fn encrypt_file(
    passphrase: &str,
    input_file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the input file into memory
    let file = File::open(input_file_path).expect("Tried to encrypt non-existent file");
    let mut reader = BufReader::new(file);
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data)?;

    // Create a MagicCrypt instance with the given passphrase
    let mc = new_magic_crypt!(passphrase, 256);

    // Encrypt the input data
    let ciphertext = mc.encrypt_bytes_to_bytes(&input_data[..]);

    // Write the encrypted data to a new file with the .enc extension
    let mut f = File::create(
        Path::new(input_file_path)
            .with_file_name("data")
            .with_extension("enc"),
    )?;
    f.write_all(ciphertext.as_slice())?;

    // Delete the original input file
    // input_file.seek(SeekFrom::Start(0))?;
    fs::remove_file(input_file_path)?;

    Ok(())
}

pub fn decrypt_file(
    passphrase: &str,
    input_file: &File,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Read the input file into memory
    let mut reader = BufReader::new(input_file);
    let mut ciphertext = Vec::new();
    reader.read_to_end(&mut ciphertext)?;

    // Create a MagicCrypt instance with the given passphrase
    let mc = new_magic_crypt!(passphrase, 256);
    // Encrypt the input data
    let res = mc.decrypt_bytes_to_bytes(&ciphertext[..]);

    if res.is_err() {
        return Err("Failed to decrypt file".into());
    }

    Ok(res.unwrap())
}
