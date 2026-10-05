<!--
  CEO dashboard (`/ceo`): the whole picture on ONE screen, no scrolling —
  sized for an iPad (9th gen), 2160 × 1620 device pixels = 1080 × 810 CSS
  pixels at 2×. Everything is in `em` off a size that follows the viewport's
  short side, so the same layout fits that screen and a literal 2160 × 1620
  one, and the grid rows share the height rather than stacking past it.

  All figures come from the existing views (metrics, insights, requisitions,
  succession); a figure that cannot be
  loaded shows "—", never 0. The headcount trend is month-end headcount
  computed from hire and termination dates (`/metrics`), so it needs no
  snapshot job.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import {
    listRequisitions,
    successionGaps,
    workforceInsights,
    workforceMetrics,
  } from "#lib/api/wpm.js";
  import { delta, trendDates, type TrendPoint } from "#lib/ceo.js";
  import { mean, rate } from "#lib/format.js";
  import { t, tp } from "#lib/i18n.svelte.js";
  import TrendChart from "#lib/components/TrendChart.svelte";

  type Metrics = Awaited<ReturnType<typeof workforceMetrics>>;
  type Insights = Awaited<ReturnType<typeof workforceInsights>>["insights"];

  let metrics = $state<Metrics | null>(null);
  let insights = $state<Insights | null>(null);
  let trend = $state<TrendPoint[]>([]);
  let gaps = $state<number | null>(null);
  let openRoles = $state<number | null>(null);

  const dash = "—";
  const today = new Date().toISOString().slice(0, 10);

  /** A failed load is "no figure" for that tile, never an error over the whole screen. */
  async function settle<T>(promise: Promise<T>): Promise<T | null> {
    try {
      return await promise;
    } catch {
      return null;
    }
  }

  onMount(() => {
    // The page owns the whole viewport: the layout's body gets a class that
    // turns off its page padding and scrolling (see the global rules below).
    document.body.classList.add("ceo-screen");
    void (async () => {
      const dates = trendDates(today);
      const [m, ins, g, open, ...months] = await Promise.all([
        settle(workforceMetrics()),
        settle(workforceInsights()),
        settle(successionGaps()),
        settle(listRequisitions("open")),
        ...dates.map((d) => settle(workforceMetrics({ from: d, to: d }))),
      ]);
      metrics = m;
      insights = ins?.insights ?? null;
      gaps = g ? g.gaps.length : null;
      openRoles = open ? open.length : null;
      trend = dates.flatMap((d, i) => {
        const month = months[i];
        return month ? [{ date: d, value: month.headcount.closing }] : [];
      });
    })();
    return () => document.body.classList.remove("ceo-screen");
  });

  const change = $derived(metrics ? delta(metrics.headcount.closing, metrics.headcount.opening) : null);
  const arrow = (d: "up" | "down" | "flat") => (d === "up" ? "▲" : d === "down" ? "▼" : "■");
</script>

<svelte:head><title>{t("nav.ceo")} — WPM</title></svelte:head>

<div class="ceo" data-testid="ceo">
  <h1 class="cx-sr-only">{t("nav.ceo")}</h1>

  <section class="cx-tile cx-kpi" data-testid="kpi-headcount">
    <span class="cx-label">{t("metrics.headcount")}</span>
    <strong class="cx-hero">{metrics ? metrics.headcount.closing : dash}</strong>
    <span class="cx-sub">
      {#if change}
        <span class="cx-delta" data-direction={change.direction}>
          {arrow(change.direction)}
          {change.abs > 0 ? "+" : ""}{change.abs}{#if change.pct !== null}&nbsp;({change.pct.toFixed(1)}%){/if}
        </span>
        {t("ceo.vs12")}
      {:else}{dash}{/if}
    </span>
  </section>

  <section class="cx-tile cx-kpi" data-testid="kpi-turnover">
    <span class="cx-label">{t("metrics.turnover")}</span>
    <strong class="cx-hero">{metrics ? (rate(metrics.turnover_rate) ?? dash) : dash}</strong>
    <span class="cx-sub">{metrics ? `${metrics.leavers} ${t("metrics.leavers")}` : dash}</span>
  </section>

  <section class="cx-tile cx-kpi" data-testid="kpi-open">
    <span class="cx-label">{t("dash.openRequisitions")}</span>
    <strong class="cx-hero">{openRoles ?? dash}</strong>
    <span class="cx-sub">
      {#if metrics?.time_to_fill}
        {mean(metrics.time_to_fill.median_days)} {t("metrics.days")} · {t("metrics.timeToFill")}
      {:else}{dash}{/if}
    </span>
  </section>

  <section class="cx-tile cx-kpi" data-testid="kpi-gaps">
    <span class="cx-label">{t("dash.successionGaps")}</span>
    <strong class="cx-hero">{gaps ?? dash}</strong>
    <span class="cx-sub">
      {#if gaps === null}{dash}
      {:else if gaps === 0}<span class="cx-status cx-good">✓ {t("ceo.noGaps")}</span>
      {:else}<span class="cx-status cx-serious">▲ {t("ceo.attention")}</span>{/if}
    </span>
  </section>

  <section class="cx-tile cx-trend">
    <span class="cx-label">{t("ceo.trend")}</span>
    <div class="cx-chart">
      {#if trend.length > 0}
        <TrendChart points={trend} label={t("ceo.trend")} />
      {:else}<span class="cx-muted">{dash}</span>{/if}
    </div>
  </section>

  <section class="cx-tile cx-list cx-insights" data-testid="ceo-insights">
    <span class="cx-label">{t("metrics.insights")}</span>
    {#if insights === null}
      <span class="cx-muted">{dash}</span>
    {:else if insights.length === 0}
      <span class="cx-muted">{t("metrics.noInsights")}</span>
    {:else}
      <ul>
        {#each insights.slice(0, 4) as i (i.code)}
          <li>
            <span class="cx-status" class:cx-serious={i.severity === "attention"}>
              {i.severity === "attention" ? `▲ ${t("ceo.attention")}` : `● ${t("ceo.info")}`}
            </span>
            {tp(`insights.${i.code}.observation`, i.params) ?? i.observation}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

</div>

<style>
  /* One screen, no scroll. The page is the viewport minus the top bar. */
  :global(body.ceo-screen) {
    height: 100dvh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  :global(body.ceo-screen main) {
    flex: 1;
    min-height: 0;
    max-width: none;
    margin: 0;
    padding: 0.5rem 0.75rem 0.75rem;
    width: 100%;
    box-sizing: border-box;
  }

  .ceo {
    /* Light tokens (the dataviz reference palette); dark below. */
    --viz-surface: #fcfcfb;
    --viz-ink: #0b0b0b;
    --viz-ink-2: #52514e;
    --viz-muted: #898781;
    --viz-grid: #e1e0d9;
    --viz-axis: #c3c2b7;
    --viz-border: rgba(11, 11, 11, 0.1);
    --viz-series-1: #2a78d6;
    --viz-good-text: #006300;
    --viz-serious: #ec835a;

    /* Everything is `em`: it scales with the viewport's short side, so the
       grid keeps its proportions from a 1080 × 810 iPad up to 2160 × 1620. */
    font-size: clamp(10px, 1.75vmin, 40px);
    height: 100%;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    grid-template-rows: minmax(0, 0.75fr) minmax(0, 1.25fr);
    gap: 0.9em;
    color: var(--viz-ink);
  }
  @media (prefers-color-scheme: dark) {
    :global(:root:not([data-theme="light"])) .ceo {
      --viz-surface: #1a1a19;
      --viz-ink: #ffffff;
      --viz-ink-2: #c3c2b7;
      --viz-grid: #2c2c2a;
      --viz-axis: #383835;
      --viz-border: rgba(255, 255, 255, 0.1);
      --viz-series-1: #3987e5;
      --viz-good-text: #0ca30c;
    }
  }
  :global(:root[data-theme="dark"]) .ceo {
    --viz-surface: #1a1a19;
    --viz-ink: #ffffff;
    --viz-ink-2: #c3c2b7;
    --viz-grid: #2c2c2a;
    --viz-axis: #383835;
    --viz-border: rgba(255, 255, 255, 0.1);
    --viz-series-1: #3987e5;
    --viz-good-text: #0ca30c;
  }

  .cx-tile {
    background: var(--viz-surface);
    border: 1px solid var(--viz-border);
    border-radius: 0.6em;
    padding: 0.8em 1em;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 0.25em;
  }
  .cx-kpi {
    justify-content: center;
  }
  .cx-trend {
    grid-column: span 2;
  }
  .cx-insights {
    grid-column: span 2;
  }
  .cx-label {
    color: var(--viz-ink-2);
    font-size: 1em;
    font-weight: 600;
  }
  .cx-hero {
    font-size: 4.2em;
    line-height: 1.05;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .cx-sub {
    color: var(--viz-ink-2);
    font-size: 0.95em;
  }
  .cx-delta[data-direction="up"] {
    color: var(--viz-good-text);
  }
  .cx-status {
    font-weight: 600;
  }
  .cx-status.cx-good {
    color: var(--viz-good-text);
  }
  .cx-status.cx-serious {
    color: var(--viz-ink);
  }
  .cx-status.cx-serious::first-letter {
    color: var(--viz-serious);
  }
  .cx-chart {
    flex: 1;
    min-height: 0;
  }
  .cx-list ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.6em;
    font-size: 1.2em;
    overflow: hidden;
  }
  .cx-muted {
    color: var(--viz-muted);
  }
  .cx-sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
</style>
