use actix_multipart::Multipart;
use actix_web::dev::ServiceRequest;
use actix_web::web::Bytes;
use actix_web::{error, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use futures::TryStreamExt;
use actix_web::HttpRequest;

use crate::args::ARGS;

pub async fn auth_validator(
    req: ServiceRequest,
    creds: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    match (
        ARGS.auth_basic_username.as_ref(),
        ARGS.auth_basic_password.as_ref(),
        creds.password(),
    ) {
        (Some(conf_user), Some(conf_pwd), Some(cred_pwd))
            if creds.user_id() == conf_user && conf_pwd == cred_pwd =>
        {
            Ok(req)
        }
        _ => Err((error::ErrorBadRequest("Invalid login details."), req)),
    }
}

pub async fn password_from_multipart(mut payload: Multipart) -> Result<String, Error> {
    let mut password = String::new();

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == Some("password") {
            let password_bytes = field.bytes(1024).await.unwrap_or(Ok(Bytes::new()))?;
            password = String::from_utf8_lossy(&password_bytes).to_string();
        }
    }
    Ok(password)
}

/// Extract user information from Google IAP headers
///
/// Google IAP sends user information in the following headers:
/// - X-Goog-Authenticated-User-Email: The user's email address
/// - X-Goog-Authenticated-User-Id: The user's unique ID
///
/// Returns the user's email if available, otherwise the user ID, or None if neither is present
pub fn extract_google_iap_user(req: &HttpRequest) -> Option<String> {
    // First try to get the email
    if let Some(email) = req.headers().get("X-Goog-Authenticated-User-Email") {
        if let Ok(email_str) = email.to_str() {
            // The header format is "accounts.google.com:user@example.com"
            // We want to extract just the email part
            if let Some(email_part) = email_str.split(':').nth(1) {
                return Some(email_part.to_string());
            }
        }
    }

    // Fallback to user ID
    if let Some(user_id) = req.headers().get("X-Goog-Authenticated-User-Id") {
        if let Ok(user_id_str) = user_id.to_str() {
            // The header format is "accounts.google.com:123456789"
            // We want to extract just the ID part
            if let Some(id_part) = user_id_str.split(':').nth(1) {
                return Some(format!("User ID: {}", id_part));
            }
        }
    }

    None
}
