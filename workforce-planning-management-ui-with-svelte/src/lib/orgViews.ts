// Org-chart view modes. The service derives one thing — the manager
// forest (WPM-T2); these helpers only *regroup the same nodes* for
// display, so every view lists exactly the same people.

import type { OrgNode } from "./api/types";

export const ORG_VIEWS = ["manager", "department", "level"] as const;
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
