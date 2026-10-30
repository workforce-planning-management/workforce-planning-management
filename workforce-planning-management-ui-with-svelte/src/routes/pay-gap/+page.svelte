<!--
  Pay gap (`/pay-gap`): the categories of workers HR sets up (a stated basis,
  the job titles in each) and the seven gender pay gap indicators the EU Pay
  Transparency Directive asks for. An aggregate: a figure shows only when both
  sexes reach the floor in it, no worker is named and no head count is given.
  The report needs a payroll-level read; the categories are HR's. The
  service audits the read and the CSV.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    changePayCategory,
    createPayCategory,
    getPayGapReport,
    listPayCategories,
    money,
    payGapCsvUrl,
    removePayCategory,
  } from "#lib/api/wpm.js";
  import type { Excluded, Gap, GapFigure, PayCategory, PayGapReport } from "#lib/api/wpm.js";
  import JointAssessment from "#lib/components/JointAssessment.svelte";
  import { basisPoints } from "#lib/format.js";
  import { currencyCode as readCurrency } from "#lib/money.js";
  import { i18n, t, tf } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let asOf = $state("");
  let currencyCode = $state("");
  let variable = $state("all_known");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });
  const currencyChosen = $derived(readCurrency(currencyCode) ?? "");
  const options = $derived({ ...(asOf ? { asOf } : {}), ...(currencyChosen ? { currency: currencyChosen } : {}), variable });

  let categories = $state<PayCategory[] | null>(null);
  let answer = $state<PayGapReport | null>(null);
  let categoryError = $state<string | null>(null);
  let reportError = $state<string | null>(null);

  // category form: one at a time, new or being corrected
  let editing = $state<string | null>(null); // a pid, "new", or null
  let formName = $state("");
  let formBasis = $state("");
  let formTitles = $state("");

  const loadCategories = guardedLoad(
    () => listPayCategories(organization),
    (rows) => {
      categories = rows;
      categoryError = null;
    },
    (cause) => {
      categoryError = errorText(cause);
    },
  );
  const loadReport = guardedLoad(
    () => getPayGapReport(organization, options),
    (found) => {
      answer = found;
      reportError = null;
    },
    (cause) => {
      answer = null;
      reportError = errorText(cause);
    },
  );
  $effect(() => {
    void organization;
    if (organization) void loadCategories();
  });
  $effect(() => {
    void organization;
    void options.asOf;
    void options.currency;
    void options.variable;
    if (organization) void loadReport();
  });

  const titlesOf = (text: string) =>
    text
      .split("\n")
      .map((x) => x.trim())
      .filter((x) => x.length > 0);

  function startNew() {
    editing = "new";
    formName = "";
    formBasis = "";
    formTitles = "";
  }
  function startEdit(c: PayCategory) {
    editing = c.pid;
    formName = c.name;
    formBasis = c.basis;
    formTitles = c.job_titles.join("\n");
  }
  async function save() {
    categoryError = null;
    const body = { organization_ref: organization, name: formName, basis: formBasis, job_titles: titlesOf(formTitles) };
    try {
      if (editing === "new") await createPayCategory(body);
      else if (editing) await changePayCategory(editing, body);
      editing = null;
      await Promise.all([loadCategories(), loadReport()]);
    } catch (cause) {
      categoryError = errorText(cause);
    }
  }
  async function remove(pid: string) {
    categoryError = null;
    try {
      await removePayCategory(pid);
      await Promise.all([loadCategories(), loadReport()]);
    } catch (cause) {
      categoryError = errorText(cause);
    }
  }

  const floor = $derived(answer?.report.floor ?? "");
  const why = (reason: string) => t(`payGap.reason_${reason}` as "payGap.reason_no_workers");
  const left = (e: Excluded) => (e.status === "exactly" ? String(e.value) : tf("payGap.under_floor", { floor }));
  const gapText = (g: Gap) => basisPoints(g.gap_bp);
  const pay = (minor: number) => money(minor, answer?.currency, i18n.locale);
</script>

{#snippet gap(f: GapFigure<Gap>)}
  {#if f.status === "shown"}
    <td data-testid="gap-female">{pay(f.value.female_minor)}</td>
    <td data-testid="gap-male">{pay(f.value.male_minor)}</td>
    <td data-testid="gap-value">{gapText(f.value)}</td>
  {:else if f.status === "withheld"}
    <td colspan="3" class="muted" data-testid="gap-withheld">{tf("payGap.withheld", { floor })}</td>
  {:else}
    <td colspan="3" class="muted" data-testid="gap-unknown">{why(f.value)}</td>
  {/if}
{/snippet}

<svelte:head><title>{tf("pages.payGap.page_title", { payGap: t("nav.payGap") })}</title></svelte:head>

<h1>{t("nav.payGap")}</h1>
<p class="muted">{t("payGap.hint")}</p>

{#if organizations.length === 0}
  <p class="muted">{t("org.noOrganizations")}</p>
{:else}
  <p>
    <label>
      {t("policies.organization")}
      <select bind:value={organization} data-testid="pay-gap-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  </p>

  <h2>{t("payGap.categories")}</h2>
  {#if categoryError}<ErrorNote testid="category-error" message={categoryError} />{/if}
  {#if categories && categories.length === 0}
    <p class="muted" data-testid="categories-none">{t("payGap.categories_none")}</p>
  {:else if categories}
    <ul data-testid="category-list">
      {#each categories as c (c.pid)}
        <li data-testid="category-row">
          <strong>{c.name}</strong>
          <span class="muted">— {c.job_titles.join(", ")}</span>
          <br />
          <span class="muted">{c.basis}</span>
          <button type="button" data-testid="category-edit" onclick={() => startEdit(c)}>{t("payGap.edit")}</button>
          <button type="button" data-testid="category-delete" onclick={() => void remove(c.pid)}>{t("common.remove")}</button>
        </li>
      {/each}
    </ul>
  {/if}
  {#if editing}
    <form
      data-testid="category-form"
      onsubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <label>{t("policies.name")} <input data-testid="category-name" bind:value={formName} maxlength="120" required /></label>
      <label>{t("payGap.basis")} <textarea data-testid="category-basis" bind:value={formBasis} rows="2" required></textarea></label>
      <label>{t("payGap.job_titles")} <textarea data-testid="category-titles" bind:value={formTitles} rows="4" required></textarea></label>
      <button type="submit" data-testid="category-save">{t("common.save")}</button>
      <button type="button" onclick={() => (editing = null)}>{t("policies.rename_cancel")}</button>
    </form>
  {:else}
    <p><button type="button" data-testid="category-new" onclick={startNew}>{t("payGap.add_category")}</button></p>
  {/if}

  <h2>{t("payGap.indicators")}</h2>
  <p>
    <label>{t("payRange.as_of")} <input type="date" bind:value={asOf} data-testid="pay-gap-as-of" /></label>
    <label>{t("payRange.choose_currency")} <input bind:value={currencyCode} maxlength="3" size="4" data-testid="pay-gap-currency" /></label>
    <label>
      {t("payGap.variable_basis")}
      <select bind:value={variable} data-testid="pay-gap-variable">
        <option value="all_known">{t("payGap.all_known")}</option>
        <option value="recipients_only">{t("payGap.recipients_only")}</option>
      </select>
    </label>
  </p>
  {#if reportError}<ErrorNote testid="error" prefixed message={reportError} />{/if}

  {#if answer}
    {@const r = answer.report}
    <section class="panel" data-testid="pay-gap-basis">
      <p>{answer.basis.pay}</p>
      <p class="muted">{answer.basis.variable}</p>
      <p class="muted">{answer.derivation}</p>
      <p class="muted" data-testid="pay-gap-audited">{t("payRange.audited")}</p>
      <p>
        {tf("payRange.currency", { currency: answer.currency ?? "—" })}
        {#if answer.reporting}
          · <span data-testid="pay-gap-reporting">{tf("payGap.reporting", { years: answer.reporting.every_years, first: answer.reporting.first_report_on, next: answer.reporting.next_report_on })}</span>
        {:else}
          · <span data-testid="pay-gap-reporting">{t("payGap.no_reporting")}</span>
        {/if}
      </p>
      <p><a href={payGapCsvUrl(organization, options)} data-testid="pay-gap-csv">{t("payGap.csv")}</a></p>
    </section>

    <p class="muted">{t("payGap.gap_note")} {t("payGap.quartile_note")}</p>
    <div class="data-grid-scroll">
      <table data-testid="pay-gap-indicators">
        <thead>
          <tr><th>{t("payGap.indicators")}</th><th>{t("payGap.female")}</th><th>{t("payGap.male")}</th><th>{t("payGap.gap")}</th></tr>
        </thead>
        <tbody>
          <tr data-testid="indicator-row"><td>{t("payGap.mean_gap")}</td>{@render gap(r.mean_gap)}</tr>
          <tr data-testid="indicator-row"><td>{t("payGap.mean_variable_gap")}</td>{@render gap(r.mean_variable_gap)}</tr>
          <tr data-testid="indicator-row"><td>{t("payGap.median_gap")}</td>{@render gap(r.median_gap)}</tr>
          <tr data-testid="indicator-row"><td>{t("payGap.median_variable_gap")}</td>{@render gap(r.median_variable_gap)}</tr>
          <tr data-testid="indicator-row">
            <td>{t("payGap.receiving_variable")}</td>
            {#if r.receiving_variable.status === "shown"}
              <td>{basisPoints(r.receiving_variable.value.female_bp)}</td><td>{basisPoints(r.receiving_variable.value.male_bp)}</td><td></td>
            {:else if r.receiving_variable.status === "withheld"}
              <td colspan="3" class="muted">{tf("payGap.withheld", { floor })}</td>
            {:else}
              <td colspan="3" class="muted">{why(r.receiving_variable.value)}</td>
            {/if}
          </tr>
          {#each r.quartiles as q, i (i)}
            <tr data-testid="quartile-row">
              <td>{tf("payGap.quartile", { n: i + 1 })}</td>
              {#if q.status === "shown"}
                <td>{basisPoints(q.value.female_bp)}</td><td>{basisPoints(q.value.male_bp)}</td><td></td>
              {:else if q.status === "withheld"}
                <td colspan="3" class="muted">{tf("payGap.withheld", { floor })}</td>
              {:else}
                <td colspan="3" class="muted">{why(q.value)}</td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <h2>{t("payGap.by_category")}</h2>
    {#if r.categories.length === 0}
      <p class="muted" data-testid="by-category-none">{t("payGap.categories_none")}</p>
    {:else}
      <div class="data-grid-scroll">
        <table data-testid="pay-gap-categories">
          <thead>
            <tr><th>{t("payGap.category")}</th><th>{t("payGap.basic")}</th><th>{t("payGap.female")}</th><th>{t("payGap.male")}</th><th>{t("payGap.gap")}</th></tr>
          </thead>
          <tbody>
            {#each r.categories as c (c.category)}
              <tr data-testid="category-gap-row"><td>{c.category}</td><td>{t("payGap.basic")}</td>{@render gap(c.basic)}</tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    <h2>{t("payGap.exclusions")}</h2>
    <ul data-testid="pay-gap-exclusions">
      <li>{t("payGap.excl_no_sex_declared")}: {left(r.exclusions.no_sex_declared)}</li>
      <li>{t("payGap.excl_sex_not_female_or_male")}: {left(r.exclusions.sex_not_female_or_male)}</li>
      <li>{t("payGap.excl_no_pay_recorded")}: {left(r.exclusions.no_pay_recorded)}</li>
      <li>{t("payGap.excl_variable_unknown")}: {left(r.exclusions.variable_unknown)}</li>
      <li>{t("payGap.excl_uncategorised")}: {left(r.exclusions.uncategorised)}</li>
    </ul>
  {:else if !reportError}
    <p>{t("common.loading")}</p>
  {/if}

  <JointAssessment {organization} {options} />
{/if}
