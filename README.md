![image](https://github.com/user-attachments/assets/c83f4952-e9ab-4a9a-8353-721a7b1da491)

# BitVault

A privacy-focused, self-hosted pastebin and file sharing service built in Rust. BitVault lets you securely share text snippets, upload files, and shorten URLs — all from a single, lightweight executable with no external dependencies. Everything stays on your server, under your control.

BitVault is a maintained fork of [Microbin](https://github.com/szabodanika/microbin), with telemetry removed, updated dependencies, and a focus on privacy and simplicity.

TOR HiddenService for demonstration: [http://bvaultb5wmxcxxapep7tnq2zrkjva6ry7uekfcsxpsxts2ve5qmf4iyd.onion/](http://bvaultb5wmxcxxapep7tnq2zrkjva6ry7uekfcsxpsxts2ve5qmf4iyd.onion/)

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

## License

BitVault is available under the [GPL-3](LICENSE) License. The original Microbin project was published under [BSD 3-Clause License](ORIGINAL_LICENSE).

© Dániel Szabó 2022-2023, under BSD-3-Clause
© Mario Stöckl, from 2024-05-27, under GPL-3 License.
