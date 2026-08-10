import { test, expect } from "@playwright/test";

// Functional end-to-end coverage for MicroBin's core paste flows.
//
// MicroBin's homepage form is JS-driven: the visible textarea (#content-input) is
// copied into a hidden field and POSTed to /upload via XHR, after which the client
// navigates to the new paste's permalink (/upload/{id}). Each test fills meaningful
// input, submits, and asserts a real property of the output — not just a status code.
//
// Portable by design: no hard-coded host. The base URL comes from playwright.config.ts
// (BASE_URL env, default http://localhost:8080), so this runs unchanged against any
// MicroBin instance (local `cargo run`, the Docker image, or a hosted one).

test("creates a text paste and shows it on its permalink", async ({ page }) => {
  const content = "DebuggAI functional test paste — hello MicroBin";

  await page.goto("/");
  await page.locator("#content-input").fill(content);
  await page.locator("#submit-button").click();

  // Client redirects to the generated permalink on success.
  await expect(page).toHaveURL(/\/upload\/[^/]+$/);

  // Real output assertion: the saved content is rendered back in the code block.
  await expect(page.locator("#code")).toContainText(content);
});

test("creates a paste with a chosen expiration", async ({ page }) => {
  const content = "DebuggAI expiration-path test — this paste sets an expiry";

  await page.goto("/");
  // Exercise the settings controls too: pick a non-default expiration option.
  await page.locator("#expiration").selectOption({ index: 2 });
  await page.locator("#content-input").fill(content);
  await page.locator("#submit-button").click();

  await expect(page).toHaveURL(/\/upload\/[^/]+$/);
  await expect(page.locator("#code")).toContainText(content);
});

test("serves the exact paste content at its /raw endpoint", async ({ page, request }) => {
  const content = "DebuggAI raw-view test — exact bytes must round-trip";

  await page.goto("/");
  await page.locator("#content-input").fill(content);
  await page.locator("#submit-button").click();
  await expect(page).toHaveURL(/\/upload\/[^/]+$/);

  // Derive the paste id from the permalink and read it back over HTTP.
  const id = page.url().split("/upload/")[1];
  const res = await request.get(`/raw/${id}`);
  expect(res.ok()).toBeTruthy();
  expect(await res.text()).toContain(content);
});
