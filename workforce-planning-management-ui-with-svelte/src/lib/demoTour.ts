// The guided tour of the demo: every page worth seeing, in an order that tells the story, grouped by the
// tour sections of the public tour page (`tour.<section>.title` / `.body`). Pure data and one helper, so the
// banner and the `/demo` page cannot disagree about the order.

import type { StringKey } from "./i18n.svelte";

export type DemoStep = {
  /** The page, without its locale prefix. */
  path: string;
  /** The catalogue key of the page's name (the same one the menu uses). */
  name: StringKey;
};

export type DemoSection = {
  /** The tour section id: its words are `tour.<id>.title` and `tour.<id>.body`. */
  id: string;
  steps: DemoStep[];
};

export const DEMO_SECTIONS: DemoSection[] = [
  {
    id: "hiring",
    steps: [{ path: "/requisitions", name: "nav.requisitions" }],
  },
  {
    id: "people",
    steps: [
      { path: "/workers", name: "nav.workers" },
      { path: "/directory", name: "nav.directory" },
      { path: "/org-chart", name: "nav.orgChart" },
      { path: "/groups", name: "nav.groups" },
      { path: "/movements", name: "nav.movements" },
      { path: "/job-levels", name: "nav.jobLevels" },
      { path: "/pay-scales", name: "nav.payScales" },
    ],
  },
  { id: "rota", steps: [{ path: "/rota", name: "nav.rota" }] },
  {
    id: "growth",
    steps: [
      { path: "/roles", name: "nav.roles" },
      { path: "/skills", name: "nav.skills" },
      { path: "/skill-gaps", name: "nav.skillGaps" },
      { path: "/learning", name: "nav.learning" },
      { path: "/development", name: "nav.development" },
      { path: "/cpd", name: "nav.cpd" },
      { path: "/mentorship", name: "nav.mentorship" },
      { path: "/me", name: "nav.me" },
      { path: "/kanban", name: "nav.kanban" },
    ],
  },
  { id: "wellbeing", steps: [{ path: "/wellbeing", name: "nav.wellbeing" }] },
  {
    id: "payroll",
    steps: [
      { path: "/payroll", name: "nav.payroll" },
      { path: "/expenses", name: "nav.expenseQueue" },
      { path: "/benchmarks", name: "nav.benchmarks" },
    ],
  },
  { id: "privacy", steps: [{ path: "/privacy", name: "nav.privacy" }] },
  {
    id: "dashboard",
    steps: [
      { path: "/ceo", name: "nav.ceo" },
      { path: "/metrics", name: "nav.metrics" },
      { path: "/announcements", name: "nav.announcements" },
    ],
  },
  {
    id: "yours",
    steps: [
      { path: "/planning", name: "nav.planning" },
      { path: "/workforce", name: "nav.workforce" },
      { path: "/contingent", name: "nav.contingent" },
      { path: "/capacity", name: "nav.capacity" },
      { path: "/resilience", name: "nav.resilience" },
      { path: "/oversight", name: "nav.oversight" },
      { path: "/cutover", name: "nav.cutover" },
      { path: "/warnings", name: "nav.warnings" },
      { path: "/change", name: "nav.change" },
    ],
  },
];

/** Every step in tour order. */
export const DEMO_STEPS: DemoStep[] = DEMO_SECTIONS.flatMap((s) => s.steps);

/**
 * Where `rest` (a path with its locale prefix removed) falls in the tour: its place, the step before and
 * the step after (null at either end). A page that is not a step has no place.
 */
export function stepAt(rest: string): {
  index: number;
  total: number;
  previous: DemoStep | null;
  next: DemoStep | null;
} | null {
  const index = DEMO_STEPS.findIndex(
    (s) => rest === s.path || rest.startsWith(`${s.path}/`),
  );
  if (index < 0) return null;
  return {
    index,
    total: DEMO_STEPS.length,
    previous: DEMO_STEPS[index - 1] ?? null,
    next: DEMO_STEPS[index + 1] ?? null,
  };
}
