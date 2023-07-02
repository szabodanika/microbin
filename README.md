
![Screenshot](.github/index.png)

# MicroBin

![Build](https://github.com/szabodanika/microbin/actions/workflows/rust.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/microbin.svg)](https://crates.io/crates/microbin)
[![Docker Image](https://github.com/szabodanika/microbin/actions/workflows/release.yml/badge.svg)](https://hub.docker.com/r/danielszabo99/microbin)
[![Docker Pulls](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)
[![Support Server](https://img.shields.io/discord/662017309162078267.svg?color=7289da&label=Discord&logo=discord&style=flat-square)](https://discord.gg/3DsyTN7T)

MicroBin is a super tiny, feature rich, configurable, self-contained and self-hosted paste bin web application. It is very easy to set up and use, and will only require a few megabytes of memory and disk storage. It takes only a couple minutes to set it up, why not give it a try now?


Run our quick docker setup script ([DockerHub](https://hub.docker.com/r/danielszabo99/microbin)):
```bash
bash <(curl -s https://microbin.eu/docker.sh)
```

Or install it manually from [Cargo](https://crates.io/crates/microbin):

```bash
cargo install microbin;
curl -L -O https://raw.githubusercontent.com/;szabodanika/microbin/master/.env;
source .env;
microbin
```

On our website [microbin.eu](https://microbin.eu) you will find the following:

- [Screenshots](https://microbin.eu/screenshots/)
- [Quickstart Guide](https://microbin.eu/quickstart/)
- [Documentation](https://microbin.eu/documentation/)
- [Donations and Sponsorships](https://microbin.eu/donate/)
- [Community](https://microbin.eu/community/)

## Features

- Is very small
- Entirely self-contained executable, MicroBin is a single file!
- Animal names instead of random numbers for pasta identifiers (64 animals)
- Server-side and client-side encryption
- File uploads (eg. `server.com/file/pig-dog-cat`)
- Raw text serving (eg. `server.com/raw/pig-dog-cat`)
- URL shortening and redirection
- QR code support
- Very simple database (JSON + files) for portability, easy backups and integration
- SQLite support
- Private and public, editable and final, automatically and never expiring uploads
- Syntax highlighting
- Automatic dark mode and custom styling support with very little CSS and only vanilla JS (see [`water.css`](https://github.com/kognise/water.css))
- Most of the above can be toggled on and off!

## What is an upload?

In MicroBin, an upload can be:

- A text that you want to paste from one machine to another, eg. some code,
- A file that you want to share, eg. a video that is too large for Discord, a zip with a code project in it or an image,
- A URL redirect.

## When is MicroBin useful?

You can use MicroBin:

- As a URL shortener/redirect service,
- To send long texts to other people,
- To send large files to other people,
- To serve content on the web, eg. configuration files for testing, images, or any other file content using the Raw functionality,
- To move files between your desktop and a server you access from the console,
- As a "postbox" service where people can upload their files or texts, but they cannot see or remove what others sent you - just disable the upload page
- To take notes! Simply create an editable upload.

...and many other things, why not get creative?

MicroBin and MicroBin.eu are available under the [BSD 3-Clause License](LICENSE).

© Dániel Szabó 2022-2023
