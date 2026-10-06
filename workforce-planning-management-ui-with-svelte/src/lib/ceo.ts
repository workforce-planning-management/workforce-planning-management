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
      iso(
        new Date(Date.UTC(t.getUTCFullYear(), t.getUTCMonth() - back + 1, 0)),
      ),
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

/** The periods the dashboard offers for turnover, time-to-fill and insights. */
export const RANGES = ["30d", "90d", "12m", "ytd"] as const;
export type Range = (typeof RANGES)[number];

/** The default period: the last twelve months. */
export const DEFAULT_RANGE: Range = "12m";

/** `raw` as a known range, else the default (so a stale or edited URL is harmless). */
export function parseRange(raw: string | null | undefined): Range {
  return (RANGES as readonly string[]).includes(raw ?? "")
    ? (raw as Range)
    : DEFAULT_RANGE;
}

/**
 * The `from`/`to` dates (inclusive, ISO) for `range` ending `today`: 30 or
 * 90 days counting today, twelve months back (the same date last year,
 * clamped to the month's end — the service's own default), or since 1 January.
 */
export function rangeDates(
  range: Range,
  today: string,
): { from: string; to: string } {
  const t = new Date(`${today}T00:00:00Z`);
  const day = (d: Date): string => d.toISOString().slice(0, 10);
  switch (range) {
    case "30d":
      return { from: day(new Date(t.getTime() - 29 * 86_400_000)), to: today };
    case "90d":
      return { from: day(new Date(t.getTime() - 89 * 86_400_000)), to: today };
    case "ytd":
      return { from: `${t.getUTCFullYear()}-01-01`, to: today };
    default: {
      const year = t.getUTCFullYear() - 1;
      const month = t.getUTCMonth();
      const lastDay = new Date(Date.UTC(year, month + 1, 0)).getUTCDate();
      return {
        from: day(
          new Date(Date.UTC(year, month, Math.min(t.getUTCDate(), lastDay))),
        ),
        to: today,
      };
    }
  }
}
