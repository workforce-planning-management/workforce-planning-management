// Dates, numbers and ranges in a page's locale: the behaviour of the WPM UI's `src/lib/locale-format.ts` (WPM-T375,
// WPM-T377; spec/worldwide.md, WPM-R214), for a site that has no i18n store, so the locale is a required argument
// (the UI's is an optional last one). Keep the two in step: the functions here have the UI's names, `Intl` options
// and results, except that a civil date that does not exist ("2026-02-30") is shown as written, not as 2 March.
//
// Every figure a person reads goes through `Intl` in the page's locale: a German page shows "1.234", a Bengali one
// its own digits, a date reads "11 Oct 2026" or "11.10.2026" rather than "2026-10-11". A civil date (no time, no
// zone) is the same day everywhere, so it is formatted in UTC. An unknown value is "—", never 0.

import { isRtl } from "./site.js";

/** What an absent or unknown figure shows. */
export const DASH = "—";

/** The end of a range that is not known. */
export const OPEN_END = "…";

/** The separator between two ends when `formatRange` cannot be used: an en dash, which has no direction. */
const RANGE_SEPARATOR = " – ";

type Num = number | null | undefined;
type DateStyle = "short" | "medium" | "long" | "full";

function nf(loc: string, options: Intl.NumberFormatOptions): Intl.NumberFormat {
  try {
    return new Intl.NumberFormat(loc, options);
  } catch {
    return new Intl.NumberFormat("en", options);
  }
}

function dtf(loc: string, options: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  try {
    return new Intl.DateTimeFormat(loc, options);
  } catch {
    return new Intl.DateTimeFormat("en", options);
  }
}

/** A number in the locale's format: grouping, the locale's decimal mark and digits. */
export function formatNumber(n: Num, loc: string, options: Intl.NumberFormatOptions = {}): string {
  if (n === null || n === undefined || !Number.isFinite(n)) return DASH;
  return nf(loc, options).format(n);
}

/** A count or other whole number ("1,234", "১,২৩৪"); a fraction is rounded. */
export function formatInteger(n: Num, loc: string): string {
  return formatNumber(n, loc, { maximumFractionDigits: 0 });
}

const CIVIL = /^(\d{4})-(\d{2})-(\d{2})$/;

function instant(iso: string): { date: Date; civil: boolean } | null {
  const s = iso.trim();
  if (CIVIL.test(s)) {
    const date = new Date(`${s}T00:00:00Z`);
    // "2026-02-30" parses to 2 March: refuse a day that does not exist rather than show another.
    return Number.isNaN(date.getTime()) || date.toISOString().slice(0, 10) !== s ? null : { date, civil: true };
  }
  if (!/^\d{4}-\d{2}-\d{2}T/.test(s)) return null;
  const date = new Date(s);
  return Number.isNaN(date.getTime()) ? null : { date, civil: false };
}

/** Whether `value` is an ISO civil date ("2026-10-30") that exists. */
export function isCivilDate(value: unknown): value is string {
  return typeof value === "string" && CIVIL.test(value) && instant(value) !== null;
}

/**
 * An ISO date as a date in the locale ("11 Oct 2026" in en-001, "11.10.2026" in de-001). `null`, `undefined` or ""
 * is "—"; text that is not an ISO date is shown as given.
 */
export function formatDate(iso: string | null | undefined, loc: string, style: DateStyle = "medium"): string {
  if (iso === null || iso === undefined || iso === "") return DASH;
  const at = instant(iso);
  if (!at) return iso;
  return dtf(loc, { dateStyle: style, ...(at.civil ? { timeZone: "UTC" } : {}) }).format(at.date);
}

/**
 * Two ISO dates as one range in the locale, with the locale's own separator ("1–30 Oct 2026" in en-001,
 * "01.–30.10.2026" in de-001), which has no direction, so it reads correctly right to left. A missing end is "…";
 * a value that is not an ISO date is shown as given, with an en dash between.
 */
export function formatDateRange(
  start: string | null | undefined,
  end: string | null | undefined,
  loc: string,
  style: DateStyle = "medium",
): string {
  const a = start ? instant(start) : null;
  const b = end ? instant(end) : null;
  if (a && b && a.civil === b.civil && a.date.getTime() <= b.date.getTime()) {
    return dtf(loc, { dateStyle: style, ...(a.civil ? { timeZone: "UTC" } : {}) }).formatRange(a.date, b.date);
  }
  const side = (v: string | null | undefined) =>
    v === null || v === undefined || v === "" ? OPEN_END : formatDate(v, loc, style);
  return `${side(start)}${RANGE_SEPARATOR}${side(end)}`;
}

/** An arrow meaning "goes to", pointing along the locale's writing direction: "→", or "←" in Arabic and Urdu. */
export function arrow(loc: string): string {
  return isRtl(loc) ? "←" : "→";
}
