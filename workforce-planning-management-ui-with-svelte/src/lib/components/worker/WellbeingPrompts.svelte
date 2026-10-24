<!--
  Health entitlements the worker has been prompted about, and the anonymous pulse surveys that are open
  (WPM-R78). A reply is recorded for the worker; a pulse score is not tied to them. Each panel's failure
  to load is shown in its own place by the page that fetched it.
-->
<script lang="ts">
  import { acknowledgeWellbeing, submitPulse, type PulseSurvey, type WellbeingPrompt } from "#lib/api/wpm.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { t } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";

  let {
    pid,
    prompts,
    surveys,
    promptsError = null,
    pulseError = null,
    onchange,
    onfail,
  }: {
    pid: string;
    prompts: WellbeingPrompt[];
    surveys: PulseSurvey[];
    promptsError?: string | null;
    pulseError?: string | null;
    /** Called after a change that the page should read again. */
    onchange: () => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let pulseThanks = $state<Set<string>>(new Set());

  async function respond(
    entitlementPid: string,
    response: "booked" | "done" | "declined" | "dismissed",
  ) {
    onfail(null);
    try {
      await acknowledgeWellbeing(pid, entitlementPid, response);
      await onchange();
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  async function sendPulse(surveyPid: string, score: number) {
    onfail(null);
    try {
      await submitPulse(surveyPid, pid, score);
      pulseThanks = new Set([...pulseThanks, surveyPid]);
    } catch (cause) {
      onfail(errorText(cause));
    }
  }
</script>

{#if promptsError}
  <h2>{t("wb.prompts")}</h2>
  <ErrorNote testid="panel-error-wellbeing" prefixed message={promptsError} />
{/if}
{#if prompts.length}
  <h2>{t("wb.prompts")}</h2>
  <ul class="panel" data-testid="wellbeing-prompts">
    {#each prompts as prompt (prompt.entitlement_pid)}
      <li>
        <strong>{prompt.name}</strong>
        <span class="chip">{prompt.entitlement_kind === "benefit" ? t("wb.kind.benefit") : t("wb.kind.health")}</span>
        {#if prompt.kind === "reminder"}<span class="chip">{t("wb.reminder")}</span>{/if}
        <br />
        {prompt.description}
        {#if prompt.info_url}
          · <a href={prompt.info_url} target="_blank" rel="noreferrer">{t("wb.info")}</a>
        {/if}
        <br />
        <button onclick={() => void respond(prompt.entitlement_pid, "booked")}>{t("wb.booked")}</button>
        <button onclick={() => void respond(prompt.entitlement_pid, "done")}>{t("wb.done")}</button>
        <button onclick={() => void respond(prompt.entitlement_pid, "declined")}>{t("wb.declined")}</button>
        <button onclick={() => void respond(prompt.entitlement_pid, "dismissed")}>{t("wb.dismissed")}</button>
      </li>
    {/each}
  </ul>
{/if}

{#if pulseError}
  <h2>{t("wb.pulse")}</h2>
  <ErrorNote testid="panel-error-pulse" prefixed message={pulseError} />
{/if}
{#if surveys.length}
  <h2>{t("wb.pulse")}</h2>
  <ul class="panel" data-testid="pulse-surveys">
    {#each surveys as survey (survey.pid)}
      <li>
        <strong>{survey.name}</strong><br />
        {survey.question}<br />
        {#if pulseThanks.has(survey.pid)}
          <span class="muted">{t("wb.pulseThanks")}</span>
        {:else}
          {#each [1, 2, 3, 4, 5] as score (score)}
            <button onclick={() => void sendPulse(survey.pid, score)}>{score}</button>
          {/each}
        {/if}
      </li>
    {/each}
  </ul>
{/if}

