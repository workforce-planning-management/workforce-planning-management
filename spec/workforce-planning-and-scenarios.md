# Workforce planning depth: configurable pay scales, analysis, services, and versioned scenarios

Proposed 2026-11-20 from customer feedback ([plan.md](plan.md) section X). Requirements **WPM-R254–R266**, **WPM-R275** and **WPM-R276**,
decisions **WPM-D207–D213**, tasks **WPM-T553–T573** and **WPM-T596–T601** in [tasks.md](tasks.md). **Nothing here is built.**
The owner decided on 2026-11-20 that **WPM tracks per-person allocation** (WPM-D211, R275); the other decisions are still the
owner's to confirm. It extends [strategic-workforce-planning.md](strategic-workforce-planning.md) (plans, forecast, gaps),
[delivery-capacity.md](delivery-capacity.md) (pools, demand, start check), [positions.md](positions.md) (establishment,
current versus target), [pay-scales.md](pay-scales.md) and [job-levels.md](job-levels.md) (bands and steps), and
[staff-records-and-pay.md](staff-records-and-pay.md) (assignments, cost centres).

> ⚠️ Demo software ([regulatory.md](regulatory.md)). NHS Agenda for Change is the **worked example**, not the design
> (WPM-R214): every scale, band, and pay point is dated data that an authorized person enters and sources.

## What the customer asked for, and what WPM already does

The feedback lists eight goals, fifteen needs, two bonus frameworks, and six scenario capabilities. Each was checked against
`spec/` on 2026-11-20.

| Asked for | Already built or specified | Gap | Here |
| --- | --- | --- | --- |
| Headcount and FTE | Workers' FTE, snapshots by basis (R35, R84), establishment (R152), assignments (R161) | None at the record level; no single view | R266 |
| Allocation across projects, products, services | Programme demand per pool and month (R57); post funding by programme (R83) | Demand from **products and services** has no place; a service has no staffing floor; **no per-person allocation** (declined by WPM-D45 until the owner's decision of 2026-11-20) | R258, R275 |
| Roles, titles, grades, pay bands, structure | Role profiles (R34), job levels and a role's band (R52, R53), org chart, cost-centre tree (R163) | No **organizational unit** hierarchy (directorate, department, team); department is a text attribute | R257 |
| AfC band and pay point per post or role | A role's band (R53); a worker's band and step (R54) | A **post** (position) has no band or pay point; no reporting by pay point | R255 |
| Authorized users configure bands, points, values | Scales are transcribed in code (WPM-D38, `pay_scale.rs`) | **No configuration**: a new year or local scale needs a release | R254 |
| Analyse by role, title, band, point, FTE, area, cost, alone or combined | Separate aggregates: by assignment category (R162), cost centre (R163), compa-ratio per band (R154) | **No combined analysis**; combining dimensions raises re-identification risk | R256 |
| Over-allocation, under-utilization, conflicts | Pool `over` and the constraint pool (R59); assignment FTE over 100 (R161); shift conflicts | **Under-use (slack) is not reported**; no conflict between claims on a critical service; no person-level over-allocation or clash with leave | R259, R275 |
| Planned versus actual effort | Nothing; no timesheets by decision (WPM-D45) | **No actuals** | R260, R275 |
| Dynamic reallocation | Plan levers include redeploy (R37); start-check delay (R60) | **No suggestion** to move capacity between pools | R259 |
| Prevent bottlenecks in critical services | Constraint pool, operations reservation (R56), key-person risk (R61) | **Services are not first-class** | R258 |
| Internal before contractors | Partner commitments (R58), contingent view (R86) | **No internal-first check** before a partner request or contractor requisition | R259 |
| Workforce as-is versus to-be | Forecast gap (R37), org-shape diff (R153, without cost and skills), establishment | Cost and skills difference not built; no combined view | R263 |
| Skills as-is versus to-be | Competency gap per demand line (R37), skills gap (R43) | Not rolled up per unit or scenario | R263 |
| Model structures, delivery models, directorates, teams | Target role nodes per plan (R153) | No unit levels; no service delivery model; one target per plan | R257, R261 |
| Real-time visibility; accurate forecasting | Derived on read; transparent rules with `insufficient_history` (WPM-D27) | No overview; **forecast accuracy is never measured** | R265, R266 |
| Integration with financial planning | Plan budget, on-cost, hiring cost (WPM-T51); financial coding (R164) | **No cost forecast by cost centre and month, and no budget import** | R264 |
| What-if: budget up, down, or frozen | One budget per plan | No budget change or freeze as a scenario lever | R261 |
| What-if: shortages, recruitment delays | Attrition assumption (R36) | No supply shock; hires land with **no time-to-fill lag** | R261 |
| What-if: strategic priority changes | Objectives on demand lines (R38) | No priority order deciding what survives a constraint | R261 |
| Scenario comparison and version control | Several plans coexist; one active (R36) | **No lineage, no immutable version, no side-by-side comparison** | R261 |
| Impact on workforce, cost, training, skills, schedules, timelines | Separate views; start-check delay | No single impact answer per scenario; schedules belong to PPM (WPM-D55) | R262 |
| ITIL (Version 5): product, service, AI governance | AI deployer register (R148) | Not mapped; ITIL text is licensed | R258, WPM-T569 |
| Agile Manifesto, customer-led work | Capacity by team (WPM-D45) suits stable teams | Not stated | WPM-D213, WPM-T570 |

## Requirements

### WPM-R254 — Pay scales are configuration that an authorized person maintains

*As a pay-scale administrator I can add and correct pay scales, bands, pay points and their values, so that a new pay award or
a local scale needs no release.*

- **A scale is dated data**: id, organization (or `shared` for a national scale), name, jurisdiction, currency, effective from,
  an optional effective to, the hours a full-time value assumes (whole minutes a week), and a required **source** (the circular
  or agreement, with its date). Bands carry a code, an order, a `closed` flag and their **pay points**; each pay point carries an
  order, a value in minor units, and the years on it before eligibility to progress (or none). Allowances are named values.
- **A draft, then a publication.** A draft is edited freely. Publishing needs a **second authorized person** (maker and checker,
  never the same) and makes the version **immutable**. A correction is a new version that names what it corrects, so every past
  figure stays checkable (the intent of WPM-D38 is kept: values are transcribed from a source, never derived by applying a
  percentage, though the form may *offer* an uplift as a draft for a person to check against the source).
- **Who.** A new policy action `pay_scale_admin`, held by HR or payroll as the deployer configures. Everyone signed in reads a
  published scale, as today. Every create, edit, publish, and retirement is audited; a draft is visible to administrators only.
- **The shipped NHS 2026/27 scale becomes seed data** (`national-2026-27`), labelled as a worked example, with its source.
  Pay positions (R54), role bands (R53), and pay-range analysis (R154) read the store, never the code constant.
- **Not done:** importing a scale from a file (a later task once the shape is settled); local supplements such as a high-cost
  area allowance (a named allowance can record one, but nothing applies it).

### WPM-R255 — A post carries its grade

*As HR I can record the band and pay point a post is graded at, and report the establishment by band and pay point.*

- A position (R152) and an assignment (R161) gain an optional **grade**: `{scale, band, pay_point?}`, validated against a
  published scale on the effective date. Unrecorded is `unknown`, never the entry point.
- A grade is a **reference, not an amount** (WPM-D208): the position still has no pay column, and its cost is derived from the
  scale, shown only to callers who may read salaries.
- The establishment view (R152) adds headcount, budgeted FTE, held FTE, and vacancy **by band and pay point**, with the floor
  for callers who may not see holders.
- A worker's own band and step (R54) may differ from their post's grade (protected pay, acting up, a trainee); the difference is
  shown to HR, never reconciled automatically.

### WPM-R256 — Workforce analysis over combined dimensions

*As a planner I can analyse headcount, FTE, vacancy, and cost by any combination of role, job title, band, pay point, FTE band,
organizational unit, cost centre, and engagement basis.*

- `POST /api/analysis/workforce` takes up to **three dimensions**, the measures, an as-of date, and optional filters, and returns
  a table. The query is a small allowlisted shape, not free SQL.
- **Every cell has the floor** (10 by default, as elsewhere). Because a combination can isolate a person by subtraction, the pure
  core also applies **complementary suppression**: where withholding one cell leaves it derivable from a row or column total,
  the next smallest cell in that row or column is withheld too (WPM-D209).
- **Cost** is a measure only for callers who may read salaries (the plan cost rule); others see `withheld`. Cost is derived from
  salary or, where asked, from the post grade on the configured scale, and the answer names its basis.
- Every analysis call is audited with its dimensions and filters, never its cells.
- **Not done:** saved views; charts beyond a table and a bar chart; export (R264 is the financial export).

### WPM-R257 — Organizational units and service delivery models

*As a planner I can model the organization as directorates, departments, and teams, and say which units deliver which services.*

- **An organizational unit tree** per organization, with configurable level names (for example directorate, department, team),
  dated validity, and a parent. A unit is never deleted; it is closed. The worker's `department` attribute resolves to a unit;
  existing free-text values are **listed as unmatched** for HR to resolve, never guessed (as cost centres are, R163).
- A unit may map to one or more cost centres; the two trees stay separate, because one is how the organization is managed and
  the other how it is costed.
- **A service delivery model** records which units deliver which services (R258) and with which pools. A plan's target
  structure (R153) can be expressed in units as well as role nodes.
- ABAC scoping keeps the `department` attribute; a scope by unit subtree is a later decision.

### WPM-R258 — Products and services make demand; a critical service has a floor

*As a service owner I can say what a product or service needs from a pool each month, and the minimum it must never fall
below.*

- A demand claim (R57) names a **programme, a product, or a service** by `EntityRef` URN. Products and services are owned
  elsewhere (a service catalogue or configuration management database); WPM keeps the reference only (WPM-D134).
- A service may be marked **critical** with a **floor** per pool: the FTE it needs to run safely. The capacity view reports
  `below_floor` for any month where the pool's supply, after the active demand of everything else, would leave a critical
  service under its floor. That is the bottleneck warning; it never blocks a claim.
- The operations reservation (R56) can be broken down by service, so "kept for running live services" says which.
- ITIL (Version 5) terms (product, service, value stream) are **mapped by name** where the deployer uses them; no licensed text
  is copied (WPM-D100).

### WPM-R259 — Allocation signals and reallocation suggestions

*As a planner I can see where capacity is over-committed, under-used, or in conflict, and what could move.*

- **Under-use:** a pool-month whose committed demand is below a configured share of supply (default 80%) reports `slack` with the
  FTE amount. A pool with no demand recorded is `unknown`, not slack (WPM-D46).
- **Conflict:** two active claims that together put a critical service below its floor (R258) are listed as a conflict, naming
  the claims, never people.
- **Reallocation suggestions:** for each `over` pool-month, pools with `slack` the same month whose members hold the over pool's
  required skills at its minimum proficiency, with the FTE that could move. A suggestion carries its evidence and is never applied
  automatically (WPM-D28, WPM-D50).
- **Internal first** (WPM-D210): a partner request (R58) or a contractor requisition (R82) shows the internal slack that matches
  its pool or role profile. Proceeding despite slack needs a recorded reason, kept with the decision; nothing is refused.
- Person-level over-allocation, availability, and clashes with leave are R275. A reallocation suggestion may list the
  **available people** who hold the skill, as evidence behind the lever (WPM-D28): in name order, never ranked or scored, and
  only to callers who may read allocations.

### WPM-R260 — Planned versus actual, at pool level

*As a planner I can compare the FTE a programme, product, or service was planned to use with what it used.*

- **Actuals are aggregates per pool, demand source, and month**: FTE used, entered by the pool owner or received from PPM through
  the bridge (PPM owns project actuals). Each figure carries its source and when it was recorded. A month with none is `unknown`.
- The view shows planned and actual, and the variance, per month and cumulative. **Planned** is the active claim, or, where
  the pool tracks allocation, the sum of confirmed allocations (R275), and the view says which.
- **Actuals stay aggregates** for now: WPM records who is *planned* to work on what, not what each person *did*. Per-person
  actual effort would be a timesheet; the owner's decision covers allocation, and timesheets remain out (WPM-D211). Deriving
  actuals from time entries is not done, because they record attendance and overtime, not what the time was spent on.

### WPM-R275 — Per-person allocation

*As a resource manager I can allocate a person's time across programmes, products, and services month by month, and see who is
over-allocated, who is available, and where an allocation clashes with leave.* (Decided by the owner on 2026-11-20, WPM-D211.)

- **An allocation** is a worker's assignment (R161) × a demand source (programme, product, or service URN, R258) × an optional
  pool × a month range × a share of that assignment's FTE in hundredths (`fte_centi`). Status: `tentative`, `confirmed`, or
  `ended`. A pid, a source, and a reason note; who made it and when.
- **Tracking is switched on per pool or unit** (`allocation_tracked`). In a tracked scope a person with no allocation that
  month is **unallocated: a known zero**. In an untracked scope, allocation is `unknown`, never zero (WPM-D46), so a partial
  roll-out cannot report false availability.
- **Signals, per person and month** (pure core):
  - `over_allocated` when confirmed allocations exceed the person's available FTE: assignment FTE less approved leave, by the
    same day-weighting as pool supply. The amount is shown; nothing is refused, because the person decides.
  - `available` when below a configured share (default 80%), with the free FTE.
  - `leave_clash` when a confirmed allocation falls in a month whose approved leave leaves too little time. The kind of leave
    is never read or shown.
  - `assignment_over_allocated` when one assignment carries more than its own FTE, even where the person's total fits.
  - **Per allocation**, `runs_past_end` with the date, when it runs into a month after the assignment's or the engagement's
    last day (R79). This is a signal, not a refusal, because an end can be extended. Starting before the assignment, or after
    it has ended, is refused.
  - Over-allocation and a leave clash partition the cases that do not fit: over the contract is over-allocated whatever the
    leave. A person with nothing to give that month (not started, ended, or on leave all month) is never `available`.
- **Reconciles with pool demand**: per pool, source, and month, the sum of confirmed allocations is compared with the claim
  (R57) and shown as `staffed`, `understaffed`, or `overstaffed` by how much. Pool supply (R56) is unchanged; allocation says
  who, demand says how much.
- **Who may read named allocations** (WPM-D211): the person (their own, on `/me`), their line managers, HR, **resource
  managers** (WPM-R276) in the person's organization, and the owner of the demand source for allocations to that source where
  the deployer's policy grants it. Everyone else sees **counts and FTE with the floor**. Every read of another person's
  allocations is audited. **Writes** are by HR and resource managers; a resource manager does not write their own (pure rule
  `allocation::may_write`).
  The person is told in-app when an allocation of theirs is confirmed or changed.
- **No score.** No per-person utilization rate, ranking, or league table exists anywhere, and allocation feeds no review,
  appraisal, or pay record. An allocation says what a person is planned to do, never how well (WPM-D211).
- **Bridge.** PPM still receives counts and FTE per pool and source (WPM-D134); named allocations do not cross the bridge.
- **Records.** Subject-access export, erasure (allocations are deleted with the worker; the pool-level totals already reported
  stay), and a retention period in the schedule (default: 2 years after the allocation ends, for the deployer to replace).
- **Scenarios** (R261) copy allocations as **hypothetical** lines with the person replaced by their role and unit, so a
  scenario never names anyone (WPM-D26).
- **Built, pure core (WPM-T597, 2026-11-20).** `allocation` in the domain crate: statuses and their moves, validation,
  day-weighted contracted and available FTE (reusing pool supply's day count), the signals above, `sum_confirmed`, and
  `reconcile`.
- **Built, records and API (WPM-T598, 2026-11-20).** Table `allocations`; `GET`/`POST /api/workers/{pid}/allocations` (the
  person's allocations with each month's standing), `PUT`/`DELETE /api/allocations/{pid}`, `POST /api/allocations/{pid}/status`,
  `GET /api/allocations` (the named list for HR and resource managers, paged, in name order, optionally by source) and
  `GET /api/allocations/summary` (per pool, source and month, against the active claim, the floor for everyone else).
  **Differences from the requirement as written:** tracking is switched on **per organization**
  (`allocation_tracked` on `PUT /api/capacity/settings`), not per pool or unit, until organizational units exist (WPM-R257);
  the person reads their own through `/api/workers/{pid}/allocations`, with no `/me` route; a source's owner is never a reader,
  because WPM records no owner of a programme; changes are in the audit trail, with no history table; retention is the
  `time_and_leave` kind. No screen yet (WPM-T599).

### WPM-R276 — The resource manager role

*As an organization administrator I can name the people who plan who works on what, without giving them HR's rights.*
(Decided by the owner on 2026-11-20.)

- **Two ways to hold it**, as for HR: an organization membership role `resource_manager` (granted and revoked by an `org_admin`
  only; an `hr_admin` cannot grant it, and the role grants nothing to others), or an identity-provider role
  `wpm-resource-manager`, which maps to `attrs.resource_manager = ["true"]` (Keycloak realm role, Entra app role). **Writes need
  the token role**, because the policy decides a write from the token before the handler runs; it passes on that role for the
  allocation routes alone (`self_service::ATTRIBUTE_WRITES`). The membership role alone reads (WPM-T598).
- **What it allows:** reading named allocations of people in the organization, and creating, changing and ending their
  allocations (R275), except their own. It reads what allocation planning needs: assignments' FTE and dates, approved leave as
  **dates only**, engagement end dates, skills and pool membership.
- **What it does not allow:** salary, payslips, pay position, review content, the kind of leave, adjustments, the injury
  record, equality data, or any other HR record. It is **not a privileged caller** for the need-to-know rule (WPM-R114): holding
  it alone gives the same reads as any member of the organization, plus allocations. It writes nothing outside allocations.
- Every named read of another person's allocations by a resource manager is audited, as for anyone (R275).
- **Built so far (WPM-T601, 2026-11-20):** the vocabulary (`resource_manager` in the organization roles), the Keycloak claim
  mapping and test realm role, the pure access rules (`allocation::read_access`, `named_read_is_audited`, `may_write`), the grant
  rule's test, and the role's label in the 13 base locales. **Not built:** the routes that enforce it (WPM-T598).

### WPM-R261 — Scenarios with levers, lineage, and versions

*As a planner I can copy a plan into a scenario, change its levers, compare it with others, and keep every version.*

- **A scenario is a plan with a parent**: it starts as a copy of a base plan's demand lines, objectives, assumptions, and target
  structure, and records which version it came from.
- **Levers** are named assumptions, each shown in every answer:
  - **budget**: a change by percentage or amount from a month, or a freeze; a freeze stops new hires from that month;
  - **hiring freeze by unit, role, or position state** (positions are frozen, never abolished, by a scenario);
  - **shortage**: a loss of FTE in a pool, role, or unit from a month (sickness, industrial action, a supplier ending);
  - **recruitment lag**: calendar months from approval to start, per role profile or unit, defaulting to the observed time to
    fill where there is enough history and `unknown` otherwise; hires land after the lag, not on the target date;
  - **priority**: an order over objectives; when budget or supply is short, demand lines are met in that order and the unmet
    lines are listed.
- **Versions**: saving a scenario for comparison **publishes an immutable numbered version** with who and when; drafts change
  freely. A version is never edited; a change is a new version (WPM-D212).
- **Comparison**: two to four versions side by side, with the difference in each impact (R262) and in each lever.
- At most one plan is `active` per organization (R36); a scenario is never active until a person adopts it, which archives the
  previous active plan with its versions.

### WPM-R262 — Impact analysis per scenario version

*As a planner I can see what a scenario does to the workforce, cost, skills, training, and timelines.*

- One answer per version, assembled from the existing derivations, each with its own derivation and exclusions:
  - **workforce**: headcount and FTE gap by unit and month (R37, with the levers of R261);
  - **cost**: run-rate cost by cost centre and month, with on-cost and recruitment cost where recorded, against the budget, never
    mixing currencies (WPM-T51), withheld from callers who may not read salaries;
  - **skills**: the competency gap per role profile and unit (R37, R43);
  - **training**: the learning hours and cost to close reskill gaps, from learning economics (R227–R230), as estimates that say
    what they rest on;
  - **timelines**: for each programme, product, or service with demand, the start check's earliest delay (R60) under the
    scenario's supply. The **schedule consequence is PPM's** (WPM-D55): WPM sends the capacity answer through the what-if bridge
    (R209) and shows PPM's reply with its source and age when one is linked.
- A figure WPM cannot compute is `unknown` with the reason.

### WPM-R263 — As-is versus to-be

*As a director I can see the workforce and the skills we have against what the plan or scenario needs, per unit.*

- One view per unit and target date: headcount, FTE, and establishment as-is; the to-be from the chosen version; the gap; and the
  competency gap per required skill.
- Finishes R153: the **cost and skills difference** of roles added and removed, using the post grade (R255) as the unit cost
  where the plan has no other.

### WPM-R264 — Workforce cost to and from financial planning

*As finance I can take the workforce cost forecast into the financial plan and give planners the budget they plan against.*

- **Export**: cost by cost centre, financial code (R164), month, and scenario version, as CSV and JSON, for callers who may read
  salaries; every export audited.
- **Import**: budget envelopes per cost centre and month (amount and currency), dated and sourced, which the cost impact (R262)
  is measured against. An import that names an unknown or closed cost centre is refused line by line, never partly guessed.
- No ledger posting; WPM states figures and a finance system owns the books (as R164).

### WPM-R265 — Forecast accuracy is measured

*As a planner I can see how far past forecasts were from what happened, so the forecast earns trust or loses it honestly.*

- Every published version (R261) keeps its forecast. When snapshots (R35) exist for a forecast month, the view shows forecast
  against actual headcount and FTE per unit, and the mean absolute error, with the number of months compared.
- Too few months compared is `insufficient_history`, not an accuracy figure (WPM-D27).

### WPM-R266 — A planning overview

*As a leader I can see capacity, skills coverage, and commitment in one place, each saying how current it is.*

- A read-only page: headcount and FTE now, vacancy, pools `over`, `slack`, and `below_floor` in the next three months, critical
  roles with no bench, and the active plan's gap. Each tile names its as-of time and links to the view behind it.
- "Real time" means **derived on read** from current records, not a stream; snapshots are dated.

## Design decisions (proposed)

- **WPM-D207 A pay scale is dated, sourced data, published by two people.** Replaces the "in code" part of WPM-D38 and keeps the
  rest: values transcribed from a named source, never derived; a published version immutable; a correction a new version. Two
  people because a wrong value changes everyone's pay position at once.
- **WPM-D208 A grade is a reference, not pay.** A post's band and pay point are a pointer into a scale; WPM-D104 (no pay on a
  position) holds. Cost is always derived, and shown only to those who may read salaries.
- **WPM-D209 Combined analysis suppresses by complement.** A per-cell floor alone is not enough once dimensions combine, because
  totals let a hidden cell be recovered. Withhold the next smallest cell so no withheld cell is derivable from what is shown, and
  cap dimensions at three. Rejected: free slicing with a floor only.
- **WPM-D210 Internal first is evidence, not a block.** The check shows matching internal slack and asks for a reason; the
  person decides, as with the start check (WPM-D50).
- **WPM-D211 WPM tracks per-person allocation; it does not track per-person effort.** *Decided by the owner, 2026-11-20.*
  Amends WPM-D45 ("no per-person allocation percentages") and WPM-D59 ("no per-person time allocations"): a person's planned
  share of time per programme, product, or service is recorded (R275). What stays from them: **no timesheets**, so actual effort
  is a pool aggregate (R260); supply is still computed from FTE and leave; a position is still a seat (WPM-D104). The guards are
  what make the record acceptable: tracking switched on per scope so absence is never false availability, named reads limited
  to the person, their managers, HR and the source's owner, audited, no utilization score or ranking, and no use in reviews or pay.
  *Rewritten on 2026-11-20:* the first draft of this decision, the same day, kept WPM-D45 and declined per-person allocation.
- **WPM-D212 A compared scenario is an immutable version.** Comparison and forecast accuracy are only honest against figures that
  cannot change afterwards. Drafts stay editable; versions do not.
- **WPM-D213 Customer-led delivery is served by stable team capacity.** In the spirit of the Agile Manifesto, WPM plans capacity
  for stable teams and pools and lets the work be re-planned often (monthly claims, cheap scenarios). It does not measure
  individuals' velocity or story points. ITIL (Version 5) and the Manifesto are mapped by name; no licensed text is copied.

## Data and privacy summary

New tables (when built): `pay_scales`, `pay_scale_bands`, `pay_points`, `pay_scale_allowances` (reference data, no personal
data); `org_units`, `org_unit_cost_centres`, `service_delivery` (none); `service_floors` (none); `demand_actuals` (aggregates,
none); `plan_versions`, `plan_levers` (none); `budget_envelopes` (none); **`allocations` (personal data: who is planned to work on
what, R275)**, with `allocation_tracked` flags on pools and units. Columns added: a grade on `positions` and
`assignments` (a reference; the assignment's grade is a salary-class fact with the audience of WPM-D41). Allocations are the one new
kind of personal record; they do not cross the bridge, and no new personal field leaves WPM; analysis and exports respect the floor and salary visibility. The domain model, auth sensitivity map, audit catalogue,
and glossary change in the tasks that build each part.

## Not decided

See [plan.md](plan.md) section X, "Open questions".

## Sources

- The customer's feedback, pasted by the owner on 2026-11-20 (not stored in the repository).
- PeopleCert, *ITIL AI Governance (Version 5)*, <https://www.peoplecert.org/browse-certifications/it-governance-and-service-management/ITIL-1/itil-ai-governance-version-5-4234>,
  read as a search excerpt on 2026-11-20: AI governance across product management, service management, operations, and
  transformation. Partner FAQs say the 34 practices are available only to PeopleCert Plus members, so the practice names are
  **not yet checked** (WPM-T569).
- NHS Agenda for Change: [pay-scales.md](pay-scales.md) (the 2026/27 circular, read 2026-10-06).
