// Org-chart view modes. The service derives one thing — the manager
// forest (WPM-T2); these helpers only *regroup the same nodes* for
// display, so every view lists exactly the same people.

import type { OrgNode } from "./api/types";

export const ORG_VIEWS = ["manager", "department", "level", "tenure", "location"] as const;
export type OrgView = (typeof ORG_VIEWS)[number];

/** A node with its depth in the manager forest (a root is level 1). */
export interface OrgEntry {
  node: OrgNode;
  level: number;
}

/** Every node in the forest, depth-first, with its level. */
export function flatten(roots: OrgNode[], level = 1): OrgEntry[] {
  return roots.flatMap((node) => [
    { node, level },
    ...flatten(node.reports, level + 1),
  ]);
}

/** Nodes grouped by department, departments and members sorted by name. */
export function byDepartment(
  roots: OrgNode[],
): Array<{ department: string; members: OrgNode[] }> {
  const groups = new Map<string, OrgNode[]>();
  for (const { node } of flatten(roots)) {
    const members = groups.get(node.department) ?? [];
    members.push(node);
    groups.set(node.department, members);
  }
  return [...groups.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([department, members]) => ({
      department,
      members: members.sort((a, b) =>
        a.display_name.localeCompare(b.display_name),
      ),
    }));
}

/** Nodes grouped by depth, top of the organization first. */
export function byLevel(
  roots: OrgNode[],
): Array<{ level: number; members: OrgNode[] }> {
  const groups = new Map<number, OrgNode[]>();
  for (const { node, level } of flatten(roots)) {
    const members = groups.get(level) ?? [];
    members.push(node);
    groups.set(level, members);
  }
  return [...groups.entries()]
    .sort(([a], [b]) => a - b)
    .map(([level, members]) => ({
      level,
      members: members.sort((a, b) =>
        a.display_name.localeCompare(b.display_name),
      ),
    }));
}

/** Tenure bands, longest-serving first; unknown bands sort last. */
export const TENURE_BANDS = [
  "over_10y",
  "5_to_10y",
  "3_to_5y",
  "1_to_3y",
  "under_1y",
  "not_started",
] as const;

/** A language-neutral label for a tenure band, in years. */
export function tenureLabel(band: string): string {
  switch (band) {
    case "over_10y":
      return "10+";
    case "5_to_10y":
      return "5–10";
    case "3_to_5y":
      return "3–5";
    case "1_to_3y":
      return "1–3";
    case "under_1y":
      return "<1";
    default:
      return "—";
  }
}

/** Nodes grouped by tenure band, longest-serving first. */
export function byTenure(
  roots: OrgNode[],
): Array<{ band: string; members: OrgNode[] }> {
  const groups = new Map<string, OrgNode[]>();
  for (const { node } of flatten(roots)) {
    const members = groups.get(node.tenure) ?? [];
    members.push(node);
    groups.set(node.tenure, members);
  }
  const rank = (band: string) => {
    const i = (TENURE_BANDS as readonly string[]).indexOf(band);
    return i === -1 ? TENURE_BANDS.length : i;
  };
  return [...groups.entries()]
    .sort(([a], [b]) => rank(a) - rank(b))
    .map(([band, members]) => ({
      band,
      members: members.sort((a, b) =>
        a.display_name.localeCompare(b.display_name),
      ),
    }));
}

/**
 * Nodes grouped by work location, locations sorted by name, with the
 * people whose location is not recorded in a separate trailing group
 * (`location: null`) — "unknown" is not a place.
 */
export function byLocation(
  roots: OrgNode[],
): Array<{ location: string | null; members: OrgNode[] }> {
  const groups = new Map<string | null, OrgNode[]>();
  for (const { node } of flatten(roots)) {
    const key = node.location ?? null;
    const members = groups.get(key) ?? [];
    members.push(node);
    groups.set(key, members);
  }
  return [...groups.entries()]
    .sort(([a], [b]) => {
      if (a === null) return b === null ? 0 : 1;
      if (b === null) return -1;
      return a.localeCompare(b);
    })
    .map(([location, members]) => ({
      location,
      members: members.sort((a, b) =>
        a.display_name.localeCompare(b.display_name),
      ),
    }));
}
