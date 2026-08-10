import { defineConfig, devices } from "@playwright/test";

// Self-contained Playwright config for MicroBin's E2E suite.
// Point it at a running MicroBin instance with BASE_URL (defaults to the standard
// local dev port). This suite is intentionally independent of the Rust CI.
export default defineConfig({
  testDir: ".",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: [["list"]],
  use: {
    baseURL: process.env.BASE_URL || "http://localhost:8080",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  timeout: 30_000,
  expect: { timeout: 10_000 },
});
