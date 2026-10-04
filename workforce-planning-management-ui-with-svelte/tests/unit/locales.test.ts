import { describe, expect, it } from "vitest";
import {
  LOCALE_ALIASES,
  LOCALES,
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
