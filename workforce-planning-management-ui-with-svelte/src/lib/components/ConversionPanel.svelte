<!--
  A worker's conversion plan (WPM-R98, WPM-T164): what a person intends for a fixed-term or
  contractor engagement, with who proposed it and who approved it. A plan is a decision someone
  records, never an outcome of length of service. HR reads; the worker's line manager or a
  privileged caller proposes, approves, abandons and carries out (the service refuses anyone else
  and says why). Only "carry out" changes the worker. A caller who may not read plans (a line
  manager who is not HR) gets the proposal form alone, and is told it was sent without seeing the
  plan back. The worker themselves, and a permanent worker with no plan, see nothing.
-->
<script lang="ts">
  import {
    CONVERSION_FUNDING_KINDS,
    CONVERSION_INTENTS,
    moveConversionPlan,
    proposeConversionPlan,
    workerConversionPlans,
    type ConversionIntent,
    type ConversionPlan,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let {
    workerPid,
    employmentType,
    terminated = false,
    own = false,
  }: { workerPid: string; employmentType: string; terminated?: boolean; own?: boolean } =
    $props();

  let plans = $state<ConversionPlan[] | null>(null);
  let error = $state<string | null>(null);
  let actionError = $state<string | null>(null);
  let version = $state(0);
  /** The read was refused: this caller may propose (if the service allows) but not read plans. */
  let proposeOnly = $state(false);
  let sent = $state(false);

  let intent = $state<ConversionIntent>("convert");
  let targetOn = $state("");
  let fundingKind = $state("");
  let fundingEndsOn = $state("");
  let reviewOn = $state("");
  let reason = $state("");

  const notTheirs = (cause: unknown) =>
    cause instanceof ApiError && [401, 403, 404].includes(cause.status);

  $effect(() => {
    const pid = workerPid;
    void version;
    let current = true;
    void (async () => {
      try {
        const found = await workerConversionPlans(pid);
        if (!current) return;
        plans = found;
        proposeOnly = false;
        error = null;
      } catch (cause) {
        if (!current) return;
        plans = null;
        proposeOnly = cause instanceof ApiError && cause.status === 403;
        error = notTheirs(cause) ? null : cause instanceof Error ? cause.message : String(cause);
      }
    })();
    return () => {
      current = false;
    };
  });

  const open = $derived(plans?.find((p) => p.status === "proposed" || p.status === "approved"));
  const past = $derived((plans ?? []).filter((p) => p !== open));
  const canPropose = $derived(!open && !terminated && employmentType !== "permanent");
  const shown = $derived(plans !== null && (plans.length > 0 || canPropose));
  const showProposeOnly = $derived(proposeOnly && !own && !terminated && employmentType !== "permanent");

  async function act(run: () => Promise<unknown>) {
    actionError = null;
    sent = false;
    try {
      await run();
      version += 1;
      sent = proposeOnly;
    } catch (cause) {
      actionError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function propose(event: SubmitEvent) {
    event.preventDefault();
    await act(async () => {
      await proposeConversionPlan(workerPid, {
        intent,
        target_on: targetOn,
        ...(fundingKind ? { post_funding_kind: fundingKind } : {}),
        ...(fundingEndsOn ? { post_funding_ends_on: fundingEndsOn } : {}),
        ...(reviewOn ? { review_on: reviewOn } : {}),
        ...(reason.trim() ? { reason: reason.trim() } : {}),
      });
      targetOn = fundingEndsOn = reviewOn = reason = "";
      fundingKind = "";
    });
  }
</script>

{#snippet planView(plan: ConversionPlan)}
  <dl>
    <dt>{t("conversion.intent")}</dt>
    <dd data-testid="conversion-intent">{tv(plan.intent)}</dd>
    <dt>{t("common.status")}</dt>
    <dd data-testid="conversion-status">{tv(plan.status)}</dd>
    <dt>{t("conversion.target")}</dt>
    <dd>{plan.target_on}</dd>
    {#if plan.review_on}
      <dt>{t("conversion.review")}</dt>
      <dd>{plan.review_on}</dd>
    {/if}
    {#if plan.post_funding_kind}
      <dt>{t("conversion.funding")}</dt>
      <dd>
        {tv(plan.post_funding_kind)}{#if plan.post_funding_ends_on}
          · {t("engagement.fundingEnds")}: {plan.post_funding_ends_on}{/if}
      </dd>
    {/if}
    <dt>{t("conversion.proposedBy")}</dt>
    <dd>{plan.proposed_by ?? "—"}</dd>
    {#if plan.approved_by}
      <dt>{t("conversion.approvedBy")}</dt>
      <dd>{plan.approved_by}</dd>
    {/if}
    {#if plan.settled_on}
      <dt>{t("conversion.settledOn")}</dt>
      <dd>{plan.settled_on}</dd>
    {/if}
    {#if plan.reason}
      <dt>{t("conversion.reason")}</dt>
      <dd>{plan.reason}</dd>
    {/if}
  </dl>
  {#if plan.flags.length > 0}
    <p data-testid="conversion-flags">
      {#each plan.flags as flag (flag)}
        <span class="chip"><span aria-hidden="true">⚠ </span>{tv(flag)}</span>
      {/each}
    </p>
  {/if}
{/snippet}

{#snippet proposeForm()}
  <form onsubmit={propose} data-testid="conversion-form">
    <h3>{t("conversion.propose")}</h3>
    <label>
      {t("conversion.intent")}
      <select data-testid="conversion-intent-input" bind:value={intent}>
        {#each CONVERSION_INTENTS as i (i)}<option value={i}>{tv(i)}</option>{/each}
      </select>
    </label>
    <label>
      {t("conversion.target")}
      <input data-testid="conversion-target-input" type="date" bind:value={targetOn} required />
    </label>
    <label>
      {t("conversion.funding")}
      <select data-testid="conversion-funding-input" bind:value={fundingKind}>
        <option value="">—</option>
        {#each CONVERSION_FUNDING_KINDS as k (k)}<option value={k}>{tv(k)}</option>{/each}
      </select>
    </label>
    {#if fundingKind && fundingKind !== "core"}
      <label>
        {t("engagement.fundingEnds")}
        <input type="date" bind:value={fundingEndsOn} />
      </label>
    {/if}
    <label>
      {t("conversion.review")}
      <input type="date" bind:value={reviewOn} />
    </label>
    <label>
      {t("conversion.reasonInput")}
      <textarea maxlength="500" rows="3" bind:value={reason}></textarea>
    </label>
    <button type="submit" data-testid="conversion-propose">{t("conversion.propose")}</button>
  </form>
{/snippet}

{#if showProposeOnly}
  <section class="panel" data-testid="conversion-propose-only">
    <h2>{t("conversion.title")}</h2>
    <p class="muted">{t("conversion.proposeOnly")}</p>
    {#if actionError}
      <p class="error" data-testid="conversion-action-error">{actionError}</p>
    {/if}
    {#if sent}
      <p data-testid="conversion-sent" role="status">{t("conversion.sent")}</p>
    {/if}
    {@render proposeForm()}
  </section>
{:else if error}
  <p class="error" data-testid="conversion-error">{t("common.error")}: {error}</p>
{:else if shown && plans}
  <section class="panel" data-testid="conversion">
    <h2>{t("conversion.title")}</h2>
    <p class="muted">{t("conversion.hint")}</p>
    {#if actionError}
      <p class="error" data-testid="conversion-action-error">{actionError}</p>
    {/if}

    {#if open}
      <div data-testid="conversion-open">
        {@render planView(open)}
        <p>
          {#if open.status === "proposed"}
            <button type="button" data-testid="conversion-approve" onclick={() => void act(() => moveConversionPlan(open.pid, "approve"))}>
              {t("conversion.approve")}
            </button>
          {/if}
          {#if open.status === "approved" && open.intent === "convert"}
            <button type="button" data-testid="conversion-done" onclick={() => void act(() => moveConversionPlan(open.pid, "done"))}>
              {t("conversion.done")}
            </button>
          {/if}
          <button type="button" data-testid="conversion-abandon" onclick={() => void act(() => moveConversionPlan(open.pid, "abandon"))}>
            {t("conversion.abandon")}
          </button>
        </p>
      </div>
    {:else}
      <p class="muted" data-testid="conversion-none">{t("conversion.none")}</p>
    {/if}

    {#if canPropose}
      {@render proposeForm()}
    {/if}

    {#if past.length > 0}
      <h3>{t("conversion.history")}</h3>
      <ul data-testid="conversion-history">
        {#each past as plan (plan.pid)}
          <li>
            {tv(plan.intent)} · {tv(plan.status)} · {plan.target_on}{#if plan.settled_on}
              {" "}<span class="muted">· {plan.settled_on}</span>{/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

<style>
  form label {
    display: block;
    margin-block: 0.5rem;
  }
  form textarea {
    display: block;
    width: 100%;
    max-width: 40rem;
  }
</style>
