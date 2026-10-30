<script lang="ts">
  import { formatDateRange } from "#lib/locale-format.js";
  import { page } from "$app/state";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listRunsPage } from "#lib/api/wpm.js";
  import { t, l, tv } from "#lib/i18n.svelte.js";
  import { createPagedLoader } from "#lib/paged.svelte.js";
  import LoadMore from "#lib/components/LoadMore.svelte";
  import type { PayrollRun } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  const loader = createPagedLoader<PayrollRun>(() => listRunsPage);
  const runs = $derived(loader.items);
  const error = $derived(loader.error);

  $effect(() => {
    void loader.load();
  });

  const columns = $derived<DataGridColumn[]>([
    {
      id: "period_start",
      header: t("columns.period"),
      sortable: true,
      rowHeader: true,
      cell: periodCell,
    },
    {
      id: "status",
      header: t("common.status"),
      sortable: true,
      accessor: (r) => tv((r as unknown as PayrollRun).status),
      cell: statusCell,
    },
  ]);
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

{#snippet periodCell({ row }: { row: DataGridRow })}
  {@const run = row as unknown as PayrollRun}
  <a href={l(`/payroll/${run.pid}`)}>{formatDateRange(run.period_start, run.period_end)}</a>
{/snippet}

{#snippet statusCell({ formatted }: { formatted: string })}
  <span class="chip">{formatted}</span>
{/snippet}

<h1>{t("nav.payroll")}</h1>

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if runs === null}
  <p>{t("common.loading")}</p>
{:else}
  <DataTable
    testid="runs"
    label={t("nav.payroll")}
    columns={columns}
    rows={runs as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
    pageSize={25}
  />
  <LoadMore shown={runs.length} total={loader.total} more={loader.more} loading={loader.loadingMore} onmore={() => loader.showMore()} />
  {#if loader.moreError}<ErrorNote testid="more-error" message={loader.moreError} />{/if}
{/if}
