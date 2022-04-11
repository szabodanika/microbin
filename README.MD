# MicroBin

![Screenshot](git/index.png)

MicroBin is a super tiny and simple self hosted pastebin app written in Rust. The executable is around 6MB and it uses 2MB memory (plus your pastas).

Features:
- No CSS or JS, super lightweight and simple
- Animal names instead of random numbers for pasta identifiers
- Automatically expiring pastas
- Never expiring pastas
- Listing and manually removing pastas

Needed improvements:
- Persisting pastas on disk (currently they are lost on restart)
- Removing pasta after N reads
- File uploads
- URL shortening

![Screenshot](git/pasta.png)