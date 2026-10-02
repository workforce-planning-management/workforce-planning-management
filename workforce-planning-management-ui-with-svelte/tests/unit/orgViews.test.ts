import { describe, expect, it } from "vitest";

import type { OrgNode } from "../../src/lib/api/types";
import { byDepartment, byLevel, flatten } from "../../src/lib/orgViews";

const node = (
  pid: string,
  department: string,
  reports: OrgNode[] = [],
): OrgNode => ({
  pid,
  display_name: pid,
  job_title: "x",
  department,
  reports,
});

const forest = [
  node("ceo", "exec", [
    node("eng-lead", "eng", [node("dev-b", "eng"), node("dev-a", "eng")]),
    node("ops-lead", "ops"),
  ]),
];

describe("org views", () => {
  it("flatten lists every node once with its depth", () => {
    expect(flatten(forest).map((e) => [e.node.pid, e.level])).toEqual([
      ["ceo", 1],
      ["eng-lead", 2],
      ["dev-b", 3],
      ["dev-a", 3],
      ["ops-lead", 2],
    ]);
  });

  it("byDepartment groups and sorts without losing anyone", () => {
    const groups = byDepartment(forest);
    expect(groups.map((g) => g.department)).toEqual(["eng", "exec", "ops"]);
    expect(groups[0]?.members.map((m) => m.pid)).toEqual([
      "dev-a",
      "dev-b",
      "eng-lead",
    ]);
    expect(groups.flatMap((g) => g.members)).toHaveLength(5);
  });

  it("byLevel groups by depth, top first", () => {
    const levels = byLevel(forest);
    expect(levels.map((l) => [l.level, l.members.length])).toEqual([
      [1, 1],
      [2, 2],
      [3, 2],
    ]);
  });

  it("an empty forest yields empty views", () => {
    expect(flatten([])).toEqual([]);
    expect(byDepartment([])).toEqual([]);
    expect(byLevel([])).toEqual([]);
  });
});
