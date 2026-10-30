<!--
  Pay scales (`/pay-scales`): the national banded pay scale, as
  published in its pay circular, and a lookup for where a salary sits on a band.
  Reference data only: nothing is stored against a person, and a salary typed
  into the lookup is sent, answered and forgotten.
-->
<script lang="ts">
  import { formatDate, formatPercent } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getPayScale, listPayScales, money, moneyWhole, payPosition } from "#lib/api/wpm.js";
  import { t, tp, i18n, tf } from "#lib/i18n.svelte.js";
  import { toMinor } from "#lib/money.js";
  import type { PayLookup, PayScale } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

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
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  type Band = PayScale["bands"][number];
  type Step = Band["steps"][number];
  // A band with two steps has no intermediate one: its middle cell is unknown ("—").
  const stepsOf = (b: Band): Array<Step | undefined> =>
    b.steps.length === 3 ? b.steps : [b.steps[0], undefined, b.steps[1]];
  const stepColumn = (
    id: string,
    header: string,
    index: number,
    cell: DataGridColumn["cell"],
  ): DataGridColumn => ({
    id,
    header,
    sortable: true,
    align: "end",
    accessor: (r) => stepsOf(r as unknown as Band)[index]?.annual_minor ?? null,
    cell,
  });
  const columns = $derived<DataGridColumn[]>([
    {
      id: "code",
      header: t("payScales.band"),
      sortable: true,
      rowHeader: true,
      cellClass: "cx-band",
      cell: bandCell,
    },
    stepColumn("entry", t("payScales.entry"), 0, entryCell),
    stepColumn("intermediate", t("payScales.intermediate"), 1, middleCell),
    stepColumn("top", t("payScales.top"), 2, topCell),
  ]);

  // Tenths of a percent: 33 is 3.3% ("33e-3" is exactly 0.033).
  const perCent = (tenths: number): string => formatPercent(Number(`${tenths}e-3`), 1);

  async function ask(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!scale) return;
    lookupError = null;
    lookup = null;
    // Read in the page's own number format ("35.114,50" in German) and the scale's own minor unit.
    const minor = salary.trim() === "" ? undefined : toMinor(salary, scale.currency, i18n.locale);
    if (minor === null) {
      lookupError = t("payScales.badSalary");
      return;
    }
    try {
      lookup = await payPosition(scale.id, {
        band,
        ...(minor !== undefined && { salary_minor: minor }),
        ...(step.trim() !== "" && months.trim() !== "" && {
          step: Number(step),
          months_on_step: Number(months),
        }),
      });
    } catch (cause) {
      lookupError = errorText(cause);
    }
  }
</script>

{#snippet bandCell({ row }: { row: DataGridRow })}
  {@const b = row as unknown as Band}
  {b.code}
  {#if b.closed}<span class="chip">{t("payScales.closed")}</span>{/if}
{/snippet}

{#snippet entryCell({ row }: { row: DataGridRow })}{@render stepView(row, 0)}{/snippet}
{#snippet middleCell({ row }: { row: DataGridRow })}{@render stepView(row, 1)}{/snippet}
{#snippet topCell({ row }: { row: DataGridRow })}{@render stepView(row, 2)}{/snippet}

{#snippet stepView(row: DataGridRow, index: number)}
  {@const cell = stepsOf(row as unknown as Band)[index]}
  {#if cell && scale}
    {moneyWhole(cell.annual_minor, scale.currency, i18n.locale)}
    {#if cell.years_to_next}
      <span class="muted">· {cell.years_to_next} {t("payScales.years")}</span>
    {/if}
  {:else}—{/if}
{/snippet}

<svelte:head><title>{tf("pages.payScales.page_title", { payScales: t("nav.payScales") })}</title></svelte:head>

<h1>{t("nav.payScales")}</h1>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if scale}
  <p data-testid="pay-scale-source">
    <strong>{scale.name}</strong> — {t("payScales.from")}
    {formatDate(scale.effective_from)}, +{perCent(scale.uplift_tenths_percent)}.
    {t("payScales.source")}: {scale.source}.
  </p>
  <p class="muted">{t("payScales.basis")}</p>

  <DataTable
    testid="pay-scale-table"
    label={t("payScales.band")}
    columns={columns}
    rows={scale.bands as unknown as DataGridRow[]}
    rowId={(row) => String(row.code)}
    rowAttrs={(row) => ({ "data-testid": `band-${row.code}` })}
  />

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

  {#if lookupError}<ErrorNote testid="pay-lookup-error" message={lookupError} />{/if}
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
  :global(.cx-band) {
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
