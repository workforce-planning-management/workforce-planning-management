// Wire types mirroring the WPM service's JSON (the service spec is
// the contract; drift is a test failure in the stubbed e2e suite).

/** One employment relationship (salary fields null when masked). */
export interface Worker {
  pid: string;
  person_ref: string;
  upstream_worker_ref: string | null;
  organization_ref: string;
  worker_number: string;
  display_name: string;
  status: string;
  employment_type: string;
  fte_percent: number;
  department: string;
  job_title: string;
  /** Free-text work location; `null` when not recorded. */
  location: string | null;
  manager_pid: string | null;
  salary_minor: number | null;
  salary_currency: string | null;
  hired_on: string;
  terminated_on: string | null;
}

/**
 * One organization membership for the signed-in caller — a person can
 * hold several of these at once (no switcher; every org they belong
 * to is visible together). `employed` is `true` when this membership
 * is a literal employment relationship (`worker_pid` set); `false`
 * for a staff/admin-only role grant in an org they aren't employed
 * in.
 */
export interface MyOrganization {
  pid: string;
  person_ref: string;
  organization_ref: string;
  worker_pid: string | null;
  employed: boolean;
  role: string;
  starts_on: string;
  ends_on: string | null;
}

/** One org-chart node (recursive). */
export interface OrgNode {
  pid: string;
  display_name: string;
  job_title: string;
  department: string;
  /** Tenure band (whole months of service): `under_1y` … `over_10y`, or
   *  `not_started` for a future hire date. */
  tenure: string;
  /** Free-text work location; `null` when not recorded. */
  location: string | null;
  reports: OrgNode[];
}

/** One level on a published job-level ladder. */
export interface JobLevel {
  number: number;
  code: string;
  title: string;
  summary: string;
  /** As the source states it; null when it does not say (never a guess). */
  experience: string | null;
  management_equivalent: string | null;
}

/** A job-level ladder (e.g. Google's technical levels). Carries no pay. */
export interface JobLevelFramework {
  id: string;
  name: string;
  organization: string;
  track: string;
  source: string;
  levels: JobLevel[];
}

/** A row of the framework list. */
export interface JobLevelFrameworkSummary {
  id: string;
  name: string;
  organization: string;
  track: string;
  source: string;
  levels: string[];
}

/** A job level as held by a worker or assigned to a role. */
export interface HeldLevel {
  framework: string;
  framework_name: string | null;
  /** Null only if the ladder was retired from the service. */
  level: JobLevel | null;
}

/** A worker's current level; readable by the worker and HR only. */
export interface WorkerJobLevel extends HeldLevel {
  effective_on: string;
  on_behalf: boolean;
}

/** A pay band as attached to a role: entry and top pay and the steps. */
export interface RolePayBand {
  scale: string;
  scale_name: string | null;
  band: string;
  closed?: boolean;
  currency?: string;
  entry_minor?: number;
  top_minor?: number;
  steps?: PayStep[];
}

/** A role's grade: a level and/or a pay band, linked by whoever edits the role. */
export interface RoleGrade {
  role_profile: string;
  job_level: HeldLevel | null;
  pay_band: RolePayBand | null;
}

/** Where a worker stands on progression, with dates (eligibility, not a promise). */
export type PayStanding =
  | { kind: "at_top" }
  | { kind: "due"; eligible_on: string; next_annual_minor: number }
  | {
      kind: "not_yet";
      eligible_on: string;
      days_remaining: number;
      next_annual_minor: number;
    };

/** A worker's band and step on a pay scale; readable by the worker and HR only. */
export interface WorkerPayPosition {
  scale: string;
  scale_name: string | null;
  band: string;
  step: number;
  currency: string | null;
  annual_minor: number | null;
  step_since: string;
  on_behalf: boolean;
  progression: PayStanding | null;
}

/** One pay point: annual full-time pay in minor units. */
export interface PayStep {
  annual_minor: number;
  /** Whole years on this step before eligible for the next; null on the top step. */
  years_to_next: number | null;
}

export interface PayBand {
  code: string;
  closed: boolean;
  steps: PayStep[];
}

export interface PayAllowance {
  code: string;
  name: string;
  amount_minor: number;
}

/** A pay scale (e.g. NHS Agenda for Change, Wales), transcribed from its circular. */
export interface PayScale {
  id: string;
  name: string;
  framework: string;
  nation: string;
  currency: string;
  effective_from: string;
  uplift_tenths_percent: number;
  source: string;
  minutes_per_week: number;
  bands: PayBand[];
  allowances: PayAllowance[];
}

/** A row of the scale list. */
export interface PayScaleSummary {
  id: string;
  name: string;
  framework: string;
  nation: string;
  currency: string;
  effective_from: string;
  uplift_tenths_percent: number;
  source: string;
  bands: string[];
}

export type PayPosition =
  | { kind: "below_entry"; shortfall_minor: number }
  | { kind: "on_step"; step: number }
  | { kind: "between_steps"; below_step: number }
  | { kind: "above_top"; excess_minor: number };

export type PayProgression =
  | { kind: "at_top" }
  | { kind: "due"; next_annual_minor: number }
  | { kind: "not_yet"; months_remaining: number; next_annual_minor: number };

/** The answer to "where does this salary sit?" — stateless, nothing stored. */
export interface PayLookup {
  scale: string;
  band: string;
  closed_to_new_entrants: boolean;
  currency: string;
  basis: string;
  position: PayPosition | null;
  progression: PayProgression | null;
}

/** One employee-directory row: nothing sensitive (no pay, dates or person ref). */
export interface DirectoryEntry {
  pid: string;
  display_name: string;
  job_title: string;
  department: string;
  location: string | null;
  organization_ref: string;
  manager_name: string | null;
  /** On approved leave today (never says what kind). */
  away_today: boolean;
  /** While away: who is covering, when someone is. */
  covered_by: string | null;
  /** Names of the on-call rotas this person is on call for today. */
  on_call: string[];
}

/** One emergency contact (visible only to the worker and HR). */
export interface EmergencyContact {
  pid: string;
  name: string;
  relationship: string;
  phone: string;
  alt_phone: string | null;
  email: string | null;
  /** 1 is the first person to call. */
  priority: number;
  note: string | null;
  on_behalf: boolean;
}

/** One backup: a colleague who covers when the worker is out. */
export interface Backup {
  pid: string;
  backup_pid: string;
  backup_name: string | null;
  backup_title: string | null;
  /** 1 is the first backup to ask. */
  priority: number;
  starts_on: string | null;
  ends_on: string | null;
  note: string | null;
  on_behalf: boolean;
}

/** Who covers for a worker on a day; `covered_by` is null when nobody can. */
export interface Cover {
  on: string;
  worker_pid: string;
  covered_by: string | null;
  covered_by_name: string | null;
  covered_by_title: string | null;
  backups_named: number;
}

/** An on-call rota in a list, with who is on call today. */
export interface RotaSummary {
  pid: string;
  organization_ref: string;
  name: string;
  description: string | null;
  period_days: number;
  starts_on: string;
  members: number;
  on_call_today: { worker_pid: string; name: string | null } | null;
}

/** Why someone is on call: their turn, covering for someone away, or a swap. */
export type OnCallSource = "rotation" | "skipped" | "override";

/** A stretch of days with the same person on call (`worker_pid` null: nobody available). */
export interface OnCallRun {
  from: string;
  to: string;
  worker_pid: string | null;
  worker_name: string | null;
  source: OnCallSource | null;
}

/** One rota with its schedule, swaps and days-on-call per member. */
export interface RotaView extends Omit<RotaSummary, "members"> {
  members: Array<{
    position: number;
    worker_pid: string;
    name: string | null;
    job_title: string | null;
  }>;
  overrides: Array<{
    pid: string;
    worker_pid: string;
    worker_name: string | null;
    starts_on: string;
    ends_on: string;
    note: string | null;
  }>;
  window: { from: string; to: string };
  runs: OnCallRun[];
  load: Array<{ worker_pid: string; name: string | null; days: number }>;
}

/** A request that a colleague take the requester's on-call days in a window. */
export interface SwapRequest {
  pid: string;
  rota_pid: string;
  rota_name: string | null;
  requester_pid: string;
  requester_name: string | null;
  taker_pid: string;
  taker_name: string | null;
  starts_on: string;
  ends_on: string;
  note: string | null;
  status: "requested" | "accepted" | "declined" | "cancelled";
}

/** One announcement in the feed (plain text). */
export interface Announcement {
  pid: string;
  organization_ref: string;
  title: string;
  body: string;
  pinned: boolean;
  publish_on: string;
  expires_on: string | null;
  /** The one department it is for; null = everyone. */
  department: string | null;
  /** Up to three https links. */
  links: Array<{ label: string; url: string }>;
  status: "scheduled" | "live" | "expired";
  author: string | null;
  /** How many have read it — present for editors only; never who. */
  read_count?: number;
}

/** One skill gap for a person (a need they have not met). */
export interface SkillGap {
  skill_pid: string;
  skill: string | null;
  category: string | null;
  required: number;
  importance: "critical" | "important" | "useful" | string;
  sources: Array<"role" | "target" | "aspiration">;
  declared: number | null;
  status: "below" | "undeclared";
  /** Levels short; null when not declared (unknown, not a number). */
  shortfall: number | null;
  priority: number;
}

/** One skill's roll-up across the workforce (counts only). */
export interface WorkforceSkillGap {
  skill_pid: string;
  skill: string | null;
  category: string | null;
  needed_by: number;
  met: number;
  below: number;
  undeclared: number;
  total_shortfall: number;
  critical_below: number;
  score: number;
  departments: Array<{ department: string; below: number }>;
}

/** A catalogue course that builds a skill. */
export interface SkillCourse {
  pid: string;
  course_ref: string;
  title: string;
  hours: number;
  levels: number;
}

/** The training that would close one gap, and what it rests on. */
export interface TrainingRecommendation {
  courses: Array<{
    course_ref: string;
    title: string;
    hours: number;
    levels: number;
  }>;
  course_levels: number;
  estimated_levels: number;
  hours: number;
  basis: "courses" | "mixed" | "estimate";
}

/** One scheduled item of a training plan. */
export interface TrainingPlanItem {
  skill_pid: string;
  skill: string | null;
  importance: string;
  required: number;
  declared: number;
  shortfall: number;
  priority: number;
  recommendation: TrainingRecommendation;
  starts_on: string | null;
  ends_on: string | null;
  weeks: number;
  cumulative_hours: number;
}

/** A person's training plan. */
export interface TrainingPlan {
  worker_pid: string;
  weekly_hours: number;
  start: string;
  total_hours: number;
  total_weeks: number;
  finish_on: string | null;
  plan: TrainingPlanItem[];
  assess_first: Array<{
    skill_pid: string;
    skill: string | null;
    required: number;
    importance: string;
  }>;
}

/** Training hours across the workforce (hours and counts only). */
export interface TrainingDemand {
  as_of: string;
  workers_considered: number;
  people_with_gaps: number;
  total_hours: number;
  average_hours_per_person: number | null;
  skills: Array<{
    skill_pid: string;
    skill: string | null;
    people: number;
    hours: number;
    people_on_estimate: number;
  }>;
  departments: Array<{ department: string; people: number; hours: number }>;
}

/** A joiner or leaver record, with checklist progress. */
export interface Movement {
  pid: string;
  kind: "joiner" | "leaver";
  worker_pid: string;
  worker_name: string;
  department: string;
  job_title: string;
  organization_ref: string;
  effective_on: string;
  reason: string | null;
  status: "open" | "completed" | "cancelled";
  notes: string | null;
  completed_at: string | null;
  progress: { closed: number; total: number; overdue: number };
}

/** One dated checklist item. */
export interface MovementItem {
  pid: string;
  position: number;
  title: string;
  category: string;
  due_on: string;
  assignee_pid: string | null;
  assignee_name: string | null;
  done_on: string | null;
  done_by: string | null;
  skipped_reason: string | null;
  state: "done" | "skipped" | "overdue" | "due_today" | "upcoming";
}

/** One thing a leaver still holds. */
export interface HeldItem {
  kind: string;
  subject_pid: string;
  label: string;
  can_reassign: boolean;
  needs_new_holder: boolean;
}

/** One entry of the handover audit trail. */
export interface HandoverAction {
  kind: string;
  subject_pid: string;
  label: string | null;
  action: "reassigned" | "closed" | "revoked";
  to_worker_pid: string | null;
  to_worker_name: string | null;
  note: string | null;
  performed_by: string | null;
  performed_at: string;
}

/** One funded job opening. */
export interface Requisition {
  pid: string;
  organization_ref: string;
  department: string;
  job_title: string;
  headcount: number;
  salary_min_minor: number | null;
  salary_max_minor: number | null;
  salary_currency: string | null;
  status: string;
  opened_on: string | null;
}

/** One application row. */
export interface Application {
  pid: string;
  requisition_pid: string;
  candidate_pid: string;
  stage: string;
  notes: string | null;
}

/** One onboarding checklist item. */
export interface OnboardingItem {
  pid: string;
  worker_pid: string;
  name: string;
  mandatory: boolean;
  status: string;
  waived_reason: string | null;
}

/** One leave entitlement (balance) row. */
export interface LeaveEntitlement {
  pid: string;
  worker_pid: string;
  kind: string;
  year: number;
  entitled_days: number;
  used_days: number;
}

/** One leave request. */
export interface LeaveRequest {
  pid: string;
  worker_pid: string;
  kind: string;
  start_on: string;
  end_on: string;
  days: number;
  status: string;
  negative_balance: boolean;
}

/** One payroll run. */
export interface PayrollRun {
  pid: string;
  organization_ref: string;
  period_start: string;
  period_end: string;
  status: string;
}

/** One payslip (amounts zeroed when masked). */
export interface Payslip {
  pid: string;
  run_pid: string;
  worker_pid: string;
  currency: string;
  gross_minor: number;
  deductions: { label: string; amount_minor: number }[];
  net_minor: number;
}

/** One benchmark band. */
export interface Benchmark {
  pid: string;
  job_title: string;
  currency: string;
  min_minor: number;
  median_minor: number;
  max_minor: number;
  source: string;
  as_of: string;
}

/**
 * A ratio the service already computed — `numerator`/`denominator` plus
 * the derived `value`, or `null` when the denominator was zero. A zero
 * denominator must render as "no data", never as `0%`: "we measured and
 * it was zero" and "we had nothing to measure" are different claims, and
 * only the service knows which one is true. See `#lib/format.ts`.
 */
export interface Ratio {
  numerator: number;
  denominator: number;
  value: number | null;
}

/** One benchmark-comparison row (flags only, no amounts). */
export interface ComparisonRow {
  worker_pid: string;
  job_title: string;
  department: string;
  benchmark_pid: string | null;
  flag: "below_min" | "within" | "above_max" | null;
}

/** One review row. */
export interface Review {
  pid: string;
  cycle_pid: string;
  worker_pid: string;
  reviewer_ref: string;
  status: string;
  rating: number | null;
  content: string | null;
}

/** One training enrolment. */
export interface TrainingEnrollment {
  pid: string;
  worker_pid: string;
  course_ref: string;
  status: string;
  completed_on: string | null;
  certificate_expires_on: string | null;
}

/** One succession plan with candidates. */
export interface SuccessionEntry {
  plan: {
    pid: string;
    role_title: string;
    department: string;
    criticality: number;
  };
  candidates: {
    pid: string;
    worker_pid: string;
    readiness: string;
    rank: number;
  }[];
}
