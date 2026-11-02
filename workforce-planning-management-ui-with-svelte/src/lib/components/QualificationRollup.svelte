<!--
  The qualification gap by role (WPM-R171, WPM-T435): for each role people currently hold, per qualification it asks for, how
  many are met, declared only, expired or missing, and the share met. A group under the floor shows nothing ("too few people"),
  never a zero. It names no one; a person with no role recorded is not counted.
-->
<script lang="ts">
  import { formatPercent } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import { qualificationRollup } from "#lib/api/wpm.js";
  import type { QualificationRollupRow } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let { organization, reads = 0 }: { organization: string; reads?: number } =
    $props();

  let rows = $state<QualificationRollupRow[] | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const chosen = organization;
    void reads;
    error = null;
    let stale = false;
    void (async () => {
      try {
        const found = await qualificationRollup(chosen);
        if (!stale) rows = found.requirements;
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  const count = (n: number | undefined, row: QualificationRollupRow) =>
    row.hidden || n === undefined ? "—" : String(n);

  const table = $derived(
    (rows ?? []).map((r) => ({
      id: `${r.role_profile_pid}/${r.title}`,
      role: r.job_title,
      title: r.title,
      met: count(r.met, r),
      declared_only: count(r.declared_only, r),
      expired: count(r.expired, r),
      missing: count(r.missing, r),
      rate: r.hidden
        ? t("mandatoryLearning.hidden")
        : r.rate === undefined
          ? "—"
          : formatPercent(r.rate),
    })),
  );

  const columns = $derived<DataGridColumn[]>([
    { id: "role", header: t("qualifications.rollupRole"), sortable: true, rowHeader: true },
    { id: "title", header: t("qualifications.name"), sortable: true },
    { id: "met", header: t("qualifications.standingMet"), align: "end" },
    { id: "declared_only", header: t("qualifications.standingDeclared"), align: "end" },
    { id: "expired", header: t("qualifications.standingExpired"), align: "end" },
    { id: "missing", header: t("qualifications.standingMissing"), align: "end" },
    { id: "rate", header: t("qualifications.rollupRate"), align: "end" },
  ]);
</script>

<section class="panel" data-testid="qualification-rollup">
  <h2>{t("qualifications.rollupTitle")}</h2>
  <p class="muted">{t("qualifications.rollupHint")}</p>
  {#if error}<ErrorNote testid="rollup-error" message={error} />{/if}
  {#if rows}
    {#if rows.length === 0}
      <p data-testid="rollup-none">{t("qualifications.rollupNone")}</p>
    {:else}
      <DataTable
        testid="qualification-rollup-table"
        label={t("qualifications.rollupTitle")}
        columns={columns}
        rows={table as unknown as DataGridRow[]}
        rowId={(row) => String(row.id)}
      />
    {/if}
  {/if}
</section>
