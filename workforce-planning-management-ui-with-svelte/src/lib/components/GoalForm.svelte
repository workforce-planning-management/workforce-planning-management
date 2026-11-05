<!--
  The one form for a goal (WPM-R224): proposing your own, proposing one for someone below you, adding a PADR's objective, or changing
  a proposal. It can link a skill or a step on the person's plan, and carry a reference to an objective held elsewhere, which is
  shown as written and never opened (WPM-D166). A goal with several measures shows the first here and keeps the others.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { listDevelopmentPlans, listSkills, type GoalBody, type GoalView } from "#lib/api/wpm.js";
  import { formatDate } from "#lib/locale-format.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let {
    workerPid,
    submit,
    submitLabel,
    initial = null,
    startsOn = "",
    endsOn = "",
  }: {
    workerPid: string;
    submit: (body: GoalBody) => Promise<unknown>;
    submitLabel: string;
    initial?: GoalView | null;
    startsOn?: string;
    endsOn?: string;
  } = $props();

  type Option = { pid: string; label: string };

  // The form starts from these once; its fields are then the reader's, and a caller whose starting values change re-keys the form.
  const seed = untrack(() => ({ goal: initial?.goal ?? null, measures: initial?.measures ?? [], startsOn, endsOn }));
  const first = seed.measures[0] ?? null;
  const others = seed.measures.slice(1);

  let error = $state<string | null>(null);
  let skills = $state<Option[]>([]);
  let steps = $state<Option[]>([]);
  let title = $state(seed.goal?.title ?? "");
  let description = $state(seed.goal?.description ?? "");
  let periodStart = $state(seed.goal?.period_start ?? seed.startsOn);
  let periodEnd = $state(seed.goal?.period_end ?? seed.endsOn);
  let measureName = $state(first?.name ?? "");
  let measureKind = $state<"number" | "milestone">(first?.kind ?? "number");
  let unit = $state(first?.unit ?? "");
  let startValue = $state<number | null>(first?.start_value ?? null);
  let targetValue = $state<number | null>(first?.target_value ?? null);
  let skillPid = $state(seed.goal?.skill_pid ?? "");
  let planItemPid = $state(seed.goal?.plan_item_pid ?? "");
  let alignsTo = $state(seed.goal?.aligns_to ?? "");

  const load = guardedLoad(
    () => Promise.all([listSkills(), listDevelopmentPlans(workerPid)]),
    ([catalogue, plans]) => {
      const named = new Map(catalogue.map((skill) => [skill.pid, skill.name]));
      skills = catalogue.map((skill) => ({ pid: skill.pid, label: skill.name }));
      steps = plans.plans.flatMap((plan) =>
        plan.items.map((item) => ({
          pid: item.pid,
          label: [named.get(item.skill_pid) ?? t("common.unknown"), tv(item.method), item.due_on && formatDate(item.due_on)]
            .filter(Boolean)
            .join(" · "),
        })),
      );
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  async function send(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    const number = measureKind === "number";
    const body: GoalBody = {
      title: title.trim(),
      ...(description.trim() ? { description: description.trim() } : {}),
      period_start: periodStart,
      period_end: periodEnd,
      ...(skillPid ? { skill_pid: skillPid } : {}),
      ...(planItemPid ? { plan_item_pid: planItemPid } : {}),
      ...(alignsTo.trim() ? { aligns_to: alignsTo.trim() } : {}),
      measures: [
        ...(measureName.trim()
          ? [
              {
                name: measureName.trim(),
                kind: measureKind,
                ...(number && unit.trim() ? { unit: unit.trim() } : {}),
                ...(number && startValue !== null ? { start_value: startValue } : {}),
                ...(number && targetValue !== null ? { target_value: targetValue } : {}),
              },
            ]
          : []),
        ...others.map((row) => ({
          name: row.name,
          kind: row.kind,
          ...(row.unit ? { unit: row.unit } : {}),
          ...(row.start_value !== null ? { start_value: row.start_value } : {}),
          ...(row.target_value !== null ? { target_value: row.target_value } : {}),
        })),
      ],
    };
    try {
      await submit(body);
      if (!seed.goal) {
        title = description = measureName = unit = skillPid = planItemPid = alignsTo = "";
        startValue = targetValue = null;
      }
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<form onsubmit={send} data-testid="goal-form">
  {#if error}<ErrorNote testid="goal-form-error" message={error} />{/if}
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
  <details open={Boolean(skillPid || planItemPid || alignsTo)}>
    <summary>{t("goals.links")}</summary>
    <label>
      {t("goals.buildsSkill")}
      <select bind:value={skillPid} data-testid="goal-skill-pick">
        <option value=""></option>
        {#each skills as row (row.pid)}<option value={row.pid}>{row.label}</option>{/each}
      </select>
    </label>
    <label>
      {t("goals.planStep")}
      <select bind:value={planItemPid} data-testid="goal-plan-pick">
        <option value=""></option>
        {#each steps as row (row.pid)}<option value={row.pid}>{row.label}</option>{/each}
      </select>
    </label>
    <label>
      {t("goals.alignsTo")}
      <input bind:value={alignsTo} maxlength="200" data-testid="goal-aligns-input" />
    </label>
    <p class="muted">{t("goals.alignsHint")}</p>
  </details>
  <button type="submit" data-testid="goal-propose">{submitLabel}</button>
</form>

<style>
  form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
    min-width: 0;
    max-width: 100%;
  }
  form input,
  form select {
    max-width: 100%;
    box-sizing: border-box;
  }
</style>
