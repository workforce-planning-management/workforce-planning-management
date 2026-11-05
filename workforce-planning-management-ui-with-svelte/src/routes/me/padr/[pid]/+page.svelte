<!--
  One personal development review (WPM-R172): its latest version, its objectives and its two signatures. The reviewer and the person
  write it until either signs; the first signature freezes the content and the objectives, which are the person's goals in the
  period (WPM-R225). Whether a wellbeing conversation was offered is a yes or no, and nothing here holds what was said. A correction
  is a new version, with the signed one kept.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import GoalCard from "#lib/components/GoalCard.svelte";
  import GoalForm from "#lib/components/GoalForm.svelte";
  import PadrVersionView from "#lib/components/PadrVersionView.svelte";
  import {
    commentOnPadr,
    correctPadr,
    editPadr,
    getPadr,
    listDevelopmentPlans,
    listSkills,
    proposeGoalFor,
    proposeMyGoal,
    signPadr,
    type GoalBody,
    type PadrNeed,
    type PadrView,
  } from "#lib/api/wpm.js";
  import { formatDate } from "#lib/locale-format.js";
  import { l, t, tf } from "#lib/i18n.svelte.js";

  const pid = $derived(page.params.pid ?? "");

  let view = $state<PadrView | null>(null);
  let steps = $state<Array<{ pid: string; label: string }>>([]);
  let error = $state<string | null>(null);
  let periodStart = $state("");
  let periodEnd = $state("");
  let summary = $state("");
  let wellbeing = $state<"" | "yes" | "no">("");
  let needs = $state<PadrNeed[]>([]);
  let comment = $state("");

  const latest = $derived(view?.versions.at(-1) ?? null);
  // An objective starts with this version's period, as a value for the form's date fields.
  const objectiveFrom = $derived(latest?.version.period_start ?? "");
  const objectiveTo = $derived(latest?.version.period_end ?? "");
  const earlier = $derived((view?.versions.slice(0, -1) ?? []).toReversed());
  const writes = $derived(view?.role === "reviewer" || view?.role === "reviewee");
  const editable = $derived(writes && latest !== null && !latest.frozen);
  const mySignature = $derived(
    view?.role === "reviewer"
      ? latest?.version.reviewer_signed_on
      : view?.role === "reviewee"
        ? latest?.version.reviewee_signed_on
        : null,
  );
  const stepLabels = $derived(new Map(steps.map((step) => [step.pid, step.label])));

  const load = guardedLoad(
    async () => {
      const found = await getPadr(pid);
      const [catalogue, plans] = await Promise.all([listSkills(), listDevelopmentPlans(found.worker_pid)]);
      return { found, catalogue, plans };
    },
    ({ found, catalogue, plans }) => {
      view = found;
      const named = new Map(catalogue.map((skill) => [skill.pid, skill.name]));
      steps = plans.plans.flatMap((plan) =>
        plan.items.map((item) => ({ pid: item.pid, label: named.get(item.skill_pid) ?? t("common.unknown") })),
      );
      const version = found.versions.at(-1)?.version;
      if (version) {
        periodStart = version.period_start;
        periodEnd = version.period_end;
        summary = version.values_summary ?? "";
        wellbeing = version.wellbeing_offered === null ? "" : version.wellbeing_offered ? "yes" : "no";
        needs = version.development_needs.map((row) => ({ ...row }));
        comment = version.reviewee_comment ?? "";
      }
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void pid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  function save(event: SubmitEvent) {
    event.preventDefault();
    void run(() =>
      editPadr(pid, {
        period_start: periodStart,
        period_end: periodEnd,
        ...(summary.trim() ? { values_summary: summary.trim() } : {}),
        wellbeing_offered: wellbeing === "" ? null : wellbeing === "yes",
        development_needs: needs
          .filter((row) => row.need.trim())
          .map((row) => ({ need: row.need.trim(), plan_item_pid: row.plan_item_pid || null })),
      }),
    );
  }

  function saveComment(event: SubmitEvent) {
    event.preventDefault();
    void run(() => commentOnPadr(pid, comment));
  }

  async function addObjective(body: GoalBody) {
    if (view?.role === "reviewee") await proposeMyGoal(body);
    else if (view) await proposeGoalFor(view.worker_pid, body);
    await load();
  }
</script>

<svelte:head><title>{t("titles.padr")}</title></svelte:head>

<p><a href={l("/me/padr")}>{t("padr.back")}</a></p>
<h1>{t("padr.title")}{#if view}: {view.display_name}{/if}</h1>

{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if !view || !latest}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else}
  <p class="muted" data-testid="padr-version">{tf("padr.version", { version: latest.version.version })}</p>
  {#if view.completed_on}
    <p data-testid="padr-completed">{t("padr.completedOn")}: {formatDate(view.completed_on)}</p>
  {/if}

  {#if editable}
    <form class="panel" onsubmit={save} data-testid="padr-form">
      <label>{t("padr.periodStart")} <input type="date" bind:value={periodStart} required /></label>
      <label>{t("padr.periodEnd")} <input type="date" bind:value={periodEnd} required /></label>
      <label>
        {t("padr.values")}
        <textarea bind:value={summary} maxlength="2000" rows="4" data-testid="padr-values"></textarea>
      </label>
      <label>
        {t("padr.wellbeing")}
        <select bind:value={wellbeing} data-testid="padr-wellbeing">
          <option value=""></option>
          <option value="yes">{t("padr.yes")}</option>
          <option value="no">{t("padr.no")}</option>
        </select>
      </label>
      <p class="muted">{t("padr.wellbeingHint")}</p>
      <h2>{t("padr.needs")}</h2>
      {#each needs as row, index (index)}
        <div data-testid="padr-need">
          <label>{t("padr.need")} <input bind:value={row.need} maxlength="500" data-testid="padr-need-text" /></label>
          <label>
            {t("goals.planStep")}
            <select bind:value={row.plan_item_pid}>
              <option value={null}></option>
              {#each steps as step (step.pid)}<option value={step.pid}>{step.label}</option>{/each}
            </select>
          </label>
          <button type="button" onclick={() => needs.splice(index, 1)}>{t("common.remove")}</button>
        </div>
      {/each}
      <button type="button" data-testid="padr-add-need" onclick={() => needs.push({ need: "", plan_item_pid: null })}>
        {t("padr.addNeed")}
      </button>
      <button type="submit" data-testid="padr-save">{t("common.save")}</button>
    </form>

    <h2>{t("padr.objectives")}</h2>
    <p class="muted">{t("padr.objectivesHint")}</p>
    {#each view.objectives as objective (objective.goal.pid)}
      <GoalCard
        view={objective}
        party={view.role === "reviewee" ? "worker" : "manager"}
        onchange={load}
      />
    {:else}
      <p class="muted" data-testid="padr-no-objectives">{t("padr.noObjectives")}</p>
    {/each}
    <details class="panel" data-testid="padr-add-objective">
      <summary>{t("padr.addObjective")}</summary>
      {#key `${objectiveFrom}/${objectiveTo}`}
        <GoalForm
          workerPid={view.worker_pid}
          startsOn={objectiveFrom}
          endsOn={objectiveTo}
          submitLabel={t("goals.proposeButton")}
          submit={addObjective}
        />
      {/key}
    </details>
  {:else}
    <PadrVersionView version={latest} {stepLabels} liveObjectives={view.objectives} onchange={load} />
  {/if}

  <section class="panel" data-testid="padr-signatures">
    <h2>{t("padr.signatures")}</h2>
    <p data-testid="padr-reviewer-signed">
      {t("padr.reviewer")}:
      {latest.version.reviewer_signed_on ? formatDate(latest.version.reviewer_signed_on) : t("padr.notSigned")}
    </p>
    <p data-testid="padr-reviewee-signed">
      {t("padr.reviewee")}:
      {latest.version.reviewee_signed_on ? formatDate(latest.version.reviewee_signed_on) : t("padr.notSigned")}
    </p>
    {#if writes && !mySignature}
      <p class="muted">{t("padr.signHint")}</p>
      <button type="button" data-testid="padr-sign" onclick={() => void run(() => signPadr(pid))}>{t("padr.sign")}</button>
    {/if}
  </section>

  {#if view.role === "reviewee"}
    <form class="panel" onsubmit={saveComment}>
      <label>
        {t("padr.comment")}
        <textarea bind:value={comment} maxlength="2000" rows="3" data-testid="padr-comment"></textarea>
      </label>
      <button type="submit" data-testid="padr-comment-save">{t("common.save")}</button>
    </form>
  {:else if latest.version.reviewee_comment}
    <p data-testid="padr-comment-read">{t("padr.comment")}: {latest.version.reviewee_comment}</p>
  {/if}

  {#if writes && latest.frozen}
    <button type="button" data-testid="padr-correct" onclick={() => void run(() => correctPadr(pid))}>{t("padr.correct")}</button>
    <p class="muted">{t("padr.correctHint")}</p>
  {/if}

  {#if earlier.length > 0}
    <h2>{t("padr.earlier")}</h2>
    {#each earlier as version (version.version.pid)}
      <details class="panel" data-testid="padr-earlier">
        <summary>{tf("padr.version", { version: version.version.version })}</summary>
        <PadrVersionView {version} {stepLabels} liveObjectives={[]} onchange={load} />
      </details>
    {/each}
  {/if}
{/if}

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
  form select,
  form textarea {
    max-width: 100%;
    box-sizing: border-box;
  }
</style>
