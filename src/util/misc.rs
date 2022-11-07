use std::time::{SystemTime, UNIX_EPOCH};

use crate::args::ARGS;
use linkify::{LinkFinder, LinkKind};
use qrcode_generator::QrCodeEcc;
use std::fs;

use crate::{dbio, Pasta};

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
            // remove the file itself
            if let Some(file) = &p.file {
                if fs::remove_file(format!(
                    "./pasta_data/public/{}/{}",
                    p.id_as_animals(),
                    file.name()
                ))
                .is_err()
                {
                    log::error!("Failed to delete file {}!", file.name())
                }

                // and remove the containing directory
                if fs::remove_dir(format!("./pasta_data/public/{}/", p.id_as_animals())).is_err() {
                    log::error!("Failed to delete directory {}!", file.name())
                }
            }
            false
        }
    });

    dbio::save_to_file(pastas);
}

pub fn string_to_qr_svg(str: &str) -> String {
    qrcode_generator::to_svg_to_string(str, QrCodeEcc::Low, 256, None::<&str>).unwrap()
}

pub fn is_valid_url(url: &str) -> bool {
    let finder = LinkFinder::new();
    let spans: Vec<_> = finder.spans(url).collect();
    spans[0].as_str() == url && Some(&LinkKind::Url) == spans[0].kind()
}
