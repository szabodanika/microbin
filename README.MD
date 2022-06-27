
![Screenshot](git/index.png)

# MicroBin


![Build](https://github.com/szabodanika/microbin/actions/workflows/rust.yml/badge.svg)
![crates.io](https://img.shields.io/crates/v/microbin.svg)


MicroBin is a super tiny, feature rich, configurable, self-contained and self-hosted paste bin web application. It is very easy to set up and use, and will only require a few megabytes of memory and disk storage. It takes only a couple minutes to set it up, why not give it a try now?

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/szabodanika/microbin)
 
Or install from Cargo: 

`cargo install microbin`

And run with your custom configuration:

`microbin --port 8080 --highlightsyntax --editable`

### Features
- Is very small
- Animal names instead of random numbers for pasta identifiers (64 animals)
- File uploads (eg. server.com/file/pig-dog-cat)
- Raw pasta text (eg. server.com/raw/pig-dog-cat)
- URL shortening and redirection
- Very simple database (JSON + files) for portability, easy backups and integration
- Listing and manually removing pastas (/pastalist)
- Private and public pastas
- Editable and final pastas
- Never expiring pastas
- Automatically expiring pastas
- Syntax highlighting
- Entirely self-contained executable, MicroBin is a single file!
- Automatic dark mode (follows system preferences)
- Very little CSS and absolutely no JS (see [water.css](https://github.com/kognise/water.css))
- Most of the above can be toggled on and off!
 
## 1 Usage

### What is a "pasta" anyway?

In microbin, a pasta can be:
- A text that you want to paste from one machine to another, eg. some code,
- A file that you want to share, eg. a video that is too large for Discord, a zip with a code project in it or an image,
- A URL redirect.

### When is MicroBin useful?

You can use MicroBin
- As a URL shortener/redirect service,
- To send long texts to other people,
- To send large files to other people,
- To serve content on the web, eg. configuration files for testing, images, or any other file content using the Raw functionality,
- To move files between your desktop and a server you access from the console,
- As a "postbox" service where people can upload their files or texts, but they cannot see or remove what others sent you - just disable the pastalist page
- To take notes! Simply create an editable pasta.

...and many other things, why not get creative?

### Creating a Pasta

Navigate to the root of your server, for example https://microbin.myserver.com/. This should show you a form where you will at the very least see an expiration selector, a file attachment input, a content text field and a green save button. Depending on your configuration there miight also be a syntax highlight selector, an editable checkbox and a private ceckbox.

Use the expiration dropdown to choose how long you want your pasta to exist. When the selected time has expired, it will be removed from the server. The content can be any text, including plain text, code, html, even a URL. A URL is a special case, because when you open the pasta again, it will redirect you to that URL instead of showing it as a text. Entering content is optional, and so is the file attachment. If you want, you can even submit a pasta completely empty.

You will be redirected to the URL of the pasta, which will end with a few animal names. If you remember those animals, you can simply type them in on another machine and open your pasta elsewhere.

If you have editable pastas enabled and you check the editable checkbox, then later on there will be an option to change the text content of your pasta. Selecting the private checkbox will simply prevent your pasta to show up on the pasta list page, if that is enabled.

If you have syntax higlighting enabled, then select your language from the dropdown, or leave it as none if you just want to upload plain with no highlighting.

### Listing Pastas

If you have pasta listing enabled, then there is a pasta list option in the navigation bar, which will list all the pastas on the server in two groups: regular pastas and URL redirects (pastas containing nothing but a URL). If you have private pastas enabled, they will not show up here at all.

From the pasta list page, you will be able to view individual pastas by clicking on their animal identifiers on the lest, view their raw contrent by clicking on the Raw button, remove them, and if you have editable pastas enabled, then open them in edit view.

### Use MicroBin from the console with cURL

Simple text Pasta: `curl -d "expiration=10min&content=This is a test pasta" -X POST https://microbin.myserver.com/create`

File contents: `curl -d "expiration=10min&content=$( < mypastafile.txt )" -X POST https://microbin.myserver.com/create`

Available expiration options:
`1min`, `10min`, `1hour`, `24hour`, `1week`, `never`

Use cURL to read the pasta: `curl https://microbin.myserver.com/rawpasta/fish-pony-crow`,

or to download the pasta: `curl https://microbin.myserver.com/rawpasta/fish-pony-crow > output.txt` (use /file instead of /rawpasta to download attached file).

## 2 Installation

### From Cargo

Install from Cargo:

`cargo install microbin`

Remember, MicroBin will create your database and file storage wherever you execute it. I recommend that you create a folder for it first and execute it there:

`mkdir ~/microbin/`

`cd ~/microbin/`

`microbin --port 8080 --highlightsyntax --editable`

### Building MicroBin

Simply clone the repository, build it with `cargo build --release` and run the `microbin` executable in the created `target/release/` directory. It will start on port 8080. You can change the port with `-p` or `--port` CL arguments. For other arguments see [the Wiki](https://github.com/szabodanika/microbin/wiki).

```
git clone https://github.com/szabodanika/microbin.git
cd microbin
cargo build --release
./target/release/microbin -p 80
```

### Building Docker Image

MicroBin includes a Dockerfile. To build the image, follow these steps:

```
git clone https://github.com/szabodanika/microbin.git
cd microbin
docker build -t microbin-docker .
```

Then, for `docker compose` you can repurpose the following example in your compose file:

```
services:
  paste:
    image: microbin-docker
    restart: always
    ports:
     - "80:8080"
    volumes:
     - ./microbin-data:/usr/local/bin/pasta_data
```

**Note:** If you are getting the following error about domain name resolution:

```
warning: spurious network error (2 tries remaining): failed to resolve address for github.com: Temporary failure in name resolution; class=Net (12)
warning: spurious network error (1 tries remaining): failed to resolve address for github.com: Temporary failure in name resolution; class=Net (12)
```

You might need to run `docker build` with the `--network` option:

```
docker build --network host -t microbin-docker .
```

### MicroBin as a service

To install it as a service on your Linux machine, create a file called `/etc/systemd/system/microbin.service`, paste this into it with the `[username]` and `[path to installation directory]` replaced with the actual values. If you installed MicroBin from cargo, your executable will be in your cargo directory, e.g. `/Users/daniel/.cargo/bin/microbin`.

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

Here is my `microbin.service` for example, with some optional arguments:


```
[Unit]
Description=MicroBin
After=network.target

[Service]
Type=simple
Restart=always
User=ubuntu
RootDirectory=/

# This is the directory where I want to run microbin. It will store all the pastas here.
WorkingDirectory=/home/ubuntu/server/microbin

# This is the location of my executable - I also have 2 optional features enabled
ExecStart=/home/ubuntu/server/microbin/target/release/microbin --editable --highlightsyntax

# I keep my installation in the home directory, so I need to add this
ProtectHome=off

[Install]
WantedBy=multi-user.target
```

Then start the service with `systemctl start microbin` and enable it on boot with `systemctl enable microbin`. To update your MicroBin, simply update or clone the repository again, build it again, and then restart the service with `systemctl restart microbin`. An update will never affect your existing pastas, unless there is a breaking change in the data model (in which case MicroBin just won't be able to import your DB), which will always be mentioned explicitly.

### NGINX configuration

```
server {
	# I have HTTPS enabled using certbot - you can use HTTP of course if you want!
  listen 443 ssl; # managed by Certbot

	server_name	microbin.myserver.com;
	
	location / {
			# Make sure to change the port if you are not running MicroBin at 8080!
    	proxy_pass	        http://127.0.0.1:8080$request_uri;
	    proxy_set_header	Host $host;
	    proxy_set_header	X-Forwarded-Proto $scheme;
	    proxy_set_header    X-Real-IP $remote_addr;
	    proxy_set_header    X-Forwarded-For $proxy_add_x_forwarded_for;
	}

	# Limit content size - I have 1GB because my MicroBin server is private, no one else will use it.
	client_max_body_size 1024M;
	
  ssl_certificate /etc/letsencrypt/live/microbin.myserver.com/fullchain.pem; # managed by Certbot
  ssl_certificate_key /etc/letsencrypt/live/microbin.myserver.com/privkey.pem; # managed by Certbot
  include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
  ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot
}

```

## 3 Command Line Arguments

There is an ever expanding list of customisations built into MicroBin so you can use it the way you want. Instead of a configuration file, we simply use arguments that you pass to the executable, making the workflow even simpler. Read the following options and if you cannot find what you need, you can always open an issue at our GitHub repository and request a new feature!

### --auth-username [AUTH_USERNAME]

Require username for HTTP Basic Authentication when visiting the service. If `--auth-username` is set but `--auth-password ` is not, just leave the password field empty when logging in. You can also just go to https://username:password@yourserver.net or https://username@yourserver.net if password is not set instead of typing into the password

### --auth-password [AUTH_PASSWORD]

Require password for HTTP Basic Authentication when visiting the service. Will not have any affect unless `--auth-username` is also set. If `--auth-username` is set but `--auth-password ` is not, just leave the password field empty when logging in. You can also just go to https://username:password@yourserver.net or https://username@yourserver.net if password is not set instead of typing into the password prompt.

### --editable

Enables editable pastas. You will still be able to make finalised pastas but there will be an extra checkbox to make your new pasta editable from the pasta list or the pasta view page.

### --footer_text [TEXT]

Replaces the default footer text with your own. If you want to hide the footer, use --hide-footer instead.

### -h, --help

Show all commands in the terminal.

### --hide-footer

Hides the footer on every page.

### --hide-header

Hides the navigation bar on every page.

### --hide-logo

Hides the MicroBin logo from the navigation bar on every page.

### --no-listing

Disables the /pastalist endpoint, essentially making all pastas private.

### --highlightsyntax

Enables syntax highlighting support. When creating a new pasta, a new dropdown selector will be added where you can select your pasta's syntax, or just leave it empty for no highlighting.

### -p, --port [PORT]

Default value: 8080

Sets the port for the server will be listening on.

### --private

Enables private pastas. Adds a new checkbox to make your pasta private, which then won't show up on the pastalist page. With the URL to your pasta, it will still be accessible.

### --pure-html

Disables main CSS styling, just uses a few in-line stylings for the layout. With this option you will lose dark-mode support.

### --readonly

Disables adding/editing/removing pastas entirely.

### --title [TITLE]

Replaces "MicroBin" with your title of choice in the navigation bar.

### -t, --threads [THREADS]

Default value: 1

Number of workers MicroBin is allowed to have. Increase this to the number of CPU cores you have if you want to go beast mode, but for personal use one worker is enough.

### -V, --version

Displays your MicroBin's version information.

### --wide

Changes the maximum width of the UI from 720 pixels to 1080 pixels.
