# MicroBin Plus JSON API

## Authentication

All API endpoints sit behind the same basic auth middleware as the web UI. If `MICROBIN_BASIC_AUTH_USERNAME` and `MICROBIN_BASIC_AUTH_PASSWORD` are configured, include them as HTTP Basic Auth headers.

```
Authorization: Basic base64(username:password)
```

---

## Endpoints

### `GET /api/list`

Returns all public (non-private) pastes, sorted newest first.

Returns `403` if `MICROBIN_NO_LISTING` is enabled.

**Response** `200 OK`
```json
[
  {
    "id": "brave-red-fox",
    "content": "Hello world",
    "pasta_type": "text",
    "expiration": "02-09 14:30",
    "created": "02-08 12:00",
    "read_count": 5,
    "burn_after_reads": 0,
    "private": false,
    "readonly": false,
    "editable": true,
    "encrypt_server": false,
    "encrypt_client": false,
    "has_file": false,
    "file_name": null,
    "file_size": null,
    "url": "/upload/brave-red-fox",
    "raw_url": "/raw/brave-red-fox"
  }
]
```

---

### `GET /api/pasta/{id}`

Returns a single paste by its slug (animal names or hash ID). Increments read count.

Returns `403` for server-encrypted pastes (use the web interface to decrypt).
Returns `404` if the paste doesn't exist or has expired.

**Response** `200 OK`
```json
{
  "id": "brave-red-fox",
  "content": "Hello world",
  "pasta_type": "text",
  "expiration": "02-09 14:30",
  "created": "02-08 12:00",
  "read_count": 6,
  "burn_after_reads": 0,
  "private": false,
  "readonly": false,
  "editable": true,
  "encrypt_server": false,
  "encrypt_client": false,
  "has_file": false,
  "file_name": null,
  "file_size": null,
  "url": "/upload/brave-red-fox",
  "raw_url": "/raw/brave-red-fox"
}
```

---

### `POST /api/create`

Creates a new paste. Accepts `multipart/form-data`.

**Form Fields**

| Field | Required | Values | Description |
|-------|----------|--------|-------------|
| `content` | Yes (unless `file` provided) | Any text | The paste content. Auto-detected as `url` type if it's a valid URL. |
| `file` | No | File upload | Attachment. Disabled if `MICROBIN_NO_FILE_UPLOAD` is set. |
| `expiration` | No | `1min`, `10min`, `1hour`, `24hour`, `3days`, `1week`, `never` | Defaults to server's `MICROBIN_DEFAULT_EXPIRY` (default: `24hour`). |
| `burn_after` | No | `0`, `1`, `10`, `100`, `1000`, `10000` | Auto-delete after N reads. `0` = disabled. |
| `privacy` | No | `public`, `private`, `readonly` | Default: `public`. `private` enables server-side encryption. |
| `plain_key` | No | Any string | Encryption key for `private` pastes. |
| `syntax_highlight` | No | File extension (e.g. `rs`, `py`, `js`) | Syntax highlighting language. |
| `uploader_password` | Conditional | String | Required if `MICROBIN_READONLY` and `MICROBIN_UPLOADER_PASSWORD` are set. |

**Response** `201 Created`
```json
{
  "id": "brave-red-fox",
  "url": "/upload/brave-red-fox",
  "raw_url": "/raw/brave-red-fox"
}
```

**Error Responses**
- `400` — Missing content/file, or file exceeds size limit
- `401` — Invalid uploader password

---

## Examples

### Create a text paste (curl)

```bash
curl -X POST http://localhost:8080/api/create \
  -F "content=Hello from the API" \
  -F "expiration=1hour"
```

### Create a paste with a file

```bash
curl -X POST http://localhost:8080/api/create \
  -F "content=See attached" \
  -F "file=@./screenshot.png" \
  -F "expiration=24hour"
```

### Create a private paste

```bash
curl -X POST http://localhost:8080/api/create \
  -F "content=secret data" \
  -F "privacy=private" \
  -F "plain_key=mypassword"
```

### List pastes

```bash
curl http://localhost:8080/api/list
```

### Get a specific paste

```bash
curl http://localhost:8080/api/pasta/brave-red-fox
```

### With basic auth

```bash
curl -u username:password http://localhost:8080/api/list
```
