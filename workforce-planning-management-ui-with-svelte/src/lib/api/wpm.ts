// Typed WPM API surface over the BFF proxy, plus the family `money()`
// formatter. Paths mirror the service routes one-to-one — the
// Playwright suite stubs these exact paths, so drift fails loudly.

import { api } from "#lib/api/client.js";
import type {
  Application,
  Benchmark,
  Announcement,
  Backup,
  ComparisonRow,
  Cover,
  RotaSummary,
  RotaView,
  SwapRequest,
  DirectoryEntry,
  EmergencyContact,
  Worker,
  LeaveEntitlement,
  LeaveRequest,
  MyOrganization,
  OnboardingItem,
  OrgNode,
  Payslip,
  PayrollRun,
  Ratio,
  Requisition,
  Review,
  SuccessionEntry,
  TrainingEnrollment,
} from "#lib/api/types.js";

type FetchLike = { fetch?: typeof fetch };

/**
 * Format minor units + ISO-4217 as a locale-aware money string.
 * `null` renders as an em dash (the masked/absent state) — never 0,
 * which would be a lie.
 */
export function money(
  minor: number | null | undefined,
  currency: string | null | undefined,
  locale?: string,
): string {
  if (minor === null || minor === undefined || !currency) return "—";
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency,
  }).format(minor / 100);
}

/**
 * The signed-in caller's own organization memberships — every org
 * they belong to, at once (no switcher). Empty when signed out.
 */
export function listMyOrganizations(
  init?: FetchLike,
): Promise<MyOrganization[]> {
  return api("/me/organizations", init);
}

/**
 * Every organization the caller can read, expanded through org
 * confederation: each membership's own organization plus every
 * transitive descendant. A flat list of refs, not membership rows —
 * a membership in a parent confederation yields one row from
 * {@link listMyOrganizations} but several entries here. Pages that
 * render one section per readable organization (`/org-chart`,
 * `/benchmarks`) use this, not `listMyOrganizations`.
 */
export function listMyOrganizationScope(init?: FetchLike): Promise<string[]> {
  return api("/me/organizations/scope", init);
}

/** Workers list (optionally filtered). */
export function listWorkers(
  filters?: { department?: string; status?: string },
  init?: FetchLike,
): Promise<Worker[]> {
  const params = new URLSearchParams();
  if (filters?.department) params.set("department", filters.department);
  if (filters?.status) params.set("status", filters.status);
  const qs = params.size ? `?${params}` : "";
  return api(`/workers${qs}`, init);
}

/** One worker. */
export function getWorker(pid: string, init?: FetchLike): Promise<Worker> {
  return api(`/workers/${pid}`, init);
}

/** One worker lifecycle transition. */
export function changeStatus(pid: string, to: string): Promise<Worker> {
  return api(`/workers/${pid}/status`, { method: "POST", body: { to } });
}

/**
 * The employee directory: employed workers in the caller's organizations,
 * by name. `q` searches name, title, department, location and manager.
 */
export function employeeDirectory(
  filters?: { q?: string; department?: string; limit?: number },
  init?: FetchLike,
): Promise<DirectoryEntry[]> {
  const params = new URLSearchParams();
  if (filters?.q) params.set("q", filters.q);
  if (filters?.department) params.set("department", filters.department);
  if (filters?.limit) params.set("limit", String(filters.limit));
  const qs = params.size ? `?${params}` : "";
  return api(`/directory${qs}`, init);
}

/** The manager forest for one organization. */
export function orgChart(
  organization: string,
  init?: FetchLike,
): Promise<OrgNode[]> {
  return api(
    `/org-chart?organization=${encodeURIComponent(organization)}`,
    init,
  );
}

/** Requisitions (optionally by status). */
export function listRequisitions(
  status?: string,
  init?: FetchLike,
): Promise<Requisition[]> {
  return api(`/requisitions${status ? `?status=${status}` : ""}`, init);
}

/** One requisition. */
export function getRequisition(
  pid: string,
  init?: FetchLike,
): Promise<Requisition> {
  return api(`/requisitions/${pid}`, init);
}

/** One requisition transition. */
export function requisitionStatus(
  pid: string,
  to: string,
): Promise<Requisition> {
  return api(`/requisitions/${pid}/status`, { method: "POST", body: { to } });
}

/** A requisition's applications. */
export function listApplications(
  pid: string,
  init?: FetchLike,
): Promise<Application[]> {
  return api(`/requisitions/${pid}/applications`, init);
}

/** One application stage transition. */
export function applicationStage(
  pid: string,
  body: {
    to: string;
    worker_number?: string;
    salary_minor?: number;
    salary_currency?: string;
  },
): Promise<{ pid: string; stage: string; worker_pid: string | null }> {
  return api(`/applications/${pid}/stage`, { method: "POST", body });
}

/** A worker's onboarding checklist. */
export function listOnboarding(
  pid: string,
  init?: FetchLike,
): Promise<OnboardingItem[]> {
  return api(`/workers/${pid}/onboarding`, init);
}

/** Complete one checklist item. */
export function completeItem(pid: string): Promise<OnboardingItem> {
  return api(`/onboarding-items/${pid}/complete`, { method: "POST" });
}

/** A worker's leave balances. */
export function listEntitlements(
  pid: string,
  init?: FetchLike,
): Promise<LeaveEntitlement[]> {
  return api(`/workers/${pid}/leave-entitlements`, init);
}

/** A worker's leave requests. */
export function listLeaveRequests(
  pid: string,
  init?: FetchLike,
): Promise<LeaveRequest[]> {
  return api(`/workers/${pid}/leave-requests`, init);
}

/** Decide one leave request. */
export function decideLeave(
  pid: string,
  decision: "approve" | "reject" | "cancel",
): Promise<LeaveRequest> {
  return api(`/leave-requests/${pid}/${decision}`, { method: "POST" });
}

/** The rota (shifts + assignments). */
export function listShifts(
  filters?: { department?: string; date?: string },
  init?: FetchLike,
): Promise<
  {
    shift: {
      pid: string;
      department: string;
      starts_at: string;
      ends_at: string;
      required_headcount: number;
    };
    assignments: { pid: string; worker_pid: string }[];
  }[]
> {
  const params = new URLSearchParams();
  if (filters?.department) params.set("department", filters.department);
  if (filters?.date) params.set("date", filters.date);
  const qs = params.size ? `?${params}` : "";
  return api(`/shifts${qs}`, init);
}

/** A worker's reviews. */
export function listReviews(pid: string, init?: FetchLike): Promise<Review[]> {
  return api(`/workers/${pid}/reviews`, init);
}

/** A worker's training enrolments. */
export function listTraining(
  pid: string,
  init?: FetchLike,
): Promise<TrainingEnrollment[]> {
  return api(`/workers/${pid}/training-enrollments`, init);
}

/** Certificates expiring within the window. */
export function expiringTraining(
  withinDays: number,
  init?: FetchLike,
): Promise<{ as_of: string; horizon: string; expiring: TrainingEnrollment[] }> {
  return api(`/training/expiring?within_days=${withinDays}`, init);
}

/** Succession plans with ranked candidates. */
export function listSuccession(init?: FetchLike): Promise<SuccessionEntry[]> {
  return api("/succession-plans", init);
}

/** The succession gap report. */
export function successionGaps(
  init?: FetchLike,
): Promise<{ gaps: SuccessionEntry["plan"][] }> {
  return api("/succession-plans/gaps", init);
}

/** Payroll runs. */
export function listRuns(init?: FetchLike): Promise<PayrollRun[]> {
  return api("/payroll-runs", init);
}

/** One payroll run. */
export function getRun(pid: string, init?: FetchLike): Promise<PayrollRun> {
  return api(`/payroll-runs/${pid}`, init);
}

/** One run action (calculate / approve / pay / reopen). */
export function runAction(
  pid: string,
  action: "calculate" | "approve" | "pay" | "reopen",
): Promise<PayrollRun> {
  return api(`/payroll-runs/${pid}/${action}`, { method: "POST" });
}

/** A run's payslips. */
export function runPayslips(pid: string, init?: FetchLike): Promise<Payslip[]> {
  return api(`/payroll-runs/${pid}/payslips`, init);
}

/** One worker's payslips (self-service). */
export function workerPayslips(
  pid: string,
  init?: FetchLike,
): Promise<Payslip[]> {
  return api(`/workers/${pid}/payslips`, init);
}

/** Benchmarks. */
export function listBenchmarks(init?: FetchLike): Promise<Benchmark[]> {
  return api("/benchmarks", init);
}

/** The benchmark comparison (flags only). */
export function benchmarkComparison(
  organization: string,
  init?: FetchLike,
): Promise<{ organization: string; rows: ComparisonRow[] }> {
  return api(
    `/benchmarks/comparison?organization=${encodeURIComponent(organization)}`,
    init,
  );
}

// ─── Learning & development ─────────────────────────────────────────

/** The skills catalog. */
export function listSkills(init?: FetchLike): Promise<
  Array<{
    pid: string;
    name: string;
    category: string;
    external_refs: Array<{
      framework: string;
      ref: string;
      label: string | null;
      version: string | null;
    }>;
  }>
> {
  return api("/skills", init);
}

/** Skill categories (mirror `rules::learning::SKILL_CATEGORIES`). */
export const SKILL_CATEGORIES = [
  "technical",
  "leadership",
  "compliance",
  "domain",
  "other",
] as const;

/** Rename and/or recategorise a skill. */
export function updateSkill(
  pid: string,
  body: { name?: string; category?: string },
): Promise<unknown> {
  return api(`/skills/${pid}`, { method: "PUT", body });
}

/** Where a skill is used, and whether it can be deleted. */
export function skillUsage(
  pid: string,
  init?: FetchLike,
): Promise<{
  declared_by: number;
  required_by_profiles: number;
  in_development_plans: number;
  in_initiatives: number;
  total: number;
  deletable: boolean;
}> {
  return api(`/skills/${pid}/usage`, init);
}

/** Delete a skill nothing uses (the service refuses a skill in use). */
export function deleteSkill(pid: string): Promise<unknown> {
  return api(`/skills/${pid}`, { method: "DELETE" });
}

/** Fold a skill into another, keeping the stronger statement. */
export function mergeSkill(
  pid: string,
  intoPid: string,
): Promise<{ merged_into: string; records_moved: number; records_merged: number }> {
  return api(`/skills/${pid}/merge`, { method: "POST", body: { into_pid: intoPid } });
}

/** Keyword-rule category suggestions for skills still `other`. */
export function categorySuggestions(init?: FetchLike): Promise<{
  derivation: string;
  suggestions: Array<{
    pid: string;
    name: string;
    suggested: string;
    keyword: string;
  }>;
  unsuggested: number;
}> {
  return api("/skills/category-suggestions", init);
}

/** Apply the current suggestion to the chosen skills. */
export function applyCategorySuggestions(
  skillPids: string[],
): Promise<{ applied: number; skipped: number }> {
  return api("/skills/category-suggestions/apply", {
    method: "POST",
    body: { skill_pids: skillPids },
  });
}

/** Declare (upsert) a worker's proficiency in a skill (1-5). */
export function declareSkill(
  workerPid: string,
  body: { skill_pid: string; proficiency: number; target?: number },
): Promise<unknown> {
  return api(`/workers/${workerPid}/skills`, { method: "PUT", body });
}

/** The per-department skills matrix + gaps. */
export function skillsMatrix(init?: FetchLike): Promise<{
  as_of: string;
  note: string;
  matrix: Array<{
    department: string;
    skill: string | null;
    workers: number;
    average_proficiency: number;
    below_target: number;
  }>;
  gaps: Array<{
    worker_pid: string;
    department: string;
    skill: string | null;
    proficiency: number;
    target: number | null;
  }>;
}> {
  return api("/learning/skills-matrix", init);
}

/** Per-department training analytics. */
export function trainingAnalytics(init?: FetchLike): Promise<{
  as_of: string;
  horizon: string;
  note: string;
  departments: Array<{
    department: string;
    by_status: Record<string, number>;
    completion_rate: Ratio;
    certs_expiring: number;
  }>;
}> {
  return api("/learning/training-analytics", init);
}

/** Learning paths with step counts. */
export function listPaths(
  init?: FetchLike,
): Promise<
  Array<{ pid: string; name: string; summary: string | null; steps: number }>
> {
  return api("/learning-paths", init);
}

/** One path's per-member honest progress. */
export function pathProgress(
  pathPid: string,
  init?: FetchLike,
): Promise<{
  as_of: string;
  path: { pid: string; name: string };
  steps: Array<{ course_ref: string; title: string; position: number }>;
  derivation: string;
  members: Array<{
    worker_pid: string;
    display_name: string | null;
    completed_steps: number;
    total_steps: number;
  }>;
}> {
  return api(`/learning-paths/${pathPid}/progress`, init);
}

/** Strategic skill depth: per skill and per category (capability analysis). */
export function capabilityAnalysis(
  thresholds?: { minProficiency?: number; minDepth?: number },
  init?: FetchLike,
): Promise<{
  derivation: string;
  thresholds: { min_proficiency: number; min_depth: number };
  headcount: number;
  skills_in_catalog: number;
  adequately_covered: Ratio | null;
  by_category: Record<string, Record<string, number>>;
  skills: Array<{
    skill: string;
    category: string;
    declared_by: number;
    proficient: number;
    proficient_departments: number;
    proficient_share: Ratio | null;
    status: "undeclared" | "no_proficient" | "thin" | "adequate";
  }>;
}> {
  const query = new URLSearchParams();
  if (thresholds?.minProficiency) {
    query.set("min_proficiency", String(thresholds.minProficiency));
  }
  if (thresholds?.minDepth) query.set("min_depth", String(thresholds.minDepth));
  const qs = query.toString();
  return api(
    `/workforce-intelligence/capability-analysis${qs ? `?${qs}` : ""}`,
    init,
  );
}

/** Role profiles (what a role requires): list, optionally one framework's. */
export function listRoleProfiles(
  framework?: string,
  init?: FetchLike,
): Promise<
  Array<{
    pid: string;
    job_title: string;
    description: string | null;
    source_ref: string | null;
    requirement_count: number;
    framework: string | null;
    profession: string | null;
    role_name: string | null;
    level_name: string | null;
    level_order: number | null;
    management_track: boolean;
  }>
> {
  const qs = framework ? `?framework=${encodeURIComponent(framework)}` : "";
  return api(`/role-profiles${qs}`, init);
}

/** Frameworks role profiles were imported from, with attribution. */
export function listFrameworks(init?: FetchLike): Promise<
  Array<{
    slug: string;
    name: string;
    source_url: string | null;
    licence: string | null;
    attribution: string | null;
    scale_max: number;
    scale_labels: string | null;
    imported_on: string;
    note: string | null;
    profiles: number;
    roles: number;
  }>
> {
  return api("/capability-frameworks", init);
}

/** What changes going up a level in the same role of the same framework. */
export function roleProgression(
  pid: string,
  init?: FetchLike,
): Promise<{
  profile: { pid: string; job_title: string; level_order: number } | string;
  next: Array<{
    pid: string;
    job_title: string;
    level_order: number;
    management_track: boolean;
    added: Array<{ skill: string; min_proficiency: number }>;
    raised: Array<{ skill: string; from: number; to: number }>;
    unchanged: number;
    dropped: string[];
  }>;
}> {
  return api(`/role-profiles/${pid}/progression`, init);
}

/** A role profile with its required skills, critical first. */
export function getRoleProfile(
  pid: string,
  init?: FetchLike,
): Promise<{
  pid: string;
  job_title: string;
  description: string | null;
  source_ref: string | null;
  framework: {
    slug: string;
    name: string;
    licence: string | null;
    attribution: string | null;
    scale_max: number;
    scale_labels: string | null;
    note: string | null;
  } | null;
  profession: string | null;
  role_name: string | null;
  level_name: string | null;
  level_order: number | null;
  management_track: boolean;
  requirements: Array<{
    skill_pid: string;
    skill: string | null;
    category: string | null;
    min_proficiency: number;
    importance: "critical" | "important" | "useful";
    note: string | null;
    source_level: number | null;
    source_scale_max: number | null;
  }>;
}> {
  return api(`/role-profiles/${pid}`, init);
}

/** Create a role profile for a job title. */
export function createRoleProfile(body: {
  job_title: string;
  description?: string;
  source_ref?: string;
}): Promise<{ pid: string }> {
  return api("/role-profiles", { method: "POST", body });
}

/** Require a skill at a minimum proficiency (1-5); upsert. */
export function setRoleRequirement(
  profilePid: string,
  body: {
    skill_pid: string;
    min_proficiency: number;
    importance: string;
    note?: string;
  },
): Promise<unknown> {
  return api(`/role-profiles/${profilePid}/requirements`, {
    method: "PUT",
    body,
  });
}

/** Drop a requirement from a role profile. */
export function removeRoleRequirement(
  profilePid: string,
  skillPid: string,
): Promise<unknown> {
  return api(`/role-profiles/${profilePid}/requirements/${skillPid}`, {
    method: "DELETE",
  });
}

/** CPD requirement units / entry categories (mirror `rules::cpd`). */
export const CPD_UNITS = ["hours", "points"] as const;
export const CPD_CATEGORIES = [
  "course",
  "conference",
  "reading",
  "mentoring",
  "on_the_job",
  "self_study",
  "other",
] as const;

/** CPD requirements (hours or points per period). */
export function listCpdRequirements(init?: FetchLike): Promise<
  Array<{
    pid: string;
    name: string;
    unit: string;
    required: number;
    period_start: string;
    period_end: string;
    job_title: string | null;
  }>
> {
  return api("/cpd-requirements", init);
}

/** Define a CPD requirement for a period. */
export function createCpdRequirement(body: {
  name: string;
  unit: string;
  required: number;
  period_start: string;
  period_end: string;
  job_title?: string;
}): Promise<{ pid: string }> {
  return api("/cpd-requirements", { method: "POST", body });
}

/** A worker's CPD ledger. */
export function listCpdEntries(
  workerPid: string,
  init?: FetchLike,
): Promise<
  Array<{
    pid: string;
    entry_date: string;
    activity: string;
    category: string;
    unit: string;
    amount: number;
    evidence_note: string | null;
    evidence_url: string | null;
    source: string;
    verified_on: string | null;
    verified_by: string | null;
  }>
> {
  return api(`/workers/${workerPid}/cpd-entries`, init);
}

/** Record a CPD activity. */
export function createCpdEntry(
  workerPid: string,
  body: {
    entry_date: string;
    activity: string;
    category: string;
    unit: string;
    amount: number;
    evidence_note?: string;
    evidence_url?: string;
  },
): Promise<{ pid: string }> {
  return api(`/workers/${workerPid}/cpd-entries`, { method: "POST", body });
}

/** Verify a CPD entry's evidence. */
export function verifyCpdEntry(pid: string): Promise<unknown> {
  return api(`/cpd-entries/${pid}/verify`, { method: "POST" });
}

/** Recorded / verified vs each applicable requirement + registrations. */
export function cpdProgress(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  worker_pid: string;
  requirements: Array<{
    requirement_pid: string;
    name: string;
    unit: string;
    period_start: string;
    period_end: string;
    required: number;
    recorded: number;
    verified: number;
    remaining: number;
    met: boolean;
    met_verified: boolean;
  }>;
  registrations: Array<{
    body: string;
    expires_on: string | null;
    status: "no_expiry" | "valid" | "expiring" | "expired";
  }>;
}> {
  return api(`/workers/${workerPid}/cpd-progress`, init);
}

/** Aggregate CPD overview (no one named). */
export function cpdOverview(init?: FetchLike): Promise<{
  derivation: string;
  headcount: number;
  requirements: Array<{
    requirement_pid: string;
    name: string;
    unit: string;
    period_start: string;
    period_end: string;
    applicable_workers: number;
    met_recorded: { numerator: number; denominator: number; value: number } | null;
    met_verified: { numerator: number; denominator: number; value: number } | null;
  }>;
  registrations: { expiring: number; expired: number };
}> {
  return api("/cpd/overview", init);
}

/** The frameworks a person can choose a role in. */
export function listSelectableFrameworks(init?: FetchLike): Promise<
  Array<{ slug: string; name: string; roles: number; available: boolean }>
> {
  return api("/frameworks/selectable", init);
}

/** A person's current role in a framework. */
export type FrameworkRole = {
  framework: string;
  role_label: string;
  role_profile_pid: string | null;
  occupation_uri: string | null;
  selected_on: string;
};

/** The worker's current role in each framework they have chosen one in. */
export function listFrameworkRoles(
  workerPid: string,
  init?: FetchLike,
): Promise<FrameworkRole[]> {
  return api(`/workers/${workerPid}/framework-roles`, init);
}

/** Set my current role: a PCF role level or an ESCO occupation. */
export function setFrameworkRole(
  workerPid: string,
  framework: string,
  body: { role_profile_pid?: string; occupation_uri?: string },
): Promise<FrameworkRole> {
  return api(`/workers/${workerPid}/framework-roles/${framework}`, {
    method: "PUT",
    body,
  });
}

/** Clear my role selection (declared skills stay). */
export function clearFrameworkRole(
  workerPid: string,
  framework: string,
): Promise<unknown> {
  return api(`/workers/${workerPid}/framework-roles/${framework}`, {
    method: "DELETE",
  });
}

/** One skill of the selected role, with my declared level (WPM 1–5). */
export type FrameworkRoleSkill = {
  ref: string;
  label: string | null;
  declared: number | null;
  /** PCF only: what the role level expects (WPM scale) and the framework's own. */
  role_expects?: number;
  framework_level?: number | null;
  framework_scale_max?: number | null;
  importance?: string;
  wording?: string | null;
  /** ESCO only. */
  relation?: "essential" | "optional";
  skill_type?: string | null;
};

/** The selected role's skills with what I have already declared. */
export function frameworkRoleSkills(
  workerPid: string,
  framework: string,
  init?: FetchLike,
): Promise<{
  framework: string;
  role: FrameworkRole;
  skills: FrameworkRoleSkill[];
}> {
  return api(`/workers/${workerPid}/framework-roles/${framework}/skills`, init);
}

/** Select the skills I have at my own 1–5 level (`null` deselects). */
export function setFrameworkSkills(
  workerPid: string,
  framework: string,
  selections: Array<{ ref: string; proficiency: number | null }>,
): Promise<{
  declared: number;
  removed: number;
  skills_created: number;
  skills_linked: number;
}> {
  return api(`/workers/${workerPid}/framework-roles/${framework}/skills`, {
    method: "PUT",
    body: { selections },
  });
}

/** One role a person has held (or holds) in a framework. */
export type RoleHistoryRow = FrameworkRole & {
  pid: string;
  started_at: string;
  ended_at: string | null;
  current: boolean;
  recorded_by: string | null;
  on_behalf: boolean;
};

/** Every role held in a framework (or all), newest first. */
export function roleHistory(
  workerPid: string,
  framework?: string,
  init?: FetchLike,
): Promise<RoleHistoryRow[]> {
  const qs = framework ? `?framework=${framework}` : "";
  return api(`/workers/${workerPid}/role-history${qs}`, init);
}

/** Record a role held in the past, with its dates (inclusive days). */
export function addPastRole(
  workerPid: string,
  framework: string,
  body: {
    role_profile_pid?: string;
    occupation_uri?: string;
    started_on: string;
    ended_on: string;
  },
): Promise<unknown> {
  return api(`/workers/${workerPid}/framework-roles/${framework}/past`, {
    method: "POST",
    body,
  });
}

/** One interval a skill was held at a level. */
export type SkillHistoryRow = {
  pid: string;
  skill_pid: string;
  skill: string | null;
  proficiency: number;
  started_at: string;
  ended_at: string | null;
  current: boolean;
  source: string;
  recorded_by: string | null;
  on_behalf: boolean;
};

/** The timeline of skill levels. */
export function skillHistory(
  workerPid: string,
  skillPid?: string,
  init?: FetchLike,
): Promise<SkillHistoryRow[]> {
  const qs = skillPid ? `?skill_pid=${skillPid}` : "";
  return api(`/workers/${workerPid}/skill-history${qs}`, init);
}

/** Record a skill level held in the past, with its dates. */
export function addPastSkill(
  workerPid: string,
  body: {
    skill_pid: string;
    proficiency: number;
    started_on: string;
    ended_on: string;
  },
): Promise<unknown> {
  return api(`/workers/${workerPid}/skill-history/past`, { method: "POST", body });
}

/** Skills, levels and roles held at the end of a date. */
export function skillsAsOf(
  workerPid: string,
  at: string,
  init?: FetchLike,
): Promise<{
  at: string;
  roles: Array<{ framework: string; role_label: string }>;
  skills: Array<{ skill_pid: string; skill: string | null; proficiency: number }>;
}> {
  return api(`/workers/${workerPid}/skills-as-of?at=${at}`, init);
}

/** Aspiration vocabularies (mirror `rules::career`). */
export const ASPIRATION_HORIZONS = [
  "within_1y",
  "one_to_three_years",
  "beyond",
  "someday",
] as const;
export const ASPIRATION_STATUSES = [
  "idea",
  "planned",
  "in_progress",
  "achieved",
  "dropped",
] as const;

/** Who may see an aspiration: just the person, their management chain, or anyone who can view the record. */
export const VISIBILITIES = ["private", "manager", "everyone"] as const;
export type Visibility = (typeof VISIBILITIES)[number];

/** One aspiration, learning goal, or growth idea. */
export type Aspiration = {
  pid: string;
  kind: "role" | "skill";
  framework: string | null;
  role_label: string | null;
  skill: string | null;
  target_level: number | null;
  horizon: string;
  status: string;
  note: string | null;
  visibility: Visibility;
  recorded_by: string | null;
  on_behalf: boolean;
  progress: Record<string, unknown> | null;
};

/** Aspirations and growth ideas; private unless shared. */
export function listAspirations(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  viewer: "person" | "manager" | "other";
  viewer_is_the_person: boolean;
  aspirations: Aspiration[];
}> {
  return api(`/workers/${workerPid}/aspirations`, init);
}

/** Record a future role or skill target. */
export function addAspiration(
  workerPid: string,
  body: {
    kind: "role" | "skill";
    framework_slug?: string;
    role_profile_pid?: string;
    occupation_uri?: string;
    skill_pid?: string;
    target_level?: number;
    horizon: string;
    status?: string;
    note?: string;
    visibility?: Visibility;
  },
): Promise<{ pid: string }> {
  return api(`/workers/${workerPid}/aspirations`, { method: "POST", body });
}

/** Update status, horizon, target, note, or sharing. */
export function updateAspiration(
  pid: string,
  body: {
    status?: string;
    horizon?: string;
    target_level?: number;
    note?: string;
    visibility?: Visibility;
  },
): Promise<unknown> {
  return api(`/aspirations/${pid}`, { method: "PUT", body });
}

/** One person in a reporting line. */
export type OrgPerson = {
  pid: string;
  display_name: string;
  job_title: string;
  department: string;
};

/** The management chain above a worker, nearest first (level 1 = direct manager). */
export function upline(
  workerPid: string,
  init?: FetchLike,
): Promise<{ worker: OrgPerson; upline: Array<OrgPerson & { level: number; direct_manager: boolean }> }> {
  return api(`/workers/${workerPid}/upline`, init);
}

/** Everyone below a manager: direct reports (depth 1) and indirect reports. */
export function downline(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  manager: OrgPerson;
  summary: { direct: number; indirect: number; total: number };
  downline: Array<OrgPerson & { depth: number; report_kind: "direct" | "indirect" }>;
}> {
  return api(`/workers/${workerPid}/downline`, init);
}

/** A manager's downline with the aspirations each person has shared with managers or everyone. */
export function downlineAspirations(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  manager: { pid: string; display_name: string };
  viewer: "manager" | "other";
  team: Array<{
    worker_pid: string;
    display_name: string;
    job_title: string;
    department: string;
    depth: number;
    direct_report: boolean;
    aspirations: Aspiration[];
  }>;
}> {
  return api(`/workers/${workerPid}/downline-aspirations`, init);
}

/** A group: community of practice, community of interest, or other. */
export type Group = {
  pid: string;
  organization_ref: string;
  /** `confederation`: a community spanning the organization and everything beneath it. */
  scope: "organization" | "confederation";
  name: string;
  kind: "practice" | "interest" | "other";
  description: string | null;
  members: number;
};

export const GROUP_KINDS = ["practice", "interest", "other"] as const;

/** Every group with its member count. */
export function listGroups(organization?: string, init?: FetchLike): Promise<Group[]> {
  const qs = organization ? `?organization_ref=${encodeURIComponent(organization)}` : "";
  return api(`/groups${qs}`, init);
}

/** Start a group. */
export function createGroup(body: {
  organization_ref: string;
  scope?: "organization" | "confederation";
  name: string;
  kind: string;
  description?: string;
}): Promise<{ pid: string }> {
  return api("/groups", { method: "POST", body });
}

/** Every group a worker is in (several at once is normal). */
export function workerGroups(
  workerPid: string,
  includePast = false,
  init?: FetchLike,
): Promise<{
  worker_pid: string;
  groups: Array<{
    group_pid: string;
    name: string;
    kind: string;
    role: string;
    joined_at: string;
    left_at: string | null;
    current: boolean;
    on_behalf: boolean;
  }>;
}> {
  return api(`/workers/${workerPid}/groups${includePast ? "?include_past=true" : ""}`, init);
}

/** What a group knows, in aggregate; skills under the floor are withheld. */
export function groupSkills(
  groupPid: string,
  init?: FetchLike,
): Promise<{
  group: Group;
  floor: number;
  skills: Array<{
    skill_pid: string;
    skill: string | null;
    declared: number;
    coverage: number | null;
    levels: Record<"1" | "2" | "3" | "4" | "5", number>;
  }>;
  withheld_below_floor: number;
}> {
  return api(`/groups/${groupPid}/skills`, init);
}

/** The members of a group (leads first). */
export function groupMembers(
  groupPid: string,
  init?: FetchLike,
): Promise<{
  group: Group;
  members: Array<{
    worker_pid: string;
    display_name: string | null;
    job_title: string | null;
    role: string;
    joined_at: string;
    on_behalf: boolean;
  }>;
}> {
  return api(`/groups/${groupPid}/members`, init);
}

/** One end of a dotted-line relationship. */
export type DottedLink = OrgPerson & {
  note: string | null;
  started_at: string;
  ended_at: string | null;
  current: boolean;
  on_behalf: boolean;
};

/** A worker's dotted-line managers and dotted-line reports. */
export function dottedLine(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  worker: OrgPerson;
  dotted_line_managers: DottedLink[];
  dotted_line_reports: DottedLink[];
}> {
  return api(`/workers/${workerPid}/dotted-line`, init);
}

/** Give a worker a dotted-line manager (anyone; several allowed). */
export function addDottedManager(
  workerPid: string,
  managerPid: string,
  note?: string,
): Promise<unknown> {
  return api(`/workers/${workerPid}/dotted-line-managers`, {
    method: "POST",
    body: { manager_pid: managerPid, ...(note ? { note } : {}) },
  });
}

/** End a dotted-line relationship; it is kept as history. */
export function endDottedManager(workerPid: string, managerPid: string): Promise<unknown> {
  return api(`/workers/${workerPid}/dotted-line-managers/${managerPid}`, { method: "DELETE" });
}

/** A worker's emergency contacts, first-to-call first (worker and HR only). */
export function listEmergencyContacts(
  workerPid: string,
  init?: FetchLike,
): Promise<EmergencyContact[]> {
  return api(`/workers/${workerPid}/emergency-contacts`, init);
}

/** Add an emergency contact (up to five). */
export function addEmergencyContact(
  workerPid: string,
  contact: {
    name: string;
    relationship: string;
    phone: string;
    alt_phone?: string;
    email?: string;
    note?: string;
    priority?: number;
  },
): Promise<{ pid: string }> {
  return api(`/workers/${workerPid}/emergency-contacts`, {
    method: "POST",
    body: contact,
  });
}

/** Change an emergency contact. */
export function updateEmergencyContact(
  pid: string,
  changes: Partial<Omit<EmergencyContact, "pid" | "on_behalf">>,
): Promise<EmergencyContact> {
  return api(`/emergency-contacts/${pid}`, { method: "PUT", body: changes });
}

/** Remove an emergency contact. */
export function removeEmergencyContact(pid: string): Promise<unknown> {
  return api(`/emergency-contacts/${pid}`, { method: "DELETE" });
}

/** Who covers for a worker when they are out, in the order to ask. */
export function listBackups(workerPid: string, init?: FetchLike): Promise<Backup[]> {
  return api(`/workers/${workerPid}/backups`, init);
}

/** Name a backup (up to three), optionally for a dated window. */
export function addBackup(
  workerPid: string,
  backup: {
    backup_pid: string;
    priority?: number;
    starts_on?: string;
    ends_on?: string;
    note?: string;
  },
): Promise<{ pid: string }> {
  return api(`/workers/${workerPid}/backups`, { method: "POST", body: backup });
}

/** Stop naming a backup. */
export function removeBackup(pid: string): Promise<unknown> {
  return api(`/backups/${pid}`, { method: "DELETE" });
}

/** Who covers for a worker on a day (default today). */
export function workerCover(
  workerPid: string,
  on?: string,
  init?: FetchLike,
): Promise<Cover> {
  return api(`/workers/${workerPid}/cover${on ? `?on=${on}` : ""}`, init);
}

/** The announcement feed: live posts, pinned first then newest. */
export function listAnnouncements(
  options?: {
    limit?: number;
    includeAll?: boolean;
    organization?: string;
    department?: string;
  },
  init?: FetchLike,
): Promise<Announcement[]> {
  const params = new URLSearchParams();
  if (options?.organization) params.set("organization", options.organization);
  if (options?.department) params.set("department", options.department);
  if (options?.includeAll) params.set("include", "all");
  if (options?.limit) params.set("limit", String(options.limit));
  const qs = params.size ? `?${params}` : "";
  return api(`/announcements${qs}`, init);
}

/** Post an announcement (organization editors). */
export function postAnnouncement(post: {
  organization_ref: string;
  title: string;
  body: string;
  pinned?: boolean;
  publish_on?: string;
  expires_on?: string;
  department?: string;
  links?: Array<{ label: string; url: string }>;
}): Promise<{ pid: string }> {
  return api("/announcements", { method: "POST", body: post });
}

/** Mark an announcement read for a worker (themself, or HR on their behalf). */
export function markAnnouncementRead(pid: string, workerPid: string): Promise<unknown> {
  return api(`/announcements/${pid}/read`, {
    method: "POST",
    body: { worker_pid: workerPid },
  });
}

/** The announcements this person has read (ids); theirs alone. */
export function myAnnouncementReads(workerPid: string, init?: FetchLike): Promise<string[]> {
  return api(`/workers/${workerPid}/announcement-reads`, init);
}

/** Retire an announcement. */
export function retireAnnouncement(pid: string): Promise<unknown> {
  return api(`/announcements/${pid}`, { method: "DELETE" });
}

/** On-call rotas in the caller's organizations, with who is on call today. */
export function listRotas(init?: FetchLike): Promise<RotaSummary[]> {
  return api("/rotas", init);
}

/** One rota with its schedule (default 28 days from today), swaps and load. */
export function getRota(
  pid: string,
  window?: { from?: string; to?: string },
  init?: FetchLike,
): Promise<RotaView> {
  const params = new URLSearchParams();
  if (window?.from) params.set("from", window.from);
  if (window?.to) params.set("to", window.to);
  const qs = params.size ? `?${params}` : "";
  return api(`/rotas/${pid}${qs}`, init);
}

/** Create an on-call rota: members in rotation order. */
export function createRota(rota: {
  organization_ref: string;
  name: string;
  description?: string;
  period_days: number;
  starts_on: string;
  members: string[];
}): Promise<{ pid: string }> {
  return api("/rotas", { method: "POST", body: rota });
}

/** Rename, re-time or re-order a rota (`members` replaces the order). */
export function updateRota(
  pid: string,
  changes: {
    name?: string;
    description?: string;
    period_days?: number;
    starts_on?: string;
    members?: string[];
  },
): Promise<{ pid: string }> {
  return api(`/rotas/${pid}`, { method: "PUT", body: changes });
}

/** Retire a rota. */
export function retireRota(pid: string): Promise<unknown> {
  return api(`/rotas/${pid}`, { method: "DELETE" });
}

/** A swap: a worker on call for a date window regardless of the rotation. */
export function addRotaSwap(
  pid: string,
  swap: { worker_pid: string; starts_on: string; ends_on: string; note?: string },
): Promise<{ pid: string }> {
  return api(`/rotas/${pid}/overrides`, { method: "POST", body: swap });
}

/** Undo a swap. */
export function removeRotaSwap(pid: string): Promise<unknown> {
  return api(`/rota-overrides/${pid}`, { method: "DELETE" });
}

/** A rota's swap requests, newest first. */
export function listSwapRequests(rotaPid: string, init?: FetchLike): Promise<SwapRequest[]> {
  return api(`/rotas/${rotaPid}/swap-requests`, init);
}

/** Ask a colleague to take the requester's on-call days in a window. */
export function requestSwap(
  rotaPid: string,
  swap: {
    requester_pid: string;
    taker_pid: string;
    starts_on: string;
    ends_on: string;
    note?: string;
  },
): Promise<{ pid: string }> {
  return api(`/rotas/${rotaPid}/swap-requests`, { method: "POST", body: swap });
}

/** Decide or withdraw a swap request. */
export function decideSwap(
  pid: string,
  decision: "accept" | "decline" | "cancel",
): Promise<{ status: string }> {
  return api(`/rota-swap-requests/${pid}/${decision}`, { method: "POST" });
}

/** A person's open swap requests: asked of them, and asked by them. */
export function workerSwapRequests(
  workerPid: string,
  init?: FetchLike,
): Promise<{ incoming: SwapRequest[]; outgoing: SwapRequest[] }> {
  return api(`/workers/${workerPid}/swap-requests`, init);
}

/** A worker's on-call stretches across the rotas the caller can read. */
export function workerOnCall(
  workerPid: string,
  init?: FetchLike,
): Promise<
  Array<{ rota_pid: string; rota_name: string; from: string; to: string; source: string }>
> {
  return api(`/workers/${workerPid}/on-call`, init);
}

/** Move a worker to another organization; groups that no longer fit end (kept as history). */
export function transferWorker(
  workerPid: string,
  organizationRef: string,
): Promise<{
  worker_pid: string;
  from: string;
  to: string;
  ended_group_memberships: Array<string | null>;
  manager_in_other_organization: boolean;
}> {
  return api(`/workers/${workerPid}/transfer`, {
    method: "POST",
    body: { organization_ref: organizationRef },
  });
}

/** A worker joins a group, or changes their role in it. */
export function joinGroup(
  groupPid: string,
  workerPid: string,
  role: "member" | "lead" = "member",
): Promise<unknown> {
  return api(`/groups/${groupPid}/members`, {
    method: "POST",
    body: { worker_pid: workerPid, role },
  });
}

/** A worker leaves a group; the membership is closed and kept as history. */
export function leaveGroup(groupPid: string, workerPid: string): Promise<unknown> {
  return api(`/groups/${groupPid}/members/${workerPid}`, { method: "DELETE" });
}

/** Drop an aspiration. */
export function deleteAspiration(pid: string): Promise<unknown> {
  return api(`/aspirations/${pid}`, { method: "DELETE" });
}

/** Search the pinned ESCO occupations (needs at least 2 characters). */
export function searchEscoOccupations(
  q: string,
  limit = 25,
  init?: FetchLike,
): Promise<
  Array<{
    uri: string;
    label: string;
    isco_code: string | null;
    essential_skills: number;
    optional_skills: number;
  }>
> {
  return api(
    `/esco/occupations?q=${encodeURIComponent(q)}&limit=${limit}`,
    init,
  );
}

/** One ESCO occupation with its essential and optional skills. */
export function getEscoOccupation(
  uri: string,
  init?: FetchLike,
): Promise<{
  uri: string;
  label: string;
  isco_code: string | null;
  description: string | null;
  skills: Array<{
    uri: string;
    label: string;
    relation: "essential" | "optional";
    skill_type: string | null;
    reuse_level: string | null;
    draft_category: string;
    catalogue_skill_pid: string | null;
  }>;
}> {
  return api(`/esco/occupation?uri=${encodeURIComponent(uri)}`, init);
}

/** Search the pinned ESCO skills, with the catalogue skill each links to. */
export function searchEscoSkills(
  q: string,
  limit = 8,
  init?: FetchLike,
): Promise<
  Array<{
    uri: string;
    label: string;
    skill_type: string | null;
    reuse_level: string | null;
    catalogue_skill_pid: string | null;
  }>
> {
  return api(`/esco/skills?q=${encodeURIComponent(q)}&limit=${limit}`, init);
}

/** Draft a role profile from an ESCO occupation at the planner's level. */
export function seedProfileFromEsco(body: {
  occupation_uri: string;
  job_title?: string;
  default_min_proficiency: number;
  include_optional?: boolean;
}): Promise<{
  pid: string;
  requirements_created: number;
  skills_created: number;
  skills_linked: number;
}> {
  return api("/role-profiles/from-esco", { method: "POST", body });
}

/** Record a skill's reference in an external framework (e.g. ESCO). */
export function addSkillRef(
  skillPid: string,
  body: { framework_slug: string; ref: string; label?: string },
): Promise<{ pid: string }> {
  return api(`/skills/${skillPid}/refs`, { method: "POST", body });
}

/** Remove a skill's reference in a framework. */
export function removeSkillRef(
  skillPid: string,
  framework: string,
): Promise<unknown> {
  return api(`/skills/${skillPid}/refs/${encodeURIComponent(framework)}`, {
    method: "DELETE",
  });
}

/** Terms-carrying ratio (or null when there is nothing to divide). */
type Fit = {
  critical_met: { numerator: number; denominator: number; value: number } | null;
  all_met: { numerator: number; denominator: number; value: number } | null;
};

/** Roles ordered by this worker's own declared-skill fit (self-service). */
export function roleMatches(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  roles: Array<{
    role_profile_pid: string;
    job_title: string;
    requirements: number;
    fit: Fit;
  }>;
}> {
  return api(`/workers/${workerPid}/role-matches`, init);
}

/** Open requisitions in the worker's organization, with their fit. */
export function opportunities(
  workerPid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  opportunities: Array<{
    requisition_pid: string;
    job_title: string;
    department: string;
    status: string;
    fit: Fit | null;
  }>;
}> {
  return api(`/workers/${workerPid}/opportunities`, init);
}

/** The worker's own expressed interests. */
export function listMobilityInterests(
  workerPid: string,
  init?: FetchLike,
): Promise<
  Array<{
    pid: string;
    kind: "role" | "requisition";
    target_pid: string | null;
    title: string | null;
    note: string | null;
  }>
> {
  return api(`/workers/${workerPid}/mobility-interests`, init);
}

/** Express interest in a role profile or an open requisition. */
export function expressMobilityInterest(
  workerPid: string,
  body: { role_profile_pid?: string; requisition_pid?: string; note?: string },
): Promise<{ pid: string }> {
  return api(`/workers/${workerPid}/mobility-interests`, {
    method: "POST",
    body,
  });
}

/** Withdraw an expressed interest. */
export function withdrawMobilityInterest(pid: string): Promise<unknown> {
  return api(`/mobility-interests/${pid}`, { method: "DELETE" });
}

/** Aggregate interest per target — never who. */
export function mobilityInterestSummary(init?: FetchLike): Promise<{
  derivation: string;
  targets: Array<{
    kind: "role" | "requisition";
    target_pid: string;
    title: string | null;
    interested: number;
  }>;
}> {
  return api("/mobility/interest-summary", init);
}

/** Change-initiative vocabularies (mirror `rules::change`). */
export const CHANGE_KINDS = [
  "automation",
  "ai_assistance",
  "process_redesign",
  "restructure",
  "other",
] as const;
export const CHANGE_IMPACTS = ["displaced", "reshaped", "created"] as const;
export const CHANGE_TIMEFRAMES = [
  "now",
  "within_1y",
  "one_to_three_years",
  "beyond",
] as const;
export const CHANGE_STATUSES_NEXT: Record<string, string[]> = {
  draft: ["active", "cancelled"],
  active: ["completed", "cancelled"],
  completed: [],
  cancelled: [],
};

/** Change initiatives (AI / automation) with roles and skills touched. */
export function listChangeInitiatives(init?: FetchLike): Promise<
  Array<{
    pid: string;
    name: string;
    kind: string;
    status: string;
    starts_on: string | null;
    roles_affected: number;
    skills_shifting: number;
  }>
> {
  return api("/change-initiatives", init);
}

/** One initiative with its role impacts and skill shifts. */
export function getChangeInitiative(
  pid: string,
  init?: FetchLike,
): Promise<{
  pid: string;
  name: string;
  description: string | null;
  kind: string;
  status: string;
  role_impacts: Array<{
    role_profile_pid: string;
    job_title: string | null;
    impact: string;
    timeframe: string;
    note: string | null;
  }>;
  skill_shifts: Array<{
    skill_pid: string;
    skill: string | null;
    direction: string;
    note: string | null;
  }>;
}> {
  return api(`/change-initiatives/${pid}`, init);
}

/** Open a draft change initiative. */
export function createChangeInitiative(body: {
  name: string;
  kind: string;
  description?: string;
}): Promise<{ pid: string }> {
  return api("/change-initiatives", { method: "POST", body });
}

/** Move an initiative through its lifecycle. */
export function setChangeStatus(pid: string, to: string): Promise<unknown> {
  return api(`/change-initiatives/${pid}/status`, {
    method: "POST",
    body: { to },
  });
}

/** Record how a role is affected (upsert). */
export function setRoleImpact(
  pid: string,
  body: {
    role_profile_pid: string;
    impact: string;
    timeframe: string;
    note?: string;
  },
): Promise<unknown> {
  return api(`/change-initiatives/${pid}/role-impacts`, {
    method: "PUT",
    body,
  });
}

/** Record a skill rising or declining (upsert). */
export function setSkillShift(
  pid: string,
  body: { skill_pid: string; direction: string; note?: string },
): Promise<unknown> {
  return api(`/change-initiatives/${pid}/skill-shifts`, {
    method: "PUT",
    body,
  });
}

/** Aggregate readiness of the affected workforce (never names anyone). */
export function changeReadiness(
  pid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  initiative: { pid: string; name: string; status: string };
  bar: number;
  affected_workers: number;
  with_active_reskill_plan: number;
  reskill_plan_coverage: {
    numerator: number;
    denominator: number;
    value: number;
  } | null;
  roles: Array<{
    job_title: string;
    impact: string;
    timeframe: string;
    employed_workers: number;
    with_active_reskill_plan: number;
    reskill_plan_coverage: {
      numerator: number;
      denominator: number;
      value: number;
    } | null;
  }>;
  rising_skills: Array<{
    skill: string | null;
    meeting: number;
    below: number;
    undeclared: number;
    meeting_share: {
      numerator: number;
      denominator: number;
      value: number;
    } | null;
  }>;
  declining_skills: string[];
}> {
  return api(`/change-initiatives/${pid}/readiness`, init);
}

/** Workforce plans (scenarios) in the caller's organizations. */
export function listWorkforcePlans(init?: FetchLike): Promise<
  Array<{
    pid: string;
    name: string;
    organization_ref: string;
    horizon_start: string;
    horizon_end: string;
    status: string;
    attrition_bp: number | null;
    demand_lines: number;
  }>
> {
  return api("/workforce-plans", init);
}

/** A plan with its demand lines and objectives. */
export function getWorkforcePlan(
  pid: string,
  init?: FetchLike,
): Promise<{
  pid: string;
  name: string;
  organization_ref: string;
  horizon_start: string;
  horizon_end: string;
  rationale: string | null;
  attrition_bp: number | null;
  budget_minor: number | null;
  budget_currency: string | null;
  on_cost_bp: number | null;
  status: string;
  demand_lines: Array<{
    pid: string;
    department: string;
    role_profile_pid: string | null;
    job_title: string | null;
    target_on: string;
    target_headcount: number;
    note: string | null;
    objective_pids: string[];
  }>;
  objectives: Array<{ pid: string; title: string; owner: string | null }>;
}> {
  return api(`/workforce-plans/${pid}`, init);
}

/** Open a draft plan. */
export function createWorkforcePlan(body: {
  name: string;
  organization_ref: string;
  horizon_start: string;
  horizon_end: string;
  rationale?: string;
  attrition_bp?: number;
  budget_minor?: number;
  budget_currency?: string;
  on_cost_bp?: number;
}): Promise<{ pid: string }> {
  return api("/workforce-plans", { method: "POST", body });
}

/** draft → active → archived. */
export function setPlanStatus(pid: string, to: string): Promise<unknown> {
  return api(`/workforce-plans/${pid}/status`, { method: "POST", body: { to } });
}

/** Set planned headcount for a department (optionally a role) at a date. */
export function setDemandLine(
  pid: string,
  body: {
    department: string;
    role_profile_pid?: string;
    target_on: string;
    target_headcount: number;
  },
): Promise<{ pid: string }> {
  return api(`/workforce-plans/${pid}/demand-lines`, { method: "PUT", body });
}

/** Remove a demand line. */
export function removeDemandLine(pid: string, linePid: string): Promise<unknown> {
  return api(`/workforce-plans/${pid}/demand-lines/${linePid}`, {
    method: "DELETE",
  });
}

/** Add a strategic objective to a plan. */
export function addPlanObjective(
  pid: string,
  body: { title: string; owner?: string },
): Promise<{ pid: string }> {
  return api(`/workforce-plans/${pid}/objectives`, { method: "POST", body });
}

/** Replace the objectives a demand line serves. */
export function setLineObjectives(
  pid: string,
  linePid: string,
  objectivePids: string[],
): Promise<unknown> {
  return api(`/workforce-plans/${pid}/demand-lines/${linePid}/objectives`, {
    method: "PUT",
    body: { objective_pids: objectivePids },
  });
}

/** Supply projection, headcount gap and competency gaps. */
export function planForecast(
  pid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  as_of: string;
  assumptions: {
    attrition_bp: number | null;
    attrition_source: "plan_assumption" | "observed_snapshots" | "insufficient_history";
    hires_assumed: number;
  };
  departments: Array<{
    department: string;
    target_on: string;
    days_ahead: number;
    opening_headcount: number;
    planned_demand: number;
    projected_supply: number | null;
    headcount_gap: number | null;
    levers: string[] | null;
    competency_gaps: Array<{
      job_title: string | null;
      skill: string | null;
      importance: string;
      min_proficiency: number;
      needed: number;
      proficient_now: number;
      shortfall: number;
      reskill_pool: number;
    }>;
  }>;
}> {
  return api(`/workforce-plans/${pid}/forecast`, init);
}

/** Annual cost of hiring to close the gaps vs the plan budget. */
export function planCost(
  pid: string,
  currency?: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  currency: string;
  salary_visible: boolean;
  assumptions: {
    attrition_bp: number | null;
    attrition_source: string;
    on_cost_bp: number;
    min_cohort: number;
  };
  groups: Array<{
    department: string;
    target_on: string;
    hires_needed: number | null;
    unit_cost_minor: number | null;
    unit_cost_source: "benchmark" | "department_average" | null;
    annual_cost_minor: number | null;
    reason:
      | "salary_not_visible"
      | "insufficient_history"
      | "no_unit_cost"
      | null;
  }>;
  total_annual_cost_minor: number | null;
  uncosted_groups: number;
  affordability: {
    budget_minor: number;
    remaining_minor: number;
    within_budget: boolean;
  } | null;
}> {
  const qs = currency ? `?currency=${encodeURIComponent(currency)}` : "";
  return api(`/workforce-plans/${pid}/cost${qs}`, init);
}

/** Does the planned headcount serve the strategy? */
export function planAlignment(
  pid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  planned_headcount: number;
  aligned_share: { numerator: number; denominator: number; value: number } | null;
  objectives: number;
  unresourced_objectives: string[];
  unaligned_demand_lines: Array<{
    department: string;
    target_on: string;
    target_headcount: number;
  }>;
  critical_roles_without_bench: {
    in_plan_departments: number;
    all_departments: number;
  };
}> {
  return api(`/workforce-plans/${pid}/alignment`, init);
}

/** Terms-carrying ratio object (or null when there is nothing to divide). */
type RatioOrNull = {
  numerator: number;
  denominator: number;
  value: number;
} | null;

/** One worker's declared proficiency against a role profile. */
export function workerRoleGap(
  workerPid: string,
  roleProfilePid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  worker_pid: string;
  role: { pid: string; job_title: string };
  requirements: Array<{
    skill_pid: string;
    skill: string | null;
    importance: string;
    min_proficiency: number;
    declared: number | null;
    grade: "met" | "below" | "undeclared";
    shortfall: number | null;
  }>;
  critical_met: RatioOrNull;
  all_met: RatioOrNull;
}> {
  return api(
    `/workers/${workerPid}/role-gap?role_profile_pid=${roleProfilePid}`,
    init,
  );
}

/** Can the workforce staff a role? Aggregate per requirement. */
export function roleGap(
  roleProfilePid: string,
  init?: FetchLike,
): Promise<{
  derivation: string;
  role: { pid: string; job_title: string };
  headcount: number;
  requirements: Array<{
    skill: string | null;
    importance: string;
    min_proficiency: number;
    meeting: number;
    below: number;
    undeclared: number;
    coverage: RatioOrNull;
  }>;
}> {
  return api(`/role-profiles/${roleProfilePid}/gap`, init);
}

/** The shared workforce metrics (headcount, turnover, time-to-fill, …). */
export function workforceMetrics(
  period?: { from?: string; to?: string },
  init?: FetchLike,
): Promise<{
  period: { from: string; to: string };
  definitions: Record<string, string>;
  headcount: { opening: number; closing: number; opening_date: string };
  starters: number;
  leavers: number;
  turnover_rate: number | null;
  span_of_control: { managers: number; mean: number; max: number } | null;
  time_to_fill: {
    requisitions: number;
    mean_days: number;
    median_days: number;
  } | null;
}> {
  const query = new URLSearchParams();
  if (period?.from) query.set("from", period.from);
  if (period?.to) query.set("to", period.to);
  const qs = query.toString();
  return api(`/workforce-intelligence/metrics${qs ? `?${qs}` : ""}`, init);
}

/** Findings derived from the shared metrics, with the thresholds used. */
export function workforceInsights(
  period?: { from?: string; to?: string },
  init?: FetchLike,
): Promise<{
  period: { from: string; to: string };
  insights: Array<{
    code: string;
    severity: "info" | "attention";
    observation: string;
    suggestion: string;
    /** Figures behind the finding, for rendering it in the UI language. */
    params: Record<string, number>;
  }>;
  thresholds: Record<string, { value: number; meaning: string }>;
}> {
  const query = new URLSearchParams();
  if (period?.from) query.set("from", period.from);
  if (period?.to) query.set("to", period.to);
  const qs = query.toString();
  return api(`/workforce-intelligence/insights${qs ? `?${qs}` : ""}`, init);
}

/** The mentorship overview (active pairs, load, unmatched, stale). */
export function mentorshipOverview(
  days?: number,
  init?: FetchLike,
): Promise<{
  as_of: string;
  active_pairings: number;
  mentor_load: Array<{
    mentor_pid: string;
    mentor: string | null;
    active_mentees: number;
  }>;
  unmatched_workers: Array<{
    pid: string;
    display_name: string;
    department: string;
  }>;
  stale_days: number;
  stale_mentorships: Array<{
    pid: string;
    mentor: string | null;
    mentee: string | null;
    last_session: string | null;
  }>;
}> {
  return api(
    `/learning/mentorship-overview${days ? `?days=${days}` : ""}`,
    init,
  );
}

/** Change a mentorship's lifecycle status. */
export function mentorshipStatus(pid: string, to: string): Promise<unknown> {
  return api(`/mentorships/${pid}/status`, { method: "POST", body: { to } });
}

/** The advisory working-time guardrail signals (WPM-R27; flags only). */
export function workingTime(
  department?: string,
  init?: FetchLike,
): Promise<{
  as_of: string;
  reference_weeks: number;
  rest_window_days: number;
  workers_checked: number;
  derivation: string;
  flagged: Array<{
    worker_pid: string;
    display_name: string;
    department: string;
    average_weekly: {
      numerator_minutes: number;
      denominator_weeks: number;
      value_minutes_per_week: number | null;
    };
    over_48h: boolean;
    rest_breaches: Array<{
      prev_end: string;
      next_start: string;
      gap_minutes: number;
    }>;
  }>;
}> {
  const qs = department ? `?department=${encodeURIComponent(department)}` : "";
  return api(`/workforce/working-time${qs}`, init);
}

// ─── Reasonable adjustments (WPM-R33) ───────────────────────────────

/** The closed suggestion categories (navigation, never a label). */
export const ADJUSTMENT_CATEGORIES = [
  "written_instructions",
  "agendas_in_advance",
  "quieter_workspace",
  "flexible_breaks",
  "clear_priorities",
  "equipment",
  "schedule",
  "other",
] as const;

/** One adjustment request (words may be withheld on masked reads). */
export interface AdjustmentRequest {
  pid: string;
  category: string;
  status: "requested" | "agreed" | "declined" | "in_place" | "withdrawn";
  barrier?: string;
  impact?: string;
  adjustment?: string;
  decision_note?: string | null;
  decided_on: string | null;
  words_withheld: boolean;
}

/** The worker's adjustment requests ($sub-owned). */
export function listAdjustmentRequests(
  pid: string,
  init?: FetchLike,
): Promise<AdjustmentRequest[]> {
  return api(`/workers/${pid}/adjustment-requests`, init);
}

/** Ask for a change: barrier + impact + change, all required. */
export function createAdjustmentRequest(
  pid: string,
  body: {
    category: string;
    barrier: string;
    impact: string;
    adjustment: string;
  },
): Promise<{ pid: string }> {
  return api(`/workers/${pid}/adjustment-requests`, { method: "POST", body });
}

/** Decide a request (agreed | declined | in_place | withdrawn). */
export function decideAdjustment(
  pid: string,
  to: string,
  note?: string,
): Promise<AdjustmentRequest> {
  return api(`/adjustment-requests/${pid}/status`, {
    method: "POST",
    body: { to, note },
  });
}

// ─── Ergonomic (DSE) assessments (WPM-R32) ──────────────────────────

/** One DSE checklist item (equipment note only — WPM-D24). */
export interface ErgonomicItem {
  pid: string;
  name: string;
  ok: boolean | null;
  note: string | null;
}

/** One workstation assessment with its items. */
export interface ErgonomicAssessment {
  pid: string;
  workstation: string;
  status: "open" | "completed";
  assessed_on: string | null;
  open_issues: number;
  items: ErgonomicItem[];
}

/** The worker's DSE assessments. */
export function listErgonomicAssessments(
  pid: string,
  init?: FetchLike,
): Promise<ErgonomicAssessment[]> {
  return api(`/workers/${pid}/ergonomic-assessments`, init);
}

/** Open an assessment (default DSE checklist when items omitted). */
export function createErgonomicAssessment(
  pid: string,
  workstation: string,
  items?: string[],
): Promise<{ pid: string }> {
  return api(`/workers/${pid}/ergonomic-assessments`, {
    method: "POST",
    body: { workstation, items: items ?? [] },
  });
}

/** Answer one checklist item (ok | issue + equipment note). */
export function answerErgonomicItem(
  pid: string,
  ok: boolean,
  note?: string,
): Promise<ErgonomicItem> {
  return api(`/ergonomic-items/${pid}`, { method: "PUT", body: { ok, note } });
}

/** Complete an assessment (every item must be answered). */
export function completeErgonomicAssessment(
  pid: string,
): Promise<ErgonomicAssessment> {
  return api(`/ergonomic-assessments/${pid}/complete`, { method: "POST" });
}

/** Issue-flagged items by department (rota-tier visibility). */
export function ergonomicIssues(init?: FetchLike): Promise<{
  as_of: string;
  by_department: Record<string, number>;
  issues: Array<{
    department: string;
    worker_pid: string;
    display_name: string;
    workstation: string;
    item: string;
    note: string | null;
    assessment_status: string;
  }>;
  derivation: string;
}> {
  return api("/ergonomics/issues", init);
}

// ─── Notifications (WPM-R31) ────────────────────────────────────────

/** One in-app notification (reference-only body, WPM-D23). */
export interface Notification {
  pid: string;
  kind: string;
  body: string;
  data: Record<string, unknown>;
  created_at: string;
  read_at: string | null;
}

/** The worker's notifications, unread first ($sub-owned). */
export function listNotifications(
  pid: string,
  init?: FetchLike,
): Promise<Notification[]> {
  return api(`/workers/${pid}/notifications`, init);
}

/** Mark one notification read (owner-only). */
export function markNotificationRead(pid: string): Promise<Notification> {
  return api(`/notifications/${pid}/read`, { method: "POST" });
}

// ─── Subject rights & retention (WPM-R30) ───────────────────────────

/** The retention report: what the next sweep would remove. */
export function retentionReport(init?: FetchLike): Promise<{
  as_of: string;
  horizon_days: number;
  soft_deleted_past_horizon: Record<string, number>;
  expired_consent_candidates: number;
  derivation: string;
}> {
  return api("/retention", init);
}

/** Run the retention sweep (destructive; admin under enforcement). */
export function retentionSweep(): Promise<{
  horizon_days: number;
  deleted: Record<string, number>;
  rows_deleted: number;
  candidates_scrubbed: number;
}> {
  return api("/retention/sweep", { method: "POST" });
}

/** Erase (anonymise) a terminated/retired worker (destructive). */
export function eraseWorker(
  pid: string,
): Promise<{ erased: string; note: string }> {
  return api(`/workers/${pid}/erase`, { method: "POST" });
}

// ─── 360° appraisals (WPM-R29) ──────────────────────────────────────

/** One appraisal summary row (counts, never content). */
export interface AppraisalSummary {
  pid: string;
  status: "draft" | "collecting" | "shared";
  competencies: string[];
  shared_on: string | null;
  nominated: number;
  responded: number;
}

/** One nomination on the detail view: who (and whether they responded). */
export interface AppraisalNomination {
  pid: string;
  rater_pid: string;
  display_name: string | null;
  group: "self" | "manager" | "peer" | "report";
  responded: boolean;
}

/** One group block on the report: withheld, or disclosed aggregates. */
export interface AppraisalGroup {
  group: string;
  withheld: boolean;
  responses?: number;
  competencies?: Record<string, { count: number; mean: number }>;
  comments?: string[];
}

/** The subject's appraisals. */
export function listAppraisals(
  pid: string,
  init?: FetchLike,
): Promise<AppraisalSummary[]> {
  return api(`/workers/${pid}/appraisals`, init);
}

/** Open a draft 360 (self nomination is automatic). */
export function createAppraisal(
  pid: string,
  competencies: string[],
): Promise<{ pid: string }> {
  return api(`/workers/${pid}/appraisals`, {
    method: "POST",
    body: { competencies },
  });
}

/** One appraisal + its nominations (who responded, never what). */
export function getAppraisal(
  pid: string,
  init?: FetchLike,
): Promise<{
  pid: string;
  worker_pid: string;
  status: string;
  competencies: string[];
  shared_on: string | null;
  nominations: AppraisalNomination[];
}> {
  return api(`/appraisals/${pid}`, init);
}

/** Invite a rater (draft only). */
export function nominateRater(
  appraisalPid: string,
  raterPid: string,
  group: "manager" | "peer" | "report",
): Promise<{ pid: string }> {
  return api(`/appraisals/${appraisalPid}/nominations`, {
    method: "POST",
    body: { rater_pid: raterPid, group },
  });
}

/** Move the appraisal lifecycle (draft → collecting → shared). */
export function appraisalStatus(pid: string, to: string): Promise<unknown> {
  return api(`/appraisals/${pid}/status`, { method: "POST", body: { to } });
}

/** Submit one rater's response (every declared competency, 1–5). */
export function respondAppraisal(
  appraisalPid: string,
  raterPid: string,
  scores: Record<string, number>,
  comment?: string,
): Promise<{ submitted: boolean }> {
  return api(`/appraisals/${appraisalPid}/responses`, {
    method: "POST",
    body: { rater_pid: raterPid, scores, comment },
  });
}

/** One pending 360 request for a rater. */
export interface AppraisalRequest {
  appraisal_pid: string;
  subject_pid: string;
  subject: string | null;
  group: string;
  competencies: string[];
}

/** The rater's own pending 360 requests ($sub-owned). */
export function appraisalRequests(
  pid: string,
  init?: FetchLike,
): Promise<AppraisalRequest[]> {
  return api(`/workers/${pid}/appraisal-requests`, init);
}

/** The group-floored report (shared appraisals only). */
export function appraisalReport(
  pid: string,
  init?: FetchLike,
): Promise<{
  appraisal: {
    pid: string;
    worker_pid: string;
    competencies: string[];
    shared_on: string | null;
  };
  groups: AppraisalGroup[];
  derivation: string;
}> {
  return api(`/appraisals/${pid}/report`, init);
}

// ─── Wellbeing (health entitlements, WPM-R25) ───────────────────────

/** One configurable entitlement rule (non-clinical predicates only). */
export interface WellbeingEntitlement {
  pid: string;
  name: string;
  kind: "health" | "benefit";
  benefit_plan_pid: string | null;
  description: string;
  info_url: string | null;
  min_age: number | null;
  max_age: number | null;
  departments: string[];
  job_titles: string[];
  doses: number;
  active_from: string | null;
  active_until: string | null;
}

/** One live prompt (or the one multi-dose reminder) for a worker. */
export interface WellbeingPrompt {
  kind: "prompt" | "reminder";
  entitlement_kind: "health" | "benefit";
  benefit_plan_pid: string | null;
  entitlement_pid: string;
  name: string;
  description: string;
  info_url: string | null;
  doses: number;
  response: string | null;
}

/** The configured entitlement rules. */
export function listWellbeingEntitlements(
  init?: FetchLike,
): Promise<WellbeingEntitlement[]> {
  return api("/wellbeing-entitlements", init);
}

/** Add an entitlement rule (HR configuration). */
export function createWellbeingEntitlement(
  body: Partial<Omit<WellbeingEntitlement, "pid">> & {
    name: string;
    description: string;
  },
): Promise<{ pid: string }> {
  return api("/wellbeing-entitlements", { method: "POST", body });
}

/** Soft-close an entitlement rule. */
export function deleteWellbeingEntitlement(pid: string): Promise<unknown> {
  return api(`/wellbeing-entitlements/${pid}`, { method: "DELETE" });
}

/** A worker's live prompts (self-service). */
export function workerWellbeingPrompts(
  pid: string,
  init?: FetchLike,
): Promise<{
  as_of: string;
  age_known: boolean;
  derivation: string;
  prompts: WellbeingPrompt[];
}> {
  return api(`/workers/${pid}/wellbeing-prompts`, init);
}

/** Acknowledge a prompt (booked | done | declined | dismissed). */
export function acknowledgeWellbeing(
  workerPid: string,
  entitlementPid: string,
  response: "booked" | "done" | "declined" | "dismissed",
): Promise<unknown> {
  return api(`/workers/${workerPid}/wellbeing-acknowledgements`, {
    method: "POST",
    body: { entitlement_pid: entitlementPid, response },
  });
}

/** One anonymous pulse survey. */
export interface PulseSurvey {
  pid: string;
  name: string;
  question: string;
  active_from: string | null;
  active_until: string | null;
  open: boolean;
}

/** One k-floored result cell: suppressed, or disclosed with stats. */
export interface PulseCell {
  suppressed: boolean;
  count?: number;
  distribution?: number[];
  mean?: number;
}

/** The pulse surveys with their open state. */
export function listPulseSurveys(init?: FetchLike): Promise<PulseSurvey[]> {
  return api("/pulse-surveys", init);
}

/** Submit one anonymous 1–5 score (no handle comes back). */
export function submitPulse(
  surveyPid: string,
  workerPid: string,
  score: number,
): Promise<{ submitted: boolean }> {
  return api(`/pulse-surveys/${surveyPid}/responses`, {
    method: "POST",
    body: { worker_pid: workerPid, score },
  });
}

/** The k-floored aggregate results for one survey. */
export function pulseResults(
  surveyPid: string,
  init?: FetchLike,
): Promise<{
  as_of: string;
  survey: { pid: string; name: string; question: string };
  overall: PulseCell;
  departments: Array<PulseCell & { department: string }>;
  derivation: string;
}> {
  return api(`/pulse-surveys/${surveyPid}/results`, init);
}

/** HR aggregate uptake: counts only, no individuals. */
export function wellbeingUptake(init?: FetchLike): Promise<{
  as_of: string;
  derivation: string;
  entitlements: Array<{
    entitlement_pid: string;
    name: string;
    kind: "health" | "benefit";
    by_response: Record<string, number>;
    uptake_rate: Ratio;
    enrolment_conversion: Ratio | null;
  }>;
}> {
  return api("/wellbeing/uptake", init);
}
