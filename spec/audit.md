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

- `expense_claim_created|submitted|draft|cancelled|approved|rejected|reimbursed` (never an amount, title or description),
- `pay_position_set|cleared` on a worker (never the band, step or amount),
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

### Append-only and hash-chained (WPM-R115, WPM-D72)

The audit trail is protected by the database, not by convention. Two triggers on `audit_logs`:

- **Append-only.** An `UPDATE` or `DELETE` on any row is refused, whoever asks, with a SQL error naming the
  table. Only a database owner who first disables the trigger can change a row, and doing so leaves a trace
  below.
- **Hash-chained.** Each row has a `chain_seq` (its place, assigned under a transaction-scoped advisory lock so
  the order is the order of insertion), a `prev_hash` (the previous row's `entry_hash`, or 64 zeros) and an
  `entry_hash`: SHA-256 over the previous hash, the sequence, and the row's entity, record, action, actor,
  snapshot and time (UTC), each text field length-prefixed. One SQL function, `audit_entry_hash`, defines it;
  the trigger, the back-fill of rows that pre-date it, and the verifier all call it.

`GET /api/audits/verify` (privileged callers) and `cargo loco task verify_audit_chain` re-compute the chain and
report `ok`, the number of `entries`, the `head_hash` and the `first_break`. A rewritten row breaks at that row;
a removed row breaks at the row after it; the task exits with an error when the chain is broken.

**What it does not prove.** A chain proves rows were not changed or removed from the middle. Cutting rows off
the end leaves a valid shorter chain, so the operator must **record the head hash somewhere the database cannot
reach** (a log service, a ticket) on a schedule and compare. A database superuser can disable a trigger and
re-chain everything after a rewritten row; only the outside record shows it. `TRUNCATE` is not a row change and
is not blocked (a deployer revokes it from the application's role, WPM-T199). A restore from a dump keeps the
hashes (the data loads before the triggers exist); the restore drill checks the chain is intact and the head
hash identical.

**Cost.** Audit inserts serialise on the lock until their transaction ends, so write throughput is bounded by
the slowest transaction that writes an audit row, and two transactions that take row locks in opposite order
around an audit write can deadlock (the database aborts one; the request fails and can be retried). Measured
under load in WPM-T196, not yet.
