# MicroBin end-to-end tests

Functional browser tests for MicroBin's core flows, written with [Playwright](https://playwright.dev).
They are **self-contained** and **independent of the Rust build/CI** — they drive a running
MicroBin instance over HTTP, so they add coverage without changing anything about how MicroBin
is built or tested today.

## Run locally

```bash
# 1. Start MicroBin however you like, e.g.:
cargo run                      # (from the repo root) → http://localhost:8080
# or: docker run -p 8080:8080 danielszabo99/microbin

# 2. Run the tests (from this directory):
npm install
npm run test:install           # one-time: download the Chromium browser
npx playwright test            # uses BASE_URL, default http://localhost:8080
```

Point the suite at any instance with `BASE_URL`:

```bash
BASE_URL=http://localhost:8080 npx playwright test
```

## What's covered
- **Create & view a paste** — type content, save, follow the redirect to `/upload/{id}`, and
  assert the saved content is rendered back.
- **Create a paste with an expiration** — additionally exercises the settings controls.
