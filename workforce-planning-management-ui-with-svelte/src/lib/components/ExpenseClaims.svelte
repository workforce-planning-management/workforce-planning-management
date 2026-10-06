<!--
  A worker's own expense claims: start a claim, add items, submit it, and see how
  it was decided. Only the claimant, their manager and HR can see a claim, so the
  panel renders nothing for anyone else (the service refuses the read with 403).
  The person who decides is never the claimant — that is the service's rule, shown
  here only as the absence of Approve.
-->
<script lang="ts">
  import { createExpenseClaim, listExpenseClaims, money } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { ExpenseClaimSummary } from "#lib/api/types.js";
  import ExpenseClaimView from "#lib/components/ExpenseClaimView.svelte";
  import { t, tp, i18n } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  const ICON: Record<string, string> = {
    draft: "○",
    submitted: "●",
    approved: "✓",
    rejected: "▲",
    reimbursed: "✓",
    cancelled: "—",
  };

  let claims = $state<ExpenseClaimSummary[]>([]);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  let open = $state<string | null>(null);
  let title = $state("");
  let currency = $state("GBP");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      claims = await listExpenseClaims(workerPid);
      allowed = true;
    } catch (cause) {
      // Not the claimant, their manager or HR: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = message(cause);
      }
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function start(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      const made = await createExpenseClaim(workerPid, { title: title.trim(), currency });
      title = "";
      open = made.pid;
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

{#if allowed}
  <section class="panel" data-testid="expense-claims">
    <h2>{t("expenses.title")}</h2>
    <p class="muted">{t("expenses.panelHint")}</p>
    {#if error}<p class="error" data-testid="error">{error}</p>{/if}
    {#if claims.length === 0}
      <p class="muted" data-testid="expense-none">{t("expenses.none")}</p>
    {:else}
      <ul data-testid="expense-list">
        {#each claims as c (c.pid)}
          <li>
            <button type="button" class="cx-link" onclick={() => (open = open === c.pid ? null : c.pid)}>
              {c.title}
            </button>
            — {money(c.total_minor, c.currency, i18n.locale)}
            <span class="chip">{ICON[c.status]} {tp(`expenses.statuses.${c.status}`)}</span>
          </li>
        {/each}
      </ul>
    {/if}
    {#if open}
      <ExpenseClaimView claimPid={open} onchange={load} />
    {/if}
    <h3>{t("expenses.newClaim")}</h3>
    <form class="cx-form" onsubmit={start} data-testid="expense-new">
      <label>
        {t("expenses.claimTitle")}
        <input data-testid="expense-title" bind:value={title} required />
      </label>
      <label>
        {t("expenses.currency")}
        <input data-testid="expense-currency" size="4" maxlength="3" bind:value={currency} />
      </label>
      <button type="submit" data-testid="expense-create">{t("expenses.create")}</button>
    </form>
  </section>
{/if}

<style>
  .cx-link {
    background: none;
    border: none;
    padding: 0;
    text-decoration: underline;
    cursor: pointer;
    font: inherit;
    color: inherit;
  }
  .cx-form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    align-items: flex-end;
    margin-top: 0.75rem;
  }
  .cx-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    max-width: 100%;
  }
</style>
