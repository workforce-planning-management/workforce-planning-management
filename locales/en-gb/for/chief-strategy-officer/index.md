# For the Chief Strategy Officer

**The question you are likely asking:** is our workforce shaped to serve the strategy, what will we need, and what
should we do about the gap?

## What you get

| You want to know | Where WPM answers | Read |
| --- | --- | --- |
| What a role requires | Role profiles with required skills and a minimum proficiency, so a gap is measured against a role and not a person | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| What we will need | Workforce plans as draft worlds, several side by side, with stated assumptions and demand by department, role and period | WPM-R36, same file |
| Where the gap is | Projected supply against planned demand: a headcount gap and a competency gap, with levers listed as suggestions and their evidence | WPM-R37 |
| Whether it serves the strategy | Objectives linked to demand: unresourced objectives, unaligned demand, critical roles with no bench | WPM-R38 |
| Whether we can deliver a programme | Skill pools, programme demand, partner commitments, the constraint pool, a start check that is a suggestion | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| How much we rely on non-permanent staff | The share of FTE that is contractor and fixed-term, each apart, and how much funding ends in each of the next twelve months | `GET /api/workforce-intelligence/contingent` (WPM-T109, part) |
| What we plan to do about contracts ending | Conversion plans: a person's decision to convert, extend or end, with the funding beside the contract end | [engagements.md](../../../../spec/engagements.md) |
| What else is planned | The roadmap and the open task list | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## How to read a forecast

- It is a stated rule applied to stated inputs, not a model, and every result names its assumptions.
- With too little headcount history it says `insufficient_history` rather than extrapolating.
- A plan holds hypothetical headcount only. It never reads or changes a person's record.
- Levers are suggestions. The planner chooses, and nothing is decided by the software.

## Limits

- The "convert contingent to permanent" lever in plans is specified (WPM-R63) and not yet in code.
- Contingent share has no trend yet: it needs snapshots split by basis (WPM-T139, not built).
- Critical chain and critical path scheduling belong to project-portfolio-management, not here.
- No user interface exists yet for funding or conversion plans.

See also the [Chief Executive Officer](../chief-executive-officer/index.md) and
[Chief Operating Officer](../chief-operating-officer/index.md) pages.

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Counts that change are in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software with
synthetic data. Nothing here is legal advice.
