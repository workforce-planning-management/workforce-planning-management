// Formatters. The client formats; it does not compute.
//
// Every function here turns a value the API already decided into
// something readable — mirroring the content-management-system
// front-end's `#lib/format.ts` (WPM-T39). The recurring rule across all
// of them: a value that is genuinely `0` and a value that is *absent*
// are different claims, and only one of them is true — "no data" must
// never render as "0%" or "0.0", and a real zero must never collapse
// into "no data" either.

import type { Ratio } from "./api/types";
import {
  formatBasisPoints,
  formatDecimal,
  formatInteger,
  formatPercent,
} from "./locale-format";

// Every figure is written through `Intl` in the page's locale (`locale-format.ts`, WPM-T375): "67%" in
// English, "67 %" in German, Bengali digits on a Bengali page.

/**
 * A ratio as a rounded percentage, or `null` when there is nothing to
 * show.
 *
 * A zero-denominator ratio arrives as `value: null` and must **not**
 * render as `0%`: "we measured and it was zero" and "we had nothing to
 * measure" are different claims.
 */
export function percent(ratio: Ratio | null | undefined): string | null {
  if (!ratio || ratio.value === null) return null;
  return formatPercent(ratio.value);
}

/** A ratio's working, for display alongside the percentage: `"2/3"`. */
export function workings(ratio: Ratio | null | undefined): string | null {
  if (!ratio) return null;
  return `${formatInteger(ratio.numerator)}/${formatInteger(ratio.denominator)}`;
}

/**
 * A ratio rendered as `"67% (2/3)"`, or `fallback` (default `"—"`) when
 * there is nothing to show. This is the combined form every route
 * actually renders; `percent`/`workings` stay available separately for
 * a caller that lays the two out as distinct table columns.
 */
export function percentWithWorkings(
  ratio: Ratio | null | undefined,
  fallback = "—",
): string {
  const pct = percent(ratio);
  if (pct === null) return fallback;
  const w = workings(ratio);
  return w ? `${pct} (${w})` : pct;
}

/**
 * A percentage computed from a raw `done`/`total` pair, for a caller
 * that has counts rather than a service-supplied {@link Ratio} (e.g.
 * learning-path step progress). Same null-not-zero rule: `total === 0`
 * renders `fallback`, never `0%`.
 */
export function percentOf(done: number, total: number, fallback = "—"): string {
  if (total === 0) return fallback;
  return formatPercent(done / total);
}

/**
 * A mean to one decimal place, or `null` when absent (`null`/
 * `undefined`) — never let a missing sample silently render as `"0.0"`
 * or, worse, the literal string `"undefined"`.
 */
export function mean(value: number | null | undefined): string | null {
  if (value === null || value === undefined) return null;
  return formatDecimal(value, 1);
}

/**
 * A rate (a fraction, e.g. turnover) as a percentage to one decimal
 * place, or `null` when absent — "no base to divide by" is not `0.0%`.
 */
export function rate(value: number | null | undefined): string | null {
  if (value === null || value === undefined) return null;
  return formatPercent(value, 1);
}

/**
 * Basis points (hundredths of a percent) as a percentage with two decimals: `10456` is `104.56%`, `-5` is `-0.05%`
 * (in English). The service worked the figure out; this only places the point, exactly (`10456e-4`, not a division).
 */
export function basisPoints(bp: number): string {
  return formatBasisPoints(bp);
}
