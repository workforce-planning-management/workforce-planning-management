import { Temporal } from "@js-temporal/polyfill";

interface Candidate {
  instant: string;
  offset: string;
}

type Resolution =
  | { kind: "empty" | "invalidZone" | "invalidTime" | "skipped" }
  | { kind: "unique"; candidate: Candidate }
  | { kind: "repeated"; candidates: [Candidate, Candidate] };

/** WPM-D205: resolve a wall-clock entry without shifting a skipped time. */
export function resolveInjuryTime(local: string, zone: string): Resolution {
  if (!local) return { kind: "empty" };

  const timeZone = zone.trim();
  try {
    // Accept named zones and aliases supported by the browser, including UTC.
    if (!/^[A-Za-z]/.test(timeZone)) return { kind: "invalidZone" };
    new Intl.DateTimeFormat("en", { timeZone });
  } catch {
    return { kind: "invalidZone" };
  }

  try {
    const wallTime = Temporal.PlainDateTime.from(local);
    const earlier = wallTime.toZonedDateTime(timeZone, {
      disambiguation: "earlier",
    });
    if (!earlier.toPlainDateTime().equals(wallTime)) return { kind: "skipped" };

    const later = wallTime.toZonedDateTime(timeZone, {
      disambiguation: "later",
    });
    const toCandidate = (time: Temporal.ZonedDateTime): Candidate => ({
      instant: time.toInstant().toString({ smallestUnit: "millisecond" }),
      offset: `UTC${time.offset}`,
    });
    const first = toCandidate(earlier);
    const second = toCandidate(later);
    // The API expects RFC 3339 with a four-digit UTC year.
    if (![first, second].every(({ instant }) => /^\d{4}-/.test(instant))) {
      return { kind: "invalidTime" };
    }
    if (first.instant === second.instant)
      return { kind: "unique", candidate: first };
    return { kind: "repeated", candidates: [first, second] };
  } catch {
    return { kind: "invalidTime" };
  }
}
