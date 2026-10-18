<script lang="ts">
  // Privacy operations (WPM-R30): the retention report — what the
  // next sweep would remove, per table, under the floored horizon —
  // and the sweep itself (destructive; admin-only under enforcement).
  // Erasure lives on the worker profile (it is per-person).
  import { retentionReport, retentionSweep } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  type Report = Awaited<ReturnType<typeof retentionReport>>;
  type SweepResult = Awaited<ReturnType<typeof retentionSweep>>;

  let report = $state<Report | null>(null);
  let sweepResult = $state<SweepResult | null>(null);
  let error = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  async function load() {
    try {
      report = await retentionReport();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  $effect(() => {
    void load();
  });

  async function sweep() {
    actionError = null;
    try {
      sweepResult = await retentionSweep();
      await load();
    } catch (cause) {
      actionError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  const tableColumns: DataGridColumn[] = [
    { id: "table", header: t("columns.tableName"), sortable: true },
    { id: "count", header: t("columns.rowCount"), sortable: true, align: "end" },
  ];
</script>

<h1>{t("nav.privacy")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if report === null}
  <p>{t("common.loading")}</p>
{:else}
  <h2>{t("pr.retention")}</h2>
  <div class="panel" data-testid="retention-report">
    <p>{t("pr.horizon")}: <strong>{report.horizon_days}</strong></p>
    <h3>{t("pr.pastHorizon")}</h3>
    {#if Object.keys(report.soft_deleted_past_horizon).length === 0}
      <p class="muted">—</p>
    {:else}
      <DataTable
        label={t("pr.pastHorizon")}
        columns={tableColumns}
        rows={Object.entries(report.soft_deleted_past_horizon).map(([table, count]) => ({ table, count })) as unknown as DataGridRow[]}
        rowId={(row) => String(row.table)}
      />
    {/if}
    <p>{t("pr.candidates")}: <strong>{report.expired_consent_candidates}</strong></p>
    <p class="muted">{report.derivation}</p>
    <button onclick={() => void sweep()} data-testid="run-sweep">{t("pr.sweep")}</button>
    {#if actionError}
      <p class="error" data-testid="action-error">{actionError}</p>
    {/if}
    {#if sweepResult}
      <p data-testid="sweep-result">
        {sweepResult.rows_deleted} {t("pr.swept")} ·
        {t("pr.candidates")}: {sweepResult.candidates_scrubbed}
      </p>
    {/if}
  </div>
{/if}
