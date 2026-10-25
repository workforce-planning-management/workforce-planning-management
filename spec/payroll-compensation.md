# Pillar 5 — Payroll & compensation

## Payroll runs

A run covers one organization and period and is a strict state
machine: `draft → calculated → approved → paid`, each transition
audited with actor. **Calculate** produces one payslip per active
employee from pure-core arithmetic. **A contractor is never on payroll** (WPM-D58): they are paid
against invoices through their supplier, so a run produces no payslip for an `employment_type` of
`contractor` even when a rate is recorded in the salary field. A permanent, fixed-term or intern
worker is paid as usual:

- **Gross** — the salary pro-rated to the period (FTE-aware), plus
  overtime minutes × the derived rate.
- **Deductions** — kinded lines (`tax`, `social`, `pension`,
  `other`); v1 rates come from configured stub tables (percentages),
  **not** jurisdiction-complete tax law (documented limitation) —
  pension lines derive from active benefit enrollments.
- **Net** — gross − Σ deductions, with the invariant enforced in
  code (a payslip that doesn't reconcile refuses to persist) and
  every amount an overflow-checked i64 in minor units.

Recalculation is allowed from `draft`/`calculated` only; `approved`
freezes the slips; `paid` is a record of fact, not a payment
execution (bank-file export is roadmap). Payslips are visible to the
payroll persona and to each employee (their own, via self-service);
every payslip read is audited.

## Salary benchmarking

Benchmarks record market min/median/max per job title (+ optional
location) in a currency, dated `as_of`. The comparison view joins
active employees to their title's benchmark and flags:

- `below_min` — retention risk, and
- `above_max` — cost outlier,

with the portfolio-style rollup per department. Benchmarks are data
the deployment loads (import or manual entry) — WPM does not scrape
market data.

## Pay position analysis (WPM-R154; spec round WPM-T268, built WPM-T281)

How pay sits inside its band, as an aggregate. For each band of a pay scale ([pay-scales.md](pay-scales.md)):

- **Compa-ratio**: pay divided by the band's midpoint, per group (a band, a level, a department, a location), and **range
  penetration**: how far through the band from its minimum to its maximum the pay sits. Both are shown with the number of workers behind
  them. A band with no midpoint or no pay in the group gives `unknown`, not zero.
- **Outliers** by level, department and location: groups whose compa-ratio sits far from the others' (the rule and its cut-off are
  configuration, stated in the payload), shown as groups, never as named workers. **The floor applies** (WPM-D32): a group under it is
  withheld, and a total that would reveal a withheld group is not shown.
- **Who reads it.** HR and payroll roles that may already read salaries; a caller who may not sees `unknown` with the reason, as the plan cost
  view does (WPM-D93).
- **What it is not.** It is the **evidence** a future compensation review (roadmap) would use, not the review: it proposes no pay
  change and no worker's pay is compared with another's. Delivery: WPM-T281.

**Built (WPM-T281).** `GET /api/reports/pay-range?organization=&as_of=&cutoff=`, in `rules::pay_range` (pure) and
`controllers/pay_range.rs`. It needs the same unmasked salary read as the pay gap report (payroll, the service, administrators; HR
reads salaries masked and is refused, a worker is refused), and every read is audited (`pay_range_report_read`, naming the day only).
Nothing is stored: no table, no column, no export or erasure wiring.

- **Inputs.** Employed workers of the organization on the day (headcount definition). Pay is the salary on the worker record, which
  WPM holds as full-time-equivalent annual; the band is the worker's recorded pay position (scale and band), the level their
  recorded job level, the department and location from the worker record. Pay in another currency than the analysis currency (the
  `currency` query, else the most common among workers with a band) counts as no pay.
- **Definitions.** Midpoint is the middle of the band's entry and top steps (an intermediate step is not used). Compa-ratio is pay
  over midpoint in basis points (10 000 at the midpoint); range penetration is 0 at the entry step and 10 000 at the top and is **not
  clamped**, so pay outside the band shows as under 0 or over 10 000. A band of one rate (band 2) has a compa-ratio but its
  penetration is `unknown` (`no_width`). Group figures are the mean of the workers' ratios, rounded half up; integer arithmetic.
- **Groupings** by band, level (`framework:number`), department and location; a worker with no level or location is in a group
  called `unrecorded`, which is subject to the floor like any other.
- **Floor and the total (WPM-D108).** A group under the floor (10, `DEFAULT_FLOOR`) is `withheld` with no count. The total is shown
  only when no grouping has withheld groups that together are fewer than the floor (one small group would otherwise be the total less
  the others); else it is `unknown` with the reason `would_reveal`, and no outlier is called, since there is nothing to compare with.
  Workers left out (no pay; pay but no band on a known scale) are counted by reason and the count shows only from the floor, except that none is said as none (nobody is picked out by it).
- **Outliers.** A shown group whose mean compa-ratio is more than the cut-off from the mean of everyone analysed: `above` or `below`
  with the distance, else `no`. The cut-off is a query in whole percentage points, 1 to 50, default 10, and is stated in the payload.
  Groups only; no worker is named and no worker's pay is compared with another's.
- **Tests.** 18 unit tests (boundaries, the floor and one under it, the total held back and released, the cut-off exactly, single
  rate bands, no pay is not zero, a source test that an entry carries no pid or name), six deliberate faults tried and all caught; a
  request test against Postgres (groups, withholding, the total, outliers, the default cut-off, nothing named, bad input); the
  enforcement test (401, worker 403, HR 403 masked, payroll 200).
- **Not built:** a screen; a CSV; the intermediate step as a reference point; part-time pay normalisation (the salary field is read
  as full-time-equivalent, as the pay gap report reads it); a comparison across years; a pay change proposal (by design).

### Decisions

- **WPM-D108 — A total is not shown when it would give a withheld group away.** Withholding a small group is not enough if the total
  and the other groups are shown: the small group is their difference. A total is therefore shown only when, in every grouping, the
  withheld groups together reach the floor (or there are none). Groups and totals both carry no count below the floor.
