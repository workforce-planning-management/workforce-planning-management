<!--
  Ergonomic assessments (WPM-R80): a workstation, a checklist answered ok or not, and the open issues left.
-->
<script lang="ts">
  import {
    answerErgonomicItem,
    completeErgonomicAssessment,
    createErgonomicAssessment,
    type ErgonomicAssessment,
  } from "#lib/api/wpm.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { t, tv } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";

  let {
    pid,
    assessments,
    error = null,
    onchange,
    onfail,
  }: {
    pid: string;
    assessments: ErgonomicAssessment[];
    error?: string | null;
    /** Called after a change that the page should read again. */
    onchange: () => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let workstation = $state("");

  async function act(action: () => Promise<unknown>) {
    onfail(null);
    try {
      await action();
      await onchange();
    } catch (cause) {
      onfail(errorText(cause));
    }
  }
</script>

<h2>{t("erg.title")}</h2>
{#if error}<ErrorNote testid="panel-error-ergonomics" prefixed message={error} />{/if}
<div class="panel" data-testid="ergonomics">
  <input placeholder={t("erg.workstation")} aria-label={t("erg.workstation")} bind:value={workstation} />
  <button
    disabled={!workstation.trim()}
    onclick={() => void act(async () => {
      await createErgonomicAssessment(pid, workstation.trim());
      workstation = "";
    })}
  >{t("erg.new")}</button>
  {#each assessments as assessment (assessment.pid)}
    <div>
      <strong>{assessment.workstation}</strong>
      <span class="chip">{tv(assessment.status)}</span>
      {#if assessment.status === "open"}
        <button onclick={() => void act(() => completeErgonomicAssessment(assessment.pid))}>
          {t("erg.complete")}
        </button>
      {:else if assessment.open_issues > 0}
        <span class="chip">{assessment.open_issues} ⚠</span>
      {/if}
      <ul>
        {#each assessment.items as item (item.pid)}
          <li>
            {item.name}
            {#if item.ok === true}✓{:else if item.ok === false}✗ <span class="muted">{item.note}</span>{/if}
            {#if assessment.status === "open" && item.ok === null}
              <button onclick={() => void act(() => answerErgonomicItem(item.pid, true))}>{t("erg.ok")}</button>
              <button
                onclick={() => {
                  const note = window.prompt(t("erg.issue")) ?? "";
                  void act(() => answerErgonomicItem(item.pid, false, note.trim() || undefined));
                }}
              >{t("erg.issue")}</button>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
  {/each}
</div>

