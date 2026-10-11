# For the Chief Operating Officer

**The question you are likely asking:** do we have the right people, in the right place, at the right time, and
what breaks if someone is away?

## What you get

| You want to know | Where WPM answers | Read |
| --- | --- | --- |
| Can a team cover absence | Backups per person, the on-call rota with swaps and reminders, and a cover view that says plainly when nobody is available | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Time, attendance and leave | Time entries, leave requests with an approval path, accruals, and shifts | [workforce-management.md](../../../../spec/workforce-management.md) |
| Are working-time limits being respected | Advisory working-time guardrails; they warn and do not block | WPM-R27, same file |
| Capacity by skill, month by month | Skill pools, supply and programme demand, partner commitments, the constraint pool, and a start check that is a suggestion, with recorded start decisions | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| What we will need and where the gap is | Workforce plans, forecast and gap analysis by department and by role | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Which contracts end soon | An end date by engagement basis, extensions with history, reminders before an end, a list of engagements missing an end date, and conversion plans | [engagements.md](../../../../spec/engagements.md) |
| Where contingent staff are a large share | The share of FTE that is contractor and that is fixed-term, each apart, and the funding that ends month by month | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Joiners and leavers | Dated checklists, and a last-day handover | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Skills gaps and training time | Gap analysis with the evidence a recommendation rests on | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## How to read the capacity view

Capacity is planned by skill pool, not by person. A pool-month comes to one of four results, and an undeclared
skill is *unknown*, not a shortfall. The start check is a suggestion, and a person records the decision to start.
Critical chain and critical path scheduling are not here: they belong to project-portfolio-management.

## Limits

- Contingent share has no trend yet: it needs headcount snapshots split by basis (WPM-T139, not built).
- A pool for the contingent view is the department until plans define pools.
- Shift scheduling and payroll tax calculations are not a substitute for your rostering or payroll systems.
- Several views load an organisation's records into memory; no load figure has been measured
  ([readiness-assessment.md](../../../../spec/readiness-assessment.md), gap 8).

See also [Chief People Officer](../chief-people-officer/index.md) and [Chief Executive Officer](../chief-executive-officer/index.md).

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Figures that change
(tests, migrations, tasks done) are not copied here; read them in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software
with synthetic data: this page says what it does and does not do, and is not a claim that it is fit for your
organisation. Nothing here is legal advice.
