<!--
  Expense decisions (`/expenses`): the claims waiting for the caller to decide —
  those of their direct reports and, for HR, of their organization. The service
  never lists a person's own claim here, and never lets them decide it.
-->
<script lang="ts">
  import { createPagedLoader } from "#lib/paged.svelte.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import LoadMore from "#lib/components/LoadMore.svelte";
  import { expenseQueuePage, money } from "#lib/api/wpm.js";
  import type { ExpenseClaimSummary } from "#lib/api/types.js";
  import ExpenseClaimView from "#lib/components/ExpenseClaimView.svelte";
  import { t, tp, i18n, tf } from "#lib/i18n.svelte.js";

  let status = $state("submitted");
  let open = $state<string | null>(null);

  const list = createPagedLoader<ExpenseClaimSummary>(
    () => (limit, offset) => expenseQueuePage(status, limit, offset),
  );
  const claims = $derived(list.items);
  const error = $derived(list.error);
  const load = () => list.load();

  $effect(() => {
    // A different list is a different set of claims: close whichever was open.
    open = null;
    void status;
    list.reset();
    void load();
  });
</script>

<svelte:head><title>{tf("pages.expenses.page_title", { expenseQueue: t("nav.expenseQueue") })}</title></svelte:head>

<h1>{t("nav.expenseQueue")}</h1>
<p class="muted">{t("expenses.queueHint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

<p>
  <label>
    {t("expenses.statusFilter")}
    <select data-testid="expense-queue-status" bind:value={status}>
      {#each ["submitted", "approved"] as s (s)}
        <option value={s}>{tp(`expenses.statuses.${s}`)}</option>
      {/each}
    </select>
  </label>
</p>

{#if claims === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else if claims.length === 0}
  <p class="muted" data-testid="expense-queue-empty">{t("expenses.queueEmpty")}</p>
{:else}
  <ul data-testid="expense-queue">
    {#each claims as c (c.pid)}
      <li>
        <strong>{c.worker_name ?? ""}</strong> — {c.title} —
        {money(c.total_minor, c.currency, i18n.locale)}
        <button type="button" data-testid={`expense-open-${c.pid}`} onclick={() => (open = open === c.pid ? null : c.pid)}>
          {open === c.pid ? t("expenses.close") : t("expenses.open")}
        </button>
        {#if open === c.pid}
          <ExpenseClaimView claimPid={c.pid} onchange={() => load()} />
        {/if}
      </li>
    {/each}
  </ul>
  <LoadMore shown={claims.length} total={list.total} more={list.more} loading={list.loadingMore} onmore={() => list.showMore()} />
  {#if list.moreError}<ErrorNote testid="more-error" message={list.moreError} />{/if}
{/if}
