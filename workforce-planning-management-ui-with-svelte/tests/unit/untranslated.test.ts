// No visible text outside the catalogue, and no locale that quietly repeats English (WPM-R94).
//
// 1. Every component's template: any literal text a person would read (or a literal
//    placeholder, title, aria-label, alt or label) fails the test. Visible text belongs in
//    `content/locales/<locale>/ui.json` and is read with `t()`, `tf()` or `tv()`.
// 2. Every non-English `-001` locale: a value identical to English fails unless it is on the
//    allow-list, which names the reason (a brand, an acronym or a symbol, or a word that is
//    the same in that language).
// 3. Every token in the server's closed vocabularies has a `values.<token>` entry, so a status
//    or kind the server can send is never shown in English by default.

import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import allow from "./untranslated.allow.json";
import { literalText } from "./svelte-text";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..");
const locales = join(root, "content", "locales");

function walk(dir: string, accept: (file: string) => boolean): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...walk(path, accept));
    else if (accept(path)) out.push(path);
  }
  return out;
}

function flatten(node: Record<string, unknown>, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [key, value] of Object.entries(node)) {
    if (typeof value === "string") out[`${prefix}${key}`] = value;
    else Object.assign(out, flatten(value as Record<string, unknown>, `${prefix}${key}.`));
  }
  return out;
}

const catalogue = (locale: string) =>
  flatten(JSON.parse(readFileSync(join(locales, locale, "ui.json"), "utf8")));

describe("no visible text outside the catalogue", () => {
  const files = [
    ...walk(join(root, "src", "routes"), (f) => f.endsWith(".svelte")),
    ...walk(join(root, "src", "lib", "components"), (f) => f.endsWith(".svelte")),
  ];

  it("scans the templates", () => {
    expect(files.length).toBeGreaterThan(40);
  });

  it("finds literal text when there is some (the scanner works)", () => {
    const found = literalText(
      '<p class="x">Hello there</p><input placeholder="Search here" /><span>{n}</span><code>ls -l</code>',
    );
    expect(found.map((f) => f.text)).toEqual(["Hello there", 'placeholder="Search here"']);
    // An arrow or a comparison inside an expression does not end a tag early.
    expect(
      literalText('<button onclick={() => count > 1 && go()}>{t("a.b")}</button>').length,
    ).toBe(0);
  });

  it("has none", () => {
    const offenders: string[] = [];
    for (const file of files) {
      for (const finding of literalText(readFileSync(file, "utf8"))) {
        if (!(allow.literalText as string[]).includes(finding.text)) {
          offenders.push(`${relative(root, file)}: ${finding.text.slice(0, 90)}`);
        }
      }
    }
    expect(offenders, offenders.join("\n")).toEqual([]);
  });
});

describe("no locale repeats English", () => {
  const english = catalogue("en-001");
  const tolerated = allow.sameAsEnglish as Record<string, string[]>;
  for (const locale of readdirSync(locales)) {
    if (locale === "en-001" || !locale.endsWith("-001")) continue;
    it(`${locale} translates every key`, () => {
      const mine = catalogue(locale);
      const ok = new Set(tolerated[locale] ?? []);
      const offenders = Object.entries(english)
        .filter(([key, text]) => {
          const bare = text.replace(/\{\w+\}|WPM|ISCO|[—()·%/+]/g, "");
          return (
            mine[key] === text &&
            /[A-Za-z]{3,}/.test(bare) &&
            !ok.has(text) &&
            !(allow.anyLocale as string[]).includes(text)
          );
        })
        .map(([key, text]) => `${key}: ${text.slice(0, 70)}`);
      expect(offenders, offenders.join("\n")).toEqual([]);
    });
  }
});

describe("every server vocabulary token has a value entry", () => {
  const rules = join(root, "..", "workforce-planning-management-service-with-rust", "src", "rules");
  const tables = new Set<string>();
  const tokens = new Set<string>();

  it("reads the vocabularies", () => {
    const privacy = readFileSync(join(rules, "privacy.rs"), "utf8");
    const list = /SOFT_DELETED_TABLES: &\[&str\] = &\[([^\]]*)\]/.exec(privacy)?.[1] ?? "";
    for (const m of list.matchAll(/"([a-z_]+)"/g)) tables.add(m[1]!);
    expect(tables.size).toBeGreaterThan(50);
    const notificationKinds = new Set<string>();
    const notify = readFileSync(join(rules, "notify.rs"), "utf8");
    const kinds = /pub const KINDS: &\[&str\] = &\[([^\]]*)\]/.exec(notify)?.[1] ?? "";
    for (const m of kinds.matchAll(/"([a-z_]+)"/g)) notificationKinds.add(m[1]!);
    for (const file of walk(rules, (f) => f.endsWith(".rs"))) {
      const text = readFileSync(file, "utf8");
      for (const m of text.matchAll(/pub const ([A-Z_0-9]+): &\[&str\] = &\[([^\]]*)\];/g)) {
        for (const t of m[2]!.matchAll(/"([^"]+)"/g)) {
          const token = t[1]!;
          if (tables.has(token) || token.includes(" ")) continue;
          if (m[1] === "KINDS" && notificationKinds.has(token)) continue;
          tokens.add(token);
        }
      }
    }
    expect(tokens.size).toBeGreaterThan(150);
  });

  it("has a values entry in English for each", () => {
    const values = catalogue("en-001");
    const missing = [...tokens].filter((token) => !(`values.${token}` in values));
    expect(missing, missing.join(", ")).toEqual([]);
  });
});
