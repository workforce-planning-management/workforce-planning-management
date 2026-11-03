<!--
  Statutory and mandatory learning (`/mandatory-learning`, WPM-R170, WPM-T315): HR's roll-up of each requirement for an
  organization: counts per status and the rate in date among those counted. Unknown (a record does not say which group
  someone is in) is counted apart, in neither side of the rate. A group under the floor shows nothing, and a rate with
  nothing to divide by is "—", never 0. It names no one: the person's own status is on their own page.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { formatPercent } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import LearningAdmin from "#lib/components/LearningAdmin.svelte";
  import { learningRollup } from "#lib/api/wpm.js";
  import type {
    LearningRollupGroup,
    LearningRollupRow,
  } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let rows = $state<LearningRollupRow[] | null>(null);
  // "" is one row per requirement; otherwise one row per requirement and group, each group meeting the floor on its own.
  let by = $state<"" | "department" | "staff_group">("");
  let error = $state<string | null>(null);

  // Bumped after HR changes something, so the roll-up is read again.
  let reads = $state(0);

  $effect(() => {
    const chosen = organization;
    const grouped = by;
    void reads;
    error = null;
    if (!chosen) {
      rows = null;
      return;
    }
    let stale = false;
    void (async () => {
      try {
        const found = await learningRollup(chosen, undefined, grouped || undefined);
        if (!stale) rows = found.requirements;
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  const count = (
    n: number | undefined,
    row: { hidden: boolean },
  ) =>
    row.hidden || n === undefined ? "—" : String(n);

  const line = (
    id: string,
    r: LearningRollupRow | LearningRollupGroup,
    subject: string,
    basis: string,
    group: string,
  ) => ({
    row_id: id,
    subject,
    basis,
    group,
    compliant: count(r.compliant, r),
    due_soon: count(r.due_soon, r),
    expired: count(r.expired, r),
    never_completed: count(r.never_completed, r),
    unknown: count(r.unknown, r),
    rate: r.hidden
      ? t("mandatoryLearning.hidden")
      : r.rate === null || r.rate === undefined
        ? "—"
        : formatPercent(r.rate),
  });

  const table = $derived(
    (rows ?? []).flatMap((r) => {
      const basis = t(`mandatoryLearning.${r.basis}`);
      if (!by) return [line(r.requirement_pid, r, r.subject, basis, "")];
      return (r.groups ?? []).map((g) =>
        line(
          `${r.requirement_pid}:${g.group ?? ""}`,
          g,
          r.subject,
          basis,
          g.group ?? t("mandatoryLearning.groupNotRecorded"),
        ),
      );
    }),
  );

  const columns = $derived<DataGridColumn[]>([
    { id: "subject", header: t("mandatoryLearning.subject"), sortable: true, rowHeader: true },
    { id: "basis", header: t("mandatoryLearning.basis"), sortable: true },
    ...(by
      ? [{ id: "group", header: t(`mandatoryLearning.by_${by}`), sortable: true }]
      : []),
    { id: "compliant", header: t("mandatoryLearning.compliant"), align: "end" },
    { id: "due_soon", header: t("mandatoryLearning.due_soon"), align: "end" },
    { id: "expired", header: t("mandatoryLearning.expired"), align: "end" },
    { id: "never_completed", header: t("mandatoryLearning.never_completed"), align: "end" },
    { id: "unknown", header: t("mandatoryLearning.unknown"), align: "end" },
    { id: "rate", header: t("mandatoryLearning.rollupRate"), align: "end" },
  ]);
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

<h1>{t("mandatoryLearning.title")}</h1>
<p class="muted">{t("mandatoryLearning.hint")}</p>
<label>
  {t("mandatoryLearning.organization")}
  <select bind:value={organization} data-testid="learning-organization">
    <option value=""></option>
    {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
  </select>
</label>
<label>
  {t("mandatoryLearning.by")}
  <select bind:value={by} data-testid="learning-by">
    <option value="">{t("mandatoryLearning.by_none")}</option>
    <option value="department">{t("mandatoryLearning.by_department")}</option>
    <option value="staff_group">{t("mandatoryLearning.by_staff_group")}</option>
  </select>
</label>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}
{#if rows}
  {#if rows.length === 0}
    <p data-testid="learning-none">{t("mandatoryLearning.none")}</p>
  {:else}
    <DataTable
      testid="learning-rollup"
      label={t("mandatoryLearning.title")}
      columns={columns}
      rows={table as unknown as DataGridRow[]}
      rowId={(row) => String(row.row_id)}
    />
  {/if}
{/if}
{#if organization}
  <LearningAdmin {organization} changed={() => (reads += 1)} />
{/if}
