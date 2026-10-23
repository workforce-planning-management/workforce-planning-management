<!--
  My time off (WPM-R129): allowances and days remaining per kind, this year's requests and past absence, in
  calendar days. A kind with no allowance has no remaining figure (unknown, not zero); sick leave has no reason.
  A request is made here and a person decides it; one awaiting a decision, or approved and not started, can be cancelled.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    cancelMyLeave,
    getMyTimeOff,
    requestMyLeave,
    type MyTimeOff,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const KINDS = ["annual", "sick", "parental", "unpaid", "other"];

  let view = $state<MyTimeOff | null>(null);
  let error = $state<string | null>(null);
  let kind = $state("annual");
  let startOn = $state("");
  let endOn = $state("");
  let reason = $state("");

  const message = (cause: unknown) => (errorText(cause));
  // Cancellable by the worker: awaiting a decision, or approved and not yet started (the service checks too).
  const cancellable = (r: MyTimeOff["requests"][number]) =>
    r.status === "requested" || (r.status === "approved" && view !== null && r.start_on > view.as_of);

  async function load() {
    try {
      view = await getMyTimeOff();
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

<section class="panel" data-testid="my-time-off">
  <h2>{t("details.time.title")}</h2>
  {#if error}<ErrorNote testid="time-error" message={error} />{/if}
  {#if view}
    <div class="scroll">
    <table data-testid="time-balances">
      <thead>
        <tr>
          <th>{t("details.time.kind")}</th>
          <th>{t("details.time.entitled")}</th>
          <th>{t("details.time.taken")}</th>
          <th>{t("details.time.booked")}</th>
          <th>{t("details.time.requested")}</th>
          <th>{t("details.time.remaining")}</th>
        </tr>
      </thead>
      <tbody>
        {#each view.balances as b (b.kind)}
          <tr>
            <td>{tv(b.kind)}</td>
            <td>{b.entitled}</td>
            <td>{b.taken}</td>
            <td>{b.booked}</td>
            <td>{b.requested}</td>
            <td>{b.remaining ?? t("common.unknown")}</td>
          </tr>
        {:else}
          <tr><td colspan="6" class="muted">{t("details.time.no_allowance")}</td></tr>
        {/each}
      </tbody>
    </table>
    </div>
    <p class="muted">{tf("details.time.unit", { year: view.year })}</p>
    <ul data-testid="time-requests">
      {#each view.requests as r (r.pid)}
        <li>
          {tv(r.kind)}: {r.start_on} → {r.end_on} ({r.days}) <span class="muted">— {tv(r.status)}</span>
          {#if r.reason}<span class="muted">— {r.reason}</span>{/if}
          {#if cancellable(r)}
            <button type="button" data-testid="leave-cancel" onclick={() => void run(() => cancelMyLeave(r.pid))}>
              {t("details.time.cancel")}
            </button>
          {/if}
        </li>
      {:else}
        <li class="muted">{t("details.time.no_requests")}</li>
      {/each}
    </ul>
    {#if view.history.length > 0}
      <h3>{t("details.time.history")}</h3>
      <ul data-testid="time-history">
        {#each view.history as h (h.year)}
          <li>
            {h.year}:
            {#each h.absences as a, i (a.kind)}{i > 0 ? ", " : ""}{tv(a.kind)} {a.days}{/each}
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
  <h3>{t("details.time.request")}</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void run(async () => {
        await requestMyLeave({
          kind,
          start_on: startOn,
          end_on: endOn,
          ...(kind !== "sick" && reason.trim() ? { reason: reason.trim() } : {}),
        });
        startOn = endOn = reason = "";
      });
    }}
  >
    <label>
      {t("details.time.kind")}
      <select data-testid="leave-kind" bind:value={kind}>{#each KINDS as k (k)}<option value={k}>{tv(k)}</option>{/each}</select>
    </label>
    <label>{t("details.time.from")} <input data-testid="leave-from" type="date" bind:value={startOn} required /></label>
    <label>{t("details.time.to")} <input data-testid="leave-to" type="date" bind:value={endOn} required /></label>
    {#if kind !== "sick"}
      <label>{t("details.time.reason")} <input data-testid="leave-reason" bind:value={reason} /></label>
    {:else}
      <p class="muted">{t("details.time.sick_no_reason")}</p>
    {/if}
    <button type="submit" data-testid="leave-submit">{t("details.time.send")}</button>
  </form>
</section>

<style>
  /* A phone is too narrow for six columns: the table scrolls sideways and a heading never breaks mid-word. */
  .scroll {
    overflow-x: auto;
  }
  .scroll th,
  .scroll td:first-child {
    white-space: nowrap;
  }
</style>
