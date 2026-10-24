// A page or panel that reads again when something it depends on changes must not let a slow answer for the
// earlier input overwrite the answer to the later one (WPM-T260). Every loader goes through `guardedLoad`,
// `createLoader` or `latestOnly` (src/lib/loader.svelte.ts), or is listed here with the guard it has instead.

import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "src");

function walk(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? walk(path) : path.endsWith(".svelte") ? [path] : [];
  });
}

/** Loaders with a guard of their own, and what it is. */
const OWN_GUARD: Record<string, string> = {
  "routes/oversight/+page.svelte": "drops an answer whose organization is no longer the chosen one",
  "routes/capacity/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/cutover/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/warnings/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/resilience/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/contingent/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/skill-gaps/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/ceo/+page.svelte": "`current` flag cleared by the effect's cleanup",
  "routes/job-levels/+page.svelte": "`stale` flag cleared by the effect's cleanup",
  "routes/pay-scales/+page.svelte": "`stale` flag cleared by the effect's cleanup",
  "lib/components/ConversionPanel.svelte": "`current` flag cleared by the effect's cleanup",
  "lib/components/EngagementPanel.svelte": "`current` flag cleared by the effect's cleanup",
};

describe("loaders are guarded against a late answer", () => {
  const files = walk(root);

  it("every `async function load…` is in a file that uses the shared guard or has its own", () => {
    const offenders = files
      .filter((file) => /async function load\w*\(/.test(readFileSync(file, "utf8")))
      .map((file) => ({ rel: relative(root, file), text: readFileSync(file, "utf8") }))
      .filter(
        ({ rel, text }) =>
          !/guardedLoad|createLoader|latestOnly/.test(text) && !(rel in OWN_GUARD),
      )
      .map(({ rel }) => rel);
    expect(offenders, "use guardedLoad from #lib/loader.svelte.js").toEqual([]);
  });

  it("an effect does not fetch with a bare `await` inside an IIFE unless it is guarded or listed", () => {
    const offenders = files
      .map((file) => ({ rel: relative(root, file), text: readFileSync(file, "utf8") }))
      .filter(({ text }) => /\$effect\(\(\) => \{[^}]*void \(async \(\) => \{/.test(text))
      .filter(({ rel, text }) => !/current|stale|guardedLoad|latestOnly/.test(text) && !(rel in OWN_GUARD))
      .map(({ rel }) => rel);
    // These read once when they appear, from nothing that changes, so there is no earlier answer to overtake.
    const READS_ONCE = [
      "lib/components/RoleGap.svelte",
      "lib/components/me/ContactDetailsPanel.svelte",
      "lib/components/me/HealthPanel.svelte",
      "routes/change/+page.svelte",
      "routes/cpd/+page.svelte",
      "routes/learning/+page.svelte",
      "routes/planning/+page.svelte",
      "routes/roles/+page.svelte",
    ];
    expect(offenders.filter((f) => !READS_ONCE.includes(f))).toEqual([]);
  });

  it("the allowlist names files that exist", () => {
    const have = new Set(files.map((f) => relative(root, f)));
    for (const file of Object.keys(OWN_GUARD)) expect(have.has(file), file).toBe(true);
  });
});
