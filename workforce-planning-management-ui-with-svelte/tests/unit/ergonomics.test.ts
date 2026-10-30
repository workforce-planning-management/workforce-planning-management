// The workstation checklist as translatable content (WPM-T386; spec/worldwide.md): a default item is shown by its key in the
// reader's language, a custom item as typed, and every shipped locale holds every default item's words.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { ergonomicItemLabel } from "../../src/lib/ergonomics";
import { LOCALES, STRINGS_BY_LOCALE } from "../../src/lib/i18n.svelte";

// The keys the service stores, read from its rule so the two cannot drift.
const RULE = readFileSync(
  join(
    __dirname,
    "../../../workforce-planning-management-service-with-rust/src/rules/ergonomics.rs",
  ),
  "utf8",
);
const KEYS = [...RULE.matchAll(/key: "([a-z_]+)"/g)].map((m) => m[1]);

describe("the workstation checklist's words", () => {
  it("reads eight keys from the service's table", () => {
    expect(KEYS).toEqual([
      "screen",
      "chair",
      "keyboard_mouse",
      "desk_space",
      "lighting",
      "leg_room",
      "breaks",
      "software",
    ]);
  });

  it("shows a default item in the reader's language", () => {
    const english = "Chair adjustable, with lower-back support";
    expect(ergonomicItemLabel("chair", english, "en-001")).toBe(english);
    expect(ergonomicItemLabel("chair", english, "de-001")).toBe(
      "Stuhl verstellbar, mit Stütze für den unteren Rücken",
    );
    expect(ergonomicItemLabel("chair", english, "ar-001")).not.toBe(english);
  });

  it("shows a custom item, or a key this client does not know, as stored", () => {
    expect(ergonomicItemLabel(null, "Laptop stand present", "de-001")).toBe(
      "Laptop stand present",
    );
    expect(ergonomicItemLabel("standing_desk", "Standing desk", "de-001")).toBe(
      "Standing desk",
    );
  });

  it("is in every shipped language locale, and the English matches the service's text", () => {
    const languages = LOCALES.filter((l) => l.endsWith("-001"));
    expect(languages.length).toBeGreaterThanOrEqual(13);
    for (const locale of languages) {
      const table = STRINGS_BY_LOCALE[locale];
      const held = Object.keys(table)
        .filter((k) => k.startsWith("erg.items."))
        .map((k) => k.slice("erg.items.".length));
      expect(held, locale).toEqual(KEYS);
      for (const key of KEYS)
        expect(
          table[`erg.items.${key}`]?.trim(),
          `${locale} ${key}`,
        ).toBeTruthy();
    }
    for (const key of KEYS) {
      const text = new RegExp(`key: "${key}",\\s*text: "([^"]+)"`).exec(
        RULE,
      )?.[1];
      expect(STRINGS_BY_LOCALE["en-001"][`erg.items.${key}`], key).toBe(text);
    }
  });
});
