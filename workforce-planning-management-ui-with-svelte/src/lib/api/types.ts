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
