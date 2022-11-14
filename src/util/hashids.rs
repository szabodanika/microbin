use harsh::Harsh;
use lazy_static::lazy_static;

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
