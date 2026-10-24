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

## Pay position analysis (WPM-R154, proposed; spec round WPM-T268)

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
