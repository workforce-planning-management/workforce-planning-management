<!--
  Workforce metrics (`/metrics`): the shared metric vocabulary — headcount,
  starters, leavers, turnover, span of control, time-to-fill — for a chosen
  period, each shown beside the service's own definition. All server-derived;
  a missing figure renders "—", never 0.
-->
<script lang="ts">
  import { workforceMetrics } from "#lib/api/wpm.js";
  import { mean, rate } from "#lib/format.js";
  import { t } from "#lib/i18n.svelte.js";

  type Metrics = Awaited<ReturnType<typeof workforceMetrics>>;

  let metrics = $state<Metrics | null>(null);
  let from = $state("");
  let to = $state("");
  let error = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      metrics = await workforceMetrics({ from, to });
      from = metrics.period.from;
      to = metrics.period.to;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  $effect(() => {
    void load();
  });

  const dash = "—";
</script>

<svelte:head><title>{t("nav.metrics")} — WPM</title></svelte:head>

<h1>{t("nav.metrics")}</h1>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

<p>
  <label>
    {t("metrics.from")}
    <input type="date" data-testid="metrics-from" bind:value={from} />
  </label>
  <label>
    {t("metrics.to")}
    <input type="date" data-testid="metrics-to" bind:value={to} />
  </label>
  <button type="button" data-testid="metrics-apply" onclick={() => void load()}>
    {t("metrics.apply")}
  </button>
</p>

{#if metrics}
  {@const d = metrics.definitions}
  <table data-testid="metrics-table">
    <thead>
      <tr><th>{t("metrics.metric")}</th><th>{t("metrics.value")}</th><th>{t("metrics.definition")}</th></tr>
    </thead>
    <tbody>
      <tr>
        <td>{t("metrics.headcount")}</td>
        <td>
          {metrics.headcount.opening} → {metrics.headcount.closing}
          <span class="muted">({t("metrics.openingOn")} {metrics.headcount.opening_date})</span>
        </td>
        <td class="muted">{d.headcount}</td>
      </tr>
      <tr>
        <td>{t("metrics.starters")}</td>
        <td>{metrics.starters}</td>
        <td class="muted">{d.starters}</td>
      </tr>
      <tr>
        <td>{t("metrics.leavers")}</td>
        <td>{metrics.leavers}</td>
        <td class="muted">{d.leavers}</td>
      </tr>
      <tr>
        <td>{t("metrics.turnover")}</td>
        <td data-testid="metrics-turnover">{rate(metrics.turnover_rate) ?? dash}</td>
        <td class="muted">{d.turnover_rate}</td>
      </tr>
      <tr>
        <td>{t("metrics.span")}</td>
        <td>
          {#if metrics.span_of_control}
            {mean(metrics.span_of_control.mean)}
            <span class="muted">
              ({metrics.span_of_control.managers} {t("metrics.managers")}, {t("metrics.max")}
              {metrics.span_of_control.max})
            </span>
          {:else}{dash}{/if}
        </td>
        <td class="muted">{d.span_of_control}</td>
      </tr>
      <tr>
        <td>{t("metrics.timeToFill")}</td>
        <td data-testid="metrics-time-to-fill">
          {#if metrics.time_to_fill}
            {mean(metrics.time_to_fill.median_days)} {t("metrics.days")}
            <span class="muted">
              ({t("metrics.mean")} {mean(metrics.time_to_fill.mean_days)},
              {metrics.time_to_fill.requisitions} {t("metrics.requisitions")})
            </span>
          {:else}{dash}{/if}
        </td>
        <td class="muted">{d.time_to_fill}</td>
      </tr>
    </tbody>
  </table>
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}
