// Plural rules (WPM-T374; spec/worldwide.md, WPM-R214, WPM-D143): a string that fills a count is a plural group chosen
// with `Intl.PluralRules` in the page's locale, and every locale's file holds the CLDR categories its language uses.

import { describe, expect, it } from "vitest";

import {
  LOCALES,
  PLURAL_CATEGORIES,
  PLURAL_KEYS,
  STRINGS_BY_LOCALE,
  pluralCategories,
  pluralCategory,
  translatePlural,
} from "../../src/lib/i18n.svelte";

const KEY = "pages.cpd.requirement_s_defined" as const;

describe("plural selection", () => {
  it("English has two forms: one and other", () => {
    expect(pluralCategory(1, "en-001")).toBe("one");
    expect(pluralCategory(0, "en-001")).toBe("other");
    expect(pluralCategory(2, "en-001")).toBe("other");
    expect(translatePlural(KEY, 1, "en-001")).toBe(
      "{requirements} requirement defined.",
    );
    expect(translatePlural(KEY, 5, "en-001")).toBe(
      "{requirements} requirements defined.",
    );
  });

  it("Arabic has six: zero, one, two, few, many, other", () => {
    expect(pluralCategories("ar-001")).toEqual([...PLURAL_CATEGORIES]);
    const forms = [0, 1, 2, 3, 11, 100].map((n) => pluralCategory(n, "ar-001"));
    expect(forms).toEqual(["zero", "one", "two", "few", "many", "other"]);
    const texts = [0, 1, 2, 3, 11, 100].map((n) =>
      translatePlural(KEY, n, "ar-001"),
    );
    expect(new Set(texts).size).toBe(6);
  });

  it("Russian has one, few and many for whole numbers", () => {
    expect(
      [1, 21, 2, 4, 22, 5, 11, 0].map((n) => pluralCategory(n, "ru-001")),
    ).toEqual(["one", "one", "few", "few", "few", "many", "many", "many"]);
    expect(translatePlural(KEY, 21, "ru-001")).toBe(
      "Задано {requirements} требование.",
    );
    expect(translatePlural(KEY, 3, "ru-001")).toBe(
      "Задано {requirements} требования.",
    );
    expect(translatePlural(KEY, 5, "ru-001")).toBe(
      "Задано {requirements} требований.",
    );
  });

  it("Welsh has six, like Arabic but with its own rules", () => {
    expect([0, 1, 2, 3, 6, 4].map((n) => pluralCategory(n, "cy-001"))).toEqual([
      "zero",
      "one",
      "two",
      "few",
      "many",
      "other",
    ]);
  });

  it("Chinese and Indonesian have one form for every count", () => {
    for (const locale of ["zh-001", "id-001"]) {
      expect(pluralCategories(locale)).toEqual(["other"]);
      expect(translatePlural(KEY, 1, locale)).toBe(
        translatePlural(KEY, 7, locale),
      );
    }
    expect(translatePlural(KEY, 1, "zh-001")).toBe(
      "已定义 {requirements} 项要求。",
    );
  });

  it("a regional locale uses its language's forms", () => {
    expect(translatePlural(KEY, 1, "en-gb")).toBe(
      translatePlural(KEY, 1, "en-001"),
    );
    expect(translatePlural(KEY, 1000000, "es-es")).toBe(
      translatePlural(KEY, 1000000, "es-001"),
    );
  });

  it("no plural form appends an English letter or writes (s)", () => {
    for (const locale of LOCALES) {
      for (const [key, text] of Object.entries(STRINGS_BY_LOCALE[locale])) {
        if (PLURAL_KEYS.some((b) => key.startsWith(`${b}.`)))
          expect(text, `${locale} ${key}`).not.toMatch(/\{changes2\}|\(s\)/);
      }
    }
    // …and no English string fills a count with "(s)" any more.
    const english = Object.values(STRINGS_BY_LOCALE["en-001"]).filter((t) =>
      /\w\(s\)/.test(t),
    );
    expect(english).toEqual([]);
  });
});

describe("every plural group has the forms its locale needs", () => {
  it("finds the plural groups", () => {
    expect(PLURAL_KEYS.length).toBeGreaterThanOrEqual(9);
  });

  for (const locale of LOCALES.filter((l) => l.endsWith("-001"))) {
    it(`${locale} holds exactly ${pluralCategories(locale).join(", ")}`, () => {
      const table = STRINGS_BY_LOCALE[locale];
      const need = pluralCategories(locale);
      const problems: string[] = [];
      for (const base of PLURAL_KEYS) {
        const have = PLURAL_CATEGORIES.filter(
          (c) => table[`${base}.${c}`] !== undefined,
        );
        if (have.join() !== need.join())
          problems.push(
            `${base}: has ${have.join(",")}, needs ${need.join(",")}`,
          );
        for (const c of have)
          if (!table[`${base}.${c}`]?.trim())
            problems.push(`${base}.${c} is empty`);
      }
      expect(problems, problems.join("\n")).toEqual([]);
    });
  }

  it("a regional locale holds only categories its language uses", () => {
    for (const locale of LOCALES.filter((l) => !l.endsWith("-001"))) {
      const need = pluralCategories(locale) as string[];
      for (const key of Object.keys(STRINGS_BY_LOCALE[locale])) {
        const base = PLURAL_KEYS.find((b) => key.startsWith(`${b}.`));
        if (base)
          expect(need, `${locale} ${key}`).toContain(
            key.slice(base.length + 1),
          );
      }
    }
  });
});
