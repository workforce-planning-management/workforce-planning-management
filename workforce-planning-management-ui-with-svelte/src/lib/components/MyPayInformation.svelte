<!--
  Pay information: a worker may ask for their own pay level and the average
  pay, by sex, of their category of workers. HR answers within a stated
  number of calendar months; the answer is worked out when it is given and
  WPM keeps no copy. One request waits at a time. The panel shows only dates
  and where the request stands, never a figure.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { askForPayInformation, listMyPayInformationRequests } from "#lib/api/wpm.js";
  import type { PayInformationRequest } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let requests = $state<PayInformationRequest[] | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => listMyPayInformationRequests(),
    (rows) => {
      requests = rows;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  const waiting = $derived((requests ?? []).some((r) => r.standing !== "answered"));

  async function ask() {
    error = null;
    try {
      await askForPayInformation();
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

<section class="panel" data-testid="my-pay-information">
  <h2>{t("payInfo.title")}</h2>
  <p class="muted">{t("payInfo.hint")}</p>
  {#if error}<ErrorNote testid="error" message={error} />{/if}
  {#if requests && requests.length === 0}
    <p class="muted" data-testid="pay-info-none">{t("payInfo.none")}</p>
  {:else if requests}
    <ul data-testid="pay-info-list">
      {#each requests as r (r.pid)}
        <li data-testid="pay-info-row">
          {tf("payInfo.asked", { day: formatDate(r.asked_on) })} ·
          {#if r.standing === "answered"}
            <span data-testid="pay-info-standing">{tf("payInfo.answered", { day: formatDate(r.answered_on) })}</span>
          {:else if r.standing === "overdue"}
            <span data-testid="pay-info-standing">{tf("payInfo.overdue", { day: formatDate(r.due_on) })}</span>
          {:else}
            <span data-testid="pay-info-standing">{tf("payInfo.due", { day: formatDate(r.due_on) })}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  <button type="button" data-testid="pay-info-ask" disabled={waiting} onclick={() => void ask()}>{t("payInfo.ask")}</button>
</section>
