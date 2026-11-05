<!--
  Learning spend against budget (WPM-R227): recorded fees per department and currency, never converted. A completion with no
  single fee is counted, not priced, and no other actual is invented.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { ApiError } from "#lib/api/client.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { learningBudget, money, setLearningBudget, type LearningBudgetView } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let year = $state(new Date().getFullYear());
  let view = $state<LearningBudgetView | null>(null);
  let error = $state<string | null>(null);
  // Budgets are for HR and payroll; for anyone else the panel is simply not there.
  let refused = $state(false);
  let department = $state("");
  let amount = $state<number | null>(null);
  let currency = $state("");

  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  const load = guardedLoad(
    async () => (organization ? await learningBudget(organization, year) : null),
    (found) => {
      view = found;
    },
    (cause) => {
      if (cause instanceof ApiError && cause.status === 403) refused = true;
      else error = errorText(cause);
    },
  );

  $effect(() => {
    void organization;
    void year;
    void load();
  });

  async function set(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      await setLearningBudget({
        organization_ref: organization,
        department: department.trim(),
        year,
        amount_minor: amount ?? 0,
        currency: currency.trim().toUpperCase(),
      });
      amount = null;
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#if organizations.length > 0 && !refused}
  <section class="panel" data-testid="learning-budget">
    <h2>{t("economics.budgetTitle")}</h2>
    {#if error}<ErrorNote testid="error" message={error} />{/if}
    <label>{t("economics.year")} <input type="number" min="2000" max="2200" bind:value={year} /></label>
    {#if view}
      <table>
        <thead>
          <tr>
            <th scope="col">{t("economics.department")}</th>
            <th scope="col">{t("economics.budget")}</th>
            <th scope="col">{t("economics.spent")}</th>
            <th scope="col">{t("economics.remaining")}</th>
          </tr>
        </thead>
        <tbody>
          {#each view.departments as row (row.department)}
            {#each row.lines as line (line.currency)}
              <tr data-testid="budget-line">
                <th scope="row">{row.department}</th>
                <td>{money(line.budget_minor, line.currency)}</td>
                <td>{money(line.spent_minor, line.currency)}</td>
                <td>{money(line.remaining_minor, line.currency)}</td>
              </tr>
            {/each}
            {#if row.completions_without_fee > 0}
              <tr data-testid="budget-no-fee">
                <th scope="row">{row.department}</th>
                <td colspan="3" class="muted">
                  {tf("economics.completionsWithoutFee", { count: row.completions_without_fee })}
                </td>
              </tr>
            {/if}
          {:else}
            <tr><td colspan="4" class="muted">{t("economics.budgetNone")}</td></tr>
          {/each}
        </tbody>
      </table>
    {/if}
    <form onsubmit={set}>
      <label>{t("economics.department")} <input bind:value={department} required maxlength="200" /></label>
      <label>{t("economics.amount")} <input type="number" min="0" bind:value={amount} required /></label>
      <label>{t("economics.currency")} <input bind:value={currency} required maxlength="3" /></label>
      <button type="submit" data-testid="budget-set">{t("economics.setBudget")}</button>
    </form>
  </section>
{/if}
