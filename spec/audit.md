# Audit & events

## Audit trail

Family conventions: every mutation writes an audit row (entity kind,
pid, action, actor, snapshot). Because HR data is regulated personal
data, **sensitive reads are audited too**: employee-record reads that
include salary, payslip reads, review-content reads, succession-plan
reads, 360 report reads (`report_read`), unmasked adjustment-request
reads (`adjustments_read`), unmasked assessment-score reads, and
subject-access exports (`subject_access_exported`) each record who
read what, when. Approval and override actions (leave approvals,
checklist waivers, calibration changes, payroll approvals, adjustment
decisions, erasures with per-step counts, retention sweeps with
counts) carry their detail in the snapshot.

**One designed exception**: a pulse submission's audit row carries
**no actor** (WPM-D20) — the trail records that a submission
happened, never who made it, because an actor-stamped row would
silently defeat the structural anonymity.

## Event kinds

`requisition_opened` / `requisition_filled`, `application_staged`,
`worker_hired` / `worker_activated` / `worker_terminated`,
`onboarding_item_completed`, `time_recorded`, `leave_requested` /
`leave_approved` / `leave_rejected`, `shift_assigned`,
`benefit_enrolled`, `review_submitted` / `review_shared`,
`training_completed`, `payroll_run_calculated` /
`payroll_run_approved` — the family envelope, deduped by consumers
on `event_id`; transactional outbox under the `outbox` transport.
Later rounds added audit actions such as `acknowledged` (wellbeing),
`submitted` (pulse — actor-less; 360 responses — actor-carried),
`status_changed` (appraisals, adjustments), `answered` / `completed`
(ergonomics), `erased`, `retention_swept`, and in-app notification
kinds (`appraisal_request` / `appraisal_shared` /
`adjustment_update` — reference-only, WPM-D23).

## Actions added since WPM-T36

Each is recorded against the worker (or rota, announcement, movement) with the
actor, and — where a detail would itself be sensitive — **without it**:

- `job_level_set|cleared` on a worker (never the level), `grade_set|cleared` on a
  role profile (the level and band named — a role is not a person),
- `emergency_contact_added|updated|removed` (never the contact's details),
  `backup_added|updated|removed`;
- `created|updated|retired` and `swap_added|swap_removed|swap_requested|
  swap_accepted|swap_declined|swap_cancelled` on a rota;
- `posted|edited|retired` on an announcement;
- `joiner_opened|leaver_opened|…_completed|…_cancelled`,
  `movement_item_added|done|skipped|reopened|assigned`, and per handover action
  `handover_reassigned|handover_closed|handover_revoked` (kind, thing and new
  holder in the snapshot) — duplicated in the `handover_actions` table, which
  **survives erasure** (notes scrubbed, rows kept).

Skill-gap and training-plan reads are derived views and are not audited
individually; they expose no record the underlying reads do not.

## Integrity

State transitions + audit + outbox share one transaction; approval
races (two managers approving the same leave) are serialized with row
locks (`FOR UPDATE`, the patient-flow bed pattern); payslip
reconciliation (net = gross − deductions) is enforced before persist.
