<!--
  Pay information requests (`/pay-information`): the requests workers have made
  for their own pay level and the average pay, by sex, of their category,
  oldest due first. Working out an answer needs a payroll-level read and is
  audited; the figures are shown here to be given to the worker and are not
  stored. Recording that an answer was given is HR's and keeps dates only.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    getPayInformationAnswer,
    listPayInformationRequests,
    markPayInformationAnswered,
    money,
  } from "#lib/api/wpm.js";
  import type { GapFigure, PayInformationAnswer, PayInformationQueueItem } from "#lib/api/wpm.js";
  import { i18n, t, tf } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let status = $state("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let queue = $state<PayInformationQueueItem[] | null>(null);
  let error = $state<string | null>(null);
  let working = $state<PayInformationAnswer | null>(null);

  const load = guardedLoad(
    () => listPayInformationRequests({ ...(organization ? { organization } : {}), ...(status ? { status } : {}) }),
    (rows) => {
      queue = rows;
      error = null;
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  $effect(() => {
    void organization;
    void status;
    working = null;
    void load();
  });

  async function workOut(pid: string) {
    error = null;
    try {
      working = await getPayInformationAnswer(pid);
    } catch (cause) {
      working = null;
      error = errorText(cause);
    }
  }
  async function answered(pid: string) {
    error = null;
    try {
      await markPayInformationAnswered(pid);
      working = null;
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  const standingText = (r: PayInformationQueueItem) =>
    r.standing === "answered"
      ? tf("payInfo.answered", { day: r.answered_on ?? "" })
      : r.standing === "overdue"
        ? tf("payInfo.overdue", { day: r.due_on })
        : tf("payInfo.due", { day: r.due_on });
  const pay = (minor: number | null) => money(minor, working?.currency, i18n.locale);
  const averageText = (f: GapFigure<number>) =>
    f.status === "shown"
      ? pay(f.value)
      : f.status === "withheld"
        ? tf("payInfo.withheld", { floor: working?.small_group_floor ?? "" })
        : t(`payGap.reason_${f.value}` as "payGap.reason_no_workers");
</script>

<svelte:head><title>{tf("pages.payInformation.page_title", { payInformation: t("nav.payInformation") })}</title></svelte:head>

<h1>{t("nav.payInformation")}</h1>
<p class="muted">{t("payInfo.queue_hint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

<p>
  {#if organizations.length > 0}
    <label>
      {t("policies.organization")}
      <select bind:value={organization} data-testid="pay-info-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  {/if}
  <label>
    {t("payInfo.status")}
    <select bind:value={status} data-testid="pay-info-status">
      <option value="">—</option>
      <option value="open">{t("payInfo.status_open")}</option>
      <option value="overdue">{t("payInfo.status_overdue")}</option>
      <option value="answered">{t("payInfo.status_answered")}</option>
    </select>
  </label>
</p>

{#if queue === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else if queue.length === 0}
  <p class="muted" data-testid="pay-info-queue-empty">{t("payInfo.queue_none")}</p>
{:else}
  <ul data-testid="pay-info-queue">
    {#each queue as r (r.pid)}
      <li data-testid="pay-info-queue-row">
        <strong>{r.display_name}</strong> <span class="muted">({r.worker_number})</span> —
        {tf("payInfo.asked", { day: r.asked_on })} · {standingText(r)}
        {#if r.standing !== "answered"}
          <button type="button" data-testid="pay-info-work-out" onclick={() => void workOut(r.pid)}>{t("payInfo.work_out")}</button>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

{#if working}
  <section class="panel" data-testid="pay-info-answer">
    <h2>{t("payInfo.answer")}</h2>
    <p class="muted">{t("payInfo.answer_hint")}</p>
    <p data-testid="pay-info-own">{tf("payInfo.own_pay", { pay: working.answer.own_pay_minor === null ? t("payInfo.none_recorded") : pay(working.answer.own_pay_minor) })}</p>
    {#if working.category}
      <p data-testid="pay-info-category">{tf("payInfo.category", { name: working.category.name })} <span class="muted">— {working.category.basis}</span></p>
    {:else}
      <p class="muted" data-testid="pay-info-category">{t("payInfo.no_category")}</p>
    {/if}
    <p data-testid="pay-info-female">{tf("payInfo.female_average", { pay: averageText(working.answer.female_average_minor) })}</p>
    <p data-testid="pay-info-male">{tf("payInfo.male_average", { pay: averageText(working.answer.male_average_minor) })}</p>
    <p class="muted">{working.derivation}</p>
    <button type="button" data-testid="pay-info-mark" onclick={() => void answered(working!.request_pid)}>{t("payInfo.mark_answered")}</button>
    <button type="button" onclick={() => (working = null)}>{t("credentials.hide")}</button>
  </section>
{/if}
