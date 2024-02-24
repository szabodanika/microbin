use crate::{args::ARGS, util::hashids::to_u64 as hashid_to_u64};
use crate::util::animalnumbers::to_u64 as animal_to_u64;
use harsh::Harsh;
use lazy_static::lazy_static;

use crate::pasta::Pasta;

lazy_static! {
    pub static ref HARSH: Harsh = Harsh::builder().length(6).build().unwrap();
}

pub fn to_hashids(number: u64) -> String {
    HARSH.encode(&[number])
}

pub fn to_u64(hash_id: &str) -> Result<u64, &str> {
    let ids = HARSH
        .decode(hash_id)
        .map_err(|_e| "Failed to decode hash ID")?;
    let id = ids.first().ok_or("No ID found in hash ID")?;
    Ok(*id)
}

pub fn alias_comparator(id: &str) -> Box<dyn Fn(&Pasta) -> bool> {
    let raw_id = id.to_string();
    let id = if ARGS.hash_ids {
        hashid_to_u64(id).unwrap_or(0)
    } else {
        animal_to_u64(id).unwrap_or(0)
    };
    return Box::new(move |pasta|{
        pasta.id == id || (ARGS.enable_custom_url && pasta.custom_alias.as_ref() == Some(&raw_id))
    });
}
