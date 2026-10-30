<!--
  Read-only data table on `@lilydesignsystem/svelte-data-grid` (plain `role="table"` mode): click a
  header to sort (Shift+click adds a second key), text sorts by the page's locale, an unknown value
  (null) sorts last and shows "—", never 0. Pass `cell` snippets on a column for links, chips and buttons.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import DataGrid from "@lilydesignsystem/svelte-data-grid";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";
  import { i18n, t, tf } from "#lib/i18n.svelte.js";
  import {
    dataTableLabels,
    loadPack,
    withPackFallback,
  } from "#lib/data-table-labels.js";
  import type { LabelPack } from "@lilydesignsystem/svelte-locale-translations";
  import { formatDate, formatNumber, isCivilDate } from "#lib/locale-format.js";

  let {
    label,
    columns,
    rows,
    rowId,
    rowAttrs,
    pageSize,
    empty,
    footer,
    exportName,
    searchable,
    testid,
  }: {
    label: string;
    columns: DataGridColumn[];
    rows: DataGridRow[];
    rowId?: (row: DataGridRow, index: number) => string;
    rowAttrs?: (row: DataGridRow) => Record<string, unknown>;
    /** Rows per page; unset shows every row. Pages only appear when the rows overflow one. */
    pageSize?: number;
    /** Shown in one row spanning every column when there are no rows. */
    empty?: Snippet;
    /** Rows for a `<tfoot>` (a totals row); write them as `<tr>`s. */
    footer?: Snippet<[{ columns: DataGridColumn[]; rows: DataGridRow[] }]>;
    /**
     * File name (no extension) that turns on CSV and Excel export of the rows shown. Only for
     * tables that name no one: an export leaves the app's masking and access checks behind.
     */
    exportName?: string;
    /** Adds a text box that filters the rows on any column. */
    searchable?: boolean;
    testid?: string;
  } = $props();

  // A column with no `type`, `format` or `cell` of its own shows a number and an ISO date in the page's locale
  // ("1,234", "30 Oct 2026"; WPM-T375). It still sorts on the value, and an export still carries the value as sent.
  const read = (column: DataGridColumn, row: DataGridRow): unknown =>
    column.accessor ? column.accessor(row) : row[column.id];
  const shown = $derived(
    columns.map((column): DataGridColumn =>
      column.type || column.format || column.cell
        ? column
        : {
            ...column,
            format: (value: unknown) =>
              typeof value === "number"
                ? formatNumber(value)
                : isCivilDate(value)
                  ? formatDate(value)
                  : value === null || value === undefined
                    ? ""
                    : String(value),
            exportValue: column.exportValue ?? ((row: DataGridRow) => read(column, row)),
          },
    ),
  );

  // Column visibility is offered only when a column is marked `hidable: true`.
  let pack = $state<LabelPack>();
  $effect(() => {
    const locale = i18n.locale;
    let current = true;
    loadPack(locale).then((p) => {
      if (current) pack = p;
    });
    return () => {
      current = false;
    };
  });
  const labels = $derived(
    withPackFallback(
      dataTableLabels(
        { t, tf },
        {
          hidable: columns.some((c) => c.hidable),
          searchable,
          exportable: !!exportName,
          rowCount: rows.length,
        },
      ),
      pack,
    ),
  );
</script>

<DataGrid
  {label}
  mode="table"
  columns={shown}
  emptyValue="—"
  {rows}
  {rowId}
  {rowAttrs}
  pageSize={pageSize && rows.length > pageSize ? pageSize : undefined}
  {labels}
  exportFilename={exportName}
  locale={i18n.locale}
  {empty}
  {footer}
  data-testid={testid}
/>
