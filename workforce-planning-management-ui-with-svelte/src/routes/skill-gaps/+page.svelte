<!--
  Skill gaps across the workforce (`/skill-gaps`): per skill, how many people
  need it (their current role's requirement or their own target), meet it, are
  below it, or have not declared it — ranked by importance × levels short.
  Counts only: nobody is named, and private aspirations are never included.
-->
<script lang="ts">
  import { formatDecimal, formatNumber } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { trainingDemand, workforceSkillGaps } from "#lib/api/wpm.js";
  import type { TrainingDemand, WorkforceSkillGap } from "#lib/api/types.js";
  import SkillTrainingCatalogue from "#lib/components/SkillTrainingCatalogue.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let department = $state("");
  let departments = $state<string[]>([]);
  let skills = $state<WorkforceSkillGap[] | null>(null);
  let demand = $state<TrainingDemand | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const dept = department;
    let current = true;
    void (async () => {
      try {
        const [result, hours] = await Promise.all([
          workforceSkillGaps({ department: dept || undefined, limit: 50 }),
          trainingDemand({ department: dept || undefined }).catch(() => null),
        ]);
        if (!current) return;
        skills = result.skills;
        demand = hours;
        error = null;
        departments = [
          ...new Set([...departments, ...result.skills.flatMap((s) => s.departments.map((d) => d.department))]),
        ].sort((a, b) => a.localeCompare(b));
      } catch (cause) {
        if (current) error = errorText(cause);
      }
    })();
    return () => {
      current = false;
    };
  });

  const gapColumns = $derived<DataGridColumn[]>([
    { id: "skill", pinned: "start", header: t("nav.skills"), sortable: true, rowHeader: true, accessor: (r) => r.skill ?? r.skill_pid, cell: gapSkillCell },
    { id: "needed_by", header: t("skillgaps.needed"), sortable: true, align: "end" },
    { id: "below", header: t("skillgaps.below"), sortable: true, align: "end" },
    { id: "undeclared", header: t("skillgaps.undeclared"), sortable: true, align: "end" },
    { id: "total_shortfall", header: t("skillgaps.short"), sortable: true, align: "end" },
    { id: "critical_below", header: t("skillgaps.criticalBelow"), sortable: true, align: "end" },
    { id: "score", header: t("skillgaps.score"), sortable: true, align: "end", cell: scoreCell },
    {
      id: "departments",
      header: t("skillgaps.departments"),
      accessor: (r) => (r as unknown as WorkforceSkillGap).departments.map((d) => `${d.department} (${d.below})`).join(", "),
      cell: gapDepartmentsCell,
    },
  ]);
  const demandColumns = $derived<DataGridColumn[]>([
    { id: "skill", header: t("nav.skills"), sortable: true, rowHeader: true, accessor: (r) => r.skill ?? r.skill_pid },
    { id: "people", header: t("training.people"), sortable: true, align: "end" },
    { id: "hours", header: t("training.hours"), sortable: true, align: "end" },
    { id: "people_on_estimate", header: t("training.basisEstimate"), sortable: true, align: "end", cell: estimateCell },
  ]);
</script>

<svelte:head><title>{tf("pages.skillGaps.page_title", { skillGaps: t("nav.skillGaps") })}</title></svelte:head>

{#snippet gapSkillCell({ row }: { row: DataGridRow })}
  <strong>{row.skill ?? row.skill_pid}</strong> <span class="muted">{row.category ?? ""}</span>
{/snippet}

{#snippet scoreCell({ row }: { row: DataGridRow })}
  <strong>{row.score}</strong>
{/snippet}

{#snippet gapDepartmentsCell({ value }: { value: unknown })}
  <span class="muted">{value}</span>
{/snippet}

{#snippet estimateCell({ row }: { row: DataGridRow })}
  <span class="muted">{row.people_on_estimate}</span>
{/snippet}

<h1>{t("skillgaps.workforceTitle")}</h1>
<p class="muted">{t("skillgaps.hint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

<p>
  <select bind:value={department} data-testid="gaps-department" aria-label={t("common.department")}>
    <option value="">{t("directory.allDepartments")}</option>
    {#each departments as d (d)}<option value={d}>{d}</option>{/each}
  </select>
</p>

{#if skills === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else if skills.length === 0}
  <p class="muted" data-testid="gaps-none">{t("skillgaps.none")}</p>
{:else}
  <DataTable
    testid="workforce-gaps"
    label={t("skillgaps.workforceTitle")}
    columns={gapColumns}
    rows={skills as unknown as DataGridRow[]}
    rowId={(row) => String(row.skill_pid)}
    pageSize={25}
  />
{/if}

{#if demand}
  <h2>{t("training.demandTitle")}</h2>
  <p class="muted">{t("training.demandHint")}</p>
  <p data-testid="demand-total">
    <strong>{formatNumber(demand.total_hours)}</strong> {t("training.hours")} ·
    {formatNumber(demand.people_with_gaps)} {t("training.people")}
    {#if demand.average_hours_per_person !== null}
      · {t("training.avg")}: {formatDecimal(demand.average_hours_per_person, 1)}
    {/if}
  </p>
  {#if demand.skills.length > 0}
    <DataTable
      testid="demand-skills"
      label={t("training.demandTitle")}
      columns={demandColumns}
      rows={demand.skills as unknown as DataGridRow[]}
      rowId={(row) => String(row.skill_pid)}
    />
  {/if}
  {#if demand.departments.length > 0}
    <p class="muted" data-testid="demand-departments">
      {demand.departments.map((d) => `${d.department}: ${d.hours} (${d.people})`).join(" · ")}
    </p>
  {/if}
{/if}

<SkillTrainingCatalogue />
