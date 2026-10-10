# Delivery capacity: skill pools, programme demand, partners, start check (WPM-R56–R60)

What is **built** of the delivery-capacity proposal in [plan.md](plan.md) section A:
the pure arithmetic (WPM-T104, WPM-T105), the records (WPM-T106) and the view and start
check (WPM-T107). The rest of that proposal (the spec round WPM-T103, key people, funding
horizons, oversight levels, signals) is not built. No user interface exists yet.

## The model in one paragraph

Capacity is planned by **skill pool**, not by person (WPM-D45). A pool groups role profiles
and skills; a worker counts toward it through a role profile or a skill at a minimum
proficiency. Programmes claim FTE from a pool month by month. Partner organizations record the
aggregate FTE they can add. The service answers: how much can the pool supply, is it
over-committed, which pool is the constraint, and can this programme start?

## Supply

For each month, a pool's supply is the sum over the workers who count toward it of
`FTE × available days ÷ days in the month`, less the pool's **operations reservation** (the
share kept for running live services). A day is available when the worker is employed and not on
**approved** leave (a requested or rejected leave does not count; the kind of leave is never read).
A worker counts in every pool they qualify for, so pool supplies must not be added together. A
role profile matches a worker by their chosen framework role or by job title. A pool with no
members has a supply of 0, which is a known zero.

All FTE is in **hundredths** (`fte_centi`, 100 = one FTE). A month is its first day.

## Demand and partners

- A **programme demand** is a claim by a programme (a `thing:` URN owned by
  project-portfolio-management, WPM-D55) on a pool for a month. Its status is `proposed` (shown, not
  counted), `active` (counted) or `closed`. A plain write can set `proposed` or `closed`; a claim
  becomes `active` only through a **recorded start decision** (WPM-D50).
- A **partner commitment** is an organization's aggregate FTE for a pool and month, `requested`,
  `committed` or `confirmed`. It never names partner staff. Committed and confirmed capacity can
  cover a shortfall; requested cannot.

## Each pool-month comes to one of four results

| Result | Meaning |
| --- | --- |
| `fits` | Supply covers the active demand |
| `fits_with_partners` | Supply falls short and committed or confirmed partner capacity covers the rest |
| `over` | Short, and every partner has answered |
| `unknown` | Short, and at least one partner has made **no commitment** that month. A missing commitment is unknown, never zero (WPM-D46) |

The **constraint pool** is the pool with the largest total shortfall (in `over` months only) over
the horizon; a tie goes to the lowest id. Months that cannot be said do not count toward it.

## The start check is a suggestion

`POST /api/capacity/start-check` takes a programme's proposed claims (or what-if lines, which
store nothing) and says, per line, whether it fits, is short (and by how much), cannot be said
(a silent partner), or falls outside the horizon. It also gives the **earliest delay in months**
at which every line is known to fit (the whole programme moves together; a delay that pushes any
line past the horizon, or into a month that cannot be said, does not count), and the
**work-in-progress check**: how many programmes would be active on the constraint pool against
the organization's limit. It is a capacity answer only; scheduling belongs to
project-portfolio-management (WPM-D55).

`POST /api/capacity/start-decisions` records a person's decision, `proceed` or `defer`, with a
**required reason**. The check is re-run on the server. A decision to proceed is allowed when the
check fails: the person decides, and the evidence is kept with the decision. `proceed` makes the
programme's proposed claims active. The audit entry names the decision, not the reason text.

## Endpoints

| Endpoint | Purpose |
| --- | --- |
| `GET/POST /api/skill-pools`, `GET/PUT/DELETE /api/skill-pools/{pid}` | Pools and their reservation |
| `POST /api/skill-pools/{pid}/members`, `DELETE …/members/{member_pid}` | Role profile, or skill with minimum proficiency |
| `GET/PUT /api/programme-demands`, `DELETE /api/programme-demands/{pid}` | Claims; `PUT` upserts by programme, pool and month |
| `GET/PUT /api/partner-commitments`, `DELETE /api/partner-commitments/{pid}` | Partner capacity |
| `GET /api/capacity?organization=&from=&months=` | Pool × month view (default 12 months, at most 36) |
| `GET/PUT /api/capacity/settings` | The work-in-progress limit |
| `POST /api/capacity/start-check` | The check; stores nothing |
| `GET/POST /api/capacity/start-decisions` | The decisions |

## Data

Six tables, none holding personal data: `skill_pools`, `skill_pool_members`,
`programme_demands`, `partner_commitments`, `capacity_settings`, `start_decisions`. Pools,
members, demand and commitments are soft-deleted. Programme and partner are `EntityRef` URNs; WPM
keeps no copy of either. A decision records who decided (an actor identifier, as other records do).

## Limits and what was not decided

- **Programme URN type.** The entity-reference crate has no programme type, so a programme is a
  `thing:` URN. If project-portfolio-management gets its own type, the check moves to it.
- **Authorization.** Writes are HR writes under the built-in policy; there is no separate planner
  role in the policy, so "planner" in the plan is not enforced distinctly. The start check is a
  `POST`, so it needs write access though it changes nothing.
- **Proposed claims of other programmes** are not counted when checking one programme.
- **Rounding.** Supply is rounded half up to a hundredth of an FTE per pool-month.
- **Not built:** a user interface, the funding horizon, the key-person list, oversight levels and
  capacity signals (rest of plan.md section A to D), and the spec round (WPM-T103), which will
  fold this note into `delivery-capacity-and-oversight.md`.
