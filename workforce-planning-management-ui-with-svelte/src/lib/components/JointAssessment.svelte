<!--
  Joint pay assessment: for each category of workers, whether the Directive's
  joint assessment is due. It is due when the category's gap in basic pay is at
  or over the threshold, no justification on objective, gender-neutral
  criteria has been recorded, and the gap was not remedied by the end of the
  period counted from the date of reporting. This reports; a person records the
  justification, the remedy and the assessment, and nothing here says anyone
  was paid unfairly. Needs a payroll-level read to see; HR records.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { ASSESSMENT_OUTCOMES, getJointAssessment, putAssessmentRecord } from "#lib/api/wpm.js";
  import type { AssessmentSignal, GapQuery, JointAssessment } from "#lib/api/wpm.js";
  import { basisPoints } from "#lib/format.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  let { organization, options }: { organization: string; options: GapQuery } = $props();

  let answer = $state<JointAssessment | null>(null);
  let error = $state<string | null>(null);

  let editing = $state<string | null>(null); // a category name
  let reportedOn = $state("");
  let justified = $state(false);
  let justification = $state("");
  let remedied = $state("");
  let assessed = $state("");
  let outcome = $state("");
  let representatives = $state("");

  const load = guardedLoad(
    () => getJointAssessment(organization, options),
    (found) => {
      answer = found;
      error = null;
    },
    (cause) => {
      answer = null;
      error = errorText(cause);
    },
  );
  $effect(() => {
    void organization;
    void options.asOf;
    void options.currency;
    void options.variable;
    if (organization) void load();
  });

  function edit(c: JointAssessment["categories"][number]) {
    editing = c.category;
    reportedOn = c.reported_on ?? answer?.as_of ?? "";
    justified = c.record?.justified ?? false;
    justification = c.record?.justification ?? "";
    remedied = c.record?.remedied_on ?? "";
    assessed = c.record?.assessed_on ?? "";
    outcome = c.record?.outcome ?? "";
    representatives = c.record?.representatives ?? "";
  }

  async function save() {
    if (!editing) return;
    error = null;
    try {
      await putAssessmentRecord({
        organization_ref: organization,
        category: editing,
        reported_on: reportedOn,
        justified,
        ...(justified && justification ? { justification } : {}),
        ...(remedied ? { remedied_on: remedied } : {}),
        ...(assessed ? { assessed_on: assessed } : {}),
        ...(assessed && outcome ? { outcome } : {}),
        ...(representatives ? { representatives } : {}),
      });
      editing = null;
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  const signalText = (s: AssessmentSignal) => {
    switch (s.status) {
      case "due":
        return tf("jointAssessment.due", { day: s.remedy_by });
      case "pending":
        return s.remedy_by ? tf("jointAssessment.pending", { day: s.remedy_by }) : t("jointAssessment.pending_unreported");
      case "not_due":
        return t(`jointAssessment.not_due_${s.why}` as "jointAssessment.not_due_justified");
      default:
        return t(`jointAssessment.unknown_${s.why}` as "jointAssessment.unknown_gap_withheld");
    }
  };
</script>

<section data-testid="joint-assessment">
  <h2>{t("jointAssessment.title")}</h2>
  <p class="muted">{t("jointAssessment.hint")}</p>
  {#if error}<ErrorNote testid="assessment-error" message={error} />{/if}
  {#if answer}
    <p data-testid="assessment-config">{tf("jointAssessment.config", { threshold: basisPoints(answer.threshold_bp), months: answer.remedy_months })}</p>
    {#if answer.categories.length === 0}
      <p class="muted" data-testid="assessment-none">{t("payGap.categories_none")}</p>
    {:else}
      <div class="data-grid-scroll">
        <table data-testid="assessment-table">
          <thead>
            <tr>
              <th>{t("payGap.category")}</th><th>{t("payGap.gap")}</th><th>{t("jointAssessment.reported")}</th><th>{t("jointAssessment.signal")}</th><th></th>
            </tr>
          </thead>
          <tbody>
            {#each answer.categories as c (c.category)}
              <tr data-testid="assessment-row">
                <td>{c.category}</td>
                <td>{c.gap.status === "shown" ? basisPoints(c.gap.value.gap_bp) : "—"}</td>
                <td>{c.reported_on ?? t("jointAssessment.not_reported")}</td>
                <td data-testid="assessment-signal">{signalText(c.signal)}</td>
                <td>
                  {#if c.record}<span class="muted" data-testid="assessment-recorded">
                    {c.record.justified ? t("jointAssessment.recorded_justified") : ""}
                    {c.record.remedied_on ? tf("jointAssessment.recorded_remedied", { day: c.record.remedied_on }) : ""}
                    {c.record.assessed_on ? `${tf("jointAssessment.recorded_assessed", { day: c.record.assessed_on })} ${tv(c.record.outcome)}` : ""}
                  </span>{/if}
                  <button type="button" data-testid="assessment-edit" onclick={() => edit(c)}>{t("jointAssessment.record")}</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if !error}
    <p>{t("common.loading")}</p>
  {/if}

  {#if editing}
    <form
      data-testid="assessment-form"
      onsubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <h3>{tf("jointAssessment.recording", { category: editing })}</h3>
      <label>{t("jointAssessment.reported")} <input type="date" bind:value={reportedOn} required data-testid="assessment-reported" /></label>
      <label><input type="checkbox" bind:checked={justified} data-testid="assessment-justified" /> {t("jointAssessment.justified")}</label>
      {#if justified}
        <label>{t("jointAssessment.justification")} <textarea rows="3" bind:value={justification} minlength="10" maxlength="1000" required data-testid="assessment-justification"></textarea></label>
      {/if}
      <label>{t("jointAssessment.remedied_on")} <input type="date" bind:value={remedied} data-testid="assessment-remedied" /></label>
      <label>{t("jointAssessment.assessed_on")} <input type="date" bind:value={assessed} data-testid="assessment-assessed" /></label>
      {#if assessed}
        <label>
          {t("jointAssessment.outcome")}
          <select bind:value={outcome} required data-testid="assessment-outcome">
            <option value="">—</option>
            {#each ASSESSMENT_OUTCOMES as o (o)}<option value={o}>{tv(o)}</option>{/each}
          </select>
        </label>
      {/if}
      <label>{t("jointAssessment.representatives")} <input bind:value={representatives} maxlength="200" data-testid="assessment-representatives" /></label>
      <button type="submit" data-testid="assessment-save">{t("common.save")}</button>
      <button type="button" onclick={() => (editing = null)}>{t("policies.rename_cancel")}</button>
    </form>
  {/if}
</section>
