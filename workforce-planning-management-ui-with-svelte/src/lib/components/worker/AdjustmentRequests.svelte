<!--
  Reasonable adjustments (WPM-R79): the worker asks for a change in their own words; whoever decides agrees,
  declines or marks it in place. When the words are withheld from the viewer, only the category and status show.
-->
<script lang="ts">
  import {
    ADJUSTMENT_CATEGORIES,
    createAdjustmentRequest,
    decideAdjustment,
    type AdjustmentRequest,
  } from "#lib/api/wpm.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { t, tv } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";

  let {
    pid,
    requests,
    error = null,
    onchange,
    onfail,
  }: {
    pid: string;
    requests: AdjustmentRequest[];
    error?: string | null;
    /** Called after a change that the page should read again. */
    onchange: () => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let adjCategory = $state("quieter_workspace");
  let adjBarrier = $state("");
  let adjImpact = $state("");
  let adjChange = $state("");

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

<h2>{t("adj.title")}</h2>
{#if error}<ErrorNote testid="panel-error-adjustments" prefixed message={error} />{/if}
<div class="panel" data-testid="adjustments">
  <p class="muted">{t("adj.hint")}</p>
  <select aria-label={t("labels.adjustmentCategory")} bind:value={adjCategory}>
    {#each ADJUSTMENT_CATEGORIES as category (category)}
      <option value={category}>{tv(category)}</option>
    {/each}
  </select>
  <input placeholder={t("adj.barrier")} aria-label={t("adj.barrier")} bind:value={adjBarrier} />
  <input placeholder={t("adj.impact")} aria-label={t("adj.impact")} bind:value={adjImpact} />
  <input placeholder={t("adj.change")} aria-label={t("adj.change")} bind:value={adjChange} />
  <button
    disabled={!adjBarrier.trim() || !adjImpact.trim() || !adjChange.trim()}
    onclick={() => void act(async () => {
      await createAdjustmentRequest(pid, {
        category: adjCategory,
        barrier: adjBarrier.trim(),
        impact: adjImpact.trim(),
        adjustment: adjChange.trim(),
      });
      adjBarrier = adjImpact = adjChange = "";
    })}
  >{t("adj.new")}</button>
  <ul>
    {#each requests as request (request.pid)}
      <li>
        <span class="chip">{tv(request.category)}</span>
        <span class="chip">{tv(request.status)}</span>
        {#if !request.words_withheld}
          <br />{request.barrier} → {request.adjustment}
          {#if request.decision_note}<br /><span class="muted">{request.decision_note}</span>{/if}
        {/if}
        {#if request.status === "requested"}
          <button
            onclick={() => {
              const note = window.prompt(t("adj.note")) ?? "";
              void act(() => decideAdjustment(request.pid, "agreed", note.trim() || undefined));
            }}
          >{t("adj.agree")}</button>
          <button onclick={() => void act(() => decideAdjustment(request.pid, "declined"))}>{t("adj.decline")}</button>
        {:else if request.status === "agreed"}
          <button onclick={() => void act(() => decideAdjustment(request.pid, "in_place"))}>{t("adj.inPlace")}</button>
        {/if}
      </li>
    {/each}
  </ul>
</div>

