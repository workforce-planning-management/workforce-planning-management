import { defineConfig, devices } from "@playwright/test";

// The preview port is `PW_PORT` (default 4173). Playwright reuses *whatever* is
// already listening there, so if another project's server holds 4173 the specs
// would run against the wrong app: set `PW_PORT` to a free port.
const PORT = process.env.PW_PORT ?? "4173";

// Smoke tests run against the Vite dev server. The backend is stubbed
// per-test via `page.route`, so no running Rust service is required.
export default defineConfig({
  testDir: "tests/e2e",
  timeout: 30_000,
  // A page can sit idle for several seconds when many pages load at once at the start of a parallel run
  // (the dashboard specs waited 5.4 s for requests that were already stubbed, while the server answered
  // every request in under 75 ms; alone they take 0.2 s). The default 5 s expect timeout turned that
  // into a failure, so assertions wait up to 20 s: a real failure still fails, only later.
  expect: { timeout: 20_000 },
  fullyParallel: true,
  use: {
    baseURL: `http://localhost:${PORT}`,
    // Pin Accept-Language so an unprefixed visit lands on en-001, not en-us.
    locale: "en",
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  // Test the production build via `vite preview`: it serves static,
  // correctly-typed ES modules, avoiding the `vite dev` cold-start
  // dependency-optimisation race that flakes module loading.
  webServer: {
    command: `npm run build && npm run preview -- --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
