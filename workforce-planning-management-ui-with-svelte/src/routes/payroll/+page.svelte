<script lang="ts">
  import { page } from "$app/state";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listRuns } from "#lib/api/wpm.js";
  import { t, l, tv } from "#lib/i18n.svelte.js";
  import { createLoader } from "#lib/loader.svelte.js";
  import type { PayrollRun } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  const loader = createLoader(() => listRuns());
  const runs = $derived(loader.data);
  const error = $derived(loader.error);

  $effect(() => {
    void loader.load();
  });

  const columns = $derived<DataGridColumn[]>([
    {
      id: "period_start",
      header: "",
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
  <a href={l(`/payroll/${run.pid}`)}>{run.period_start} → {run.period_end}</a>
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
{/if}
