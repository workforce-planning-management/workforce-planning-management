import { defineConfig } from "vitest/config";

// Unit tests for the pure modules in src/lib (WPM-T631). The SvelteKit plugin is left out on purpose: these modules
// import no `$app` or `$env`, so they run as plain TypeScript. Playwright (tests/e2e) covers the built pages.
export default defineConfig({
  test: {
    include: ["tests/unit/**/*.test.ts"],
    coverage: {
      provider: "v8",
      include: ["src/lib/**/*.ts"],
      exclude: ["src/lib/index.ts"],
      reporter: ["text"],
      // The branches not run are `?? ""` fallbacks of `split()[0]`, which cannot be undefined, so branches are held at 95.
      thresholds: { lines: 100, statements: 100, functions: 100, branches: 95 },
    },
  },
});
