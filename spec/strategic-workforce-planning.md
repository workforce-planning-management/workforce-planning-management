# Strategic workforce planning

The analytical process of forecasting future talent and skill needs,
identifying the gaps between the current workforce and future goals, and
aligning headcount and competencies with strategy. This topic closes the
"future state" and "path" pillars that [roadmap.md](roadmap.md)'s research
backlog found missing: WPM's current-state pillar (workers, org chart,
skills, succession, [workforce intelligence](talent-development.md)) is
strong; the plan, the forecast, and the gap between them are not yet
represented.

## The model in one picture

```text
 strategy ──► objectives ──► demand (headcount × competencies, per period)
                                   │
 current state ──► supply projection ──┤      gap = demand − projected supply
 (employed now, trend attrition,       │        ├─ headcount gap   (per department, period)
  succession, pipelines)               │        └─ competency gap  (per role: required vs held)
                                       ▼
                         levers: hire · reskill/upskill · promote · redeploy
```

Three questions, three surfaces:

| Question | Surface | Requirement |
|---|---|---|
| What will we need? | Workforce plans (scenarios) with demand lines; supply forecast | WPM-R36, WPM-R37 |
| Where is the gap? | Headcount gap + competency gap per demand line | WPM-R37 |
| Is it aligned with strategy? | Objectives linked to demand lines; alignment view | WPM-R38 |

Two foundations the rest depends on: **role profiles with required
skills** (WPM-R34) — a "target" is per worker today, not per role — and a
**headcount history** (WPM-R35), without which no forecast can be honest.

## Requirements

### WPM-R34 — Role profiles and required skills

*As a workforce planner I can say what a role requires, so competency gaps
can be measured against a role rather than a person.*

- A role profile is keyed by job title (the same key the benchmarks use)
  with a set of required skills, each with a minimum proficiency 1–5 (the
  existing scale) and an optional importance (`critical` / `important` /
  `useful`).
- Skills reference the existing catalogue; no second skills model.
- Reads are plain; writes need the HR `write` action. Soft-delete; audited.

### WPM-R35 — Headcount snapshots

*As a planner I can see headcount history, because a forecast needs a
trend and history cannot be reconstructed later.*

- A periodic job records, per organization × department × date: employed
  headcount, FTE (hundredths), starters and leavers since the previous
  snapshot — all via the one `is_employed_on` definition (WPM-T44).
- Idempotent per (organization, department, date). Append-only; no edits.
- Aggregates only: no worker identity, salary, or sensitive attribute.

### WPM-R36 — Workforce plans (scenarios) and demand lines

*As a planner I can model a future state — and several alternatives —
without touching live records.*

- A plan: name, organization, horizon (start/end), strategic rationale,
  explicit **assumptions** (attrition rate, optional growth), status
  `draft → active → archived`.
- Demand lines: department × (optional) role profile × period ×
  target headcount. A plan holds hypothetical headcount only; it never
  creates, changes, or reads individual worker rows (WPM-D26).
- Several plans can coexist for comparison (scenario modeling); at most
  one is `active` per organization.

### WPM-R37 — Forecast and gap analysis

*As a planner I can see projected supply against planned demand, and
where the shortfall is — in people and in skills.*

- **Supply projection** per department per period, from the plan's
  assumptions: opening employed headcount, expected leavers (the
  assumption attrition rate, defaulting to the trailing observed rate
  from snapshots), known moves (succession readiness, pipeline "ready"
  members, development plans landing in the period).
- **Headcount gap** = demand − projected supply, per department per
  period; negative = surplus.
- **Competency gap** per demand line with a role profile: for each
  required skill, workers currently proficient at the minimum
  (employed only), against the headcount the line needs — graded with the
  same depth bands as capability analysis (WPM-T41).
- **Levers** are listed as *suggestions with their evidence*, never
  decisions: hire externally, reskill (workers with the adjacent skills
  and open development plans), promote (succession bench), redeploy
  (surplus departments). The planner chooses.
- Every payload names its assumptions and derivation; a forecast with
  fewer than the minimum snapshots reports `insufficient_history`, not an
  extrapolation (WPM-D27).

### WPM-R38 — Strategic alignment

*As a CHRO I can see whether headcount and competencies serve the
strategy, and what does not.*

- A plan carries **objectives** (title, optional owner). Demand lines
  link to one or more objectives.
- Alignment view: share of planned headcount tied to an objective;
  objectives with no demand (unresourced); demand lines with no
  objective (unaligned); critical roles (succession, WPM-T22) with no
  bench against the active plan.
- Terms-carrying ratios, null-not-zero, as for every intelligence view.

## Design decisions

### WPM-D26 — A plan is a draft world, not a layer on the live one

Every table today is live state. A plan holds only **aggregate
hypothetical headcount** and never references worker rows, so a scenario
cannot leak into the live org chart, payroll, or audit trail, and a
discarded scenario leaves no residue. The live org chart's derivation is
reused for *comparison* (current vs target shape), not for storage.

### WPM-D27 — Forecasting is transparent rules, not a model

The service has no ML dependency and this feature does not add one. A
projection is arithmetic over **stated assumptions** the planner can read
and change; the trailing-attrition default is shown as the assumption it
is. With too little history the answer is `insufficient_history`. A
forecast that cannot show its working is not offered.

### WPM-D28 — Gaps are aggregate and levers are suggestions

Workforce planning decides about *positions and capabilities*, not about
people. Gap views are per department / role / skill; the one place
individuals appear is as **evidence behind a lever** (who is proficient,
who is on the bench) under the audit and masking rules those lists
already have. The tool never ranks or selects individuals for an outcome
— that remains a human decision, which also keeps it clear of automated
decision-making obligations ([regulatory.md](regulatory.md)).

## Delivery order

Tracked in [tasks.md](tasks.md) (WPM-T45–T50): headcount snapshot job →
role profiles → plans + demand lines → forecast and gap analysis (pure
core first) → alignment view → front-end. Snapshots first because history
accrues only with time.
