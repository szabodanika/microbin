import { defineConfig, devices } from "@playwright/test";
export default defineConfig({
  testDir: ".",
  use: { baseURL: process.env.BASE_URL || "http://localhost:8080", trace: "on-first-retry" },
  reporter: [["list"]],
  timeout: 30000,
  expect: { timeout: 10000 },
});
