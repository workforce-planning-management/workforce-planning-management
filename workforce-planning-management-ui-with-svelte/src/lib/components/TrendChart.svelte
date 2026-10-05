<!--
  A single-series line chart for the CEO dashboard. One series, so no legend
  box (the title names it) and the last value is labelled directly. Thin 2px
  line, hairline grid, a 9px end marker with a surface ring, and a hover /
  touch crosshair with a tooltip. A visually-hidden table carries the same
  numbers for screen readers. Colors come from the `--viz-*` tokens the
  page defines, so it follows light / dark.
-->
<script lang="ts">
  import { paddedRange, type TrendPoint } from "#lib/ceo.js";
  import { i18n } from "#lib/i18n.svelte.js";

  let { points, label }: { points: TrendPoint[]; label: string } = $props();

  // Fixed viewBox; the SVG scales to its box, so the chart fits any tile.
  const W = 400;
  const H = 250;
  const PAD = { l: 34, r: 54, t: 12, b: 22 };

  const range = $derived(paddedRange(points.map((p) => p.value)));
  const x = (i: number) =>
    PAD.l + (points.length < 2 ? 0 : (i * (W - PAD.l - PAD.r)) / (points.length - 1));
  const y = (v: number) =>
    PAD.t + ((range[1] - v) / (range[1] - range[0])) * (H - PAD.t - PAD.b);
  const path = $derived(points.map((p, i) => `${i === 0 ? "M" : "L"}${x(i)},${y(p.value)}`).join(" "));
  const ticks = $derived([range[0], (range[0] + range[1]) / 2, range[1]].map((v) => Math.round(v)));
  const month = (iso: string) =>
    new Intl.DateTimeFormat(i18n.locale, { month: "short", timeZone: "UTC" }).format(
      new Date(`${iso}T00:00:00Z`),
    );

  let hover = $state<number | null>(null);
  function track(event: PointerEvent) {
    const box = (event.currentTarget as SVGSVGElement).getBoundingClientRect();
    const vx = ((event.clientX - box.left) / box.width) * W;
    let best = 0;
    for (let i = 1; i < points.length; i++) {
      if (Math.abs(x(i) - vx) < Math.abs(x(best) - vx)) best = i;
    }
    hover = best;
  }
</script>

<figure class="trend" data-testid="trend-chart">
  <svg
    viewBox={`0 0 ${W} ${H}`}
    role="img"
    aria-label={label}
    onpointermove={track}
    onpointerdown={track}
    onpointerleave={() => (hover = null)}
  >
    {#each ticks as tick (tick)}
      <line class="grid" x1={PAD.l} x2={W - PAD.r} y1={y(tick)} y2={y(tick)} />
      <text class="tick" x={PAD.l - 6} y={y(tick) + 3} text-anchor="end">{tick}</text>
    {/each}
    {#each points as p, i (p.date)}
      {#if i % 2 === 0 || i === points.length - 1}
        <text class="tick" x={x(i)} y={H - 6} text-anchor="middle">{month(p.date)}</text>
      {/if}
    {/each}
    <path class="line" d={path} />
    {#if points.at(-1)}
      {@const last = points.at(-1)!}
      <circle class="end" cx={x(points.length - 1)} cy={y(last.value)} r="4.5" />
      <text class="value" x={x(points.length - 1) + 9} y={y(last.value) + 4}>{last.value}</text>
    {/if}
    {#if hover !== null && points[hover]}
      {@const p = points[hover]!}
      <line class="cross" x1={x(hover)} x2={x(hover)} y1={PAD.t} y2={H - PAD.b} />
      <circle class="end" cx={x(hover)} cy={y(p.value)} r="4.5" />
      <g transform={`translate(${Math.min(x(hover) + 8, W - 92)},${PAD.t})`}>
        <rect class="tipbox" width="84" height="30" rx="4" />
        <text class="tiptext" x="8" y="13">{month(p.date)} {p.date.slice(8)}</text>
        <text class="tiptext strong" x="8" y="25">{p.value}</text>
      </g>
    {/if}
  </svg>
  <!-- The wrapper clips: a table ignores `overflow`, so its rows would spill past
       a 1px box and stretch the page's scroll height. -->
  <div class="sr-only">
    <table>
      <caption>{label}</caption>
      <tbody>
        {#each points as p (p.date)}<tr><th scope="row">{p.date}</th><td>{p.value}</td></tr>{/each}
      </tbody>
    </table>
  </div>
</figure>

<style>
  .trend {
    margin: 0;
    width: 100%;
    height: 100%;
    min-height: 0;
  }
  svg {
    width: 100%;
    height: 100%;
    display: block;
    touch-action: none;
  }
  .grid {
    stroke: var(--viz-grid);
    stroke-width: 1;
  }
  .line {
    fill: none;
    stroke: var(--viz-series-1);
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .end {
    fill: var(--viz-series-1);
    stroke: var(--viz-surface);
    stroke-width: 2;
  }
  .cross {
    stroke: var(--viz-axis);
    stroke-width: 1;
  }
  .tick {
    fill: var(--viz-ink-2);
    font-size: 10px;
  }
  .value {
    fill: var(--viz-ink);
    font-size: 12px;
    font-weight: 600;
  }
  .tipbox {
    fill: var(--viz-surface);
    stroke: var(--viz-border);
  }
  .tiptext {
    fill: var(--viz-ink-2);
    font-size: 9px;
  }
  .tiptext.strong {
    fill: var(--viz-ink);
    font-size: 11px;
    font-weight: 600;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
</style>
