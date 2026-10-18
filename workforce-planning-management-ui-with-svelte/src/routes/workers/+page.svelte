<script lang="ts">
  import { listWorkers, money } from "#lib/api/wpm.js";
  import { i18n, t, l } from "#lib/i18n.svelte.js";
  import type { Worker } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let workers = $state<Worker[] | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        workers = await listWorkers();
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  // A masked salary renders the translated "Hidden" token and sorts last, never as a fake zero.
  const columns = $derived<DataGridColumn[]>([
    { id: "worker_number", header: t("emp.number"), sortable: true },
    { id: "display_name", header: t("common.name"), sortable: true, pinned: "start", cell: nameCell },
    { id: "job_title", header: t("common.jobTitle"), sortable: true },
    { id: "department", header: t("common.department"), sortable: true },
    { id: "status", header: t("common.status"), sortable: true },
    {
      id: "salary_minor",
      header: t("emp.salary"),
      sortable: true,
      align: "end",
      emptyValue: t("common.masked"),
      // The grid formats before it checks for a missing value, so a masked (null) salary must not reach `money`.
      format: (v, r) =>
        v == null ? "" : money(Number(v), String(r.salary_currency), i18n.locale),
    },
  ]);
</script>

{#snippet nameCell({ row }: { row: DataGridRow })}
  <a href={l(`/workers/${row.pid}`)}>{row.display_name}</a>
{/snippet}

<h1>{t("nav.workers")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if workers === null}
  <p>{t("common.loading")}</p>
{:else}
  <DataTable
    testid="worker-table"
    label={t("nav.workers")}
    {columns}
    rows={workers as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
    pageSize={25}
    searchable
  />
{/if}
