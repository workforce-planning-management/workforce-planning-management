<!--
  One expense claim in full: its items, and whatever the caller may do with it.
  The actions shown come from the service's `can` flags, never from guessing who the
  caller is — so the claimant never sees Approve on their own claim, and a manager
  never sees Add item. A possible duplicate is flagged (▲), not refused.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    addExpenseItem,
    getExpenseClaim,
    moveExpenseClaim,
    money,
    removeExpenseItem,
  } from "#lib/api/wpm.js";
  import type { ExpenseClaim } from "#lib/api/types.js";
  import { t, tp, i18n } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let { claimPid, onchange }: { claimPid: string; onchange?: () => void } = $props();

  const CATEGORIES = [
    "travel",
    "accommodation",
    "meals",
    "equipment",
    "training",
    "subscriptions",
    "other",
  ];
  const ICON: Record<string, string> = {
    draft: "○",
    submitted: "●",
    approved: "✓",
    rejected: "▲",
    reimbursed: "✓",
    cancelled: "—",
  };

  let claim = $state<ExpenseClaim | null>(null);
  let error = $state<string | null>(null);
  let date = $state("");
  let category = $state("travel");
  let amount = $state("");
  let description = $state("");
  let receipt = $state("");
  let note = $state("");
  let paidOn = $state("");

  const message = (cause: unknown) => (errorText(cause));

  type Item = ExpenseClaim["items"][number];
  const item = (row: DataGridRow) => row as unknown as Item;
  const itemColumns = $derived<DataGridColumn[]>([
    { id: "incurred_on", type: "date", header: t("expenses.date"), sortable: true },
    {
      id: "category",
      header: t("expenses.category"),
      sortable: true,
      format: (v) => tp(`expenses.categories.${v}`) ?? String(v),
    },
    {
      id: "amount_minor",
      header: t("expenses.amount"),
      sortable: true,
      align: "end",
      format: (_v, r) => money(item(r).amount_minor, claim?.currency ?? "", i18n.locale),
    },
    { id: "description", header: t("expenses.description"), cell: descriptionCell },
    { id: "remove", header: t("common.actions"), cell: removeCell },
  ]);

  const load = guardedLoad(
    () => getExpenseClaim(claimPid),
    (found) => {
      claim = found;
    },
    (cause) => {
      error = message(cause);
    },
  );

  $effect(() => {
    void claimPid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
      onchange?.();
    } catch (cause) {
      error = message(cause);
    }
  }

  function addItem(event: SubmitEvent) {
    event.preventDefault();
    const pounds = Number(amount);
    if (!date || !Number.isFinite(pounds) || pounds <= 0) {
      error = t("expenses.amount");
      return;
    }
    const body = {
      incurred_on: date,
      category,
      amount_minor: Math.round(pounds * 100),
      ...(description.trim() && { description: description.trim() }),
      ...(receipt.trim() && { receipt_ref: receipt.trim() }),
    };
    // Clear at submit time, not when the request returns: whatever is typed for the
    // next item meanwhile must survive. Put the values back if the request fails.
    const kept = { amount, description, receipt };
    amount = "";
    description = "";
    receipt = "";
    void run(async () => {
      try {
        await addExpenseItem(claimPid, body);
      } catch (cause) {
        ({ amount, description, receipt } = kept);
        throw cause;
      }
    });
  }

  function reject() {
    if (!note.trim()) {
      error = t("expenses.rejectNeedsNote");
      return;
    }
    void run(() => moveExpenseClaim(claimPid, "reject", { note: note.trim() }));
  }
</script>

{#snippet descriptionCell({ row }: { row: DataGridRow })}
  {item(row).description ?? ""}
  {#if item(row).receipt_ref}<span class="muted">({item(row).receipt_ref})</span>{/if}
  {#if item(row).possible_duplicate}
    <span class="chip" data-testid="expense-duplicate">▲ {t("expenses.possibleDuplicate")}</span>
  {/if}
{/snippet}

{#snippet removeCell({ row }: { row: DataGridRow })}
  {#if claim?.can.edit}
    <button type="button" data-testid="expense-remove" onclick={() => run(() => removeExpenseItem(item(row).pid))}>
      {t("expenses.remove")}
    </button>
  {/if}
{/snippet}

{#if error}<ErrorNote testid="expense-error" message={error} />{/if}
{#if claim}
  <div class="cx-claim" data-testid="expense-claim">
    <p>
      <strong>{claim.title}</strong>
      <span class="chip" data-testid="expense-status">
        {ICON[claim.status]}
        {tp(`expenses.statuses.${claim.status}`)}
      </span>
      · {t("expenses.claimant")}: {claim.worker_name}
    </p>
    {#if claim.description}<p class="muted">{claim.description}</p>{/if}
    {#if claim.decision_note}
      <p data-testid="expense-decision-note">
        {t("expenses.decisionNote")}: {claim.decision_note}
      </p>
    {/if}
    {#if claim.reimbursed_on}
      <p>✓ {t("expenses.reimbursedOn")} {claim.reimbursed_on}</p>
    {/if}

    <DataTable
      testid="expense-items"
      label={t("expenses.itemsHeading")}
      columns={itemColumns}
      rows={claim.items as unknown as DataGridRow[]}
      rowId={(row) => String(row.pid)}
    >
      {#snippet empty()}<span class="muted">{t("expenses.noItems")}</span>{/snippet}
      {#snippet footer()}
        {#if claim}
          <tr>
            <th colspan="2">{t("expenses.total")}</th>
            <td data-testid="expense-total">{money(claim.total_minor, claim.currency, i18n.locale)}</td>
            <td colspan="2"></td>
          </tr>
        {/if}
      {/snippet}
    </DataTable>

    {#if claim.can.edit}
      <form class="cx-form" onsubmit={addItem} data-testid="expense-item-form">
        <label>{t("expenses.date")}<input type="date" data-testid="expense-date" bind:value={date} /></label>
        <label>
          {t("expenses.category")}
          <select data-testid="expense-category" bind:value={category}>
            {#each CATEGORIES as c (c)}<option value={c}>{tp(`expenses.categories.${c}`)}</option>{/each}
          </select>
        </label>
        <label>
          {t("expenses.amount")}
          <input inputmode="decimal" data-testid="expense-amount" bind:value={amount} />
        </label>
        <label>
          {t("expenses.description")}
          <input data-testid="expense-description" bind:value={description} />
        </label>
        <label>
          {t("expenses.receipt")}
          <input data-testid="expense-receipt" bind:value={receipt} />
        </label>
        <button type="submit" data-testid="expense-add">{t("expenses.add")}</button>
      </form>
    {/if}

    <p class="cx-actions">
      {#if claim.can.submit}
        <button data-testid="expense-submit" onclick={() => run(() => moveExpenseClaim(claimPid, "submit"))}>
          {t("expenses.submit")}
        </button>
      {/if}
      {#if claim.can.withdraw}
        <button data-testid="expense-withdraw" onclick={() => run(() => moveExpenseClaim(claimPid, "withdraw"))}>
          {t("expenses.withdraw")}
        </button>
      {/if}
      {#if claim.can.cancel}
        <button data-testid="expense-cancel" onclick={() => run(() => moveExpenseClaim(claimPid, "cancel"))}>
          {t("expenses.cancel")}
        </button>
      {/if}
    </p>

    {#if claim.can.decide}
      <div class="cx-form" data-testid="expense-decide">
        <label>
          {t("expenses.note")}
          <input data-testid="expense-note" bind:value={note} />
        </label>
        <button data-testid="expense-approve" onclick={() => run(() => moveExpenseClaim(claimPid, "approve", note.trim() ? { note: note.trim() } : {}))}>
          {t("expenses.approve")}
        </button>
        <button data-testid="expense-reject" onclick={reject}>{t("expenses.reject")}</button>
      </div>
    {/if}
    {#if claim.can.reimburse}
      <div class="cx-form" data-testid="expense-pay">
        <label>
          {t("expenses.reimbursedOn")}
          <input type="date" data-testid="expense-paid-on" bind:value={paidOn} />
        </label>
        <button
          data-testid="expense-reimburse"
          onclick={() => run(() => moveExpenseClaim(claimPid, "reimburse", paidOn ? { reimbursed_on: paidOn } : {}))}
        >
          {t("expenses.reimburse")}
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .cx-claim {
    border-inline-start: 3px solid currentColor;
    padding-inline-start: 0.75rem;
    margin: 0.75rem 0;
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
  .cx-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
</style>
