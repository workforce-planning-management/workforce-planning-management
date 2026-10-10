# Pay scales: a national public-sector pay scale (WPM-R51)

A banded public-sector pay and grading scale, issued by the government in a pay
circular each year. WPM carries the scale as **reference data** and answers
"where does this salary sit, and is a step up due?". Delivered as WPM-T92 and
WPM-T96, and made generic in WPM-T102 (see [tasks.md](tasks.md)).

## Research (2026-10-06)

Primary source: the government's pay circular for 2026/27, Annex 1. It states:

- the 2026/27 scales apply **from 1 April 2026** and are the previous year's
  **plus 3.3%**; the sleeping-in and on-call allowances rise 3.3%;
- the structure is the one below, which differs from other published scales;
  a separate advance payment is excluded from the uplift;
- **bank workers** are paid the same scales (not modelled — WPM has no
  bank-worker concept).

| Band | Entry | years | Intermediate | years | Top |
| --- | ---: | :-: | ---: | :-: | ---: |
| 1 (closed) | £26,300 | | | | |
| 2 | £26,300 | | | | |
| 3 | £26,300 | 2 | | | £27,890 |
| 4 | £28,819 | 3 | | | £31,626 |
| 5 | £32,557 | 2 | £35,114 | 2 | £39,631 |
| 6 | £40,559 | 2 | £42,805 | 3 | £48,841 |
| 7 | £50,129 | 2 | £52,712 | 3 | £57,365 |
| 8a | £58,379 | 2 | £61,317 | 3 | £65,723 |
| 8b | £67,583 | 2 | £71,952 | 3 | £78,530 |
| 8c | £80,698 | 2 | £85,611 | 3 | £92,984 |
| 8d | £95,773 | 2 | £101,643 | 3 | £110,448 |
| 9 | £114,475 | 2 | £121,377 | 3 | £131,732 |

"years" is the circular's *years until eligible for pay progression* from that
step. Band 1 is closed to new entrants; band 2 is a single rate. Allowances:
sleeping-in £44.82; on-call £26.05 (weekday or weekend) and £52.08 (public
holiday). Annual figures are **full-time, 37.5 hours a week**.

Secondary sources (pay-calculator sites) were used only to find the circular;
their band minimum/maximum agree with it, but **their structure does not** (they
omit the intermediate step), so the circular is the only source of the figures.

## WPM-R51 — Pay scales

*As anyone signed in I can see the national pay scale, and check where a
salary sits on a band and whether a step up is due.*

- `GET /api/pay-scales` lists the scales (id, nation, effective date, uplift,
  source circular, band codes). `GET /api/pay-scales/{id}` returns the whole
  scale: every band, step (pence) and years-to-progression, and the allowances.
  Id today: `national-2026-27`. An unknown id is **404**.
- `GET /api/pay-scales/{id}/position?band=&salary_minor=[&step=&months_on_step=]`
  places a **full-time-equivalent annual** salary on a band: `on_step`,
  `between_steps`, `below_entry` (with the shortfall) or `above_top` (with the
  excess). With `step` and `months_on_step` it also says whether progression is
  `due`, `not_yet` (months remaining) or `at_top`, with the next step's pay.
  Band codes are case-insensitive. Missing band, unknown band, nothing asked,
  negative salary, `step` without `months_on_step` (or the reverse) and a step
  not on the band are **422**.
- Money is minor units; the week is whole minutes (2250), never a float.
- UI: `/pay-scales` — the table (entry, intermediate, top, with years; band 1
  marked closed), the allowances, and the lookup form.
- **Reference data.** A new year is a new scale added in
  `src/rules/pay_scale.rs` with its circular named in `source`; the old scale
  stays so past figures remain checkable.

## WPM-D38 — A scale is transcribed, dated and sourced; a lookup stores nothing

- The figures are **transcribed from the circular**, never derived by applying a
  percentage to last year's, so a rounding rule cannot make WPM disagree with the
  employer. Each scale carries `effective_from` and `source`.
- The lookup is **stateless**: nothing is stored against a person, so it adds no
  table, no export or erasure wiring and no salary masking surface. A caller who
  asks about a worker has already read the salary through the audited, maskable
  `GET /api/workers/{pid}`.
- A salary **between** steps is reported as such, not rejected: protected pay and
  pro-rata figures are real. The caller normalises part-time pay to full-time
  equivalent first; the response states the basis.
- **Eligibility is only the circular's years on the step.** Whether the
  employer's own conditions for progression are met is not modelled, and the
  wording says "eligible", never "will be paid".

## WPM-R54 — A worker's pay band and step, and when they can move up

*As a person I can record which band and step of the scale I am on and since when,
and see when I become eligible for the next step; I am told when that is near.*

- `GET|PUT|DELETE /api/workers/{pid}/pay-position`. One current position per worker:
  `{scale, band, step, step_since?}` (`step` is 1-based; `step_since` defaults to today
  and cannot be in the future). The scale, band and step are validated against the
  reference scale — an unknown scale or band, a step off the band (band 2 is a single
  rate: only step 1) or a future date is **422**. `GET` returns `{"pay_position": null}`
  until one is recorded; `DELETE` of none is **404**.
- The response shows what the step pays (pence, currency) and the **progression
  standing on today**: `at_top`; `due` (with the date they became eligible and the next
  step's pay); or `not_yet` (the eligibility date, whole `days_remaining`, the next step's
  pay). The eligibility date is `step_since` plus the circular's years on that step (a
  leap day clamps to the end of February).
- **The worker and HR only**, as for a job level; edits record who and whether on
  the person's behalf; the audit entry (`pay_position_set`, `pay_position_cleared`) names
  no band, step or amount.
- **Reminder:** the loco task `pay_progression_reminders [days_ahead:0–90]
  [as_of:YYYY-MM-DD]` (default 30 calendar days; **schedule daily**) tells each person who
  becomes eligible within the window, once (idempotent per eligibility date), if they are
  still employed on that date. The in-app notification (`pay_step_due`) says only
  "You become eligible to move up a pay step on <date>." — **no band, step or amount**.
- UI: a **Pay band and step** panel on `/me` and the worker's page (hidden for anyone
  who may not see it). Migration `m20261006_000048_pay_positions` (and
  `m20261009_000050_generic_pay_scale_id`, which moves stored ids to
  `national-2026-27`); in the
  subject-access export; deleted on erasure.

## WPM-D41 — A pay position is a salary; a reminder promises nothing

- A band and step **is** a salary, so it has the audience of the sensitive records
  (WPM-D40): the person and HR only — stricter than the salary read, which a manager may
  have unmasked. The audit entry, the notification and the logs never carry the band,
  step or amount.
- **Eligible, not "will move".** Eligibility is the years on the step the circular sets;
  the employer's own conditions are not modelled. Every surface says "eligible".
- It is **recorded, not reconciled**: WPM does not compare the position with the salary
  field (a salary may be pro-rata or protected, and what the field means is the
  employer's), nor change one when the other changes.
- The reminder reaches **the person**, not their manager or HR: telling a third party
  would disclose pay position to someone the record is not shared with.

## Not done

- Telling HR or a manager that someone becomes eligible; a team view of who is due
  (a count of one names someone); pay-position history.
- Comparing a position with the recorded salary (see WPM-D41); flagging a salary that is
  off-scale; linking requisitions to a band. (A role's band is WPM-R53, in
  [job-levels.md](job-levels.md).)
- Scales from other jurisdictions or other years (each has its own circular).
- Bank-worker hourly rates; hourly rates from the annual figure (the divisor is a
  contractual choice).
- Applying the on-call allowance to the rota ([people-directory-and-cover.md](people-directory-and-cover.md)).
