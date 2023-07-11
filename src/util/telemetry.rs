extern crate reqwest;
extern crate serde;
extern crate serde_json;
use std::{
    thread,
    time::{Duration, Instant},
};

use serde_json::json;

use crate::args::ARGS;

pub fn start_telemetry_thread() {
    // Start a new thread that calls the send_telemetry function every 24 hours
    thread::spawn(|| {
        let mut last_run = Instant::now();
        loop {
            let _ = send_telemetry();

            // Wait for 24 hours since the last run
            let next_run = last_run + Duration::from_secs(60 * 60 * 24);
            let now = Instant::now();
            if next_run > now {
                thread::sleep(next_run - now);
            }
            last_run = Instant::now();
        }
    });
}

fn send_telemetry() -> Result<(), reqwest::Error> {
    // Convert the telemetry object to JSON
    let json_body = json!(ARGS.to_owned().without_secrets().to_owned()).to_string();

    // Send the telemetry data to the API
    reqwest::blocking::Client::new()
        .post("https://api.microbin.eu/telemetry/")
        .header("Content-Type", "application/json")
        .body(json_body)
        .send()?;

    Ok(())
}
