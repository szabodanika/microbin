# MicroBin

![Screenshot](git/index.png)

MicroBin is a super tiny and simple self hosted pastebin app written in Rust. The executable is only a few megabytes and uses very little memory (plus your pastas). 

### Installation
Simply clone the repository, build it with `cargo build --release` and run the `microbin` executable in the created `target/release/` directory. It will start on port 8080 but you can change this with `-p` or `--port` arguments. 

To install it as a service on your Linux machine, create a file called `/etc/systemd/system/microbin.service`, paste this into it with the value of `ExecStart` replaced with the actual path to microbin on your machine.

```
[Unit]
Description=MicroBin
After=network.target
[Service]
Type=simple
Restart=always
ExecStart=/home/pi/microbin/target/release/microbin
[Install]
WantedBy=multi-user.target
```

Then start the service with `systemctl start microbin` and enable it on boot with `systemctl enable microbin`.

### Features
- Very little CSS and no JS, super lightweight and simple (see [water.css](https://github.com/kognise/water.css))
- Animal names instead of random numbers for pasta identifiers
- Automatically expiring pastas
- Never expiring pastas
- Listing and manually removing pastas
- URL shortening and redirection

### Needed improvements
- Persisting pastas on disk (currently they are lost on restart)
- Removing pasta after N reads
- File uploads
- ~~URL shortening~~ (added on 23 April 2022)

