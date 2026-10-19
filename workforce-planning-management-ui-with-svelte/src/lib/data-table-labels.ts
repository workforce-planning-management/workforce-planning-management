// Every string the data grid shows or announces, from the catalogue, for one table. Pure (the
// translators are passed in) so a test can build the bundle for each locale and ask the grid's
// `missingLabels` whether anything it needs is absent.
import {
  labelsFor,
  type LabelPack,
} from "@lilydesignsystem/svelte-locale-translations";
import type { StringKey } from "#lib/i18n.svelte.js";

export type Say = {
  t: (key: StringKey) => string;
  tf: (key: StringKey, params?: Record<string, unknown>) => string;
};

export type TableOptions = {
  hidable?: boolean;
  searchable?: boolean;
  exportable?: boolean;
  rowCount: number;
};

export function dataTableLabels({ t, tf }: Say, o: TableOptions) {
  return {
    columnVisibility: o.hidable ? t("dataTable.columns") : undefined,
    columnVisibilityOption: (header: string) =>
      tf("dataTable.showColumn", { column: header }),
    search: o.searchable ? t("dataTable.search") : undefined,
    searchField: o.searchable ? t("dataTable.searchField") : undefined,
    searchAllFields: o.searchable ? t("dataTable.searchAllFields") : undefined,
    filterAnnouncement: (matches: number, total: number) =>
      tf("dataTable.filtered", { matches, total }),
    pagination: t("dataTable.pagination"),
    previousPage: t("dataTable.previousPage"),
    nextPage: t("dataTable.nextPage"),
    pageStatus: (page: number, pages: number, rows: number) =>
      tf("dataTable.pageStatus", { page, pages, rows }),
    pageAnnouncement: (page: number, pages: number) =>
      tf("dataTable.pageStatus", { page, pages, rows: o.rowCount }),
    sortAnnouncement: (header: string, direction: string) =>
      tf("dataTable.sorted", {
        header,
        direction:
          direction === "ascending"
            ? t("dataTable.ascending")
            : direction === "descending"
              ? t("dataTable.descending")
              : t("dataTable.unsorted"),
      }),
    export: o.exportable ? t("dataTable.export") : undefined,
    exportCsv: o.exportable ? t("dataTable.exportCsv") : undefined,
    exportTsv: o.exportable ? t("dataTable.exportTsv") : undefined,
    exportExcel: o.exportable ? t("dataTable.exportExcel") : undefined,
    exportAnnouncement: (_format: string, count: number, file: string) =>
      tf("dataTable.exported", { count, file }),
    copy: o.exportable ? t("dataTable.copy") : undefined,
    copyAnnouncement: (count: number) => tf("dataTable.copied", { count }),
    copyFailed: t("dataTable.copyFailed"),
  };
}

// The Lily label packs, one per language, loaded only for the page's locale (a separate chunk each).
// WPM's own catalogue stays authoritative (it is content, editable in the CMS); a pack only fills a
// label the catalogue does not name at all, so a grid feature added later is announced in the
// reader's language instead of left unnamed. Both are unreviewed drafts (content/locales/REVIEW.md).
const packs = import.meta.glob<LabelPack>(
  "../../node_modules/@lilydesignsystem/svelte-locale-translations/label-packs/{ar,bn,cy,de,en,es,fr,hi,id,pt,ru,ur,zh}-001.json",
  { import: "default" },
);

/** The pack that matches a locale's language: `de-de` -> `de-001`. */
export function packLocale(locale: string): string {
  return `${locale.split(/[-_]/)[0]}-001`;
}

export async function loadPack(locale: string): Promise<LabelPack | undefined> {
  const wanted = `/${packLocale(locale)}.json`;
  const key = Object.keys(packs).find((k) => k.endsWith(wanted));
  const load = key ? packs[key] : undefined;
  return load ? await load() : undefined;
}

/**
 * `ours` over the pack's data-grid labels. A key `ours` names, even as undefined (the table does not
 * offer that control), is never filled from the pack.
 */
export function withPackFallback(
  ours: Record<string, unknown>,
  pack: LabelPack | undefined,
): Record<string, unknown> {
  if (!pack) return ours;
  const fallback = labelsFor("data-grid", pack);
  const out: Record<string, unknown> = { ...ours };
  for (const [key, value] of Object.entries(fallback)) {
    if (!(key in ours)) out[key] = value;
  }
  return out;
}
