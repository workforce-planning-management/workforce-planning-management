// Every string the data grid shows or announces, from the catalogue, for one table. Pure (the
// translators are passed in) so a test can build the bundle for each locale and ask the grid's
// `missingLabels` whether anything it needs is absent.
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
