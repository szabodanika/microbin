# microbin end-to-end tests

Functional browser tests for microbin's core flows, written with [Playwright](https://playwright.dev).
They are **self-contained** and **independent of the project's build/CI** — they drive a running
microbin instance over HTTP, so they add coverage without changing anything about how microbin
is built or tested today.

## Run locally

```bash
# 1. Start microbin, e.g.:
docker run -p 8080:8080 danielszabo99/microbin

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
- **Create & view a paste** — fill the upload form and submit, then follow the redirect to the paste permalink and assert the content is rendered back.
- **Create after visiting a paste permalink** — load a paste permalink first, then create and view a paste.
- **Create from the paste list** — open the `/list` view, then create and view a paste.
- **Create from the guide page** — open the `/guide` page, then create and view a paste.
- **Create via list -> guide** — visit the list then the guide, then create and view a paste.
- **Create via guide -> list** — visit the guide then the list, then create and view a paste.
- **Create after the remove route** — hit the `/remove/{id}` route, then create and view a paste.
- **Create after the edit route** — hit the `/edit/{id}` route, then create and view a paste.
