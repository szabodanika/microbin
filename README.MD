# MicroBin

![Screenshot](git/index.png)

MicroBin is a super tiny and simple self hosted pastebin app written in Rust. The executable is around 6MB and it uses 2MB memory (plus your pastas, because they are all stored in the memory at the moment).

### Features
- Is very small
- File uploads
- Raw pasta content (/raw/[animals])
- URL shortening and redirection
- Automatic dark mode (follows system preferences)
- Very simple database (json + files) for portability and easy backups
- Animal names instead of random numbers for pasta identifiers (64 animals)
- Automatically expiring pastas
- Never expiring pastas
- Listing and manually removing pastas (/pastalist)
- Very little CSS and absolutely no JS (see [water.css](https://github.com/kognise/water.css))

### Installation
Simply clone the repository, build it with `cargo build --release` and run the `microbin` executable in the created `target/release/` directory. It will start on port 8080. You can change the port with `-p` or `--port` CL arguments. For other arguments see [the Wiki](https://github.com/szabodanika/microbin/wiki).

To install it as a service on your Linux machine, create a file called `/etc/systemd/system/microbin.service`, paste this into it with the `[username]` and `[path to installation directory]` replaced with the actual values.

```
[Unit]
Description=MicroBin
After=network.target

[Service]
Type=simple
Restart=always
User=[username]
RootDirectory=/
WorkingDirectory=[path to installation directory]
ExecStart=[path to installation directory]/target/release/microbin

[Install]
WantedBy=multi-user.target
```

Then start the service with `systemctl start microbin` and enable it on boot with `systemctl enable microbin`.

### Create Pasta with cURL

Simple text Pasta: `curl -d "expiration=10min&content=This is a test pasta" -X POST https://microbin.myserver.com/create`

File contents: `curl -d "expiration=10min&content=$( < mypastafile.txt )" -X POST https://microbin.myserver.com/create`

Available expiration options:
`1min`, `10min`, `1hour`, `24hour`, `1week`, `never`

Use cURL to read the pasta: `curl https://microbin.myserver.com/rawpasta/fish-pony-crow`,

or to download the pasta: `curl https://microbin.myserver.com/rawpasta/fish-pony-crow > output.txt` (use /file instead of /rawpasta to download attached file).


### Needed improvements
- ~~Persisting pastas on disk (currently they are lost on restart)~~ (added on 2 May 2022)
- ~~Configuration with command line arguments (ports, enable-disable pasta list, footer, etc)~~  (added on 7 May 2022)
- ~~File uploads~~ (added on 2 May 2022)
- ~~URL shortening~~ (added on 23 April 2022)
- Removing pasta after N reads
- CLI tool (beyond wget)
- Better instructions and documentation - on GitHub and built in 

