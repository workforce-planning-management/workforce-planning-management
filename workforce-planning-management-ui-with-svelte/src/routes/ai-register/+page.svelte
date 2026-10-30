<!--
  AI and scoring register (`/ai-register`): for each feature that works
  something out about people, what the deployer has recorded under Article 26
  of the AI Act: the category it chose, the role that oversees it, when
  representatives and workers were informed, and how long its logs are kept.
  Anyone signed in reads it (it is a notice of how people are scored); HR and
  administrators record. It records what the deployer says: it does not classify a
  system, certify conformity or count as evidence of compliance, and a field with
  nothing recorded says unknown.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { AI_CATEGORIES, getAiRegister, putAiRegisterEntry } from "#lib/api/wpm.js";
  import type { AiField, AiFeatureRow, AiRegister, GateCheck } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let answer = $state<AiRegister | null>(null);
  let error = $state<string | null>(null);
  let editing = $state<string | null>(null);
  let category = $state("");
  let overseer = $state("");
  let repsOn = $state("");
  let workersOn = $state("");
  let months = $state<number | null>(null);
  let note = $state("");
  let consultation = $state("");
  let gate = $state<GateCheck | null>(null);

  const load = guardedLoad(
    () => getAiRegister(),
    (found) => {
      answer = found;
      error = null;
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  $effect(() => {
    void load();
  });

  const value = <T,>(f: AiField<T>): T | null => (f.status === "recorded" ? f.value : null);

  function edit(r: AiFeatureRow) {
    editing = r.feature.id;
    category = value(r.annex_category) ?? "";
    overseer = value(r.overseer_role) ?? "";
    repsOn = value(r.representatives_informed_on) ?? "";
    workersOn = value(r.workers_informed_on) ?? "";
    months = value(r.log_retention_months);
    note = value(r.retention_note) ?? "";
  }

  async function save() {
    if (!editing) return;
    error = null;
    try {
      const saved = await putAiRegisterEntry(editing, {
        ...(category ? { annex_category: category } : {}),
        ...(overseer ? { overseer_role: overseer } : {}),
        ...(repsOn ? { representatives_informed_on: repsOn } : {}),
        ...(workersOn ? { workers_informed_on: workersOn } : {}),
        ...(months !== null && months !== undefined ? { log_retention_months: months } : {}),
        ...(note ? { retention_note: note } : {}),
        ...(consultation ? { consultation_pid: consultation } : {}),
      });
      gate = saved.decision_gate;
      consultation = "";
      editing = null;
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#snippet recorded(label: string, text: string | null)}
  <dt>{label}</dt>
  <dd>{#if text !== null}{text}{:else}<span class="muted" data-testid="ai-unknown">{t("aiRegister.not_recorded")}</span>{/if}</dd>
{/snippet}

<svelte:head><title>{tf("pages.aiRegister.page_title", { aiRegister: t("nav.aiRegister") })}</title></svelte:head>

<h1>{t("nav.aiRegister")}</h1>
<p class="muted">{t("aiRegister.hint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if answer}
  <p data-testid="ai-applies">
    {answer.applies_now ? tf("aiRegister.applies_now", { day: formatDate(answer.applies_from) }) : tf("aiRegister.applies_later", { day: formatDate(answer.applies_from) })}
  </p>
  <ul class="plain" data-testid="ai-features">
    {#each answer.features as r (r.feature.id)}
      <li class="panel" data-testid="ai-feature">
        <h2>{r.feature.id} — {r.feature.name}</h2>
        <p class="muted">{t("common.fromService")}: {r.advisory}</p>
        <dl>
          <dt>{t("aiRegister.computed_by")}</dt><dd>{r.feature.computed_by}</dd>
          <dt>{t("aiRegister.informs")}</dt><dd>{r.feature.informs}</dd>
          <dt>{t("aiRegister.made_of")}</dt><dd>{r.feature.made_of}</dd>
          {@render recorded(t("aiRegister.category"), value(r.annex_category) === null ? null : tv(value(r.annex_category)))}
          {@render recorded(t("aiRegister.overseer"), value(r.overseer_role))}
          {@render recorded(t("aiRegister.reps_informed"), value(r.representatives_informed_on))}
          {@render recorded(t("aiRegister.workers_informed"), value(r.workers_informed_on))}
          {@render recorded(t("aiRegister.retention"), value(r.log_retention_months) === null ? null : tf("aiRegister.months", { months: value(r.log_retention_months) ?? "" }))}
          {@render recorded(t("aiRegister.retention_note"), value(r.retention_note))}
        </dl>
        {#if r.retention_below_minimum}
          <p class="warn" data-testid="ai-below-minimum">{tf("aiRegister.below_minimum", { months: answer.minimum_log_months })}</p>
        {/if}
        <button type="button" data-testid="ai-edit" onclick={() => edit(r)}>{t("aiRegister.record")}</button>
      </li>
    {/each}
  </ul>
  <p class="muted">{answer.note}</p>
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}

{#if gate && gate.standing.kind !== "no_gate"}
  <div class="panel" data-testid="ai-gate">
    <p>{tf(`gates.standing.${gate.standing.kind}`, { n: gate.standing.kind === "reference_needed" ? gate.standing.bodies : 0 })}</p>
    {#if gate.equality_impact_template}<p>{tf("gates.template", { section: gate.equality_impact_template })}</p>{/if}
    <p class="muted">{t("gates.advice")}</p>
  </div>
{/if}

{#if editing}
  <form
    class="panel"
    data-testid="ai-form"
    onsubmit={(event) => {
      event.preventDefault();
      void save();
    }}
  >
    <h2>{tf("aiRegister.recording", { feature: editing })}</h2>
    <p class="muted">{t("aiRegister.replace_hint")}</p>
    <label>
      {t("aiRegister.category")}
      <select bind:value={category} data-testid="ai-category">
        <option value="">—</option>
        {#each AI_CATEGORIES as c (c)}<option value={c}>{tv(c)}</option>{/each}
      </select>
    </label>
    <label>{t("aiRegister.overseer")} <input bind:value={overseer} maxlength="120" data-testid="ai-overseer" /></label>
    <label>{t("aiRegister.reps_informed")} <input type="date" bind:value={repsOn} data-testid="ai-reps" /></label>
    <label>{t("aiRegister.workers_informed")} <input type="date" bind:value={workersOn} data-testid="ai-workers" /></label>
    <label>{t("aiRegister.retention")} <input type="number" min="1" max="240" bind:value={months} data-testid="ai-months" /></label>
    <label>{t("aiRegister.retention_note")} <textarea rows="2" bind:value={note} maxlength="500" data-testid="ai-note"></textarea></label>
    <label>{t("aiRegister.consultation_ref")} <input bind:value={consultation} data-testid="ai-consultation" /></label>
    <button type="submit" data-testid="ai-save">{t("common.save")}</button>
    <button type="button" onclick={() => (editing = null)}>{t("policies.rename_cancel")}</button>
  </form>
{/if}

<style>
  .plain { list-style: none; padding: 0; }
  /* Label and value side by side, so a card of nine pairs is not a column of eighteen lines; one column on a phone. */
  dl { display: grid; grid-template-columns: max-content 1fr; gap: 0.25rem 1.5rem; }
  dt { font-weight: 600; }
  dd { margin: 0; }
  @media (max-width: 600px) {
    dl { grid-template-columns: 1fr; gap: 0; }
    dd { margin-bottom: 0.5rem; }
  }
  .warn { color: #b45309; font-weight: 600; }
</style>
