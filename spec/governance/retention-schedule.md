# Retention schedule (WPM-R91)

How long each kind of record is kept **after it is deleted** before the retention sweep
removes it for good. Read the live values from `GET /api/retention/schedule`.

> ⚠️ **The defaults are cautious starting points, not legal advice.** A deployer sets
> each horizon from their own legal basis, sector rules and contracts, and records the
> reasoning in their own copy of this table (the "Deployer's basis" column). This
> repository cannot know your obligations.

Horizons are in **calendar days**, counted from the day a record is soft-deleted, and never
below 30 calendar days (a lower value would turn every deletion into an immediate,
irreversible one). A kind is overridden with `WPM_RETENTION_<KIND>_DAYS`; the older
`WPM_RETENTION_DAYS` applies to every kind that has no override of its own. A junk value is
ignored.

| Kind | Default | Override variable | Tables | Why this default (to replace) | Deployer's basis |
| --- | ---: | --- | --- | --- | --- |
| `recruitment` | 180 | `WPM_RETENTION_RECRUITMENT_DAYS` | `applications`, `candidates`, `interviews`, `onboarding_items`, `requisitions` | Short: a rejected candidate has no continuing employment relationship; enough time to answer a challenge to a decision. Pool candidates are also bounded by their own consent date | deployer to complete |
| `employment` | 2190 | `WPM_RETENTION_EMPLOYMENT_DAYS` | `emergency_contacts`, `worker_aspirations`, `worker_backups`, `workers` | About six years after the employment record ends, a common limitation period for claims arising from employment | deployer to complete |
| `pay` | 2190 | `WPM_RETENTION_PAY_DAYS` | `benefit_enrollments`, `benefit_plans`, `expense_claims`, `payroll_runs`, `payslips` | Tax and payroll records carry multi-year statutory duties; six years is a cautious common figure | deployer to complete |
| `time_and_leave` | 730 | `WPM_RETENTION_TIME_AND_LEAVE_DAYS` | `leave_entitlements`, `leave_requests`, `rotas`, `shift_assignments`, `shifts`, `time_entries` | Working-time records are commonly kept for a short statutory period; leave records support later pay and entitlement queries | deployer to complete |
| `performance` | 730 | `WPM_RETENTION_PERFORMANCE_DAYS` | `appraisals`, `development_plans`, `early_career_programs`, `feedback_entries`, `goals`, `mobility_interests`, `pipeline_members`, `program_placements`, `reviews`, `review_cycles`, `succession_candidates`, `succession_plans`, `talent_pipelines` | Long enough for a review cycle to be referred back to and a disputed rating to be examined; judgemental data is not kept longer than it is useful | deployer to complete |
| `learning` | 1095 | `WPM_RETENTION_LEARNING_DAYS` | `assessment_instruments`, `assessments`, `cpd_entries`, `cpd_requirements`, `learning_paths`, `mentorships`, `path_enrollments`, `professional_registrations`, `skill_courses`, `skills`, `training_enrollments`, `worker_skills` | Training and registration evidence may be asked for after a worker leaves; three years is a middle figure | deployer to complete |
| `wellbeing` | 365 | `WPM_RETENTION_WELLBEING_DAYS` | `adjustment_requests`, `ergonomic_assessments`, `ergonomic_items`, `pulse_surveys`, `wellbeing_entitlements` | Sensitive, health-adjacent data: kept as briefly as the purpose allows. Adjustment words are scrubbed at erasure in any case (WPM-D25) | deployer to complete |
| `planning` | 365 | `WPM_RETENTION_PLANNING_DAYS` | `announcements`, `benchmarks`, `change_initiatives`, `groups`, `role_profiles`, `workforce_plans` | Plans, reference data and announcements hold no personal data of their own beyond authorship | deployer to complete |

## What the schedule does and does not cover

- **Covers** the 56 soft-deleting tables, and **candidates whose consent has expired**
  (scrubbed after the `recruitment` horizon, then removed by the next sweep).
- **Does not cover** the audit log (`audit_logs`), the headcount snapshots (aggregates, no
  names), the erasure ledger (a pid and a time, which no longer identifies anyone) or
  backups. Backups expire after `BACKUP_RETENTION_DAYS` (default 35 calendar days;
  [backup-and-restore](../operations/backup-and-restore.md)). A deployer sets a retention
  for the audit log in their own policy: this repository does not delete it.
- **Does not decide** when a record is *deleted*: that is a person's act, or the erasure of
  a subject. The schedule says how long a deleted record is held before it is removed.

## Changing a horizon

1. Decide the basis and write it in the last column.
2. Set `WPM_RETENTION_<KIND>_DAYS` and restart.
3. Check `GET /api/retention/schedule` shows the value and `source: override`.
4. Check `GET /api/retention` for how many rows are now past their horizon, **before**
   running `POST /api/retention/sweep`, which is irreversible. Take a backup first.
