// The footer links every page to the privacy notice in the app's language (WPM-R253). The notice is on the website, one
// per locale under the repository's `locales/<locale>/privacy/`; every app locale must have one there, or the link is
// a dead end in that language.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { LOCALES } from "../../src/lib/locales";
import { privacyNoticeUrl } from "../../src/lib/help";

const REPO = join(__dirname, "../../..");

describe("the privacy notice link", () => {
  it("points at the website's notice in the app's locale", () => {
    expect(privacyNoticeUrl("cy-001")).toBe(
      "https://workforce-planning-management.github.io/cy-001/privacy/",
    );
  });

  it.each(LOCALES)("%s has a notice on the website", (locale) => {
    expect(
      existsSync(join(REPO, "locales", locale, "privacy", "index.md")),
    ).toBe(true);
  });

  it("names every key the app keeps in a browser", () => {
    const notice = readFileSync(
      join(REPO, "locales", "en-001", "privacy", "index.md"),
      "utf8",
    );
    const layout = readFileSync(
      join(__dirname, "../../src/routes/+layout.svelte"),
      "utf8",
    );
    const i18n = readFileSync(
      join(__dirname, "../../src/lib/i18n.svelte.ts"),
      "utf8",
    );
    const hooks = readFileSync(
      join(__dirname, "../../src/hooks.server.ts"),
      "utf8",
    );
    const session = readFileSync(
      join(__dirname, "../../src/lib/server/session.ts"),
      "utf8",
    );
    const admin = readFileSync(
      join(__dirname, "../../src/lib/server/admin-page.ts"),
      "utf8",
    );
    // Every string constant named as a storage key or cookie in these files.
    const keys = [
      ...[
        ...layout.matchAll(/(?:THEME_KEY|THEME_KEY_LEGACY)\s*=\s*"([^"]+)"/g),
      ].map((m) => m[1]),
      ...[...layout.matchAll(/storageKey:\s*"([^"]+)"/g)].map((m) => m[1]),
      ...[
        ...i18n.matchAll(/(?:LOCALE_KEY|LEGACY_LOCALE_KEY)\s*=\s*"([^"]+)"/g),
      ].map((m) => m[1]),
      ...[...hooks.matchAll(/LOCALE_COOKIE\s*=\s*"([^"]+)"/g)].map((m) => m[1]),
      ...[
        ...session.matchAll(/SESSION_COOKIE(?:_LOCAL)?\s*=\s*"([^"]+)"/g),
      ].map((m) => m[1]),
      ...[...admin.matchAll(/const key = "([^"]+)"/g)].map((m) => m[1]),
    ];
    expect(keys.sort()).toEqual([
      "__Host-mxi_session",
      "mxi.hcm.locale",
      "mxi.hcm.theme",
      "mxi.wpm.locale",
      "mxi.wpm.text-size",
      "mxi.wpm.theme",
      "mxi_session",
      "sveltia-cms.prefs",
      "wpm-locale",
    ]);
    for (const key of keys) expect(notice, key).toContain(`\`${key}\``);
  });
});
