<!--
  Pay scales (`/pay-scales`): the NHS Agenda for Change scale for Wales, as
  published in its pay circular, and a lookup for where a salary sits on a band.
  Reference data only: nothing is stored against a person, and a salary typed
  into the lookup is sent, answered and forgotten.
-->
<script lang="ts">
  import { getPayScale, listPayScales, money, moneyWhole, payPosition } from "#lib/api/wpm.js";
  import { t, tp, i18n } from "#lib/i18n.svelte.js";
  import type { PayLookup, PayScale } from "#lib/api/types.js";

  let scale = $state<PayScale | null>(null);
  let error = $state<string | null>(null);

  let band = $state("5");
  let salary = $state("");
  let step = $state("");
  let months = $state("");
  let lookup = $state<PayLookup | null>(null);
  let lookupError = $state<string | null>(null);

  $effect(() => {
    let stale = false;
    void (async () => {
      try {
        // The first scale is the newest the service knows.
        const [first] = await listPayScales();
        const loaded = first ? await getPayScale(first.id) : null;
        if (!stale) scale = loaded;
      } catch (cause) {
        if (!stale) error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  const perCent = (tenths: number): string => `${(tenths / 10).toFixed(1)}%`;

  async function ask(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!scale) return;
    lookupError = null;
    lookup = null;
    const pounds = salary.trim() === "" ? undefined : Number(salary);
    if (pounds !== undefined && !Number.isFinite(pounds)) {
      lookupError = t("payScales.badSalary");
      return;
    }
    try {
      lookup = await payPosition(scale.id, {
        band,
        ...(pounds !== undefined && { salary_minor: Math.round(pounds * 100) }),
        ...(step.trim() !== "" && months.trim() !== "" && {
          step: Number(step),
          months_on_step: Number(months),
        }),
      });
    } catch (cause) {
      lookupError = cause instanceof Error ? cause.message : String(cause);
    }
  }
</script>

<svelte:head><title>{t("nav.payScales")} — WPM</title></svelte:head>

<h1>{t("nav.payScales")}</h1>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

{#if scale}
  <p data-testid="pay-scale-source">
    <strong>{scale.name}</strong> — {t("payScales.from")}
    {scale.effective_from}, +{perCent(scale.uplift_tenths_percent)}.
    {t("payScales.source")}: {scale.source}.
  </p>
  <p class="muted">{t("payScales.basis")}</p>

  <div class="cx-scroll">
  <table data-testid="pay-scale-table">
    <thead>
      <tr>
        <th>{t("payScales.band")}</th>
        <th>{t("payScales.entry")}</th>
        <th>{t("payScales.intermediate")}</th>
        <th>{t("payScales.top")}</th>
      </tr>
    </thead>
    <tbody>
      {#each scale.bands as b (b.code)}
        {@const cells = b.steps.length === 3 ? b.steps : [b.steps[0], undefined, b.steps[1]]}
        <tr data-testid={`band-${b.code}`}>
          <th scope="row" class="cx-band">
            {b.code}
            {#if b.closed}<span class="chip">{t("payScales.closed")}</span>{/if}
          </th>
          {#each cells as cell, i (i)}
            <td>
              {#if cell}
                {moneyWhole(cell.annual_minor, scale.currency, i18n.locale)}
                {#if cell.years_to_next}
                  <span class="muted">· {cell.years_to_next} {t("payScales.years")}</span>
                {/if}
              {:else}—{/if}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
  </div>

  <h2>{t("payScales.allowances")}</h2>
  <ul data-testid="pay-allowances">
    {#each scale.allowances as a (a.code)}
      <li>{a.name}: {money(a.amount_minor, scale.currency, i18n.locale)}</li>
    {/each}
  </ul>

  <h2>{t("payScales.lookup")}</h2>
  <p class="muted">{t("payScales.lookupHelp")}</p>
  <form class="cx-form" data-testid="pay-lookup-form" onsubmit={ask}>
    <label>
      {t("payScales.band")}
      <select data-testid="pay-band" bind:value={band}>
        {#each scale.bands as b (b.code)}<option value={b.code}>{b.code}</option>{/each}
      </select>
    </label>
    <label>
      {t("payScales.salary")}
      <input data-testid="pay-salary" inputmode="decimal" bind:value={salary} />
    </label>
    <label>
      {t("payScales.step")}
      <input data-testid="pay-step" inputmode="numeric" bind:value={step} />
    </label>
    <label>
      {t("payScales.monthsOnStep")}
      <input data-testid="pay-months" inputmode="numeric" bind:value={months} />
    </label>
    <button type="submit" data-testid="pay-ask">{t("payScales.ask")}</button>
  </form>

  {#if lookupError}<p class="error" data-testid="pay-lookup-error">{lookupError}</p>{/if}
  {#if lookup}
    <div data-testid="pay-lookup-result">
      {#if lookup.closed_to_new_entrants}
        <p>▲ {t("payScales.closedNote")}</p>
      {/if}
      {#if lookup.position}
        <p data-testid="pay-position">
          {#if lookup.position.kind === "on_step"}
            ✓ {tp("payScales.position.on_step", { step: lookup.position.step })}
          {:else if lookup.position.kind === "between_steps"}
            ● {tp("payScales.position.between_steps", { step: lookup.position.below_step })}
          {:else if lookup.position.kind === "below_entry"}
            ▲ {tp("payScales.position.below_entry", {
              amount: moneyWhole(lookup.position.shortfall_minor, lookup.currency, i18n.locale),
            })}
          {:else}
            ▲ {tp("payScales.position.above_top", {
              amount: moneyWhole(lookup.position.excess_minor, lookup.currency, i18n.locale),
            })}
          {/if}
        </p>
      {/if}
      {#if lookup.progression}
        <p data-testid="pay-progression">
          {#if lookup.progression.kind === "at_top"}
            ● {t("payScales.progression.at_top")}
          {:else if lookup.progression.kind === "due"}
            ✓ {tp("payScales.progression.due", {
              amount: moneyWhole(lookup.progression.next_annual_minor, lookup.currency, i18n.locale),
            })}
          {:else}
            ● {tp("payScales.progression.not_yet", {
              months: lookup.progression.months_remaining,
              amount: moneyWhole(lookup.progression.next_annual_minor, lookup.currency, i18n.locale),
            })}
          {/if}
        </p>
      {/if}
    </div>
  {/if}
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}

<style>
  /* Global table headers are upper-cased; a band code is a name ("8a"). */
  .cx-scroll {
    overflow-x: auto;
  }
  .cx-scroll table {
    min-width: 34rem;
  }
  .cx-band {
    text-transform: none;
  }
  .cx-form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    align-items: flex-end;
  }
  .cx-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
</style>
