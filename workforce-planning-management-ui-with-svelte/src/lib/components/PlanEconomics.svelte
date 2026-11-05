<!--
  What one development plan costs and returns (WPM-R227, WPM-R228, WPM-R230). The fees and the cost of the hours stay side by
  side, unknown stays unknown and names the missing input, and no stated benefit is unmeasured, never not viable.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import { formatDecimal, formatPercent } from "#lib/locale-format.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    developmentPlanCost,
    developmentPlanViability,
    money,
    removeMyPlanBenefit,
    stateMyPlanBenefit,
    type PlanCostView,
    type PlanViabilityView,
    type PlanFigures,
  } from "#lib/api/wpm.js";
  import { inputNames } from "#lib/economics.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { pid }: { pid: string } = $props();

  let cost = $state<PlanCostView | null>(null);
  let view = $state<PlanViabilityView | null>(null);
  let error = $state<string | null>(null);
  let year = $state(1);
  let amount = $state<number | null>(null);
  let currency = $state("");
  let source = $state("");

  const load = guardedLoad(
    () => Promise.all([developmentPlanCost(pid), developmentPlanViability(pid)]),
    ([foundCost, foundView]) => {
      cost = foundCost;
      view = foundView;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void pid;
    void load();
  });

  const percent = (bp: number) => formatPercent(bp / 10000, 2);
  const fees = $derived(Object.entries(cost?.cost.fees ?? {}));
  const measures = $derived<Array<[string, PlanFigures | null]>>(
    view
      ? [
          [t("economics.onFees"), view.viability.on_fees],
          [t("economics.withHours"), view.viability.with_hours],
        ]
      : [],
  );

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    void run(async () => {
      await stateMyPlanBenefit(pid, {
        year,
        amount_minor: amount ?? 0,
        currency: currency.trim().toUpperCase(),
        source: source.trim(),
      });
      amount = null;
      source = "";
    });
  }
</script>

{#if error}<ErrorNote testid="error" message={error} />{/if}

{#if cost}
  <dl data-testid="plan-cost">
    <dt>{t("economics.fees")}</dt>
    <dd data-testid="plan-fees">
      {#each fees as [code, minor] (code)}
        <span>{money(minor, code)}</span>
      {:else}
        <span class="muted">{t("economics.noFees")}</span>
      {/each}
      {#if cost.cost.courses_without_fee > 0}
        <span class="muted" data-testid="plan-no-fee">
          {tf("economics.withoutFee", { count: cost.cost.courses_without_fee })}
        </span>
      {/if}
    </dd>
    <dt>{t("economics.hours")}</dt>
    <dd>
      {cost.cost.hours}
      {#if cost.cost.estimated_hours > 0}
        <span class="muted">{tf("economics.estimated", { count: cost.cost.estimated_hours })}</span>
      {/if}
    </dd>
    <dt>{t("economics.hoursCost")}</dt>
    <dd data-testid="plan-hours-cost">
      {#if cost.hours_cost.status === "resolved"}
        {money(cost.hours_cost.minor, cost.hours_cost.currency)}
        <span class="muted">
          {tf("economics.hourly", { amount: money(cost.hours_cost.hourly_minor, cost.hours_cost.currency) })}
        </span>
      {:else}
        <span class="muted">{tf("economics.missing", { inputs: inputNames(cost.hours_cost.missing) })}</span>
      {/if}
    </dd>
  </dl>
  <p class="muted">{t("economics.beside")}</p>
{/if}

{#if view}
  <h3>{t("economics.viability")}</h3>
  {#if view.viability.rate_bp !== null}
    <p class="muted" data-testid="plan-rate">{tf("economics.rate", { rate: percent(view.viability.rate_bp) })}</p>
  {/if}
  {#if view.viability.missing.length > 0}
    <p class="muted" data-testid="plan-missing">
      {tf("economics.missing", { inputs: inputNames(view.viability.missing) })}
    </p>
  {/if}
  {#each measures as [label, figures] (label)}
    <h4>{label}</h4>
    {#if figures}
      <dl data-testid="plan-viability">
        <dt>{t("economics.verdictLabel")}</dt>
        <dd data-testid="plan-verdict">{t(`economics.verdict.${figures.verdict}`)}</dd>
        <dt>{t("economics.npv")}</dt>
        <dd>{money(figures.npv_minor, view.viability.currency)}</dd>
        <dt>{t("economics.payback")}</dt>
        <dd>
          {figures.payback
            ? tf("economics.paybackYears", { years: formatDecimal(figures.payback.periods + figures.payback.fraction_bp / 10000, 1) })
            : t("economics.never")}
        </dd>
      </dl>
    {:else}
      <p class="muted" data-testid="plan-unmeasured">{t("economics.verdict.unmeasured")}</p>
    {/if}
  {/each}

  <h3>{t("economics.benefits")}</h3>
  <p class="muted">{t("economics.benefitNote")}</p>
  <ul data-testid="plan-benefits">
    {#each view.benefits as benefit (benefit.pid)}
      <li>
        {t("economics.year")}
        {benefit.year}: {money(benefit.amount_minor, benefit.currency)}
        <span class="muted">· {benefit.source}</span>
        <button type="button" onclick={() => void run(() => removeMyPlanBenefit(benefit.pid))}>
          {t("economics.remove")}
        </button>
      </li>
    {/each}
  </ul>
  <form onsubmit={submit}>
    <label>{t("economics.year")} <input type="number" min="1" max="600" bind:value={year} required /></label>
    <label>
      {t("economics.amount")}
      <input type="number" min="1" bind:value={amount} required data-testid="benefit-amount" />
    </label>
    <label>{t("economics.currency")} <input bind:value={currency} required maxlength="3" data-testid="benefit-currency" /></label>
    <label>
      {t("economics.source")}
      <input bind:value={source} required maxlength="500" data-testid="benefit-source" />
    </label>
    <button type="submit" data-testid="benefit-add">{t("economics.add")}</button>
  </form>
{/if}
