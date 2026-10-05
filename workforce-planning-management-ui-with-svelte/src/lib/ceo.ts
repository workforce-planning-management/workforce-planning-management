// Pure helpers for the CEO dashboard (`/ceo`): which month-end dates the
// trend plots, and how a change is described. No clock, no I/O.

/** One point of the headcount trend. */
export interface TrendPoint {
  /** ISO date the value is for. */
  date: string;
  value: number;
}

const iso = (d: Date): string => d.toISOString().slice(0, 10);

/**
 * The dates the trend plots: the last day of each of the `months` months
 * before `today`'s month, then `today` itself — `months + 1` points,
 * oldest first. Month-ends, not "30 days apart", so each point is a month.
 */
export function trendDates(today: string, months = 6): string[] {
  const t = new Date(`${today}T00:00:00Z`);
  const out: string[] = [];
  for (let back = months; back >= 1; back--) {
    // Day 0 of the month after `back` months ago = last day of that month.
    out.push(
      iso(new Date(Date.UTC(t.getUTCFullYear(), t.getUTCMonth() - back + 1, 0))),
    );
  }
  out.push(today);
  return out;
}

/** A change between two counts. `pct` is null when there is no base to divide by. */
export interface Delta {
  abs: number;
  pct: number | null;
  direction: "up" | "down" | "flat";
}

/** Describe `current` against `previous`: never a percentage of zero. */
export function delta(current: number, previous: number): Delta {
  const abs = current - previous;
  return {
    abs,
    pct: previous === 0 ? null : (abs / previous) * 100,
    direction: abs > 0 ? "up" : abs < 0 ? "down" : "flat",
  };
}

/**
 * Pad a value range so a line never touches the plot edge, and a flat
 * series still gets a visible band. Returns `[min, max]` with `min < max`.
 */
export function paddedRange(values: number[]): [number, number] {
  if (values.length === 0) return [0, 1];
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  if (lo === hi) return [lo - 1, hi + 1];
  const pad = (hi - lo) * 0.15;
  return [lo - pad, hi + pad];
}
