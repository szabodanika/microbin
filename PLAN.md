# Implementation Plan: REST API + Expiry Countdown

## Overview

Two features:

1. **REST API** — a JSON API layer under `/api/v1/`, designed for programmatic access and AI agent integration
2. **Expiry Countdown** — a live countdown shown below paste content on the view page, with minimal JavaScript

---

## Part 1: REST API

### Design Goals (AI Agent Focus)

AI agents (LLM-driven automation, CI/CD bots, scripted workflows) need:

- **JSON in, JSON out** — no HTML parsing, no multipart forms
- **Predictable error shape** — every error is `{"error": "...", "code": "..."}` with an appropriate HTTP status
- **Bearer token auth** — simple `Authorization: Bearer <token>` header, one env var to configure
- **Stateless** — no sessions, no cookies
- **Content type** — `application/json` throughout
- **No client-side encryption via API** — client-side AES is a browser-only flow; the API exposes server-side encryption only. This is documented clearly.

### Configuration

New argument / environment variable:

```
--api-key / BITVAULT_API_KEY
```

- Optional `String`. If set, all `/api/v1/` requests (except `/api/v1/health`) must include `Authorization: Bearer <value>`.
- If not set, API routes are open (consistent with the web UI which also has no per-route auth when basic auth is disabled).
- Completely independent of `BITVAULT_BASIC_AUTH_*` (web UI auth) and `BITVAULT_ADMIN_*` (admin panel).

Add to `src/args.rs`:

```rust
#[clap(long, env = "BITVAULT_API_KEY")]
pub api_key: Option<String>,
```

### Endpoints

All routes are prefixed `/api/v1/`.

#### `GET /api/v1/health`

No auth required.

**Response 200:**
```json
{ "status": "ok", "version": "x.y.z" }
```

---

#### `POST /api/v1/paste`

Create a new text paste.

**Request body** (`application/json`):
```json
{
  "content":         "hello world",          // required
  "extension":       "rust",                 // optional, default ""
  "privacy":         "unlisted",             // optional: "public"|"unlisted"|"private", default "unlisted"
  "expiration":      "24hour",               // optional, same values as the web UI, default from ARGS.default_expiry
  "burn_after_reads": 0,                     // optional, 0 = unlimited
  "password":        "hunter2"              // required when privacy = "private"
}
```

Notes:
- `"private"` triggers server-side AES encryption (same as the web UI). Password is required.
- `"public"` and `"unlisted"` create unencrypted pastes. `"readonly"` and `"secret"` are not supported via API (readonly requires a separate encrypted-key flow; secret requires browser-side AES).
- The `expiration` value is validated against `BITVAULT_MAX_EXPIRY` exactly as the web form does.
- File uploads are not supported in this endpoint. Use the web UI for file pastas.

**Response 201:**
```json
{
  "id":         "happy-apple-banana",
  "url":        "https://example.com/upload/happy-apple-banana",
  "expires_at": 1748736000,    // Unix timestamp, null if never
  "privacy":    "unlisted"
}
```

**Errors:**
- `400` — missing content, invalid expiration, invalid privacy value
- `401` — API key required but missing/wrong
- `422` — private pasta requested without password

---

#### `GET /api/v1/paste/{id}`

Fetch a paste's metadata and content.

For **private** (server-encrypted) pastes, the decryption password must be sent in the `X-Pasta-Password` request header. If omitted for an encrypted paste, a `401` is returned with `"code": "PASSWORD_REQUIRED"`.

**Response 200:**
```json
{
  "id":               "happy-apple-banana",
  "content":          "hello world",
  "pasta_type":       "text",
  "extension":        "rust",
  "privacy":          "unlisted",
  "created_at":       1748649600,
  "expires_at":       1748736000,
  "read_count":       4,
  "burn_after_reads": 0,
  "has_file":         false,
  "url":              "https://example.com/upload/happy-apple-banana"
}
```

Notes:
- `content` is always the plaintext (decrypted server-side for private pastes).
- `"secret"` (client-encrypted) pastes are served as-is — the content field will be ciphertext. The API cannot decrypt these since the key never reaches the server.
- `expires_at` is `null` when expiration is set to never.
- This endpoint increments `read_count` and updates `last_read`, same as the web view.

**Errors:**
- `401` — API key required, or password required for private paste
- `403` — wrong password (decrypt failed)
- `404` — paste not found or expired

---

#### `DELETE /api/v1/paste/{id}`

Delete a paste.

For private pastes, the owner password must be in `X-Pasta-Password`.

**Response 204:** no body

**Errors:**
- `401` — API key required or password required
- `403` — wrong password
- `404` — not found

---

#### `PATCH /api/v1/paste/{id}`

Update the content of an editable paste.

**Request body:**
```json
{
  "content":  "updated content",
  "password": "hunter2"   // required for private pastes
}
```

**Response 200:** same shape as `GET /api/v1/paste/{id}` with updated content.

**Errors:**
- `400` — paste is not editable
- `401` — API key or password required
- `403` — wrong password
- `404` — not found

---

#### `GET /api/v1/pastes`

List all non-expired pastas. Requires API key.

**Response 200:**
```json
[
  {
    "id":         "happy-apple-banana",
    "pasta_type": "text",
    "privacy":    "public",
    "created_at": 1748649600,
    "expires_at": 1748736000,
    "read_count": 4
  }
]
```

Note: `content` is intentionally omitted from the list response.

---

### Error Response Shape

Every error has this shape:

```json
{ "error": "Human-readable message", "code": "MACHINE_READABLE_CODE" }
```

Error codes used across endpoints:

| Code | Meaning |
|---|---|
| `API_KEY_REQUIRED` | Authorization header missing or wrong |
| `PASSWORD_REQUIRED` | Pasta is encrypted; `X-Pasta-Password` header missing |
| `WRONG_PASSWORD` | Decryption failed |
| `NOT_FOUND` | Pasta not found or expired |
| `NOT_EDITABLE` | PATCH attempted on a non-editable pasta |
| `INVALID_EXPIRATION` | Expiration value not allowed |
| `INVALID_PRIVACY` | Privacy value not supported |
| `CONTENT_REQUIRED` | Empty content on create |
| `FILE_NOT_SUPPORTED` | Attempted file upload via JSON API |

---

### Implementation Steps

#### Step 1 — Add `api_key` to `Args` (`src/args.rs`)

```rust
#[clap(long, env = "BITVAULT_API_KEY")]
pub api_key: Option<String>,
```

Also add it to `Args::without_secrets()` (set to `None`).

---

#### Step 2 — Create `src/endpoints/api.rs`

New file. Contains:

1. **`ApiError` type** — implements `ResponseError` for actix-web, serialises to `{"error": "...", "code": "..."}`.

2. **Request/response structs** — all `#[derive(Deserialize)]` / `#[derive(Serialize)]`:

   ```rust
   struct CreatePasteRequest { content, extension, privacy, expiration, burn_after_reads, password }
   struct CreatePasteResponse { id, url, expires_at, privacy }
   struct PasteResponse { id, content, pasta_type, extension, privacy, created_at, expires_at, read_count, burn_after_reads, has_file, url }
   struct PasteListItem { id, pasta_type, privacy, created_at, expires_at, read_count }
   struct UpdatePasteRequest { content, password }
   ```

3. **API key guard** — a free function `require_api_key(req: &HttpRequest) -> Result<(), ApiError>`:

   ```rust
   fn require_api_key(req: &HttpRequest) -> Result<(), ApiError> {
       let Some(ref key) = ARGS.api_key else { return Ok(()); };
       let header = req.headers().get("Authorization")
           .and_then(|v| v.to_str().ok())
           .and_then(|v| v.strip_prefix("Bearer "));
       if header == Some(key.as_str()) { Ok(()) }
       else { Err(ApiError::unauthorized("API_KEY_REQUIRED", "Valid API key required")) }
   }
   ```

4. **Handler functions** — one per endpoint above:

   - `health()` — no lock needed, returns version from `crate::util::version`
   - `create_paste(data, req, body: web::Json<CreatePasteRequest>)` — mirrors the logic in `endpoints/create.rs::create()` but accepts JSON and returns JSON. Reuses `expiration_to_timestamp`, `is_valid_expiration`, `encrypt`, `insert`.
   - `get_paste(data, req, id, http_req)` — mirrors `pastaresponse()` in `endpoints/pasta.rs`. Reuses `remove_expired`, `decrypt`, `update`.
   - `delete_paste(data, req, id, http_req)` — locks mutex, finds pasta, checks password if encrypted, calls `delete()`, removes attachment directory.
   - `update_paste(data, req, id, http_req, body)` — checks `editable`, decrypts/re-encrypts if needed, calls `update()`.
   - `list_pastes(data, req, http_req)` — requires API key, calls `remove_expired`, maps to `PasteListItem`.

5. **Helper** — `privacy_string(pasta: &Pasta) -> &'static str` to map the bool flags back to a string:
   ```
   encrypt_client && encrypt_server → "secret"
   encrypt_server && !encrypt_client → "private"
   readonly → "readonly"
   !private → "public"
   _ → "unlisted"
   ```

---

#### Step 3 — Register routes in `src/main.rs`

```rust
pub mod endpoints {
    // ... existing
    pub mod api;
}
```

In the `App` builder, add a scoped service **before** the default service:

```rust
.service(
    web::scope("/api/v1")
        .route("/health",      web::get().to(api::health))
        .route("/paste",       web::post().to(api::create_paste))
        .route("/paste/{id}",  web::get().to(api::get_paste))
        .route("/paste/{id}",  web::delete().to(api::delete_paste))
        .route("/paste/{id}",  web::patch().to(api::update_paste))
        .route("/pastes",      web::get().to(api::list_pastes))
)
```

The API scope is placed **outside** the `HttpAuthentication::basic` middleware wrap — API auth is handled per-handler via `require_api_key()`, so web basic auth and API key auth are fully independent.

---

#### Step 4 — Update `README.md`

Add a section documenting:
- The `BITVAULT_API_KEY` env var
- All six endpoints with example `curl` commands
- Limitations (no file upload, no client-side encryption)

---

### AI Agent Usage Example

```bash
# Create a paste
curl -X POST https://vault.example.com/api/v1/paste \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "SELECT * FROM users;", "extension": "sql", "expiration": "1hour"}'

# Read it back
curl https://vault.example.com/api/v1/paste/happy-apple-banana \
  -H "Authorization: Bearer $API_KEY"

# Delete it
curl -X DELETE https://vault.example.com/api/v1/paste/happy-apple-banana \
  -H "Authorization: Bearer $API_KEY"
```

An LLM agent can store context, share code snippets between steps, or exfiltrate intermediate results to a readable URL — all without browser interaction.

---

## Part 2: Expiry Countdown

### Goal

Show a live "expires in X days / X hours / X minutes / Xs" countdown below the paste content on `/upload/{id}`. Never-expiring pastas show nothing.

### Approach: Minimal JS (one `<script>` block, ~12 lines)

The template already has a substantial `<script>` block for copy/decrypt/highlight. A live countdown cannot be done without JS (CSS cannot read system time). The plan is to add the smallest possible self-contained snippet.

**No new dependencies. No external scripts. No framework.**

#### 1 — Add `expires_at_unix` accessor to `Pasta` (`src/pasta.rs`)

Expose the raw expiration timestamp to the template in a way the JS snippet can read, without adding a new template variable:

```rust
pub fn expires_at_unix(&self) -> i64 {
    self.expiration  // 0 means never
}
```

This is just an alias, but makes intent clear in the template.

Alternatively, `expiration` is already public and accessible from the template as `pasta.expiration` — no new method needed if the template accesses it directly.

#### 2 — Add HTML to `templates/upload.html`

Below the read-stats `<div>` (currently line 172), add:

```html
{% if pasta.expiration != 0 %}
<p id="expiry-line" style="font-size: small"
   data-ts="{{ pasta.expiration }}">
  Expires {{ pasta.expiration_as_string() }}
</p>
{% endif %}
```

The `data-ts` attribute holds the Unix timestamp. The inner text is the server-rendered fallback: it shows the absolute date/time if JS is disabled or before the script runs. This is a graceful degradation: no JS = still useful.

#### 3 — Add countdown script in `templates/upload.html`

Appended inside the existing `<script>` block (or as a small separate `<script>` near the bottom):

```js
(function () {
  var el = document.getElementById('expiry-line');
  if (!el) return;
  var ts = +el.dataset.ts * 1000;
  function upd() {
    var d = Math.floor((ts - Date.now()) / 1000);
    if (d <= 0) { el.textContent = 'Expired'; return; }
    var h = Math.floor(d / 3600), m = Math.floor(d % 3600 / 60), s = d % 60;
    el.textContent = 'Expires in ' + (h ? h + 'h ' : '') + m + 'm ' + s + 's';
  }
  upd();
  setInterval(upd, 1000);
})();
```

12 lines. Self-contained IIFE. No dependencies. Degrades to the server-rendered date string if JS is off.

---

## File Change Summary

| File | Change |
|---|---|
| `src/args.rs` | Add `api_key: Option<String>` field + `without_secrets()` entry |
| `src/endpoints/api.rs` | New file — all API handlers, request/response types, error type |
| `src/main.rs` | Register `api` module, add `/api/v1` scope in App builder |
| `templates/upload.html` | Add expiry countdown `<p>` and 12-line JS snippet |
| `README.md` | Document API endpoints and `BITVAULT_API_KEY` |

No changes required to: `pasta.rs`, `util/db.rs`, `util/misc.rs`, `util/auth.rs`, or any other existing endpoint. The API layer is purely additive.

---

## Out of Scope

- **File upload via API** — multipart streaming is complex and the primary use case (AI agents sharing text/code) doesn't need it
- **Client-side encryption via API** — the AES key never reaches the server; this is intentional and documented
- **Readonly pastas via API** — requires a separate encrypted-key handshake that doesn't map cleanly to JSON
- **Rate limiting** — worthwhile but a separate concern; can be added as actix middleware in a follow-up
- **OpenAPI/Swagger spec** — useful but heavy; a `README` section with `curl` examples is sufficient for v1
