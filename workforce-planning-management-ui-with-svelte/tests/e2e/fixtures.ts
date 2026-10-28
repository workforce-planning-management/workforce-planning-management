// The pages' stubbed API, shared by the before-and-after record (`page-shots.spec.ts`) and the specs for what the
// split pages do (`worker-panels.spec.ts`, `roles.spec.ts`). An unstubbed /api/proxy call 404s loudly.

import type { Page } from "@playwright/test";
import { WORKER, signIn } from "./support";

export const ORG = WORKER.organization_ref;

export const json = (page: Page, pattern: string, body: unknown) =>
  page.route(pattern, (route) => route.fulfill({ json: body }));

export async function base(page: Page) {
  await signIn(page);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await json(page, "**/api/proxy/me/organizations", [
    {
      pid: "88888888-8888-4888-8888-888888888888",
      person_ref: WORKER.person_ref,
      organization_ref: ORG,
      worker_pid: WORKER.pid,
      employed: true,
      role: "member",
      starts_on: "2026-01-05",
      ends_on: null,
    },
  ]);
  await json(page, "**/api/proxy/me/organizations/scope", [ORG]);
  await json(page, "**/api/proxy/workers", [WORKER]);
}

export async function stubWorkerPage(page: Page) {
  const w = `**/api/proxy/workers/${WORKER.pid}`;
  await json(page, w, { ...WORKER, location: "London" });
  await json(page, `${w}/onboarding`, [
    {
      pid: "o1",
      worker_pid: WORKER.pid,
      name: "Sign contract",
      mandatory: true,
      status: "done",
      waived_reason: null,
    },
    {
      pid: "o2",
      worker_pid: WORKER.pid,
      name: "Laptop issued",
      mandatory: true,
      status: "pending",
      waived_reason: null,
    },
  ]);
  await json(page, `${w}/leave-entitlements`, [
    {
      pid: "e1",
      worker_pid: WORKER.pid,
      kind: "annual",
      year: 2026,
      entitled_days: 25,
      used_days: 5,
    },
  ]);
  await json(page, `${w}/leave-requests`, [
    {
      pid: "l1",
      worker_pid: WORKER.pid,
      kind: "annual",
      start_on: "2026-11-02",
      end_on: "2026-11-06",
      days: 5,
      status: "requested",
      negative_balance: false,
    },
  ]);
  await json(page, `${w}/payslips`, []);
  await json(page, `${w}/reviews`, []);
  await json(page, `${w}/training-enrollments`, []);
  await json(page, `${w}/wellbeing-prompts`, {
    prompts: [
      {
        kind: "prompt",
        entitlement_kind: "health",
        benefit_plan_pid: null,
        entitlement_pid: "p1",
        name: "Flu vaccination",
        description: "Free at the clinic.",
        info_url: null,
        doses: 1,
        response: null,
      },
    ],
  });
  await json(page, "**/api/proxy/pulse-surveys", [
    {
      pid: "s1",
      name: "October pulse",
      question: "How was your week?",
      active_from: null,
      active_until: null,
      open: true,
    },
  ]);
  await json(page, `${w}/appraisals`, [
    {
      pid: "a1",
      status: "collecting",
      competencies: ["communication", "delivery"],
      shared_on: null,
      nominated: 3,
      responded: 1,
    },
  ]);
  await json(page, `${w}/appraisal-requests`, [
    {
      appraisal_pid: "a9",
      subject_pid: "44444444-4444-4444-8444-444444444444",
      subject: "Masked Person",
      group: "peer",
      competencies: ["communication"],
    },
  ]);
  await json(page, `${w}/notifications`, [
    {
      pid: "n1",
      kind: "leave_decided",
      body: "Your leave was approved.",
      data: {},
      created_at: "2026-10-20T09:00:00Z",
      read_at: null,
    },
  ]);
  await json(page, `${w}/ergonomic-assessments`, [
    {
      pid: "g1",
      workstation: "Desk 4",
      status: "open",
      assessed_on: null,
      open_issues: 1,
      items: [],
    },
  ]);
  await json(page, `${w}/adjustment-requests`, [
    {
      pid: "j1",
      category: "quieter_workspace",
      status: "requested",
      barrier: "Open-plan noise",
      impact: "Hard to concentrate",
      adjustment: "A quiet desk",
      decided_on: null,
      words_withheld: false,
    },
  ]);
}

export async function stubPlanning(page: Page) {
  const PLAN = "11111111-1111-4111-8111-111111111111";
  const plan = {
    pid: PLAN,
    name: "FY27",
    organization_ref: ORG,
    horizon_start: "2026-01-01",
    horizon_end: "2027-12-31",
    status: "active",
    attrition_bp: 1000,
    demand_lines: 1,
  };
  await json(page, "**/api/proxy/role-profiles**", [
    {
      pid: "r1",
      job_title: "Support Agent",
      description: null,
      source_ref: null,
      requirement_count: 1,
      framework: null,
      profession: null,
      role_name: null,
    },
  ]);
  await json(page, "**/api/proxy/workforce-plans", [plan]);
  await json(page, `**/api/proxy/workforce-plans/${PLAN}`, {
    ...plan,
    rationale: "Grow support",
    budget_minor: 10000000,
    budget_currency: "GBP",
    on_cost_bp: 2000,
    demand_lines: [
      {
        pid: "d1",
        department: "support",
        role_profile_pid: "r1",
        job_title: "Support Agent",
        target_on: "2026-12-31",
        target_headcount: 12,
        target_fte: null,
        basis: "permanent",
        expected_ends_on: null,
        programme_ref: null,
        note: null,
        objective_pids: ["b1"],
      },
    ],
    objectives: [
      { pid: "b1", title: "Cut response time", owner: "Head of Support" },
    ],
  });
  await json(page, `**/api/proxy/workforce-plans/${PLAN}/forecast`, {
    derivation: "Headcount now, less attrition, against planned.",
    as_of: "2026-10-20",
    plan: { pid: PLAN, name: "FY27", status: "active" },
    assumptions: {
      attrition_bp: 1000,
      attrition_source: "plan_assumption",
      hires_assumed: 0,
    },
    departments: [],
    programmes: [],
  });
  await json(page, `**/api/proxy/workforce-plans/${PLAN}/alignment`, {
    derivation: "",
    planned_headcount: 12,
    aligned_share: null,
    objectives: 1,
    unresourced_objectives: [],
    unaligned_demand_lines: [],
    critical_roles_without_bench: {
      in_plan_departments: 0,
      all_departments: 0,
    },
  });
  await json(page, `**/api/proxy/workforce-plans/${PLAN}/escalations`, []);
  await json(page, `**/api/proxy/workforce-plans/${PLAN}/cost`, {
    derivation: "",
    currency: "GBP",
    salary_visible: true,
    assumptions: { on_cost_bp: 2000 },
    groups: [],
    total_annual_cost_minor: 0,
    uncosted_groups: 0,
    affordability: null,
  });
  await json(page, `**/api/proxy/workforce-plans/${PLAN}/tolerances`, {
    derivation: "a tolerance is the largest deviation the plan may have",
    as_of: "2026-10-20",
    overall: "amber",
    breaches_without_escalation: 0,
    dimensions: [
      {
        dimension: "time",
        unit: "calendar_days",
        set: true,
        limit: 120,
        amber_percent: 75,
        deviation: 100,
        rag: "amber",
        basis: "days past the horizon",
        reason: null,
        escalation: null,
      },
    ],
  });
  await json(page, "**/api/proxy/workforce-plans/timeline**", { plans: [] });
}

export async function stubRoles(page: Page) {
  const R1 = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
  await json(page, "**/api/proxy/skills", [
    {
      pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
      name: "Prompting",
      category: "technical",
      external_refs: [],
    },
  ]);
  await json(page, "**/api/proxy/role-profiles**", [
    {
      pid: R1,
      job_title: "Support Agent",
      description: "First line",
      source_ref: null,
      requirement_count: 1,
      framework: null,
      profession: null,
      role_name: null,
      level_order: null,
    },
  ]);
  await json(page, "**/api/proxy/capability-frameworks", []);
  await json(page, "**/api/proxy/mobility/interest-summary", {
    derivation: "Counts only.",
    targets: [],
  });
  await json(page, `**/api/proxy/role-profiles/${R1}`, {
    pid: R1,
    job_title: "Support Agent",
    description: "First line",
    source_ref: null,
    framework: null,
    profession: null,
    role_name: null,
    level_name: null,
    level_order: null,
    management_track: false,
    requirements: [
      {
        skill_pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        skill: "Prompting",
        category: "technical",
        min_proficiency: 3,
        importance: "important",
        note: null,
        source_level: null,
        source_scale_max: null,
      },
    ],
  });
  await json(page, `**/api/proxy/role-profiles/${R1}/gap`, {
    derivation: "Declarations against the requirement.",
    role: { pid: R1, job_title: "Support Agent" },
    headcount: 12,
    requirements: [
      {
        skill: "Prompting",
        importance: "important",
        min_proficiency: 3,
        meeting: 2,
        below: 7,
        undeclared: 3,
        coverage: null,
      },
    ],
  });
  await json(page, `**/api/proxy/role-profiles/${R1}/progression`, {
    profile: { pid: R1, job_title: "Support Agent", level_order: 1 },
    next: [],
  });
  await json(page, `**/api/proxy/role-profiles/${R1}/grade`, {
    profile_pid: R1,
    job_level: null,
    pay_band: null,
  });
  await json(page, "**/api/proxy/job-levels", []);
  await json(page, "**/api/proxy/pay-scales", []);
}

/** The positions page: an establishment with an over-establishment beside a vacancy, and three seats. */
export async function stubPositions(page: Page) {
  const seat = (n: number, extra: Record<string, unknown>) => ({
    pid: `aaaaaaaa-aaaa-4aaa-8aaa-00000000000${n}`,
    organization_ref: WORKER.organization_ref,
    cost_centre: "CC-100",
    title: "Support Engineer",
    budget_fte_percent: 100,
    funding_kind: "core",
    funding_ends_on: null,
    programme_ref: null,
    requisition_pid: null,
    demand_line_pid: null,
    ...extra,
  });
  const fig = (percent: number) => ({ status: "known", percent });
  await page.route(
    "**/api/proxy/positions/establishment?organization=*",
    (route) =>
      route.fulfill({
        json: {
          small_group_floor: 10,
          cost_centres: [
            {
              cost_centre: "CC-100",
              positions: 3,
              budgeted: fig(250),
              held: fig(160),
              vacant: fig(140),
              over: fig(50),
              frozen_vacant: fig(0),
              by_state: { filled: 2, open: 1 },
            },
            {
              cost_centre: "CC-200",
              positions: 1,
              budgeted: fig(100),
              held: { status: "withheld" },
              vacant: { status: "withheld" },
              over: { status: "withheld" },
              frozen_vacant: { status: "withheld" },
              by_state: { frozen: 1 },
            },
          ],
          derivation: "",
        },
      }),
  );
  await page.route("**/api/proxy/positions?organization=*", (route) =>
    route.fulfill({
      json: {
        states: [],
        positions: [
          seat(1, { state: "filled", held: true, budget_fte_percent: 50 }),
          seat(2, { state: "filled", held: true }),
          seat(3, { state: "open", held: false, title: "Analyst" }),
          seat(4, {
            state: "frozen",
            held: false,
            cost_centre: "CC-200",
            title: "Coordinator",
          }),
        ],
      },
    }),
  );
}

/** The organization comparison page: one plan, a flatter target, a small role withheld and a role moved. */
export async function stubOrgShape(page: Page) {
  const plan = {
    pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    name: "Flatten",
    organization_ref: WORKER.organization_ref,
    horizon_start: "2026-01-01",
    horizon_end: "2027-12-31",
    status: "draft",
    attrition_bp: null,
    demand_lines: 0,
  };
  const role = (title: string, extra: Record<string, unknown>) => ({
    title,
    current: { status: "shown", value: 0 },
    target: 0,
    change: "same",
    delta: 0,
    moved: false,
    current_departments: [],
    target_departments: [],
    ...extra,
  });
  await page.route("**/api/proxy/workforce-plans", (route) =>
    route.fulfill({ json: [plan] }),
  );
  await page.route(
    `**/api/proxy/workforce-plans/${plan.pid}/organization-diff`,
    (route) =>
      route.fulfill({
        json: {
          plan: plan.pid,
          target_set: true,
          small_group_floor: 10,
          derivation: "",
          diff: {
            current: {
              layers: 3,
              headcount: 13,
              span: { managers: 3, mean_hundredths: 400, max: 5 },
            },
            target: {
              layers: 3,
              headcount: 15,
              span: { managers: 2, mean_hundredths: 700, max: 12 },
            },
            roles: [
              role("Engineer", {
                current: { status: "shown", value: 10 },
                target: 12,
                change: "larger",
                delta: 2,
              }),
              role("Manager", {
                current: { status: "withheld" },
                target: 1,
                change: "smaller",
                delta: null,
              }),
              role("Head", {
                current: { status: "withheld" },
                target: 1,
                change: "same",
                delta: null,
                moved: true,
                current_departments: ["exec"],
                target_departments: ["leadership"],
              }),
              role("Product Lead", { target: 1, change: "added", delta: 1 }),
            ],
          },
        },
      }),
  );
}
