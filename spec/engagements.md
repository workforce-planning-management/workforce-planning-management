# Engagements: end dates, extensions and contractor details (WPM-R79, WPM-R80)

What is **built** of the contingent-workforce proposal in [plan.md](plan.md) section D: the pure rules
(WPM-T135) and the records and endpoints (WPM-T137). The end-of-engagement reminders (WPM-T138), the
snapshot split, programme links, the contingent workforce view and the user interface are not built.

## The basis decides the end date

`employment_type` is the **engagement basis** and is separate from the working pattern (`fte_percent`,
WPM-D56). The stored values are `permanent`, `fixed_term`, `contractor` and `intern`.

| Basis | End date (`engagement_ends_on`) |
| --- | --- |
| `permanent` | Refused |
| `fixed_term`, `contractor` | Required |
| `intern` | Allowed, not required |

The end date is the **last day** of the engagement: the worker is engaged on it and not the day after. It
must be after the start. The rule applies when a worker is created and when one is hired from an
application.

A recorded end date moves only by an **extension** (`POST /api/workers/{pid}/engagement/extensions`), which
must move it later and is kept as a dated entry (previous end, new end, reason, who), so the history shows how
often an engagement was extended. `PUT /api/workers/{pid}` can record a *missing* end date but not change one.
The audit entry names the event, not the reason.

## Existing engagements are listed, not invented

A fixed-term or contractor worker with no end date, such as a row from before the column existed, gets no
invented date. `GET /api/engagements/missing-end-date` lists them until HR records one, and the worker's
engagement shows the standing `missing_end_date`.

## Standing

`GET /api/workers/{pid}/engagement` reports where an engagement stands against the reminder window (default 60
calendar days): `open_ended`, `missing_end_date`, `running`, `ending_soon`, `past_end_undecided` and
`past_end_decided`. An engagement past its end is **never treated as continuing**. Recorded decisions arrive
with the reminders (WPM-T138), so today a past end always reads `past_end_undecided`.

## Contractor details

For a `contractor` only: a supplier (an `organization:` URN), a route (`agency`, `own_company`,
`statement_of_work`, `direct`), and a rate with its basis (`day` or `hour`, all three of amount, currency and
basis or none), and optional employment-status assessments (outcome `contractor`, `employee` or `undetermined`,
a date not in the future, and the reviewer, who is the caller). Whether a deployment requires an assessment, and
what its outcome means, is its own jurisdiction's question; the software records it.

The **rate is masked like salary**: it is in its own table, never on the worker row that many endpoints return.
Under the shipped reference policy, payroll and the worker see it; HR, who records it, and everyone else see
that a rate exists and not what it is. An unmasked read is audited as `rate_read`, with no amount. A write never
echoes the rate. Contractors are **not on payroll** (WPM-D58).

All of it is about one person: it is in the subject-access export and is erased with the worker.

## Limits

- **Table, not columns.** The plan put supplier, route and rate on `workers`; they are in
  `worker_contractor_details` so no existing worker endpoint can return the rate.
- **Retention.** The three new tables are not in the retention sweep (like the pay position); they go with the
  worker's erasure.
- **Not built:** end-of-engagement reminders and recorded decisions (WPM-T138), the funding source beside the
  end date (WPM-T161), conversion plans (WPM-T162–T164), and any user interface.
