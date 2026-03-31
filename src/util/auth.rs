// DISCLAIMER
// (c) 2024-05-27 overcuriousity - derived from the original Microbin Project by Daniel Szabo
use actix_multipart::Multipart;
use actix_web::dev::ServiceRequest;
use actix_web::web::Bytes;
use actix_web::{error, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use futures::TryStreamExt;

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

/// Basic Auth validator for the API scope. Returns a JSON 401 instead of the
/// web UI's HTML 400, keeping all API error responses machine-parseable.
pub async fn api_auth_validator(
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
        _ => {
            use actix_web::HttpResponse;
            let body = serde_json::json!({"error": "Invalid credentials", "code": "UNAUTHORIZED"});
            let response = HttpResponse::Unauthorized().json(body);
            Err((error::InternalError::from_response("Unauthorized", response).into(), req))
        }
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
