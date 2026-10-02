import { describe, expect, it } from "vitest";

import type { OrgNode } from "../../src/lib/api/types";
import {
  byDepartment,
  byLevel,
  byLocation,
  byTenure,
  flatten,
  tenureLabel,
} from "../../src/lib/orgViews";

const node = (
  pid: string,
  department: string,
  reports: OrgNode[] = [],
  tenure = "1_to_3y",
  location: string | null = null,
): OrgNode => ({
  pid,
  display_name: pid,
  job_title: "x",
  department,
  tenure,
  location,
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

  it("byTenure orders bands longest-serving first, unknown last", () => {
    const tenured = [
      node("a", "x", [node("b", "x", [], "over_10y")], "under_1y"),
      node("c", "x", [], "mystery"),
      node("d", "x", [], "not_started"),
    ];
    expect(byTenure(tenured).map((g) => g.band)).toEqual([
      "over_10y",
      "under_1y",
      "not_started",
      "mystery",
    ]);
    expect(tenureLabel("over_10y")).toBe("10+");
    expect(tenureLabel("not_started")).toBe("—");
  });

  it("byLocation sorts places and keeps unknown last", () => {
    const placed = [
      node("a", "x", [node("b", "x", [], "1_to_3y", "Cardiff")], "1_to_3y", "Bristol"),
      node("c", "x"),
      node("d", "x", [], "1_to_3y", "Cardiff"),
    ];
    const groups = byLocation(placed);
    expect(groups.map((g) => g.location)).toEqual(["Bristol", "Cardiff", null]);
    expect(groups[1]?.members.map((m) => m.pid)).toEqual(["b", "d"]);
    expect(groups.flatMap((g) => g.members)).toHaveLength(4);
  });

  it("an empty forest yields empty views", () => {
    expect(flatten([])).toEqual([]);
    expect(byDepartment([])).toEqual([]);
    expect(byLevel([])).toEqual([]);
    expect(byTenure([])).toEqual([]);
    expect(byLocation([])).toEqual([]);
  });
});
