// DISCLAIMER
// (c) 2024-05-27 Mario Stöckl - derived from the original Microbin Project by Daniel Szabo
use crate::args::ARGS;
use crate::util::{bip39words::to_bip39_words, hashids::to_hashids};
use linkify::{LinkFinder, LinkKind};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use qrcode_generator::QrCodeEcc;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static LAST_ORPHAN_CLEANUP: AtomicI64 = AtomicI64::new(0);
const ORPHAN_CLEANUP_INTERVAL_SECS: i64 = 60;

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

            // Attempt deletion under both naming schemes in case BITVAULT_HASH_IDS
            // was toggled between restarts (directory was created under the other scheme).
            for id_str in [to_bip39_words(p.id), to_hashids(p.id)] {
                let dir = format!("{}/attachments/{}", ARGS.data_dir, id_str);
                if Path::new(&dir).exists() {
                    if fs::remove_dir_all(&dir).is_err() {
                        log::error!("Failed to delete attachment directory {}!", dir);
                    }
                }
            }
            false
        }
    });

    // Throttle orphan-directory cleanup to once per minute — avoid per-request
    // filesystem scans on busy servers.
    let last = LAST_ORPHAN_CLEANUP.load(Ordering::Relaxed);
    if timenow - last >= ORPHAN_CLEANUP_INTERVAL_SECS
        && LAST_ORPHAN_CLEANUP
            .compare_exchange(last, timenow, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    {
        // Build known directory names under BOTH naming schemes while the lock
        // is still held, then hand off the actual I/O to a background thread so
        // the mutex is not blocked during the filesystem scan/deletes.
        let attachments_dir = format!("{}/attachments", ARGS.data_dir);
        let known_ids: std::collections::HashSet<String> = pastas
            .iter()
            .flat_map(|p| [to_bip39_words(p.id), to_hashids(p.id)])
            .collect();
        std::thread::spawn(move || {
            // Only delete directories older than 5 minutes to avoid racing with
            // uploads in progress (files are written before the pasta is inserted
            // into shared state, so the id won't be in known_ids yet).
            const SAFETY_SECS: u64 = 300;
            let now = SystemTime::now();

            if let Ok(entries) = fs::read_dir(&attachments_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str().map(|s| s.to_owned()) {
                        if !known_ids.contains(&name) {
                            let path = entry.path();
                            if path.is_dir() {
                                let dominated = entry.metadata().ok().and_then(|m| {
                                    m.modified().ok().map(|mt| {
                                        now.duration_since(mt)
                                            .map(|d| d.as_secs() >= SAFETY_SECS)
                                            .unwrap_or(false)
                                    })
                                });
                                if dominated == Some(true) {
                                    if fs::remove_dir_all(&path).is_err() {
                                        log::error!(
                                            "Failed to remove orphaned attachment dir {:?}",
                                            path
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

/// Resolve the attachment sub-directory name for a pasta ID, trying both naming
/// schemes in case `BITVAULT_HASH_IDS` was toggled between restarts.
pub fn resolve_attachment_id(id: u64) -> String {
    let primary = if ARGS.hash_ids { to_hashids(id) } else { to_bip39_words(id) };
    let primary_dir = format!("{}/attachments/{}", ARGS.data_dir, primary);
    if Path::new(&primary_dir).exists() {
        return primary;
    }
    let alt = if ARGS.hash_ids { to_bip39_words(id) } else { to_hashids(id) };
    let alt_dir = format!("{}/attachments/{}", ARGS.data_dir, alt);
    if Path::new(&alt_dir).exists() {
        return alt;
    }
    primary
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

    // Write the encrypted data to a new file named {original_filename}.enc
    let enc_path = format!("{}.enc", input_file_path);
    let mut f = File::create(&enc_path)?;
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
