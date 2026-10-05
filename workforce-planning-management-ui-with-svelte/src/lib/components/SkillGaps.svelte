<!--
  A person's skill gaps: what they need (their current role's requirements,
  their own targets, and — for them alone — aspirations) against what they
  have declared, ranked by importance × levels short. A skill they have not
  declared is shown as unknown ("Not declared"), never as a gap of some size.
  A development conversation starter, not a selection decision.
-->
<script lang="ts">
  import { workerSkillGaps, workerTrainingPlan } from "#lib/api/wpm.js";
  import type { SkillGap, TrainingPlan } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

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
        error = cause instanceof Error ? cause.message : String(cause);
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
</script>

<section class="panel" data-testid="skill-gaps">
  <h2>{t("skillgaps.title")}</h2>
  {#if error}
    <p class="error" data-testid="error">{error}</p>
  {:else if gaps === null}
    <p>{t("common.loading")}</p>
  {:else if gaps.length === 0}
    <p class="muted" data-testid="skill-gaps-none">{t("skillgaps.none")}</p>
  {:else}
    <table data-testid="skill-gaps-table">
      <thead>
        <tr>
          <th>{t("nav.skills")}</th>
          <th>{t("skillgaps.required")}</th>
          <th>{t("skillgaps.declared")}</th>
          <th>{t("skillgaps.short")}</th>
          <th>{t("skillgaps.priority")}</th>
          <th>{t("skillgaps.sources")}</th>
        </tr>
      </thead>
      <tbody>
        {#each gaps as g (g.skill_pid)}
          <tr data-status={g.status}>
            <td>
              <strong>{g.skill ?? g.skill_pid}</strong>
              <span class="muted">· {importance(g.importance)}</span>
            </td>
            <td>{g.required}</td>
            <td>{g.declared ?? "—"}</td>
            <td>
              {#if g.shortfall !== null}{g.shortfall}{:else}<span class="chip" title={t("skillgaps.unknownHint")}>{t("skillgaps.undeclared")}</span>{/if}
            </td>
            <td>{g.status === "below" ? g.priority : "—"}</td>
            <td>{g.sources.map(source).join(", ")}</td>
          </tr>
        {/each}
      </tbody>
    </table>
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
      <table data-testid="training-plan">
        <thead>
          <tr>
            <th>{t("nav.skills")}</th>
            <th>{t("training.hours")}</th>
            <th>{t("training.basis")}</th>
            <th>{t("training.courses")}</th>
            <th>{t("training.dates")}</th>
          </tr>
        </thead>
        <tbody>
          {#each plan.plan as item (item.skill_pid)}
            <tr>
              <td><strong>{item.skill ?? item.skill_pid}</strong></td>
              <td>{item.recommendation.hours}</td>
              <td><span class="chip">{basis(item.recommendation.basis)}</span></td>
              <td>{item.recommendation.courses.map((c) => `${c.title} (${c.hours})`).join(", ") || "—"}</td>
              <td>{item.starts_on} → {item.ends_on} <span class="muted">({item.weeks} {t("training.weeks")})</span></td>
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr data-testid="plan-total">
            <th>{t("training.total")}</th>
            <th>{plan.total_hours}</th>
            <th></th>
            <th></th>
            <th>{t("training.finish")} {plan.finish_on} ({plan.total_weeks} {t("training.weeks")})</th>
          </tr>
        </tfoot>
      </table>
    {/if}
    {#if plan.assess_first.length > 0}
      <p data-testid="plan-assess">
        <strong>{t("training.assessFirst")}:</strong>
        {plan.assess_first.map((a) => a.skill ?? a.skill_pid).join(", ")}
      </p>
    {/if}
  {/if}
</section>
