// DISCLAIMER
// (c) 2024-05-27 overcuriousity - derived from the original Microbin Project by Daniel Szabo
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::endpoints::create::{expiration_to_timestamp, is_valid_expiration};
use crate::util::bip39words::to_u64;
use crate::util::db::{delete, insert, update};
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{decrypt, encrypt, remove_expired, resolve_attachment_id};
use crate::util::version::CURRENT_VERSION;
use crate::{AppState, ARGS};

// ─── Request / Response types ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreatePasteRequest {
    pub content: String,
    pub extension: Option<String>,
    pub privacy: Option<String>,
    pub expiration: Option<String>,
    pub burn_after_reads: Option<u64>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct CreatePasteResponse {
    pub id: String,
    pub url: String,
    pub expires_at: Option<i64>,
    pub privacy: String,
}

#[derive(Serialize)]
pub struct PasteResponse {
    pub id: String,
    pub content: String,
    pub pasta_type: String,
    pub extension: String,
    pub privacy: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub read_count: u64,
    pub burn_after_reads: u64,
    pub has_file: bool,
    pub url: String,
}

#[derive(Serialize)]
pub struct PasteListItem {
    pub id: String,
    pub pasta_type: String,
    pub privacy: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub read_count: u64,
}

#[derive(Deserialize)]
pub struct UpdatePasteRequest {
    pub content: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Constant-time byte comparison to prevent timing attacks on bearer tokens.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn require_api_key(req: &HttpRequest) -> Result<(), HttpResponse> {
    let Some(ref key) = ARGS.api_key else {
        return Ok(());
    };
    let key = key.trim();
    if key.is_empty() {
        return Ok(());
    }
    let provided = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .and_then(|v| {
            let (scheme, token) = v.split_once(' ')?;
            if scheme.eq_ignore_ascii_case("bearer") { Some(token.trim()) } else { None }
        });
    if provided.map(|t| ct_eq(t.as_bytes(), key.as_bytes())).unwrap_or(false) {
        Ok(())
    } else {
        Err(api_error(401, "API_KEY_REQUIRED", "Valid API key required"))
    }
}

fn pasta_password(req: &HttpRequest) -> String {
    req.headers()
        .get("X-Pasta-Password")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

fn api_error(status: u16, code: &str, message: &str) -> HttpResponse {
    let body = ErrorResponse {
        error: message.to_string(),
        code: code.to_string(),
    };
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    )
    .json(body)
}

pub fn json_error_handler(
    err: actix_web::error::JsonPayloadError,
    _req: &HttpRequest,
) -> actix_web::Error {
    let response = api_error(400, "INVALID_JSON", &err.to_string());
    actix_web::error::InternalError::from_response(err, response).into()
}

fn privacy_string(pasta: &crate::pasta::Pasta) -> &'static str {
    if pasta.encrypt_client && pasta.encrypt_server {
        "secret"
    } else if pasta.encrypt_server {
        "private"
    } else if pasta.readonly {
        "readonly"
    } else if !pasta.private {
        "public"
    } else {
        "unlisted"
    }
}

fn expires_at(pasta: &crate::pasta::Pasta) -> Option<i64> {
    if pasta.expiration == 0 {
        None
    } else {
        Some(pasta.expiration)
    }
}

fn pasta_url(pasta: &crate::pasta::Pasta) -> String {
    format!("{}/upload/{}", ARGS.public_path_as_str(), pasta.id_as_words())
}

fn resolve_id(id: &str) -> Option<u64> {
    if ARGS.hash_ids {
        hashid_to_u64(id).ok()
    } else {
        to_u64(id).ok()
    }
}

fn timenow() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs() as i64,
        Err(_) => 0,
    }
}

fn to_paste_response(pasta: &crate::pasta::Pasta, content: String) -> PasteResponse {
    PasteResponse {
        id: pasta.id_as_words(),
        content,
        pasta_type: pasta.pasta_type.clone(),
        extension: pasta.extension.clone(),
        privacy: privacy_string(pasta).to_string(),
        created_at: pasta.created,
        expires_at: expires_at(pasta),
        read_count: pasta.read_count,
        burn_after_reads: pasta.burn_after_reads,
        has_file: pasta.has_file(),
        url: pasta_url(pasta),
    }
}

// ─── Handlers ────────────────────────────────────────────────────────────────

pub async fn not_found() -> HttpResponse {
    api_error(404, "NOT_FOUND", "endpoint not found")
}

pub async fn openapi_spec() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/yaml")
        .body(include_str!("../../openapi.yaml"))
}

pub async fn swagger_docs() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/assets/swagger.html"))
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        version: CURRENT_VERSION.title.as_ref(),
    })
}

pub async fn create_paste(
    data: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreatePasteRequest>,
) -> HttpResponse {
    if let Err(e) = require_api_key(&req) {
        return e;
    }

    if body.content.is_empty() {
        return api_error(400, "CONTENT_REQUIRED", "content must not be empty");
    }

    let privacy = body.privacy.as_deref().unwrap_or("unlisted");
    match privacy {
        "public" | "unlisted" | "private" => {}
        "readonly" | "secret" => {
            return api_error(
                400,
                "INVALID_PRIVACY",
                "readonly and secret privacy levels are not supported via the API",
            );
        }
        _ => {
            return api_error(
                400,
                "INVALID_PRIVACY",
                "privacy must be one of: public, unlisted, private",
            );
        }
    }

    if privacy == "private" && body.password.as_deref().unwrap_or("").is_empty() {
        return api_error(
            422,
            "PASSWORD_REQUIRED",
            "password is required for private pastes",
        );
    }

    let expiration_str = body
        .expiration
        .as_deref()
        .unwrap_or(ARGS.default_expiry.as_str());
    if !is_valid_expiration(expiration_str) {
        return api_error(
            400,
            "INVALID_EXPIRATION",
            "expiration value not allowed by server configuration",
        );
    }

    let now = timenow();
    let encrypt_server = privacy == "private";
    let password = body.password.as_deref().unwrap_or("").to_string();

    let content = if encrypt_server {
        encrypt(&body.content, &password)
    } else {
        body.content.clone()
    };

    use crate::util::misc::is_valid_url;
    let pasta_type = if is_valid_url(&body.content) {
        "url".to_string()
    } else {
        "text".to_string()
    };

    let new_pasta = crate::pasta::Pasta {
        id: rand::random_range(0..=8589934591u64),
        content,
        file: None,
        attachments: None,
        extension: body.extension.clone().unwrap_or_default(),
        private: privacy != "public",
        readonly: false,
        editable: ARGS.editable,
        encrypt_server,
        encrypt_client: false,
        encrypted_key: Some(String::new()),
        created: now,
        expiration: expiration_to_timestamp(expiration_str, now),
        last_read: now,
        read_count: 0,
        burn_after_reads: body.burn_after_reads.unwrap_or(0),
        pasta_type,
    };

    let response = CreatePasteResponse {
        id: new_pasta.id_as_words(),
        url: pasta_url(&new_pasta),
        expires_at: expires_at(&new_pasta),
        privacy: privacy.to_string(),
    };

    let mut pastas = data.pastas.lock().unwrap();
    pastas.push(new_pasta);
    if let Some(pasta) = pastas.last() {
        insert(Some(&pastas), Some(pasta));
    }

    HttpResponse::Created().json(response)
}

pub async fn get_paste(
    data: web::Data<AppState>,
    id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = require_api_key(&req) {
        return e;
    }

    let password = pasta_password(&req);
    let mut pastas = data.pastas.lock().unwrap();
    let id_num = match resolve_id(&id) {
        Some(n) => n,
        None => return api_error(404, "NOT_FOUND", "paste not found or expired"),
    };

    remove_expired(&mut pastas);

    let index = match pastas.iter().position(|p| p.id == id_num) {
        Some(i) => i,
        None => return api_error(404, "NOT_FOUND", "paste not found or expired"),
    };

    // Secret pastes (encrypt_client) are client-encrypted; the server returns ciphertext as-is.
    // Private pastes (encrypt_server && !encrypt_client) are decrypted server-side.
    let content = if pastas[index].encrypt_server && !pastas[index].encrypt_client {
        if password.is_empty() {
            return api_error(
                401,
                "PASSWORD_REQUIRED",
                "X-Pasta-Password header required for this paste",
            );
        }
        match decrypt(&pastas[index].content, &password) {
            Ok(s) => s,
            Err(_) => return api_error(403, "WRONG_PASSWORD", "incorrect password"),
        }
    } else {
        pastas[index].content.clone()
    };

    pastas[index].read_count += 1;
    pastas[index].last_read = timenow();
    update(Some(&pastas), Some(&pastas[index]));

    let resp = to_paste_response(&pastas[index], content);
    HttpResponse::Ok().json(resp)
}

pub async fn delete_paste(
    data: web::Data<AppState>,
    id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = require_api_key(&req) {
        return e;
    }

    let password = pasta_password(&req);
    let mut pastas = data.pastas.lock().unwrap();
    let id_num = match resolve_id(&id) {
        Some(n) => n,
        None => return api_error(404, "NOT_FOUND", "paste not found or expired"),
    };

    remove_expired(&mut pastas);

    let index = match pastas.iter().position(|p| p.id == id_num) {
        Some(i) => i,
        None => return api_error(404, "NOT_FOUND", "paste not found or expired"),
    };

    // Private pastes: content is encrypted, validate by decrypting it.
    // Secret pastes (encrypt_client): server-side key unavailable, allow delete via API key alone.
    // Readonly pastes: content is plaintext, validate via encrypted_key (if one was set).
    if pastas[index].encrypt_server && !pastas[index].encrypt_client {
        if password.is_empty() {
            return api_error(
                401,
                "PASSWORD_REQUIRED",
                "X-Pasta-Password header required to delete this paste",
            );
        }
        let content_copy = pastas[index].content.clone();
        if decrypt(&content_copy, &password).is_err() {
            return api_error(403, "WRONG_PASSWORD", "incorrect password");
        }
    } else if pastas[index].readonly {
        let encrypted_key = pastas[index].encrypted_key.as_deref().unwrap_or("");
        if !encrypted_key.is_empty() {
            if password.is_empty() {
                return api_error(
                    401,
                    "PASSWORD_REQUIRED",
                    "X-Pasta-Password header required to delete this paste",
                );
            }
            if decrypt(encrypted_key, &password).is_err() {
                return api_error(403, "WRONG_PASSWORD", "incorrect password");
            }
        }
    }

    let id_str = resolve_attachment_id(id_num);
    let dir = format!("{}/attachments/{}", ARGS.data_dir, id_str);
    if Path::new(&dir).exists() {
        if fs::remove_dir_all(&dir).is_err() {
            log::error!("API: failed to delete attachment directory {}", dir);
        }
    }

    pastas.remove(index);
    delete(Some(&pastas), Some(id_num));

    HttpResponse::NoContent().finish()
}

pub async fn update_paste(
    data: web::Data<AppState>,
    id: web::Path<String>,
    req: HttpRequest,
    body: web::Json<UpdatePasteRequest>,
) -> HttpResponse {
    if let Err(e) = require_api_key(&req) {
        return e;
    }

    if body.content.is_empty() {
        return api_error(400, "CONTENT_REQUIRED", "content must not be empty");
    }

    // Accept password via X-Pasta-Password header (consistent with GET/DELETE) or JSON body field.
    let password = req
        .headers()
        .get("X-Pasta-Password")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| body.password.as_deref().unwrap_or("").to_string());
    let mut pastas = data.pastas.lock().unwrap();
    let id_num = match resolve_id(&id) {
        Some(n) => n,
        None => return api_error(404, "NOT_FOUND", "paste not found or expired"),
    };

    remove_expired(&mut pastas);

    let index = match pastas.iter().position(|p| p.id == id_num) {
        Some(i) => i,
        None => return api_error(404, "NOT_FOUND", "paste not found or expired"),
    };

    if !pastas[index].editable {
        return api_error(400, "NOT_EDITABLE", "this paste is not editable");
    }

    if pastas[index].encrypt_client {
        return api_error(
            400,
            "INVALID_PRIVACY",
            "client-encrypted pastes cannot be updated via the API",
        );
    }

    if pastas[index].encrypt_server || pastas[index].readonly {
        if password.is_empty() {
            return api_error(
                401,
                "PASSWORD_REQUIRED",
                "X-Pasta-Password header required to update this paste",
            );
        }
        if pastas[index].readonly {
            // Readonly paste content is plaintext; validate via encrypted_key.
            let encrypted_key = pastas[index].encrypted_key.as_deref().unwrap_or("");
            if !encrypted_key.is_empty() && decrypt(encrypted_key, &password).is_err() {
                return api_error(403, "WRONG_PASSWORD", "incorrect password");
            }
            // Keep content as plaintext — readonly pastes are not encrypted at rest.
            pastas[index].content = body.content.clone();
        } else {
            // Private paste: content is encrypted, validate by decrypting it.
            let content_copy = pastas[index].content.clone();
            if decrypt(&content_copy, &password).is_err() {
                return api_error(403, "WRONG_PASSWORD", "incorrect password");
            }
            pastas[index].content = encrypt(&body.content, &password);
        }
    } else {
        pastas[index].content = body.content.clone();
    }

    update(Some(&pastas), Some(&pastas[index]));

    let resp = to_paste_response(&pastas[index], body.content.clone());
    HttpResponse::Ok().json(resp)
}

pub async fn list_pastes(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = require_api_key(&req) {
        return e;
    }

    if ARGS.no_listing {
        return api_error(403, "LISTING_DISABLED", "paste listing is disabled on this server");
    }

    let mut pastas = data.pastas.lock().unwrap();
    remove_expired(&mut pastas);

    let mut list: Vec<PasteListItem> = pastas
        .iter()
        .filter(|p| !p.private && !p.encrypt_client && !p.encrypt_server)
        .map(|p| PasteListItem {
            id: p.id_as_words(),
            pasta_type: p.pasta_type.clone(),
            privacy: privacy_string(p).to_string(),
            created_at: p.created,
            expires_at: expires_at(p),
            read_count: p.read_count,
        })
        .collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    HttpResponse::Ok().json(list)
}
