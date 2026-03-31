![image](https://github.com/user-attachments/assets/c83f4952-e9ab-4a9a-8353-721a7b1da491)

# BitVault

A privacy-focused, self-hosted pastebin and file sharing service built in Rust. BitVault lets you securely share text snippets, upload files, and shorten URLs — all from a single, lightweight executable with no external dependencies. Everything stays on your server, under your control.

BitVault is a maintained fork of [Microbin](https://github.com/szabodanika/microbin), with telemetry removed, updated dependencies, and a focus on privacy and simplicity.

## Features

- Single self-contained binary — no runtime dependencies, minimal resource usage
- Client-side and server-side encryption for secure sharing
- File uploads and raw content serving (eg. `server.com/file/my-upload`, `server.com/raw/my-upload`)
- URL shortening and redirection
- QR code generation for easy mobile access
- Syntax highlighting for pasted code
- BIP39 mnemonic words as human-readable upload identifiers
- SQLite or JSON-file database backend
- Configurable expiration, visibility, editability, and read-once ("burn after reading") uploads
- Admin panel with authentication and optional HTTP basic auth
- Automatic dark mode via [`water.css`](https://github.com/kognise/water.css), custom CSS support

## Build from source

```bash
# Install rust and git (Arch Linux example)
sudo pacman -S rust git

# Clone and build
git clone https://github.com/overcuriousity/bitvault
cd bitvault
cargo build --release
cargo run --release
```

## Systemd service

```ini
# /etc/systemd/system/bitvault.service
[Unit]
Description=BitVault
After=network.target

[Service]
Type=simple
Restart=always
RootDirectory=/
User=<insert username>
WorkingDirectory=/home/<insert username>/
ExecStart=/home/<insert username>/bitvault/target/release/bitvault

Environment="BITVAULT_ADMIN_USERNAME=admin"
Environment="BITVAULT_ADMIN_PASSWORD=changeme"
Environment="BITVAULT_PORT=8080"
Environment="BITVAULT_BIND=0.0.0.0"
Environment="BITVAULT_PUBLIC_PATH=https://bitvault.example.org"
# Environment="BITVAULT_SHORT_PATH=https://short.net"
Environment="BITVAULT_JSON_DB=false"
Environment="BITVAULT_EDITABLE=true"
Environment="BITVAULT_HIDE_HEADER=false"
Environment="BITVAULT_HIDE_FOOTER=false"
Environment="BITVAULT_HIDE_LOGO=false"
Environment="BITVAULT_NO_LISTING=false"
Environment="BITVAULT_READONLY=false"
Environment="BITVAULT_SHOW_READ_STATS=true"
Environment="BITVAULT_THREADS=2"
Environment="BITVAULT_GC_DAYS=90"
Environment="BITVAULT_WIDE=true"
Environment="BITVAULT_ETERNAL_PASTA=true"
Environment="BITVAULT_PRIVATE=true"
Environment="BITVAULT_HIGHLIGHTSYNTAX=true"
Environment="BITVAULT_QR=true"
Environment="BITVAULT_ENABLE_BURN_AFTER=true"
Environment="BITVAULT_ENABLE_READONLY=true"
Environment="BITVAULT_DEFAULT_EXPIRY=24hour"
Environment="BITVAULT_NO_FILE_UPLOAD=false"
Environment="BITVAULT_HASH_IDS=false"
Environment="BITVAULT_ENCRYPTION_CLIENT_SIDE=true"
Environment="BITVAULT_ENCRYPTION_SERVER_SIDE=true"
Environment="BITVAULT_MAX_FILE_SIZE_ENCRYPTED_MB=2048"
Environment="BITVAULT_MAX_FILE_SIZE_UNENCRYPTED_MB=2048"
# Environment="BITVAULT_BASIC_AUTH_USERNAME=something"
# Environment="BITVAULT_BASIC_AUTH_PASSWORD=something"
# Environment="BITVAULT_CUSTOM_CSS=https://myserver.com/custom.css"
Environment="BITVAULT_TITLE=BitVault"

[Install]
WantedBy=multi-user.target
```

## REST API

BitVault exposes a JSON API under `/api/v1/` for programmatic access and AI agent integration.

### Authentication

Set `BITVAULT_API_KEY` to require a bearer token on all API requests:

```bash
export BITVAULT_API_KEY=my-secret-token
```

Pass the token in the `Authorization` header:

```
Authorization: Bearer my-secret-token
```

If `BITVAULT_API_KEY` is unset, the API is open (consistent with the web UI when basic auth is also disabled).

### Endpoints

#### `GET /api/v1/health`

No authentication required.

```bash
curl https://vault.example.com/api/v1/health
# {"status":"ok","version":"1.2.0"}
```

#### `POST /api/v1/paste` — Create a paste

```bash
curl -X POST https://vault.example.com/api/v1/paste \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "SELECT * FROM users;",
    "extension": "sql",
    "privacy": "unlisted",
    "expiration": "1hour"
  }'
# {"id":"happy-apple-banana","url":"https://vault.example.com/upload/happy-apple-banana","expires_at":1748736000,"privacy":"unlisted"}
```

**Fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `content` | string | yes | Paste text content |
| `extension` | string | no | Syntax highlight language (e.g. `"rust"`, `"sql"`) |
| `privacy` | string | no | `"public"`, `"unlisted"` (default), or `"private"` |
| `expiration` | string | no | One of: `1min`, `10min`, `1hour`, `24hour`, `3days`, `1week`, `1month`, `6months`, `1year`, `2years`, `4years`, `8years`, `16years`, `never` |
| `burn_after_reads` | number | no | Auto-delete after N reads (0 = unlimited) |
| `password` | string | required if `privacy="private"` | Server-side encryption password |

#### `GET /api/v1/paste/{id}` — Get a paste

```bash
curl https://vault.example.com/api/v1/paste/happy-apple-banana \
  -H "Authorization: Bearer $API_KEY"
```

For private (encrypted) pastes, provide the password:

```bash
curl https://vault.example.com/api/v1/paste/happy-apple-banana \
  -H "Authorization: Bearer $API_KEY" \
  -H "X-Pasta-Password: hunter2"
```

#### `DELETE /api/v1/paste/{id}` — Delete a paste

```bash
curl -X DELETE https://vault.example.com/api/v1/paste/happy-apple-banana \
  -H "Authorization: Bearer $API_KEY"
```

#### `PATCH /api/v1/paste/{id}` — Update a paste (editable pastes only)

```bash
curl -X PATCH https://vault.example.com/api/v1/paste/happy-apple-banana \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "updated content"}'
```

#### `GET /api/v1/pastes` — List all pastes

```bash
curl https://vault.example.com/api/v1/pastes \
  -H "Authorization: Bearer $API_KEY"
```

### Error responses

All errors follow this shape:

```json
{"error": "Human-readable message", "code": "MACHINE_CODE"}
```

| Code | HTTP Status | Meaning |
|---|---|---|
| `API_KEY_REQUIRED` | 401 | Missing or wrong API key |
| `PASSWORD_REQUIRED` | 401 | Paste is encrypted; `X-Pasta-Password` header missing |
| `WRONG_PASSWORD` | 403 | Decryption failed |
| `NOT_FOUND` | 404 | Paste not found or expired |
| `NOT_EDITABLE` | 400 | PATCH attempted on a non-editable paste |
| `INVALID_EXPIRATION` | 400 | Expiration value not allowed |
| `INVALID_PRIVACY` | 400 | Privacy value not supported |
| `CONTENT_REQUIRED` | 400 | Empty content on create |

### Limitations

- **File upload** is not supported via the API (use the web UI)
- **Client-side encrypted** (`secret`) and **readonly** pastes cannot be created via the API
- `secret` pastes returned by `GET /api/v1/paste/{id}` will have ciphertext in `content` — the key never reaches the server

## License

BitVault is available under the [GPL-3.0](LICENSE) License. The original Microbin project was published under [BSD 3-Clause License](ORIGINALLICENSE.txt).

© Dániel Szabó 2022-2023, under BSD-3-Clause
© overcuriousity, from 2024-05-27, under GPL-3 License.
