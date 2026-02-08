use std::fs;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::args::ARGS;

lazy_static! {
    pub static ref DEVICE_TOKENS: Mutex<Vec<DeviceToken>> = Mutex::new(load_tokens());
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceToken {
    pub token: String,
    pub label: Option<String>,
    pub registered_at: i64,
}

/// The type of event that triggered the notification.
#[derive(Debug, Clone, Copy)]
pub enum PushEvent {
    Created,
    Edited,
    Deleted,
}

impl PushEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            PushEvent::Created => "created",
            PushEvent::Edited => "edited",
            PushEvent::Deleted => "deleted",
        }
    }
}

fn tokens_path() -> String {
    format!("{}/push_tokens.json", ARGS.data_dir)
}

fn load_tokens() -> Vec<DeviceToken> {
    let path = tokens_path();
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_tokens(tokens: &[DeviceToken]) {
    let path = tokens_path();
    if let Ok(data) = serde_json::to_string_pretty(tokens) {
        if let Err(e) = fs::write(&path, data) {
            log::error!("Failed to save push tokens: {}", e);
        }
    }
}

/// Register a new device token. Returns true if newly added, false if already present.
pub fn register_token(token: String, label: Option<String>) -> bool {
    let mut tokens = DEVICE_TOKENS.lock().unwrap();
    if tokens.iter().any(|t| t.token == token) {
        return false;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    tokens.push(DeviceToken {
        token,
        label,
        registered_at: now,
    });
    save_tokens(&tokens);
    true
}

/// Unregister a device token. Returns true if it was found and removed.
pub fn unregister_token(token: &str) -> bool {
    let mut tokens = DEVICE_TOKENS.lock().unwrap();
    let before = tokens.len();
    tokens.retain(|t| t.token != token);
    if tokens.len() != before {
        save_tokens(&tokens);
        true
    } else {
        false
    }
}

/// Send push notification to all registered devices via FCM.
/// This is fire-and-forget: errors are logged but do not block the caller.
pub fn notify_all(event: PushEvent, pasta_id: &str, pasta_type: &str) {
    let pasta_id = pasta_id.to_owned();
    let pasta_type = pasta_type.to_owned();
    if !ARGS.push_notifications {
        return;
    }

    let server_key = match &ARGS.fcm_server_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            log::warn!("Push notifications enabled but MICROBIN_FCM_SERVER_KEY is not set");
            return;
        }
    };

    let tokens: Vec<String> = {
        let locked = DEVICE_TOKENS.lock().unwrap();
        locked.iter().map(|t| t.token.clone()).collect()
    };

    if tokens.is_empty() {
        return;
    }

    let event_str = event.as_str();
    let title = format!("Paste {}", event_str);
    let body = match event {
        PushEvent::Created => format!("A new {} paste was created: {}", pasta_type, pasta_id),
        PushEvent::Edited => format!("Paste {} was edited", pasta_id),
        PushEvent::Deleted => format!("Paste {} was deleted", pasta_id),
    };

    let base_url = ARGS.public_path_as_str();

    // Send to all tokens in a background thread to avoid blocking the request
    std::thread::spawn(move || {
        let client = crate::util::http_client::new();

        for token in &tokens {
            let payload = json!({
                "to": token,
                "notification": {
                    "title": title,
                    "body": body,
                    "click_action": format!("{}/upload/{}", base_url, pasta_id),
                },
                "data": {
                    "event": event_str,
                    "pasta_id": pasta_id,
                    "pasta_type": pasta_type,
                    "url": format!("{}/upload/{}", base_url, pasta_id),
                }
            });

            match client
                .post("https://fcm.googleapis.com/fcm/send")
                .header("Authorization", format!("key={}", server_key))
                .header("Content-Type", "application/json")
                .body(payload.to_string())
                .send()
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        log::warn!(
                            "FCM push failed for token {}: HTTP {}",
                            &token[..token.len().min(12)],
                            resp.status()
                        );
                    }
                }
                Err(e) => {
                    log::warn!("FCM push error for token {}: {}", &token[..token.len().min(12)], e);
                }
            }
        }
    });
}
