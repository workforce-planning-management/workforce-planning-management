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

**A jurisdiction profile comes first** (WPM-T384, WPM-D153). An organization whose HR has recorded a home jurisdiction may hold a
`retention.<kind>.days` row (a floor, a ceiling or both, in calendar days, combined `bounded` across the chain); for that
organization the horizon is the resolved floor, or the ceiling where no floor is given, still never below 30 calendar days, and
`GET /api/retention/schedule?organization=` says where each came from. The sweep is one pass over every organization's rows, and a
soft-deleted row does not always say whose it is, so for each kind it takes the **longest** of the deployment's horizon (the
variables above) and every organization's profile: a profile can make the sweep keep a record longer, never delete one sooner.

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
- **Does not cover** idempotency keys (`idempotency_keys`), which are not a record of a person: a key and the
  response it earned are kept for **one calendar day** and deleted by the request that next arrives with any key
  (WPM-R132). The stored response may repeat what the first response held, which is why it is kept so briefly.
  It is not exported with a worker or erased with one; it is gone within a calendar day.
- **Does not cover** post funding (`post_fundings`), which is not soft-deleted: clearing it deletes the row at once, and
  erasing the worker deletes it with them, so there is no deleted state to hold and sweep (WPM-T109, WPM-T161). It holds a kind,
  a programme reference and an end date, never an amount.
- **Does not cover** governance commitments (`governance_commitments`), which are not soft-deleted: removing one deletes the
  row at once (WPM-T111). A row holds a role, a frequency, a length and a head count, never a person.
- **Does not cover** the cutover tables (`cutovers`, `cutover_roles`, `cutover_pools`: an organization's own plan for a go-live, naming a programme, a site, a role and a pool, never a person; removed at once with the cutover) and `oversight_packs` and `oversight_pack_withdrawals` (a frozen aggregate with no person in it, append-only by a trigger: it is never changed or removed), `early_warnings` (it names a pool or a measure and the dates they were seen and acknowledged, never a person) or the oversight tables (`oversight_levels`, `oversight_positions`, `oversight_criteria`, `oversight_measures`,
  `oversight_readings`), which are an organization's own configuration and evidence, not records of a person, and are not
  soft-deleted: a level, criterion or measure is removed at once (a measure with its readings), and a recorded position stays on the
  record (WPM-T113).
- **Does not cover** the audit log (`audit_logs`), the headcount snapshots (aggregates, no
  names; since WPM-T139 they also hold contractor and fixed-term headcount and FTE per department and date, which are counts, and
  are kept as the history a trend needs), the erasure ledger (a pid and a time, which no longer identifies anyone) or
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
