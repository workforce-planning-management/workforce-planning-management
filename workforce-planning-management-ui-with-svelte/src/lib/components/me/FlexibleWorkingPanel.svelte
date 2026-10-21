<!--
  My flexible working requests (WPM-R126): ask for a different arrangement, see the decide-by date, answer a
  counter-proposal, appeal a refusal once within the window, or withdraw. A person decides; the software only
  computes the dates and never changes a contract (an agreed change of hours is for HR to apply).
-->
<script lang="ts">
  import {
    actOnFlexibleRequest,
    listMyFlexibleRequests,
    requestFlexibleWorking,
    type MyFlexibleRequest,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const KINDS = ["hours", "times", "days", "place_of_work", "job_share", "compressed_hours", "other"];

  let requests = $state<MyFlexibleRequest[]>([]);
  let error = $state<string | null>(null);
  let kind = $state("hours");
  let start = $state("");
  let fte = $state("");
  let teamEffect = $state("");
  let manage = $state("");
  let appealing = $state<string | null>(null);
  let appealNote = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  // States in which the worker can still take the request back (the service decides the rest).
  const withdrawable = (s: string) => s === "requested" || s === "counter_proposed";

  async function load() {
    try {
      requests = await listMyFlexibleRequests();
    } catch (cause) {
      error = message(cause);
    }
  }
  $effect(() => void load());

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid="my-flexible-working">
  <h2>{t("details.flex.title")}</h2>
  <p class="muted">{t("details.flex.hint")}</p>
  {#if error}<p class="error" data-testid="flex-error">{error}</p>{/if}
  <ul data-testid="flex-requests">
    {#each requests as r (r.pid)}
      <li data-testid="flex-{r.pid}">
        <strong>{tv(r.kind)}</strong> — {tv(r.status)}
        <span class="muted">
          · {tf("details.flex.from", { date: r.proposed_start })}{r.proposed_fte_percent ? ` · ${r.proposed_fte_percent}%` : ""}
          · {tf("details.flex.decide_by", { date: r.decide_by })}
        </span>
        {#if r.overdue}<strong class="warn" data-testid="flex-overdue">{t("details.flex.overdue")}</strong>{/if}
        {#if r.decision_reason}<span class="muted"> · {tv(r.decision_reason)}</span>{/if}
        {#if r.decision_note}<span class="muted"> · {r.decision_note}</span>{/if}
        {#if r.trial_until}<span class="muted"> · {tf("details.flex.trial", { date: r.trial_until })}</span>{/if}
        {#if r.hr_to_apply_fte_percent}
          <span class="muted"> · {tf("details.flex.hr_applies", { fte: r.hr_to_apply_fte_percent })}</span>
        {/if}
        {#if r.status === "counter_proposed"}
          <p data-testid="flex-counter">
            {t("details.flex.counter")}: {r.counter_note ?? ""}{r.counter_fte_percent ? ` (${r.counter_fte_percent}%)` : ""}
            <button type="button" data-testid="flex-accept" onclick={() => void run(() => actOnFlexibleRequest(r.pid, "respond", { response: "accept" }))}>{t("details.flex.accept")}</button>
            <button type="button" data-testid="flex-decline" onclick={() => void run(() => actOnFlexibleRequest(r.pid, "respond", { response: "decline" }))}>{t("details.flex.decline")}</button>
          </p>
        {/if}
        {#if withdrawable(r.status)}
          <button type="button" data-testid="flex-withdraw" onclick={() => void run(() => actOnFlexibleRequest(r.pid, "withdraw"))}>{t("details.flex.withdraw")}</button>
        {/if}
        {#if r.can_appeal}
          {#if appealing === r.pid}
            <form
              onsubmit={(event) => {
                event.preventDefault();
                void run(async () => {
                  await actOnFlexibleRequest(r.pid, "appeal", { note: appealNote.trim() || undefined });
                  appealing = null;
                  appealNote = "";
                });
              }}
            >
              <label>{t("details.flex.appeal_note")} <input data-testid="flex-appeal-note" bind:value={appealNote} /></label>
              <button type="submit" data-testid="flex-appeal-send">{t("details.flex.appeal")}</button>
              <button type="button" onclick={() => (appealing = null)}>{t("details.cancel")}</button>
            </form>
          {:else}
            <button type="button" data-testid="flex-appeal" onclick={() => (appealing = r.pid)}>{t("details.flex.appeal")}</button>
          {/if}
        {/if}
      </li>
    {:else}
      <li class="muted">{t("details.flex.none")}</li>
    {/each}
  </ul>
  <h3>{t("details.flex.new")}</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void run(async () => {
        await requestFlexibleWorking({
          kind,
          proposed_start: start,
          ...(fte ? { proposed_fte_percent: Number(fte) } : {}),
          ...(teamEffect.trim() ? { effect_on_team: teamEffect.trim() } : {}),
          ...(manage.trim() ? { how_to_manage: manage.trim() } : {}),
        });
        start = fte = teamEffect = manage = "";
      });
    }}
  >
    <label>
      {t("details.flex.kind")}
      <select data-testid="flex-kind" bind:value={kind}>{#each KINDS as k (k)}<option value={k}>{tv(k)}</option>{/each}</select>
    </label>
    <label>{t("details.flex.start")} <input data-testid="flex-start" type="date" bind:value={start} required /></label>
    <label>{t("details.flex.fte")} <input data-testid="flex-fte" type="number" min="1" max="100" bind:value={fte} /></label>
    <label>{t("details.flex.effect")} <input data-testid="flex-effect" bind:value={teamEffect} /></label>
    <label>{t("details.flex.manage")} <input data-testid="flex-manage" bind:value={manage} /></label>
    <p class="muted">{t("details.flex.no_health")}</p>
    <button type="submit" data-testid="flex-submit">{t("details.flex.send")}</button>
  </form>
</section>
