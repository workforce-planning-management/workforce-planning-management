<!--
  Tolerances on a plan (WPM-R140): the largest deviation the plan may have, per dimension, shown as a colour, a
  shape and a word so it never rests on colour alone, with an unknown shown as unknown. A breach is escalated and
  the decision recorded. Every change goes through `run`, which saves, then reads the plan again.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import {
    decideEscalation,
    raiseEscalationLevel,
    raisePlanEscalation,
    removePlanTolerance,
    setPlanTolerance,
    type Rag,
  } from "#lib/api/wpm.js";
  import type { planEscalations, planTolerances } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  type Tolerances = Awaited<ReturnType<typeof planTolerances>>;
  type Escalations = Awaited<ReturnType<typeof planEscalations>>;

  let {
    planPid,
    tolerances,
    escalations,
    editable,
    run,
  }: {
    planPid: string;
    tolerances: Tolerances;
    escalations: Escalations;
    /** An archived plan cannot be changed. */
    editable: boolean;
    /** Do a write, then read the plan again; a failure is the page's to show. */
    run: (action: () => Promise<unknown>) => Promise<void>;
  } = $props();

  /** The dimension whose escalate or decide form is open, and what is typed in it. */
  let escalating = $state<string | null>(null);
  let deciding = $state<string | null>(null);
  let escNote = $state("");
  let decisionPick = $state("accept_deviation");
  let decisionNote = $state("");
  /** What is typed in each tolerance row, by dimension, before it is saved. */
  let draft = $state<Record<string, { limit: string; amber: string }>>({});

  // A fresh reading of the plan closes the forms and puts the saved limits back in the rows.
  $effect(() => {
    escalating = null;
    deciding = null;
    draft = Object.fromEntries(
      tolerances.dimensions.map((d) => [
        d.dimension,
        { limit: d.limit === null ? "" : String(d.limit), amber: String(d.amber_percent ?? 75) },
      ]),
    );
  });

  const RAG_SHAPE: Record<Rag, string> = { green: "●", amber: "▲", red: "■", unknown: "?", not_set: "–" };

  async function saveTolerance(dimension: string) {
    const row = draft[dimension];
    if (!row || String(row.limit).trim() === "") return;
    await run(() => setPlanTolerance(planPid, dimension, { limit: Number(row.limit), amber_percent: Number(row.amber) }));
  }

  async function escalate(dimension: string) {
    await run(() => raisePlanEscalation(planPid, { dimension, ...(escNote.trim() ? { note: escNote.trim() } : {}) }));
    escNote = "";
  }

  async function decide(pid: string) {
    await run(() => decideEscalation(pid, { decision: decisionPick, note: decisionNote }));
    decisionNote = "";
  }

  async function dropTolerance(dimension: string) {
    await run(() => removePlanTolerance(planPid, dimension));
  }
</script>

{#snippet ragChip(rag: Rag)}
  <span class="chip rag rag-{rag}" data-rag={rag}><span class="shape" aria-hidden="true">{RAG_SHAPE[rag]}</span>{tv(rag === "unknown" ? "rag_unknown" : rag)}</span>
{/snippet}

{#if tolerances}
  <h3>{t("pages.planning.tolerances")} {@render ragChip(tolerances.overall)}</h3>
  <p class="muted" data-testid="tolerance-derivation">{tolerances.derivation}</p>
  {#if tolerances.breaches_without_escalation > 0}
    <p class="warn" data-testid="breaches-without-escalation">
      {tf("pages.planning.breaches_without_escalation", { count: tolerances.breaches_without_escalation })}
    </p>
  {/if}
  <table class="tolerances" data-testid="plan-tolerances">
    <thead>
      <tr>
        <th>{t("pages.planning.tolerances")}</th>
        <th>{t("pages.planning.deviation_now")}</th>
        <th>{t("pages.planning.tolerance_limit")}</th>
        <th>{t("pages.planning.amber_from")}</th>
        <th>{t("common.status")}</th>
        <th>{t("pages.planning.escalation")}</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each tolerances.dimensions as d (d.dimension)}
        <tr data-testid="tolerance-{d.dimension}">
          <th scope="row" title={d.basis}>{tv(d.dimension)}</th>
          <td>{#if d.deviation === null}<span class="muted" title={d.reason ?? ""}>{tv("unknown")}</span>{:else}{d.deviation} {tv(d.unit)}{/if}</td>
          <td>
            {#if draft[d.dimension]}
              <input type="number" min="0" step="1" aria-label="{tv(d.dimension)}: {t('pages.planning.tolerance_limit')}" bind:value={draft[d.dimension]!.limit} /> {tv(d.unit)}
            {/if}
          </td>
          <td>
            {#if draft[d.dimension]}
              <input type="number" min="1" max="99" step="1" aria-label="{tv(d.dimension)}: {t('pages.planning.amber_from')}" bind:value={draft[d.dimension]!.amber} />
            {/if}
          </td>
          <td>{@render ragChip(d.rag)}</td>
          <td data-testid="escalation-{d.dimension}">
            {#if d.escalation}
              <span class="chip">{t("pages.planning.level")} {d.escalation.level_number}: {tv(d.escalation.level)}</span>
              <span class="muted">{tf("pages.planning.days_open_calendar", { days: d.escalation.days_open ?? 0 })}</span>
              {#if d.escalation.level_number < 3}
                <button type="button" onclick={() => void run(() => raiseEscalationLevel(d.escalation!.pid))}>{t("pages.planning.raise_level")}</button>
              {/if}
              <button type="button" onclick={() => { deciding = d.dimension; decisionNote = ""; }}>{t("pages.planning.decide")}</button>
              {#if deciding === d.dimension}
                <div>
                  <label>{t("pages.planning.decision")}
                    <select bind:value={decisionPick}>
                      {#each ["accept_deviation", "change_plan", "stop"] as option (option)}<option value={option}>{tv(option)}</option>{/each}
                    </select>
                  </label>
                  <label>{t("pages.planning.decision_note")} <input maxlength="500" bind:value={decisionNote} /></label>
                  <button type="button" disabled={decisionNote.trim() === ""} onclick={() => void decide(d.escalation!.pid)}>{t("pages.planning.decide")}</button>
                </div>
              {/if}
            {:else if d.set && (d.rag === "amber" || d.rag === "red") && editable}
              <button type="button" onclick={() => { escalating = d.dimension; escNote = ""; }}>{t("pages.planning.escalate")}</button>
              {#if escalating === d.dimension}
                <div>
                  <label>{t("pages.planning.escalation_note")} <input maxlength="500" bind:value={escNote} /></label>
                  <button type="button" onclick={() => void escalate(d.dimension)}>{t("pages.planning.escalate")}</button>
                </div>
              {/if}
            {/if}
          </td>
          <td>
            {#if editable}
              <button type="button" onclick={() => void saveTolerance(d.dimension)}>{t("pages.planning.set_tolerance")}</button>
              {#if d.set}<button type="button" onclick={() => void dropTolerance(d.dimension)}>{t("pages.planning.remove")}</button>{/if}
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

{#if escalations.some((e) => e.status === "decided")}
  <h3>{t("pages.planning.escalation_history")}</h3>
  <ul data-testid="escalation-history">
    {#each escalations.filter((e) => e.status === "decided") as e (e.pid)}
      <li>
        {tv(e.dimension)} · {formatDate(e.raised_on)} → {formatDate(e.decided_on)} · {tv(e.decision)}{#if e.decision_note} — {e.decision_note}{/if}
      </li>
    {/each}
  </ul>
{/if}


<style>
  .tolerances th:first-child {
    white-space: nowrap;
  }
  .tolerances td button {
    margin-block: 0.1rem;
  }
</style>
