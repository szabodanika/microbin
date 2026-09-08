import { test, expect } from '@playwright/test';

test("microbin-3", async ({ page }) => {
  await page.goto("/upload/monkey-dog-bat");
  await expect(page).toHaveURL(new RegExp("/upload/monkey\\-dog\\-bat"));
  await page.goto("/list");
  await expect(page).toHaveURL(new RegExp("/list"));
  await page.goto("/");
  await expect(page).toHaveURL(new RegExp("/"));
  // Let the page settle, then dismiss any welcome/consent overlay before interacting.
  await page.waitForLoadState('networkidle', { timeout: 3000 }).catch(() => {});
  await page.keyboard.press('Escape').catch(() => {});
  await page.locator("#burn_after").selectOption("1");
  if (await page.locator("#content-input").count().catch(() => 0)) await page.locator("#content-input").fill("test", { timeout: 5000 }).catch(() => {});
  await page.locator("#expiration").selectOption("1min");
  if (await page.locator("#password_field").count().catch(() => 0)) await page.locator("#password_field").fill("Test1234!Qa", { timeout: 5000 }).catch(() => {});
  await page.locator("#privacy").selectOption("unlisted");
  await page.locator("#syntax_highlight").selectOption("auto");
  await page.locator("#submit-button").click();
  await expect(page).toHaveURL(new RegExp("/upload/[^/]+"));
  await expect(page.locator("code").first()).not.toBeEmpty();
});
