<!--
  Skill gaps across the workforce (`/skill-gaps`): per skill, how many people
  need it (their current role's requirement or their own target), meet it, are
  below it, or have not declared it — ranked by importance × levels short.
  Counts only: nobody is named, and private aspirations are never included.
-->
<script lang="ts">
  import { trainingDemand, workforceSkillGaps } from "#lib/api/wpm.js";
  import type { TrainingDemand, WorkforceSkillGap } from "#lib/api/types.js";
  import SkillTrainingCatalogue from "#lib/components/SkillTrainingCatalogue.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";

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
        if (current) error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
    return () => {
      current = false;
    };
  });
</script>

<svelte:head><title>{tf("pages.skillGaps.page_title", { skillGaps: t("nav.skillGaps") })}</title></svelte:head>

<h1>{t("skillgaps.workforceTitle")}</h1>
<p class="muted">{t("skillgaps.hint")}</p>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

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
  <table data-testid="workforce-gaps">
    <thead>
      <tr>
        <th>{t("nav.skills")}</th>
        <th>{t("skillgaps.needed")}</th>
        <th>{t("skillgaps.below")}</th>
        <th>{t("skillgaps.undeclared")}</th>
        <th>{t("skillgaps.short")}</th>
        <th>{t("skillgaps.criticalBelow")}</th>
        <th>{t("skillgaps.score")}</th>
        <th>{t("skillgaps.departments")}</th>
      </tr>
    </thead>
    <tbody>
      {#each skills as s (s.skill_pid)}
        <tr>
          <td><strong>{s.skill ?? s.skill_pid}</strong> <span class="muted">{s.category ?? ""}</span></td>
          <td>{s.needed_by}</td>
          <td>{s.below}</td>
          <td>{s.undeclared}</td>
          <td>{s.total_shortfall}</td>
          <td>{s.critical_below}</td>
          <td><strong>{s.score}</strong></td>
          <td class="muted">{s.departments.map((d) => `${d.department} (${d.below})`).join(", ")}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

{#if demand}
  <h2>{t("training.demandTitle")}</h2>
  <p class="muted">{t("training.demandHint")}</p>
  <p data-testid="demand-total">
    <strong>{demand.total_hours}</strong> {t("training.hours")} ·
    {demand.people_with_gaps} {t("training.people")}
    {#if demand.average_hours_per_person !== null}
      · {t("training.avg")}: {demand.average_hours_per_person.toFixed(1)}
    {/if}
  </p>
  {#if demand.skills.length > 0}
    <table data-testid="demand-skills">
      <thead>
        <tr>
          <th>{t("nav.skills")}</th>
          <th>{t("training.people")}</th>
          <th>{t("training.hours")}</th>
          <th>{t("training.basisEstimate")}</th>
        </tr>
      </thead>
      <tbody>
        {#each demand.skills as s (s.skill_pid)}
          <tr>
            <td>{s.skill ?? s.skill_pid}</td>
            <td>{s.people}</td>
            <td>{s.hours}</td>
            <td class="muted">{s.people_on_estimate}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
  {#if demand.departments.length > 0}
    <p class="muted" data-testid="demand-departments">
      {demand.departments.map((d) => `${d.department}: ${d.hours} (${d.people})`).join(" · ")}
    </p>
  {/if}
{/if}

<SkillTrainingCatalogue />
