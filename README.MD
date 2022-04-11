# MicroBin

![Screenshot](git/index.png)

MicroBin is a super tiny and simple self hosted pastebin app written in Rust. The executable is around 6MB and it uses 2MB memory (plus your pastas).

### Installation
Simply clone the repository, build it with `cargo build --release` and run the `microbin` executable in the created `target/release/` directory. It will start on port 8080. 

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
- No CSS or JS, super lightweight and simple
- Animal names instead of random numbers for pasta identifiers
- Automatically expiring pastas
- Never expiring pastas
- Listing and manually removing pastas

### Needed improvements
- Persisting pastas on disk (currently they are lost on restart)
- Removing pasta after N reads
- File uploads
- URL shortening
- CLI tool
- Configuration with command line arguments (ports, enable-disable pasta list, footer, etc)

