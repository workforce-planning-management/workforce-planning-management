<!--
  My goals (WPM-R224, WPM-R226): the person's own goals, the goals of the people below them by name, and a form to propose one.
  The person or a manager proposes and the other agrees; HR sees and does not agree. Nothing here is a score, and no pay,
  progression or employment decision is made from a goal.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import GoalCard from "#lib/components/GoalCard.svelte";
  import { myTeamGoals, proposeMyGoal, workerGoals, type GoalView, type TeamGoals } from "#lib/api/wpm.js";
  import type { MyOrganization } from "#lib/api/types.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  const me = $derived(
    ((page.data.organizations ?? []) as MyOrganization[]).find((m) => m.worker_pid !== null)?.worker_pid ?? null,
  );

  let mine = $state<GoalView[] | null>(null);
  let team = $state<TeamGoals["team"]>([]);
  let error = $state<string | null>(null);
  let title = $state("");
  let description = $state("");
  let periodStart = $state("");
  let periodEnd = $state("");
  let measureName = $state("");
  let measureKind = $state<"number" | "milestone">("number");
  let unit = $state("");
  let startValue = $state<number | null>(null);
  let targetValue = $state<number | null>(null);

  const load = guardedLoad(
    () => Promise.all([workerGoals(me ?? ""), myTeamGoals()]),
    ([own, reports]) => {
      mine = own.goals;
      team = reports.team;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    if (me) void load();
  });

  async function propose(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    const number = measureKind === "number";
    try {
      await proposeMyGoal({
        title: title.trim(),
        ...(description.trim() ? { description: description.trim() } : {}),
        period_start: periodStart,
        period_end: periodEnd,
        measures: measureName.trim()
          ? [
              {
                name: measureName.trim(),
                kind: measureKind,
                ...(number && unit.trim() ? { unit: unit.trim() } : {}),
                ...(number && startValue !== null ? { start_value: startValue } : {}),
                ...(number && targetValue !== null ? { target_value: targetValue } : {}),
              },
            ]
          : [],
      });
      title = description = measureName = unit = "";
      startValue = targetValue = null;
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<svelte:head><title>{t("titles.myGoals")}</title></svelte:head>

<h1>{t("goals.title")}</h1>
<p class="muted">{t("goals.hint")}</p>

{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if !me}
  <p class="muted" data-testid="goals-no-worker">{t("kanban.noWorker")}</p>
{:else if mine === null}
  <p>{t("common.loading")}</p>
{:else}
  <h2>{t("goals.mine")}</h2>
  {#each mine as view (view.goal.pid)}
    <GoalCard {view} party="worker" onchange={load} />
  {:else}
    <p class="muted" data-testid="goals-none">{t("goals.none")}</p>
  {/each}

  <section class="panel">
    <h2>{t("goals.propose")}</h2>
    <form onsubmit={propose}>
      <label>{t("goals.goalTitle")} <input bind:value={title} required maxlength="200" data-testid="goal-title" /></label>
      <label>{t("goals.description")} <input bind:value={description} maxlength="2000" /></label>
      <label>{t("goals.periodStart")} <input type="date" bind:value={periodStart} required data-testid="goal-start" /></label>
      <label>{t("goals.periodEnd")} <input type="date" bind:value={periodEnd} required data-testid="goal-end" /></label>
      <label>{t("goals.measureName")} <input bind:value={measureName} maxlength="200" data-testid="measure-name" /></label>
      <label>
        {t("goals.measureKind")}
        <select bind:value={measureKind} data-testid="measure-kind">
          <option value="number">{tv("number")}</option>
          <option value="milestone">{tv("milestone")}</option>
        </select>
      </label>
      {#if measureKind === "number"}
        <label>{t("goals.unit")} <input bind:value={unit} maxlength="200" /></label>
        <label>{t("goals.start")} <input type="number" bind:value={startValue} /></label>
        <label>{t("goals.target")} <input type="number" bind:value={targetValue} data-testid="measure-target" /></label>
      {/if}
      <button type="submit" data-testid="goal-propose">{t("goals.proposeButton")}</button>
    </form>
  </section>

  {#if team.length > 0}
    <h2>{t("goals.team")}</h2>
    {#each team as person (person.worker_pid)}
      <section data-testid="team-person">
        <h3>{person.display_name}</h3>
        {#each person.goals as view (view.goal.pid)}
          <GoalCard {view} party="manager" onchange={load} />
        {/each}
      </section>
    {/each}
  {/if}
{/if}
