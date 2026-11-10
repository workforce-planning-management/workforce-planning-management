// What is required of me (WPM-R245, WPM-D192): the order the employee's one list shows things in, what needs attention, and which
// qualification a person holds answers a requirement of their role. Pure: today is an argument.
//
// The standing of a role requirement is the service's (`QualificationGapRow.standing`, from `rules::qualification::standing`). The
// page only needs to know *which* held record answers it, to show the day it expires, so `answering` mirrors the service's match
// (title and kind ignoring case and surrounding space; a minimum level needs the same framework and a recorded level at least that)
// and its preference (verified and in date, then declared and in date, then expired). It never decides the standing itself.

import { compareText } from "./locale-format";
import type {
  LearningStatus,
  MyLearningRow,
  Qualification,
  QualificationGapRow,
} from "./api/types";

/** Most urgent first; `not_applicable` is never listed. */
const LEARNING_ORDER: LearningStatus[] = [
  "expired",
  "never_completed",
  "due_soon",
  "unknown",
  "compliant",
];

/** The requirements that apply to the person, most urgent first, then by subject. */
export function orderLearning(rows: readonly MyLearningRow[]): MyLearningRow[] {
  return rows
    .filter((r) => r.status !== "not_applicable")
    .sort(
      (a, b) =>
        LEARNING_ORDER.indexOf(a.status) - LEARNING_ORDER.indexOf(b.status) ||
        compareText(a.subject, b.subject),
    );
}

/** Whether the person has something to do for this requirement. Unknown is not "to do": it is HR's record to complete. */
export function learningNeedsAttention(status: LearningStatus): boolean {
  return (
    status === "expired" ||
    status === "never_completed" ||
    status === "due_soon"
  );
}

const CERT_ORDER: QualificationGapRow["standing"][] = [
  "expired",
  "missing",
  "declared_only",
  "met",
];

/** What the role asks for, most urgent first, then by title. */
export function orderCertifications(
  rows: readonly QualificationGapRow[],
): QualificationGapRow[] {
  return [...rows].sort(
    (a, b) =>
      CERT_ORDER.indexOf(a.standing) - CERT_ORDER.indexOf(b.standing) ||
      compareText(a.title, b.title),
  );
}

/** Whether the person has something to do for this requirement of their role (declared only waits on HR, but the person can help). */
export function certificationNeedsAttention(
  standing: QualificationGapRow["standing"],
): boolean {
  return standing !== "met";
}

const same = (a: string, b: string) =>
  a.trim().toLowerCase() === b.trim().toLowerCase();

/** Whether a held qualification is the one a requirement asks for (date and verification aside), as the service matches. */
export function matches(
  held: Qualification,
  req: QualificationGapRow,
): boolean {
  if (!same(held.title, req.title)) return false;
  if (req.kind !== null && !same(req.kind, held.kind)) return false;
  if (req.framework !== null && req.min_level !== null) {
    return (
      held.framework !== null &&
      same(held.framework, req.framework) &&
      held.level !== null &&
      held.level >= req.min_level
    );
  }
  return true;
}

function rank(held: Qualification, today: string): number {
  const inDate = held.expires_on === null || held.expires_on >= today;
  if (!inDate) return 1;
  return held.status === "verified" ? 3 : 2;
}

/** The held qualification that answers `req` on `today` (ISO dates compare as text), or null when none matches. */
export function answering(
  req: QualificationGapRow,
  held: readonly Qualification[],
  today: string,
): Qualification | null {
  let best: Qualification | null = null;
  for (const h of held.filter((q) => matches(q, req))) {
    if (best === null) {
      best = h;
      continue;
    }
    const byRank = rank(h, today) - rank(best, today);
    // Among equals, the one that lasts longest: no expiry beats any date, a later date beats an earlier one.
    const later =
      (best.expires_on ?? "") !== "" &&
      (h.expires_on === null || h.expires_on > best.expires_on!);
    if (byRank > 0 || (byRank === 0 && later)) best = h;
  }
  return best;
}
