<!--
  The capacity heatmap (WPM-R56 to R60): skill pools down the side, months across. Each cell gives
  supply / demand in FTE, and its result as a mark and a word as well as a colour. A month that
  cannot be said is `unknown` (dashed, "?"): it is never drawn as a zero and never as a pass.
-->
<script lang="ts">
  import type { CellStatus } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";
  import { stateLabel } from "#lib/stateLabel.js";

  type Pool = {
    name: string;
    over_months: number;
    unknown_months: number;
    cells: Array<{
      month: string;
      supply_centi: number;
      demand_centi: number;
      status: CellStatus;
    }>;
  };

  let { months, pools }: { months: string[]; pools: Pool[] } = $props();

  const MARK: Record<CellStatus, string> = {
    fits: "✓",
    fits_with_partners: "✓+",
    over: "✕",
    unknown: "?",
  };
  const fte = (centi: number) => (centi / 100).toFixed(2);
  const label = (month: string) => month.slice(0, 7);
</script>

<div class="scroll" data-testid="heatmap">
  <table class="heat">
    <thead>
      <tr>
        <th>{t("cap.pool")}</th>
        {#each months as m (m)}<th>{label(m)}</th>{/each}
      </tr>
    </thead>
    <tbody>
      {#each pools as pool (pool.name)}
        <tr data-testid="heat-row">
          <th scope="row" class="name">
            {pool.name}
            <span class="muted small">
              {tf("cap.overMonths", { over: pool.over_months, unknown: pool.unknown_months })}
            </span>
          </th>
          {#each pool.cells as cell (cell.month)}
            <td class="cell {cell.status}" data-status={cell.status} data-testid="heat-cell">
              <span class="mark" aria-hidden="true">{MARK[cell.status]}</span>
              <span class="word">{stateLabel(cell.status)}</span>
              <span class="nums">{fte(cell.supply_centi)} / {fte(cell.demand_centi)}</span>
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .scroll {
    overflow-x: auto;
  }
  .heat {
    min-width: 36rem;
  }
  .name {
    text-transform: none;
    letter-spacing: normal;
    font-size: 0.92rem;
    color: var(--ink);
    white-space: nowrap;
  }
  .small {
    display: block;
    font-size: 0.75rem;
    font-weight: 400;
  }
  .cell {
    text-align: center;
    color: var(--ink);
    border: 1px solid var(--line);
    min-width: 5.5rem;
    line-height: 1.25;
  }
  .mark {
    display: block;
    font-weight: 700;
  }
  .word {
    display: block;
    font-size: 0.72rem;
  }
  .nums {
    display: block;
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
  }
  /* Tints are mixed into the panel colour so they read in light and dark themes alike. */
  .fits {
    background: color-mix(in srgb, var(--state-available) 20%, var(--panel));
  }
  .fits_with_partners {
    background: color-mix(in srgb, var(--state-cleaning) 20%, var(--panel));
  }
  .over {
    background: color-mix(in srgb, var(--state-closed) 28%, var(--panel));
    border-color: var(--state-closed);
  }
  .unknown {
    background: repeating-linear-gradient(
      135deg,
      color-mix(in srgb, var(--state-reserved) 22%, var(--panel)) 0 6px,
      var(--panel) 6px 12px
    );
    border: 1px dashed var(--state-reserved);
  }
</style>
