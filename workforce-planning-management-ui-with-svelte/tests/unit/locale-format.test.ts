// Numbers, percentages, FTE and dates in the page's locale (WPM-T375; spec/worldwide.md, WPM-R214, WPM-D143), and a scan
// of `src/` that fails on new hand-rolled formatting: `toFixed`, `toLocale*`, a percent sign glued to a number, a
// timestamp sliced to a date, an ISO date field put straight into markup or a `tf`/`tp` placeholder.

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

import {
  compareText,
  DASH,
  formatBasisPoints,
  formatCentiFte,
  formatDate,
  formatDateTime,
  formatDecimal,
  formatInteger,
  formatMonth,
  formatNumber,
  formatPercent,
  formatPercentValue,
  formatSigned,
  formatTime,
  numberToInput,
  parseNumber,
  parseScaled,
} from "../../src/lib/locale-format";
import { percent, rate } from "../../src/lib/format";
import { tp } from "../../src/lib/i18n.svelte";

const NBSP = " ";
const NNBSP = " ";

describe("numbers in the page's locale", () => {
  it("groups and marks decimals the locale's way, in its own digits", () => {
    expect(formatNumber(1234.5, {}, "en-001")).toBe("1,234.5");
    expect(formatNumber(1234.5, {}, "de-001")).toBe("1.234,5");
    expect(formatNumber(1234.5, {}, "fr-001")).toBe(`1${NNBSP}234,5`);
    expect(formatNumber(1234567, {}, "hi-001")).toBe("12,34,567");
    expect(formatNumber(1234.5, {}, "bn-001")).toBe("১,২৩৪.৫");
  });

  it("shows an unknown figure as a dash, never 0", () => {
    expect(formatNumber(null)).toBe(DASH);
    expect(formatNumber(undefined)).toBe(DASH);
    expect(formatNumber(Number.NaN)).toBe(DASH);
    expect(formatNumber(0, {}, "en-001")).toBe("0");
    expect(formatPercent(null)).toBe(DASH);
    expect(formatCentiFte(null)).toBe(DASH);
  });

  it("writes fixed decimals, integers and signs", () => {
    expect(formatDecimal(2.5, 2, "en-001")).toBe("2.50");
    expect(formatDecimal(2.5, 2, "de-001")).toBe("2,50");
    expect(formatInteger(1234.4, "en-001")).toBe("1,234");
    expect(formatSigned(3, {}, "en-001")).toBe("+3");
    expect(formatSigned(-1.5, {}, "en-001")).toBe("-1.5");
    expect(formatSigned(0, {}, "en-001")).toBe("0");
  });

  it("writes a percentage the locale's way, sign and space included", () => {
    expect(formatPercent(0.25, 0, "en-001")).toBe("25%");
    expect(formatPercent(0.25, 0, "de-001")).toBe(`25${NBSP}%`);
    expect(formatPercent(0.25, 0, "fr-001")).toBe(`25${NBSP}%`);
    expect(formatPercentValue(8.3, 1, "en-001")).toBe("8.3%");
    expect(formatPercentValue(8.3, 1, "de-001")).toBe(`8,3${NBSP}%`);
  });

  it("places the point of basis points and centi-FTE exactly", () => {
    expect(formatBasisPoints(10456, 2, "en-001")).toBe("104.56%");
    expect(formatBasisPoints(-5, 2, "en-001")).toBe("-0.05%");
    expect(formatBasisPoints(909, 2, "de-001")).toBe(`9,09${NBSP}%`);
    expect(formatCentiFte(150, "en-001")).toBe("1.50");
    expect(formatCentiFte(150, "de-001")).toBe("1,50");
  });

  it("is what the null-not-zero helpers in format.ts now write", () => {
    expect(percent({ numerator: 2, denominator: 3, value: 2 / 3 })).toBe("67%");
    expect(rate(0.125)).toBe("12.5%");
    expect(rate(null)).toBeNull();
  });

  it("writes a number placed in a catalogue string in the page's format", () => {
    expect(
      tp(
        "insights.headcount_shrinking.observation",
        { pct: 12.5, opening: 1200, closing: 1050 },
        "de-001",
      ),
    ).toContain("1.200");
    expect(
      tp(
        "insights.headcount_shrinking.observation",
        { pct: 12.5, opening: 1200, closing: 1050 },
        "en-001",
      ),
    ).toBe("Headcount fell 12.5% (1,200 to 1,050).");
  });
});

describe("dates in the page's locale", () => {
  it("writes an ISO date the locale's way, the same day in every time zone", () => {
    expect(formatDate("2026-10-30", "medium", "en-001")).toBe("30 Oct 2026");
    expect(formatDate("2026-10-30", "medium", "en-us")).toBe("Oct 30, 2026");
    expect(formatDate("2026-10-30", "medium", "de-001")).toBe("30.10.2026");
    expect(formatDate("2026-10-30", "medium", "zh-001")).toBe("2026年10月30日");
    expect(formatDate("2026-01-01", "medium", "en-001")).toBe("1 Jan 2026");
  });

  it("shows a missing date as a dash and leaves text that is not a date alone", () => {
    expect(formatDate(null)).toBe(DASH);
    expect(formatDate("")).toBe(DASH);
    expect(formatDate("soon")).toBe("soon");
    expect(formatDate("2026-13-45", "medium", "en-001")).toBe("2026-13-45");
  });

  it("writes a timestamp as a date, a date and time, or a time", () => {
    const at = "2026-10-30T12:00:00Z";
    expect(formatDate(at, "medium", "en-001")).toMatch(/Oct 2026$/);
    expect(formatDateTime(at, "en-001")).toMatch(/2026.*\d{1,2}[:.]\d{2}/);
    expect(formatTime(at, "en-001")).toMatch(/^\d{1,2}[:.]\d{2}/);
    // A civil date has no time of day.
    expect(formatDateTime("2026-10-30", "en-001")).toBe("30 Oct 2026");
  });

  it("writes a month as a month and year", () => {
    expect(formatMonth("2026-10", "short", "en-001")).toBe("Oct 2026");
    expect(formatMonth("2026-10-01", "long", "en-001")).toBe("October 2026");
    expect(formatMonth("2026-10", "long", "de-001")).toBe("Oktober 2026");
  });
});

describe("numbers typed in the page's locale", () => {
  it("reads the locale's decimal mark, through money.ts's parser", () => {
    expect(parseNumber("2,5", "de-001")).toBe(2.5);
    expect(parseNumber("2.5", "en-001")).toBe(2.5);
    expect(parseNumber("1.234,5", "de-001")).toBe(1234.5);
    expect(parseNumber("২.৫", "bn-001")).toBe(2.5);
    expect(parseNumber("two", "en-001")).toBeNull();
    expect(parseNumber("", "en-001")).toBeNull();
  });

  it("scales a typed percentage or FTE to the API's integer without float error", () => {
    expect(parseScaled("2,5", 2, "de-001")).toBe(250);
    expect(parseScaled("0.8", 2, "en-001")).toBe(80);
    // 1.005 * 100 is 100.49999…; "1.005e2" is exactly 100.5.
    expect(parseScaled("1.005", 2, "en-001")).toBe(101);
    expect(parseScaled("x", 2, "en-001")).toBeNull();
  });

  it("writes a number back for a text input with no grouping", () => {
    expect(numberToInput(2.5, "de-001")).toBe("2,5");
    expect(numberToInput(1234.5, "en-001")).toBe("1234.5");
    expect(numberToInput(null)).toBe("");
  });
});

describe("names sorted in the page's locale (WPM-T381)", () => {
  const sorted = (names: string[], loc: string) =>
    [...names].sort((a, b) => compareText(a, b, loc));

  it("follows the language's own order, not the runtime's", () => {
    // Swedish puts å, ä, ö after z; English and German keep them beside a and o
    expect(sorted(["Östberg", "Zola", "Åberg", "Andersson"], "sv")).toEqual([
      "Andersson",
      "Zola",
      "Åberg",
      "Östberg",
    ]);
    expect(sorted(["Östberg", "Zola", "Åberg", "Andersson"], "de")).toEqual([
      "Åberg",
      "Andersson",
      "Östberg",
      "Zola",
    ]);
    // Spanish: ñ is a letter after n
    expect(sorted(["Ñúñez", "Nuño", "Ortiz"], "es")).toEqual([
      "Nuño",
      "Ñúñez",
      "Ortiz",
    ]);
  });

  it("sorts whole names in any script, with no first or last assumed", () => {
    // a family name first is sorted as written: the whole display name is the key
    expect(sorted(["Yamada Taro", "Abe Shinzo", "Taro Abe"], "en")).toEqual([
      "Abe Shinzo",
      "Taro Abe",
      "Yamada Taro",
    ]);
    expect(
      sorted(["Пётр Иванов", "Анна Смирнова", "Иван Петров"], "ru"),
    ).toEqual(["Анна Смирнова", "Иван Петров", "Пётр Иванов"]);
    expect(sorted(["يوسف", "أحمد", "محمد"], "ar")).toEqual([
      "أحمد",
      "محمد",
      "يوسف",
    ]);
    expect(sorted(["Suharto", "ada lovelace", "Ada Byron"], "en")).toEqual([
      "Ada Byron",
      "ada lovelace",
      "Suharto",
    ]);
  });

  it("compares numbers in a label by value, and defaults to the page's locale", () => {
    expect(sorted(["Team 10", "Team 2"], "en")).toEqual(["Team 2", "Team 10"]);
    expect(compareText("a", "b")).toBeLessThan(0);
    expect(compareText("same", "same", "not a locale!")).toBe(0);
  });
});

// The scan. A line that matches is hand-rolled formatting of something a person reads. Allowed: comments, CSS, and a
// line in ALLOW with its reason.
const here = join(__dirname, "..", "..");
const ALLOW: Record<string, string> = {};
const FIELD = String.raw`[\w.?]*?(?:\w+_(?:on|at|date|start|end|until|from|due|day)\b|decide_by|remedy_by|as_of|\.date\b|\.on\b|\.month\b)`;
const RULES: [string, RegExp][] = [
  ["toFixed", /\.toFixed\(/],
  ["localeCompare (use compareText)", /\.localeCompare\(/],
  ["toLocale*", /\.toLocale(?:String|DateString|TimeString)\(/],
  ["a percent sign glued to a number", /(?:\$\{[^}]*\}|\}|\))%(?![\w;"])/],
  [
    "a timestamp sliced to a date",
    /\b\w+(?:_at|month)\b[^;\n]*?\.slice\(0, (?:7|10|16|19)\)/,
  ],
  [
    "an ISO date in markup",
    new RegExp(String.raw`\{\s*${FIELD}\s*(?:\?\?[^}]*)?\}`),
  ],
  [
    "an ISO date in a placeholder",
    new RegExp(
      String.raw`\bt[fp]\([^)]*?\b\w+:\s*\(?${FIELD}(?!\()\s*(?:\?\?\s*[^,}]+)?\)?\s*[,}]`,
    ),
  ],
  [
    "an ISO date in a template string",
    new RegExp(String.raw`\$\{\s*${FIELD}\s*\}`),
  ],
];

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...walk(path));
    else if (/\.(svelte|ts)$/.test(path)) out.push(path);
  }
  return out;
}

/** Hand-rolled formatting in one file's text, as "rule: line". */
export function handRolled(text: string, file = ""): string[] {
  const found: string[] = [];
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (/^(\/\/|\*|\/\*|<!--)/.test(line)) continue;
    if (/style=|width:|height:|type="date"/.test(line)) continue;
    if (ALLOW[`${file}:${line}`]) continue;
    for (const [name, re] of RULES)
      if (re.test(line)) found.push(`${name}: ${line.slice(0, 120)}`);
  }
  return found;
}

describe("no hand-rolled number or date formatting in src/", () => {
  it("finds it when it is there (the scan works)", () => {
    expect(handRolled("const x = (n / 100).toFixed(2);")).toHaveLength(1);
    expect(handRolled("const s = `${Math.round(v * 100)}%`;")).toHaveLength(1);
    expect(handRolled("<td>{row.target_on}</td>")).toHaveLength(1);
    expect(handRolled('{tf("payInfo.due", { day: r.due_on })}')).toHaveLength(
      1,
    );
    expect(handRolled('{tf("x", { date: r.decide_by })}')).toHaveLength(1);
    expect(handRolled("{a.performed_at.slice(0, 16)}")).toHaveLength(1);
    expect(handRolled("new Date(x).toLocaleTimeString([])")).toHaveLength(1);
    expect(handRolled("names.sort((a, b) => a.localeCompare(b))")).toHaveLength(
      1,
    );
    // …and not when the value goes through the module.
    expect(handRolled("<td>{formatDate(row.target_on)}</td>")).toEqual([]);
    expect(
      handRolled('{tf("payInfo.due", { day: formatDate(r.due_on) })}'),
    ).toEqual([]);
    expect(handRolled('<div style="width: {pct}%"></div>')).toEqual([]);
  });

  it("finds none", () => {
    const offenders: string[] = [];
    for (const file of walk(join(here, "src"))) {
      if (/locale-format\.ts$|money\.ts$/.test(file)) continue;
      const rel = relative(here, file);
      for (const f of handRolled(readFileSync(file, "utf8"), rel))
        offenders.push(`${rel}: ${f}`);
    }
    expect(offenders, offenders.join("\n")).toEqual([]);
  });
});
