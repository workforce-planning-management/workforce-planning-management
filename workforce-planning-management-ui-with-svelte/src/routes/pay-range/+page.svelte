<!--
  Pay position (`/pay-range`): where pay sits inside its band, as compa-ratio
  (pay over the band midpoint) and range penetration (how far through the
  band), per band, level, department and location. An aggregate for a
  compensation review: a group smaller than the floor shows nothing, no one
  is named, an outlier is a group, and nothing here proposes a pay change.
  Needs a payroll-level read; the service refuses anyone else and audits the
  read.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getPayRange } from "#lib/api/wpm.js";
  import type { PayRange, PayRangeFigure, PayRangeLeft, PayRangeOutlier, PayRangeStat } from "#lib/api/wpm.js";
  import { basisPoints } from "#lib/format.js";
  import { currencyCode as readCurrency } from "#lib/money.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let asOf = $state("");
  let cutoff = $state(10);
  let currencyCode = $state("");
  // Only a whole ISO-4217 shape is sent; anything shorter is still being typed.
  const currencyChosen = $derived(readCurrency(currencyCode) ?? "");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let answer = $state<PayRange | null>(null);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => getPayRange(organization, { ...(asOf ? { asOf } : {}), cutoff, ...(currencyChosen ? { currency: currencyChosen } : {}) }),
    (found) => {
      answer = found;
      error = null;
    },
    (cause) => {
      answer = null;
      error = errorText(cause);
    },
  );

  $effect(() => {
    void organization;
    void asOf;
    void cutoff;
    void currencyChosen;
    if (organization) void load();
  });

  const why = (reason: string) => t(`payRange.reason_${reason}` as "payRange.reason_no_pay");
  const stat = (f: PayRangeFigure<PayRangeStat>) => (f.kind === "shown" ? f.value : null);
  const left = (l: PayRangeLeft) => (l.kind === "exactly" ? String(l.workers) : tf("payRange.under_floor", { floor: answer?.report.floor ?? "" }));
  const outlier = (o: PayRangeOutlier) =>
    o.kind === "above"
      ? tf("payRange.above", { by: basisPoints(o.delta_bp) })
      : o.kind === "below"
        ? tf("payRange.below", { by: basisPoints(o.delta_bp) })
        : o.kind === "unknown"
          ? t("payRange.outlier_unknown")
          : "—";
</script>

{#snippet figure(f: PayRangeFigure<PayRangeStat>)}
  {#if f.kind === "shown"}
    <td data-testid="pay-range-workers">{f.value.workers}</td>
    <td>{basisPoints(f.value.compa_mean_bp)}</td>
    <td>
      {#if f.value.penetration.kind === "shown"}
        {basisPoints(f.value.penetration.value.mean_bp)}
      {:else if f.value.penetration.kind === "withheld"}
        {t("payRange.withheld")}
      {:else}
        {why(f.value.penetration.reason)}
      {/if}
    </td>
  {:else if f.kind === "withheld"}
    <td colspan="3" class="muted" data-testid="pay-range-withheld">{tf("payRange.withheld_group", { floor: answer?.report.floor ?? "" })}</td>
  {:else}
    <td colspan="3" class="muted">{why(f.reason)}</td>
  {/if}
{/snippet}

<svelte:head><title>{tf("pages.payRange.page_title", { payRange: t("nav.payRange") })}</title></svelte:head>

<h1>{t("nav.payRange")}</h1>
<p class="muted">{t("payRange.hint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organizations.length === 0}
  <p class="muted">{t("org.noOrganizations")}</p>
{:else}
  <p>
    <label>
      {t("policies.organization")}
      <select bind:value={organization} data-testid="pay-range-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
    <label>{t("payRange.as_of")} <input type="date" bind:value={asOf} data-testid="pay-range-as-of" /></label>
    <label>{t("payRange.cutoff")} <input type="number" min="1" max="100" bind:value={cutoff} data-testid="pay-range-cutoff" /></label>
    <label>{t("payRange.choose_currency")} <input bind:value={currencyCode} maxlength="3" size="4" data-testid="pay-range-currency" /></label>
  </p>

  {#if answer}
    <section class="panel" data-testid="pay-range-basis">
      <p>{answer.basis}</p>
      <p class="muted">{answer.derivation}</p>
      <p class="muted" data-testid="pay-range-audited">{t("payRange.audited")}</p>
      <p>
        {tf("payRange.currency", { currency: answer.currency ?? "—" })} ·
        {tf("payRange.excluded_no_pay", { count: left(answer.report.excluded.no_pay) })} ·
        {tf("payRange.excluded_no_band", { count: left(answer.report.excluded.no_band) })}
      </p>
    </section>

    <h2>{t("payRange.overall")}</h2>
    {#if stat(answer.report.overall)}
      {@const o = stat(answer.report.overall)!}
      <p data-testid="pay-range-overall">
        {tf("payRange.overall_line", { workers: o.workers, compa: basisPoints(o.compa_mean_bp) })}
      </p>
    {:else}
      <p class="muted" data-testid="pay-range-overall">
        {answer.report.overall.kind === "unknown" ? why(answer.report.overall.reason) : t("payRange.withheld")}
      </p>
    {/if}

    {#each answer.report.groupings as g (g.by)}
      <h2>{t(`payRange.by_${g.by}` as "payRange.by_band")}</h2>
      {#if g.groups.length === 0}
        <p class="muted" data-testid="pay-range-no-groups-{g.by}">{t("payRange.no_groups")}</p>
      {:else}
      <div class="data-grid-scroll">
        <table data-testid="pay-range-{g.by}">
          <thead>
            <tr>
              <th>{t("payRange.group")}</th>
              <th>{t("payRange.workers")}</th>
              <th>{t("payRange.compa")}</th>
              <th>{t("payRange.penetration")}</th>
              <th>{t("payRange.outlier")}</th>
            </tr>
          </thead>
          <tbody>
            {#each g.groups as group (group.key)}
              <tr data-testid="pay-range-row">
                <td>{group.key}</td>
                {@render figure(group.figure)}
                <td data-testid="pay-range-outlier">{outlier(group.outlier)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      {/if}
    {/each}
  {:else if !error}
    <p>{t("common.loading")}</p>
  {/if}
{/if}
