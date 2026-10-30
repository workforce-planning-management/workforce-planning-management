// WPM-T373 and WPM-T376: one money formatter and parser. No currency is built in, the minor unit is the currency's own
// (JPY 0, EUR 2, KWD 3, CLF 4), and an amount is read in the reader's own number format. The cases follow the sibling
// project-portfolio-management UI's tests/unit/money.test.ts, so the two apps stay alike, in WPM's locale codes.

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  currencyCode,
  exponentOf,
  minorToInput,
  money,
  moneyWhole,
  parseAmount,
  signedMoney,
  toMinor,
} from "../../src/lib/money";

describe("exponentOf", () => {
  it("is the currency's own minor unit", () => {
    expect(exponentOf("JPY", "en-001")).toBe(0);
    expect(exponentOf("EUR", "en-001")).toBe(2);
    expect(exponentOf("KWD", "en-001")).toBe(3);
    expect(exponentOf("CLF", "en-001")).toBe(4);
  });
  it("is the same in every shipped locale", () => {
    for (const locale of [
      "ar-001",
      "bn-001",
      "cy-001",
      "de-001",
      "fr-001",
      "hi-001",
      "zh-001",
    ]) {
      expect(exponentOf("JPY", locale), locale).toBe(0);
      expect(exponentOf("KWD", locale), locale).toBe(3);
    }
  });
});

describe("money", () => {
  it("scales by the currency's exponent, not by 100", () => {
    expect(money(123_456, "JPY", "en-001")).toBe("JP¥123,456");
    expect(money(1_234_567, "KWD", "en-001")).toContain("1,234.567");
    expect(money(123_450, "EUR", "en-001")).toBe("€1,234.50");
    expect(money(12_345, "CLF", "en-001")).toContain("1.2345");
  });
  it("uses the reader's own separators", () => {
    expect(money(123_450, "EUR", "de-001")).toMatch(/^1\.234,50\s€$/u);
    expect(money(123_450, "EUR", "fr-001")).toMatch(/^1\s234,50\s€$/u);
    expect(money(1500, "JPY", "de-001")).toMatch(/^1\.500\s¥$/u);
  });
  it("renders an unknown amount or a missing currency as an em dash, never zero", () => {
    expect(money(null, "EUR", "en-001")).toBe("—");
    expect(money(undefined, "EUR", "en-001")).toBe("—");
    expect(money(100, null, "en-001")).toBe("—");
    expect(money(100, "", "en-001")).toBe("—");
  });
  it("shows the sign of a change", () => {
    expect(signedMoney(1000, "JPY", "en-001")).toBe("+JP¥1,000");
    expect(signedMoney(-5, "EUR", "en-001")).toBe("-€0.05");
    expect(signedMoney(0, "EUR", "en-001")).toBe("€0.00");
  });
});

describe("moneyWhole", () => {
  it("drops the fraction of a whole amount in the currency's own minor unit", () => {
    expect(moneyWhole(2_630_000, "GBP", "en-001")).toBe("£26,300");
    expect(moneyWhole(2_630_050, "GBP", "en-001")).toBe("£26,300.50");
    expect(moneyWhole(26_300, "JPY", "en-001")).toBe("JP¥26,300");
    expect(moneyWhole(26_300_000, "KWD", "en-001")).toMatch(/26,300$/u);
    expect(moneyWhole(26_300_500, "KWD", "en-001")).toContain("26,300.500");
    expect(moneyWhole(null, "KWD", "en-001")).toBe("—");
  });
});

describe("parseAmount and toMinor", () => {
  it("reads a decimal comma where the locale uses one", () => {
    expect(parseAmount("1,5", "de-001")).toBe(1.5);
    expect(parseAmount("12,50", "de-001")).toBe(12.5);
    expect(parseAmount("1 234,50", "fr-001")).toBe(1234.5);
    expect(parseAmount("1.234,50", "de-001")).toBe(1234.5);
    expect(toMinor("1 234,50", "EUR", "fr-001")).toBe(123_450);
  });
  it("reads a decimal point where the locale uses one", () => {
    expect(parseAmount("1,234.50", "en-001")).toBe(1234.5);
    expect(parseAmount("1234.5", "en-001")).toBe(1234.5);
  });
  it("reads the locale's own digits", () => {
    expect(parseAmount("১২৩৪.৫০", "bn-001")).toBe(1234.5);
  });
  it("never turns text into a number", () => {
    expect(parseAmount("", "en-001")).toBeNull();
    expect(parseAmount("abc", "en-001")).toBeNull();
    expect(parseAmount("1.2.3", "en-001")).toBeNull();
    expect(parseAmount("12 EUR", "en-001")).toBeNull();
  });
  it("scales by the exponent without float error", () => {
    expect(toMinor("1500", "JPY", "en-001")).toBe(1500);
    expect(toMinor("1.234", "KWD", "en-001")).toBe(1234);
    expect(toMinor("1.005", "EUR", "en-001")).toBe(101);
    expect(toMinor("0,5", "EUR", "de-001")).toBe(50);
    expect(toMinor("1.2345", "CLF", "en-001")).toBe(12345);
    expect(toMinor("x", "EUR", "en-001")).toBeNull();
  });
});

describe("minorToInput", () => {
  it("is what the same locale reads back", () => {
    for (const [minor, cur, loc] of [
      [150_050, "EUR", "de-001"],
      [1500, "JPY", "en-001"],
      [1_234_567, "KWD", "fr-001"],
      [12_345, "CLF", "en-001"],
    ] as const) {
      const text = minorToInput(minor, cur, loc);
      expect(toMinor(text, cur, loc)).toBe(minor);
    }
    expect(minorToInput(150_050, "EUR", "de-001")).toBe("1500,50");
    expect(minorToInput(1500, "JPY", "en-001")).toBe("1500");
  });
});

describe("currencyCode", () => {
  it("is three letters, upper-cased, and nothing is assumed", () => {
    expect(currencyCode("jpy")).toBe("JPY");
    expect(currencyCode(" EUR ")).toBe("EUR");
    expect(currencyCode("")).toBeNull();
    expect(currencyCode("EU")).toBeNull();
    expect(currencyCode("E1R")).toBeNull();
  });
});

describe("no built-in currency and no fixed 100 in the UI (WPM-T373, WPM-T376)", () => {
  const SRC = join(__dirname, "../../src");
  const files = (dir: string): string[] =>
    readdirSync(dir).flatMap((name) => {
      const p = join(dir, name);
      if (statSync(p).isDirectory()) return files(p);
      return /\.(ts|svelte)$/.test(name) && !name.endsWith(".d.ts") ? [p] : [];
    });
  const sources = files(SRC).map(
    (p) => [p.slice(SRC.length + 1), readFileSync(p, "utf8")] as const,
  );

  it("no form starts with a currency", () => {
    for (const [file, text] of sources) {
      expect(text, file).not.toMatch(
        /\$state(<[^>]*>)?\(\s*["'][A-Z]{3}["']\s*\)/,
      );
    }
    const read = (p: string) => readFileSync(join(SRC, p), "utf8");
    expect(read("lib/components/ExpenseClaims.svelte")).toMatch(
      /let currency = \$state\(""\)/,
    );
    expect(read("routes/planning/+page.svelte")).toMatch(
      /let budgetCurrency = \$state\(""\)/,
    );
  });

  it("no file divides or multiplies money by a fixed 100", () => {
    for (const [file, text] of sources) {
      expect(text, file).not.toMatch(
        /(minor|pounds|pence|budget|salary|amount)\w* ?[*/%] ?100\b/i,
      );
      expect(text, file).not.toMatch(
        /\b100 ?\* ?(minor|pounds|budget|salary|amount)/i,
      );
    }
  });
});
