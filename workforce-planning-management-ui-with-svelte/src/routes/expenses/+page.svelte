<!--
  Expense decisions (`/expenses`): the claims waiting for the caller to decide —
  those of their direct reports and, for HR, of their organization. The service
  never lists a person's own claim here, and never lets them decide it.
-->
<script lang="ts">
  import { expenseQueue, money } from "#lib/api/wpm.js";
  import type { ExpenseClaimSummary } from "#lib/api/types.js";
  import ExpenseClaimView from "#lib/components/ExpenseClaimView.svelte";
  import { t, tp, i18n, tf } from "#lib/i18n.svelte.js";

  let status = $state("submitted");
  let claims = $state<ExpenseClaimSummary[] | null>(null);
  let error = $state<string | null>(null);
  let open = $state<string | null>(null);

  async function load(current: string) {
    try {
      claims = await expenseQueue(current);
      error = null;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  $effect(() => {
    // A different list is a different set of claims: close whichever was open.
    open = null;
    void load(status);
  });
</script>

<svelte:head><title>{tf("pages.expenses.page_title", { expenseQueue: t("nav.expenseQueue") })}</title></svelte:head>

<h1>{t("nav.expenseQueue")}</h1>
<p class="muted">{t("expenses.queueHint")}</p>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

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
          <ExpenseClaimView claimPid={c.pid} onchange={() => load(status)} />
        {/if}
      </li>
    {/each}
  </ul>
{/if}
