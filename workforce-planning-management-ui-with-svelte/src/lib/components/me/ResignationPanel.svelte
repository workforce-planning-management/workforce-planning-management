<!--
  My resignation (WPM-R127): logged by the worker with a proposed last day that allows the notice, and an
  optional reason from a closed list. A person accepts it; nothing ends here and no one is ever terminated by it.
  The reason is the worker's own: a manager or HR is never shown it.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    getMyResignation,
    logMyResignation,
    withdrawMyResignation,
    type MyResignation,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const REASONS = [
    "new_role",
    "career_change",
    "relocation",
    "retirement",
    "study_or_travel",
    "family_or_caring",
    "working_pattern",
    "pay_or_benefits",
    "management_or_culture",
    "other",
    "prefer_not_to_say",
  ];

  let resignation = $state<MyResignation | null>(null);
  let error = $state<string | null>(null);
  let lastDay = $state("");
  let reason = $state("");
  let confirming = $state(false);

  const message = (cause: unknown) => (errorText(cause));

  const load = guardedLoad(
    async () => (await getMyResignation()).resignation,
    (found) => {
      resignation = found;
    },
    (cause) => {
      error = message(cause);
    },
  );
  $effect(() => void load());

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      confirming = false;
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid="my-resignation">
  <h2>{t("details.resign.title")}</h2>
  <p class="muted">{t("details.resign.hint")}</p>
  {#if error}<ErrorNote testid="resign-error" message={error} />{/if}
  {#if resignation}
    <p data-testid="resign-status">
      {tv(resignation.status)} — {tf("details.resign.logged", { date: resignation.logged_on })}
      · {tf("details.resign.proposed", { date: resignation.proposed_last_day })}
      {#if resignation.agreed_last_day}· {tf("details.resign.agreed", { date: resignation.agreed_last_day })}{/if}
      {#if resignation.reason}<span class="muted"> · {tv(resignation.reason)}</span>{/if}
    </p>
    {#if resignation.status === "logged"}
      <button type="button" data-testid="resign-withdraw" onclick={() => void run(() => withdrawMyResignation())}>{t("details.resign.withdraw")}</button>
    {/if}
  {/if}
  {#if !resignation || resignation.status !== "logged"}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        if (!confirming) {
          confirming = true;
          return;
        }
        void run(async () => {
          await logMyResignation({ proposed_last_day: lastDay, ...(reason ? { reason } : {}) });
          lastDay = reason = "";
        });
      }}
    >
      <label>{t("details.resign.last_day")} <input data-testid="resign-last-day" type="date" bind:value={lastDay} required /></label>
      <label>
        {t("details.resign.reason")}
        <select data-testid="resign-reason" bind:value={reason}>
          <option value="">{t("details.resign.no_reason")}</option>
          {#each REASONS as r (r)}<option value={r}>{tv(r)}</option>{/each}
        </select>
      </label>
      {#if confirming}<p class="warn" data-testid="resign-confirm-text">{t("details.resign.confirm")}</p>{/if}
      <button type="submit" data-testid="resign-submit">{confirming ? t("details.resign.confirm_button") : t("details.resign.log")}</button>
      {#if confirming}<button type="button" onclick={() => (confirming = false)}>{t("details.cancel")}</button>{/if}
    </form>
  {/if}
</section>
