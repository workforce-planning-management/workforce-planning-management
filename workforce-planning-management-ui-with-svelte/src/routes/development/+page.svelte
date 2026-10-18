<script lang="ts">
  import { expiringTraining, listSuccession, successionGaps } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";
  import CalendarPanel from "#lib/components/CalendarPanel.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";
  import type { SuccessionEntry, TrainingEnrollment } from "#lib/api/types.js";

  let succession = $state<SuccessionEntry[] | null>(null);
  let gaps = $state<SuccessionEntry["plan"][]>([]);
  let expiring = $state<TrainingEnrollment[]>([]);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        const [plans, gapReport, training] = await Promise.all([
          listSuccession(),
          successionGaps(),
          expiringTraining(90),
        ]);
        succession = plans;
        gaps = gapReport.gaps;
        expiring = training.expiring;
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  const successionColumns = $derived<DataGridColumn[]>([
    { id: "role_title", header: t("columns.role"), sortable: true },
    { id: "department", header: t("common.department"), sortable: true },
    { id: "criticality", header: t("columns.criticality"), sortable: true },
    { id: "candidates", header: t("columns.candidates"), cell: candidatesCell },
  ]);
  const expiringColumns = $derived<DataGridColumn[]>([
    { id: "course_ref", header: t("columns.course"), sortable: true },
    { id: "certificate_expires_on", type: "date", header: t("columns.certificateExpires"), sortable: true },
  ]);
</script>

{#snippet candidatesCell({ row }: { row: DataGridRow })}
  {@const candidates = row.candidates as Array<{ pid: string; rank: number; readiness: string }>}
  {#each candidates as candidate (candidate.pid)}
    <span class="chip">#{candidate.rank} {candidate.readiness}</span>
  {/each}
{/snippet}

<h1>{t("nav.development")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if succession === null}
  <p>{t("common.loading")}</p>
{:else}
  <h2>{t("dash.successionGaps")}</h2>
  <ul class="panel" data-testid="gaps">
    {#each gaps as gap (gap.pid)}
      <li><strong>{gap.role_title}</strong> {tf("pages.development.criticality", { department: gap.department, criticality: gap.criticality })}</li>
    {:else}
      <li class="muted">—</li>
    {/each}
  </ul>

  <h2>{t("dev.succession")}</h2>
  <DataTable
    testid="succession"
    label={t("dev.succession")}
    columns={successionColumns}
    rows={succession.map((e) => ({ ...e.plan, candidates: e.candidates })) as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />

  <h2>{t("dev.expiring")}</h2>
  <DataTable
    testid="expiring"
    label={t("dev.expiring")}
    columns={expiringColumns}
    rows={expiring as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
  <CalendarPanel
    testid="expiring-calendar"
    label={t("calendar.expiring")}
    events={expiring.flatMap((e) =>
      e.certificate_expires_on ? [{ id: e.pid, date: e.certificate_expires_on, title: e.course_ref }] : [],
    )}
  />
{/if}
