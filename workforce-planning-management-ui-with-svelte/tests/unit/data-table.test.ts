// The data tables: every locale supplies every string the grid needs, and only the tables
// named here offer an export (WPM-R133, WPM-D84).

import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { missingLabels } from "@lilydesignsystem/svelte-data-grid";
import { describe, expect, it } from "vitest";
import { dataTableLabels, type Say } from "../../src/lib/data-table-labels";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const locales = join(root, "content", "locales");

function flatten(
  node: Record<string, unknown>,
  prefix = "",
): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [key, value] of Object.entries(node)) {
    if (typeof value === "string") out[`${prefix}${key}`] = value;
    else
      Object.assign(
        out,
        flatten(value as Record<string, unknown>, `${prefix}${key}.`),
      );
  }
  return out;
}

function sayFor(locale: string): Say {
  const catalogue = flatten(
    JSON.parse(readFileSync(join(locales, locale, "ui.json"), "utf8")),
  );
  const t = (key: string) => catalogue[key] ?? "";
  return {
    t: t as Say["t"],
    tf: ((key: string, params: Record<string, unknown> = {}) =>
      t(key).replace(/\{(\w+)\}/g, (whole, name: string) =>
        name in params ? String(params[name]) : whole,
      )) as Say["tf"],
  };
}

describe("the grid's labels, per locale", () => {
  const all = readdirSync(locales).filter((l) => l.endsWith("-001"));
  it("finds the locales", () => expect(all.length).toBe(13));

  for (const locale of all) {
    it(`${locale} supplies every label a table with paging, search, columns and export needs`, () => {
      const labels = dataTableLabels(sayFor(locale), {
        hidable: true,
        searchable: true,
        exportable: true,
        rowCount: 40,
      });
      const missing = missingLabels(
        {
          pagination: true,
          hidableColumns: true,
          search: true,
          sorting: true,
          exporting: true,
        },
        labels,
      ).map((m) => `${m.key} (${m.kind})`);
      expect(missing, missing.join(", ")).toEqual([]);
      // The extra controls the wrapper turns on: the search column select and the pager landmark.
      for (const key of [
        "searchField",
        "searchAllFields",
        "pagination",
        "exportTsv",
        "copy",
      ] as const) {
        expect(labels[key], key).toBeTruthy();
      }
    });
  }

  it("a table with no export, search or hidden columns offers none of them", () => {
    const labels = dataTableLabels(sayFor("en-001"), { rowCount: 3 });
    expect(labels.export).toBeUndefined();
    expect(labels.exportCsv).toBeUndefined();
    expect(labels.copy).toBeUndefined();
    expect(labels.search).toBeUndefined();
    expect(labels.columnVisibility).toBeUndefined();
  });
});

describe("which tables offer an export (WPM-R133)", () => {
  // Aggregate tables that name no one. Adding a name here is a spec change.
  const allowed = [
    "contingent-departments",
    "cpd-overview",
    "metrics",
    "pay-bands",
    "training-analytics",
  ];

  function walk(dir: string): string[] {
    return readdirSync(dir).flatMap((name) => {
      const path = join(dir, name);
      return statSync(path).isDirectory()
        ? walk(path)
        : path.endsWith(".svelte")
          ? [path]
          : [];
    });
  }

  it("is exactly the aggregate tables, and no file passes a computed name", () => {
    const found: string[] = [];
    for (const file of [
      ...walk(join(root, "src", "routes")),
      ...walk(join(root, "src", "lib")),
    ]) {
      const text = readFileSync(file, "utf8");
      if (file.endsWith("DataTable.svelte")) continue;
      for (const m of text.matchAll(/exportName=(\{[^}]*\}|"[^"]*")/g)) {
        const literal = /^"([^"]+)"$/.exec(m[1]!);
        expect(
          literal,
          `${relative(root, file)} passes ${m[1]}`,
        ).not.toBeNull();
        found.push(literal![1]!);
      }
    }
    expect(found.sort()).toEqual(allowed);
  });
});
