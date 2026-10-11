import { defineConfig, devices } from "@playwright/test";

// The same UI against a REAL service, no stubs (WPM-T143). Opt in: start the Loco API on :5150 with
// `WPM_REQUIRE_AUTH=0` and a database holding the demo data from `tests/live/README.md`, then
//
//   WPM_LIVE_API=1 PW_PORT=4510 pnpm exec playwright test -c playwright.live.config.ts
//
// Without `WPM_LIVE_API=1` every spec here is skipped, so the stubbed suite never needs a service.
const PORT = process.env.PW_PORT ?? "4510";

export default defineConfig({
  testDir: "tests/live",
  timeout: 60_000,
  expect: { timeout: 20_000 },
  fullyParallel: false,
  workers: 1,
  use: { baseURL: `http://localhost:${PORT}`, locale: "en" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: `npm run build && npm run preview -- --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
