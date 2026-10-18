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
  import { dataTableLabels } from "#lib/data-table-labels.js";

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

  // Column visibility is offered only when a column is marked `hidable: true`.
  const labels = $derived(
    dataTableLabels(
      { t, tf },
      {
        hidable: columns.some((c) => c.hidable),
        searchable,
        exportable: !!exportName,
        rowCount: rows.length,
      },
    ),
  );
</script>

<DataGrid
  {label}
  mode="table"
  {columns}
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
