<!--
  Workforce metrics (`/metrics`): the shared metric vocabulary — headcount,
  starters, leavers, turnover, span of control, time-to-fill — for a chosen
  period, each shown beside the service's own definition. All server-derived;
  a missing figure renders "—", never 0.
-->
<script lang="ts">
  import { workforceInsights, workforceMetrics } from "#lib/api/wpm.js";
  import { mean, rate } from "#lib/format.js";
  import { t, tp, tf } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  type Metrics = Awaited<ReturnType<typeof workforceMetrics>>;

  type Insights = Awaited<ReturnType<typeof workforceInsights>>["insights"];

  let metrics = $state<Metrics | null>(null);
  let insights = $state<Insights | null>(null);
  let from = $state("");
  let to = $state("");
  let error = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      const [m, i] = await Promise.all([
        workforceMetrics({ from, to }),
        workforceInsights({ from, to }),
      ]);
      metrics = m;
      insights = i.insights;
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

  const columns = $derived<DataGridColumn[]>([
    { id: "label", header: t("metrics.metric") },
    {
      id: "value",
      header: t("metrics.value"),
      cell: valueCell,
      exportValue: (row) => valueText(String(row.id)),
      cellAttrs: ({ row }) =>
        row.id === "turnover"
          ? { "data-testid": "metrics-turnover" }
          : row.id === "timeToFill"
            ? { "data-testid": "metrics-time-to-fill" }
            : {},
    },
    { id: "definition", header: t("metrics.definition"), cellClass: "muted" },
  ]);

  /** The value as plain text, for an export (the cell shows the same figures with markup). */
  function valueText(id: string): string {
    if (!metrics) return dash;
    switch (id) {
      case "headcount":
        return `${metrics.headcount.opening} → ${metrics.headcount.closing}`;
      case "starters":
        return String(metrics.starters);
      case "leavers":
        return String(metrics.leavers);
      case "turnover":
        return rate(metrics.turnover_rate) ?? dash;
      case "span":
        return metrics.span_of_control ? String(mean(metrics.span_of_control.mean)) : dash;
      case "timeToFill":
        return metrics.time_to_fill
          ? `${mean(metrics.time_to_fill.median_days)} ${t("metrics.days")}`
          : dash;
      default:
        return dash;
    }
  }

  const metricRows = $derived.by((): DataGridRow[] => {
    if (!metrics) return [];
    const d = metrics.definitions;
    return [
      { id: "headcount", label: t("metrics.headcount"), definition: d.headcount },
      { id: "starters", label: t("metrics.starters"), definition: d.starters },
      { id: "leavers", label: t("metrics.leavers"), definition: d.leavers },
      { id: "turnover", label: t("metrics.turnover"), definition: d.turnover_rate },
      { id: "span", label: t("metrics.span"), definition: d.span_of_control },
      { id: "timeToFill", label: t("metrics.timeToFill"), definition: d.time_to_fill },
    ];
  });
</script>

{#snippet valueCell({ row }: { row: DataGridRow })}
  {#if metrics}
    {#if row.id === "headcount"}
      {metrics.headcount.opening} → {metrics.headcount.closing}
      <span class="muted">({t("metrics.openingOn")} {metrics.headcount.opening_date})</span>
    {:else if row.id === "starters"}
      {metrics.starters}
    {:else if row.id === "leavers"}
      {metrics.leavers}
    {:else if row.id === "turnover"}
      {rate(metrics.turnover_rate) ?? dash}
    {:else if row.id === "span"}
      {#if metrics.span_of_control}
        {mean(metrics.span_of_control.mean)}
        <span class="muted">
          ({metrics.span_of_control.managers} {t("metrics.managers")}, {t("metrics.max")}
          {metrics.span_of_control.max})
        </span>
      {:else}{dash}{/if}
    {:else if row.id === "timeToFill"}
      {#if metrics.time_to_fill}
        {mean(metrics.time_to_fill.median_days)} {t("metrics.days")}
        <span class="muted">
          ({t("metrics.mean")} {mean(metrics.time_to_fill.mean_days)},
          {metrics.time_to_fill.requisitions} {t("metrics.requisitions")})
        </span>
      {:else}{dash}{/if}
    {/if}
  {/if}
{/snippet}

<svelte:head><title>{tf("pages.metrics.page_title", { metrics: t("nav.metrics") })}</title></svelte:head>

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
  <DataTable
    testid="metrics-table"
    exportName="metrics"
    label={t("nav.metrics")}
    columns={columns}
    rows={metricRows}
    rowId={(row) => String(row.id)}
    rowAttrs={(row) => ({ "data-testid": `metric-${row.id}` })}
  />
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}

{#if insights}
  <h2>{t("metrics.insights")}</h2>
  {#if insights.length === 0}
    <p class="muted" data-testid="insights-none">{t("metrics.noInsights")}</p>
  {:else}
    <ul data-testid="insights-list">
      {#each insights as insight (insight.code)}
        <li data-severity={insight.severity}>
          <!-- Rendered from the finding's code and figures in the UI
               language; the server's English text is the fallback for a
               code this client does not know. -->
          <strong
            >{tp(`insights.${insight.code}.observation`, insight.params) ??
              insight.observation}</strong
          >
          <span class="muted"
            >{tp(`insights.${insight.code}.suggestion`) ??
              insight.suggestion}</span
          >
        </li>
      {/each}
    </ul>
  {/if}
{/if}
