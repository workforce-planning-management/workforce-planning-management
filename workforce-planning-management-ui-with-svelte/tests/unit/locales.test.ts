import { readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  CMS_UI_LANGUAGES,
  CMS_UI_LOCALE,
  LOCALE_ALIASES,
  LOCALES,
  localeFromNavigator,
  localePath,
  negotiateLocale,
  normaliseLocale,
  splitLocale,
} from "../../src/lib/locales";

describe("locale routes", () => {
  it("aliases every bare language to its -001 locale", () => {
    expect(LOCALE_ALIASES.en).toBe("en-001");
    expect(LOCALE_ALIASES.cy).toBe("cy-001");
    // Regional locales have no alias of their own.
    expect(LOCALE_ALIASES["en-gb"]).toBeUndefined();
    for (const locale of LOCALES.filter((l) => l.endsWith("-001"))) {
      expect(LOCALE_ALIASES[locale.slice(0, -4)]).toBe(locale);
    }
  });

  it("splits a canonical prefix, an alias prefix, and no prefix", () => {
    expect(splitLocale("/en-001/workers")).toEqual({
      locale: "en-001",
      alias: false,
      rest: "/workers",
    });
    expect(splitLocale("/cy/workers/abc")).toEqual({
      locale: "cy-001",
      alias: true,
      rest: "/workers/abc",
    });
    expect(splitLocale("/cy-001")).toEqual({
      locale: "cy-001",
      alias: false,
      rest: "/",
    });
    expect(splitLocale("/workers")).toEqual({
      locale: null,
      alias: false,
      rest: "/workers",
    });
    expect(splitLocale("/")).toEqual({ locale: null, alias: false, rest: "/" });
  });

  it("prefixes app paths and leaves external links and anchors alone", () => {
    expect(localePath("cy-001", "/workers")).toBe("/cy-001/workers");
    expect(localePath("cy-001", "/")).toBe("/cy-001");
    expect(localePath("en-gb", "/tour#why")).toBe("/en-gb/tour#why");
    expect(localePath("en-001", "https://example.com/x")).toBe(
      "https://example.com/x",
    );
    expect(localePath("en-001", "#top")).toBe("#top");
  });

  it("normalises legacy spellings and negotiates Accept-Language", () => {
    expect(normaliseLocale("de_DE")).toBe("de-de");
    expect(normaliseLocale("PT-br")).toBe("pt-001");
    expect(normaliseLocale("xx")).toBeNull();
    expect(negotiateLocale("xx, fr-CA;q=0.8, en;q=0.5")).toBe("fr-001");
    expect(negotiateLocale(null)).toBe("en-001");
  });
});

describe("CMS config", () => {
  it("is generated from the English strings and up to date", async () => {
    const { config } = await import("../../scripts/cms-config.mjs");
    const { readFileSync } = await import("node:fs");
    expect(readFileSync("static/admin/config.yml", "utf8")).toBe(config());
  });
});

describe("CMS interface language", () => {
  it("maps every content locale to a language Sveltia ships", () => {
    for (const locale of LOCALES) {
      expect(CMS_UI_LANGUAGES, locale).toContain(CMS_UI_LOCALE[locale]);
    }
  });
});

describe("browser language to locale route", () => {
  it("matches an underscore tag to the exact regional locale, else its language", () => {
    // The app has no `cy-gb`, so Welsh in Great Britain gets the Welsh locale.
    expect(localeFromNavigator({ language: "cy_GB" })).toBe("cy-001");
    expect(localeFromNavigator({ language: "cy-GB" })).toBe("cy-001");
    // Where a regional locale exists, it wins over the language's -001.
    expect(localeFromNavigator({ language: "en_GB" })).toBe("en-gb");
    expect(localeFromNavigator({ language: "en-US" })).toBe("en-us");
    expect(localeFromNavigator({ language: "de-DE" })).toBe("de-de");
    expect(localeFromNavigator({ language: "es-ES" })).toBe("es-es");
    expect(localeFromNavigator({ language: "es-MX" })).toBe("es-001");
    expect(localeFromNavigator({ language: "zh-Hant-TW" })).toBe("zh-001");
  });

  it("walks navigator.languages in order and skips what the app does not serve", () => {
    expect(localeFromNavigator({ languages: ["ja-JP", "fr-CA", "en"] })).toBe(
      "fr-001",
    );
    expect(
      localeFromNavigator({ languages: ["en-GB", "cy"], language: "cy" }),
    ).toBe("en-gb");
    // `languages` wins over a differing `language`.
    expect(localeFromNavigator({ languages: ["pt-BR"], language: "ru" })).toBe(
      "pt-001",
    );
  });

  it("says null — not the default — when nothing matches, so the caller chooses", () => {
    expect(localeFromNavigator({ language: "ja-JP" })).toBeNull();
    expect(localeFromNavigator({ languages: ["ja", "ko"] })).toBeNull();
    expect(localeFromNavigator({ languages: [], language: "" })).toBeNull();
    expect(localeFromNavigator({})).toBeNull();
    expect(localeFromNavigator({ language: null, languages: null })).toBeNull();
  });
});

describe("locale directory names", () => {
  // `<language>-<region>`: ISO 639 language, then an ISO 3166-1 alpha-2 country or the
  // UN M.49 numeric `001`; lower-case. A bare language is a URL alias, never a directory.
  const SHAPE = /^[a-z]{2,3}-(?:[a-z]{2}|\d{3})$/;
  const dirs = readdirSync(join(process.cwd(), "content/locales"), {
    withFileTypes: true,
  })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name);

  it("names every directory <language>-<region>, never a bare language", () => {
    expect(dirs.length).toBeGreaterThan(0);
    for (const name of dirs) expect(name, name).toMatch(SHAPE);
  });

  it("has exactly the directories the app serves", () => {
    expect([...dirs].sort()).toEqual([...LOCALES].sort());
  });

  it("would reject the bare spellings", () => {
    for (const bad of ["en", "cy", "en_GB", "EN-GB", "en-1", "en-gbr"]) {
      expect(SHAPE.test(bad), bad).toBe(false);
    }
  });
});
