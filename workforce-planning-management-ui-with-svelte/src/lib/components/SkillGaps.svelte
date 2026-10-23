<!--
  A person's skill gaps: what they need (their current role's requirements,
  their own targets, and — for them alone — aspirations) against what they
  have declared, ranked by importance × levels short. A skill they have not
  declared is shown as unknown ("Not declared"), never as a gap of some size.
  A development conversation starter, not a selection decision.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { workerSkillGaps, workerTrainingPlan } from "#lib/api/wpm.js";
  import type { SkillGap, TrainingPlan } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let { workerPid }: { workerPid: string } = $props();

  let gaps = $state<SkillGap[] | null>(null);
  let error = $state<string | null>(null);
  let plan = $state<TrainingPlan | null>(null);
  // Blank = the service's default pace for this person's FTE.
  let weekly = $state<number | null>(null);

  $effect(() => {
    void workerPid;
    void (async () => {
      try {
        gaps = (await workerSkillGaps(workerPid)).gaps;
        error = null;
      } catch (cause) {
        error = errorText(cause);
      }
    })();
  });

  // The plan reloads when the pace changes; a late answer for an older pace is dropped.
  $effect(() => {
    void workerPid;
    const pace = weekly;
    let current = true;
    void (async () => {
      try {
        const next = await workerTrainingPlan(workerPid, pace ? { weeklyHours: pace } : undefined);
        if (current) plan = next;
      } catch {
        if (current) plan = null;
      }
    })();
    return () => {
      current = false;
    };
  });

  const basis = (b: string) =>
    b === "courses"
      ? t("training.basisCourses")
      : b === "mixed"
        ? t("training.basisMixed")
        : t("training.basisEstimate");

  type PlanItem = TrainingPlan["plan"][number];
  const planItem = (row: DataGridRow) => row as unknown as PlanItem;
  const planColumns = $derived<DataGridColumn[]>([
    {
      id: "skill",
      header: t("nav.skills"),
      sortable: true,
      accessor: (r) => planItem(r).skill ?? planItem(r).skill_pid,
      cell: planSkill,
    },
    {
      id: "hours",
      header: t("training.hours"),
      sortable: true,
      accessor: (r) => planItem(r).recommendation.hours,
    },
    {
      id: "basis",
      header: t("training.basis"),
      accessor: (r) => planItem(r).recommendation.basis,
      cell: planBasis,
    },
    {
      id: "courses",
      header: t("training.courses"),
      accessor: (r) =>
        planItem(r)
          .recommendation.courses.map((c) => `${c.title} (${c.hours})`)
          .join(", ") || null,
    },
    { id: "starts_on", header: t("training.dates"), sortable: true, cell: planDates },
  ]);

  const source = (s: SkillGap["sources"][number]) =>
    s === "role"
      ? t("skillgaps.sourceRole")
      : s === "target"
        ? t("skillgaps.sourceTarget")
        : t("skillgaps.sourceAspiration");

  const importance = (i: string) =>
    i === "critical"
      ? t("skillgaps.impCritical")
      : i === "important"
        ? t("skillgaps.impImportant")
        : t("skillgaps.impUseful");

  const gapColumns = $derived<DataGridColumn[]>([
    {
      id: "skill",
      header: t("nav.skills"),
      sortable: true,
      accessor: (r) => {
        const g = r as unknown as SkillGap;
        return g.skill ?? g.skill_pid;
      },
      cell: skillCell,
    },
    { id: "required", header: t("skillgaps.required"), sortable: true, align: "end" },
    { id: "declared", header: t("skillgaps.declared"), sortable: true, align: "end" },
    { id: "shortfall", header: t("skillgaps.short"), sortable: true, align: "end", cell: shortCell },
    {
      id: "priority",
      header: t("skillgaps.priority"),
      sortable: true,
      align: "end",
      accessor: (r) => {
        const g = r as unknown as SkillGap;
        return g.status === "below" ? g.priority : null;
      },
    },
    {
      id: "sources",
      header: t("skillgaps.sources"),
      sortable: true,
      accessor: (r) => (r as unknown as SkillGap).sources.map(source).join(", "),
    },
  ]);
</script>

{#snippet planSkill({ row }: { row: DataGridRow })}
  <strong>{planItem(row).skill ?? planItem(row).skill_pid}</strong>
{/snippet}

{#snippet planBasis({ value }: { value: unknown })}
  <span class="chip">{basis(String(value))}</span>
{/snippet}

{#snippet planDates({ row }: { row: DataGridRow })}
  {planItem(row).starts_on} → {planItem(row).ends_on}
  <span class="muted">({planItem(row).weeks} {t("training.weeks")})</span>
{/snippet}

{#snippet skillCell({ row }: { row: DataGridRow })}
  {@const g = row as unknown as SkillGap}
  <strong>{g.skill ?? g.skill_pid}</strong>
  <span class="muted">· {importance(g.importance)}</span>
{/snippet}

{#snippet shortCell({ row }: { row: DataGridRow })}
  {@const g = row as unknown as SkillGap}
  {#if g.shortfall !== null}{g.shortfall}{:else}<span class="chip" title={t("skillgaps.unknownHint")}>{t("skillgaps.undeclared")}</span>{/if}
{/snippet}

<section class="panel" data-testid="skill-gaps">
  <h2>{t("skillgaps.title")}</h2>
  {#if error}
    <ErrorNote testid="error" message={error} />
  {:else if gaps === null}
    <p>{t("common.loading")}</p>
  {:else if gaps.length === 0}
    <p class="muted" data-testid="skill-gaps-none">{t("skillgaps.none")}</p>
  {:else}
    <DataTable
      testid="skill-gaps-table"
      label={t("skillgaps.title")}
      columns={gapColumns}
      rows={gaps as unknown as DataGridRow[]}
      rowId={(row) => String(row.skill_pid)}
      rowAttrs={(row) => ({ "data-status": row.status })}
    />
  {/if}

  {#if plan}
    <h3>{t("training.title")}</h3>
    <p class="muted">{t("training.hint")}</p>
    <label>
      {t("training.weekly")}
      <input
        type="number"
        min="1"
        max="40"
        placeholder={String(plan.weekly_hours)}
        bind:value={weekly}
        data-testid="plan-weekly"
      />
    </label>
    {#if plan.plan.length === 0}
      <p class="muted" data-testid="plan-none">{t("training.noPlan")}</p>
    {:else}
      <DataTable
        testid="training-plan"
        label={t("training.title")}
        columns={planColumns}
        rows={plan.plan as unknown as DataGridRow[]}
        rowId={(row) => String(row.skill_pid)}
      >
        {#snippet footer()}
          <tr data-testid="plan-total">
            <th>{t("training.total")}</th>
            <th>{plan?.total_hours}</th>
            <th></th>
            <th></th>
            <th>{t("training.finish")} {plan?.finish_on} ({plan?.total_weeks} {t("training.weeks")})</th>
          </tr>
        {/snippet}
      </DataTable>
    {/if}
    {#if plan.assess_first.length > 0}
      <p data-testid="plan-assess">
        <strong>{t("training.assessFirst")}:</strong>
        {plan.assess_first.map((a) => a.skill ?? a.skill_pid).join(", ")}
      </p>
    {/if}
  {/if}
</section>
