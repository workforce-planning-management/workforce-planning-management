// The translation-review register (content/locales/REVIEW.md) names every locale and never claims a review nobody did.
// Key parity (every locale holds every key) is pinned in `wpm.test.ts`; this keeps the register honest beside it.

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const LOCALES_DIR = join(__dirname, "../../content/locales");

interface Row {
  locale: string;
  writtenBy: string;
  reviewer: string;
  reviewedOn: string;
  scope: string;
  outcome: string;
}

/** The register's table rows, by the locale in the first column. */
export function rows(markdown: string): Row[] {
  return markdown
    .split("\n")
    .filter((line) => /^\|\s*`[a-z]{2,3}-[a-z0-9]+`/.test(line))
    .map((line) => {
      const c = line
        .split("|")
        .slice(1, -1)
        .map((x) => x.trim());
      return {
        locale: (c[0] ?? "").replace(/`/g, ""),
        writtenBy: c[2] ?? "",
        reviewer: c[3] ?? "",
        reviewedOn: c[4] ?? "",
        scope: c[5] ?? "",
        outcome: c[6] ?? "",
      };
    });
}

const NOT = /^(none|not reviewed)$/i;

/** Why a row cannot stand, or null. A review is claimed only with a reviewer, a date, a scope and an outcome. */
export function problem(row: Row): string | null {
  if (!row.writtenBy) return "does not say who wrote the translation";
  const fields = [row.reviewer, row.reviewedOn, row.scope, row.outcome];
  const none = fields.filter((f) => NOT.test(f)).length;
  if (none === 4) return null; // not reviewed: all four say so
  if (none > 0)
    return "claims a review but leaves the reviewer, date, scope or outcome as not reviewed";
  if (!/^\d{4}-\d{2}-\d{2}$/.test(row.reviewedOn))
    return "reviewed on is not a date (YYYY-MM-DD)";
  return null;
}

describe("translation-review register", () => {
  const register = readFileSync(join(LOCALES_DIR, "REVIEW.md"), "utf8");
  const table = rows(register);
  const served = readdirSync(LOCALES_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory() && d.name.endsWith("-001"))
    .map((d) => d.name)
    .filter((n) => n !== "en-001")
    .sort();

  it("has a row for every translated locale and none for a locale that does not exist", () => {
    expect(table.map((r) => r.locale).sort()).toEqual(served);
  });

  it("names a locale once", () => {
    const names = table.map((r) => r.locale);
    expect(new Set(names).size).toBe(names.length);
  });

  it("every row says who wrote it and claims a review only in full", () => {
    const bad = table
      .map((r) => [r.locale, problem(r)] as const)
      .filter(([, p]) => p !== null)
      .map(([l, p]) => `${l}: ${p}`);
    expect(bad, bad.join("\n")).toEqual([]);
  });

  it("says in words that the translations are unreviewed, while any row is", () => {
    if (table.some((r) => NOT.test(r.reviewer))) {
      expect(register).toMatch(/has not been reviewed by a person/);
    }
  });

  describe("the checker itself", () => {
    const row = (over: Partial<Row>): Row => ({
      locale: "xx-001",
      writtenBy: "AI assistant, 2026-10-10",
      reviewer: "none",
      reviewedOn: "not reviewed",
      scope: "not reviewed",
      outcome: "not reviewed",
      ...over,
    });

    it("accepts not reviewed, and a full review", () => {
      expect(problem(row({}))).toBeNull();
      expect(
        problem(
          row({
            reviewer: "A. Translator",
            reviewedOn: "2026-11-02",
            scope: "all keys",
            outcome: "accepted with changes",
          }),
        ),
      ).toBeNull();
    });

    it("refuses a review claimed in part, or with no date", () => {
      expect(problem(row({ reviewer: "A. Translator" }))).toMatch(
        /claims a review/,
      );
      expect(
        problem(
          row({
            reviewer: "A. Translator",
            reviewedOn: "last week",
            scope: "all",
            outcome: "ok",
          }),
        ),
      ).toMatch(/not a date/);
      expect(problem(row({ writtenBy: "" }))).toMatch(/who wrote/);
    });

    it("reads the rows of the table and skips everything else", () => {
      const md =
        "| Locale | Language |\n| --- | --- |\n| `de-001` | German | AI | none | not reviewed | not reviewed | not reviewed |\nplain text\n";
      expect(rows(md).map((r) => r.locale)).toEqual(["de-001"]);
    });
  });
});
