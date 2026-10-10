# Self-service, requests and workplace health requirements (WPM-R124–R130, proposed)

Requested 2026-10-11 as six capabilities. **Built so far (2026-10-11): the `/api/me` surface and its write
allow-list, contact details, emergency contacts by the worker, my time-off, self-service leave requests,
resignations, flexible working requests, equality monitoring and workplace health requirements** (WPM-T205–T214,
WPM-T216; API only, no screen). **All six requested capabilities exist at the API level; the screen (WPM-T215) is not built**; the tasks are in [tasks.md](tasks.md) Phase 21. Generic by design: any legal right a deployer must honour (for example a
statutory right to request flexible working, a notice period, an occupational-health duty) is a
**configuration they own**, never hard-coded, and the examples below are examples.

> ⚠️ **Two of these cut across earlier design decisions and are bounded on purpose.** Equality
> monitoring stores protected characteristics, and workplace immunization records are health data. Until
> now WPM held neither (its principle is "what must not be stored gets no column", WPM-D17, D24, D25, and
> `regulatory.md`: employment facts, not demographics). They are built only as the **bounded exceptions**
> WPM-D74 and WPM-D75, **off by default**, and a deployer switches each on only by recording the lawful
> basis they rely on. Both are special-category data under UK and EU data protection law (Article 9) and
> need the deployer's own impact assessment first. Nothing here is legal advice.

## The surface: `/api/me`

Everything a worker does about themself lives under `/api/me/...`, so the person is the caller's own
`sub` and never a path parameter someone could change. The blanket policy lets only HR, service and
administrator callers write; a self-service write is therefore an **explicit allow-list** of routes that
the guard lets through and whose handler checks that the record is the caller's own (WPM-D76). All other
writes are unchanged.

## WPM-R124 — Self-service updates

A worker updates, without HR: **home address**, **telephone numbers**, **personal e-mail** and their
**emergency contacts** (which exist today but are written by HR).

- Contact details are in their own table, not on the worker row that many endpoints return. A worker
  reads and changes their own; HR and payroll read them (need-to-know, `rules/access.rs`: self and
  privileged only); a manager does not.
- A change is audited **without the values**; the previous value is not kept (minimisation). A change of
  address tells payroll *that* it changed, not what it is.
- Exported with the worker and erased with them. Validation is by shape (length, a telephone number's
  characters), never by lookup.

## WPM-R125 — Equality and diversity monitoring (bounded, off by default)

Voluntary, self-declared protected characteristics (for example ethnic origin, religion or belief,
sexual orientation, disability status, sex, age band) so an employer can monitor the equality of its
workforce and the quality of that data (WPM-D74).

- **Off until a deployer enables it and records the lawful basis** (`WPM_EQUALITY_MONITORING_BASIS`). The
  categories and their allowed values are the deployer's configuration; every category always offers
  **prefer not to say**, and a worker can change or withdraw a declaration at any time.
- Held in its own table, readable and writable **by the worker only**. **No one else reads an individual's
  declaration**: not the manager, not HR, not payroll.
- The only output is an **aggregate** (per department and category), with small groups withheld below a
  floor the deployer sets (default 10), and a **completeness** figure (how many have declared) that
  names no value. That feeds the adverse-impact check of the equality impact assessment
  ([equality-impact/template.md](equality-impact/template.md), WPM-R100) as a grouping the deployer
  chooses.
- **Never an input to any score, ranking, selection or decision about a person.** A test fails if any
  registered score reads these columns (the guard of WPM-T169, extended).
- Exported with the worker, erased with them, not in the retention sweep, in the DPIA and the record of
  processing as special-category data.

## WPM-R126 — Flexible working requests

A worker asks for a different working arrangement; a person decides.

- A request names the kind (hours, times, days, place of work, job share, compressed hours, other), a
  proposed start, what the worker thinks the effect on the team is and how it might be managed. The
  manager decides, within a **decide-by date the software computes** from a configurable period (the
  default and the number of requests allowed in a year are the deployer's: a statutory regime may fix
  both), to **approve**, **approve with changes** (a counter-proposal the worker accepts or refuses),
  or **refuse** with a reason from a closed list the deployer configures plus a note. A refusal can be
  **appealed** once. Overdue requests are flagged to the manager and HR.
- An approved arrangement may carry a **trial period**. Approving a change of hours produces a
  **proposal** for HR to apply to the worker's `fte_percent`; the software **never changes a contract
  by itself** (WPM-D77). It also feeds the delivery-capacity view as a suggestion.
- The request, its decision and its reasons are the worker's and their manager's and HR's; the audit
  names the event, not the text.

## WPM-R127 — Resignations

A worker logs their intent to resign; a person accepts it.

- `POST /api/me/resignation` (built) records the date, the **proposed last day** and an optional reason from a
  closed list (never required; free text is not asked for). The earliest last day is computed from the
  notice rule the deployer configures. The worker can **withdraw** before it is accepted.
- The manager and HR are told *that* it was logged, not the reason. HR **accepts** it, sets the agreed last
  day and, in one transaction, opens the leaver process (the joiner and leaver movements that exist) and
  sets the engagement end. The software **never terminates anyone** (WPM-D77); a withdrawal after
  acceptance is a conversation, not a button.
- Reasons feed an aggregate only (by department, with the small-group floor), and are not in any score.

## WPM-R128 — Workplace health requirements (bounded, off by default)

Records of the **corporate requirements** a role or area sets, such as required immunizations (WPM-D75).

- A deployer defines requirements (a name and the roles or areas that need them) and a re-check period.
  For each worker the record is: the requirement, a **status** (`up_to_date`, `due`, `overdue`,
  `exempt_recorded`, `declined`), the date it was recorded and the next due date. **No clinical detail**:
  no diagnosis, no product or batch, no reason for an exemption (an exemption is *recorded as exempt* by
  occupational health, and the reason stays in their system). The record is written by an
  **occupational-health role**; the worker reads their own.
- A manager sees only **cleared** or **not cleared** for a requirement their area has, never the status
  detail, and only for their reports. HR sees the same. Occupational health sees the detail.
- Reminders go to the worker (a configurable number of calendar days ahead). Aggregates give compliance
  by area with the small-group floor.
- Off until enabled with a recorded lawful basis (`WPM_HEALTH_REQUIREMENTS_BASIS`). Exported with the
  worker and erased with them. A health requirement is never an input to any score.

## WPM-R129 — My time-off

A single read-only view, `GET /api/me/time-off`, built on the leave data that exists:

- per leave kind and year: the **allowance**, days **taken**, days **booked** (approved, in the future),
  days **requested** and awaiting a decision, and days **remaining**, all in calendar days or business
  days as the deployer's rule says (each figure states which);
- **history** by year: the absences taken, as kind, dates and length. **Sick leave shows dates and days
  only, never a reason** (WPM-D73); and it is the worker's own, so it does not go to the manager from here;
- the worker's open and past requests with their status, and a way to request or cancel (existing routes).

No derived "absence score" or trigger point is computed: that would be a score about a person and needs an
equality impact assessment first (WPM-R99).

## WPM-R130 — The `/api/me` surface and its guard

The allow-list of self-service write routes (WPM-D76), the rule that the handler resolves the person from
the verified token, a test that lists every write route a plain signed-in caller can reach and fails when a
new one appears, and a screen, **My details**, in the interface for R124–R129.

## Decisions

- **WPM-D74 Equality monitoring data is the one deliberate exception to "no demographics", and it is
  bounded.** Opt-in per worker and per deployment, self-declared, readable only by the worker,
  aggregate-only with a small-group floor, never an input to a person-level decision. It supersedes the
  reading of WPM-D17's principle that WPM holds no protected characteristic, for self-declared monitoring
  data only.
- **WPM-D75 A workplace health requirement records the status, never the reason.** The record answers
  "is this person cleared for this requirement?" and nothing about their health beyond it. It is the narrow
  exception to the rule that WPM holds no health data, and is off by default.
- **WPM-D76 Self-service writes are an explicit allow-list under `/api/me`.** The policy still gates every
  other write; a new self-service route is added to the list and to its test, or it does not exist.
- **WPM-D77 A request from a worker is decided by a person, and the software computes the dates.** Flexible
  working and resignation never change a contract, an end date or a pay figure by themselves; they produce
  a decision to take and, when it is taken, a proposal or a recorded change. (Compare WPM-D66.)

## What is not specified

The legal duties of a particular jurisdiction (which requests are statutory, the notice a contract needs,
which requirements an occupational-health regime imposes); retention periods for the new tables (the
deployer's schedule, with the defaults reasoned in [governance/retention-schedule.md](governance/retention-schedule.md));
and translation of the new screens (the catalogue rules of WPM-R94 apply).
