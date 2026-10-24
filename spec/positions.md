# Positions, and the organization now and as planned

A **position** is a budgeted seat that exists whether or not anyone holds it (WPM-R152, WPM-D104). The **current-versus-target
organization** view compares the live structure with the one a plan intends (WPM-R153). Proposed in [plan.md](plan.md)
section N; common in public sector and health, where the establishment (the approved number of posts) is a control in its own
right ([strategic-workforce-planning.md](strategic-workforce-planning.md), [engagements.md](engagements.md) for post funding).

## WPM-R152 — Positions

- **A position** has a cost centre, a budgeted **FTE**, a funding kind (the same kinds as post funding: `core`, `time_limited`,
  `external`, `programme`, with the programme as a `thing:` URN and an end date where the kind needs one), a state and, once held,
  a **link** to the worker. It carries **no amount of pay** (WPM-D104): the budget is FTE, and cost is derived from the pay scale
  where the caller may see salaries, as the plan cost view does.
- **States and moves.** `planned → approved → open → filled`, with `frozen` (not to be filled for now) and `abolished` (ended)
  reachable from the states a deployer's rules allow. `open` means a requisition exists for it; `filled` means a worker holds it. A
  state change is a person's decision, audited; a move the rules refuse is a `422` naming the state it is in (the same shape as
  the requisition and plan lifecycles).
- **Vacancy and over-establishment** are arithmetic over the positions, not records: **vacancy** is budgeted FTE minus the FTE
  held; **over-establishment** is FTE held beyond budget, shown beside the position that holds it, never hidden by netting against a
  vacancy elsewhere. A position with no holder FTE recorded is vacancy `unknown`, not the whole budget.
- **Links** to a plan demand line (a plan's target headcount becomes positions on approval, never silently), to the requisition that
  fills it, and to the post funding of the worker who holds it. Each link is one pid; nothing is copied.
- **Views.** An **establishment view per cost centre**: budgeted, held, vacant, over, and by state. A position's holder is HR-only
  (a manager sees the count and the state, as with plan lines); the establishment totals have no holder in them.
- **Not done.** No per-person time allocation across positions (WPM-D45, WPM-D59); no salary on a position; no automatic
  requisition on `open`.

**Built, pure core only (WPM-T278).** `rules::position`: the six states and fifteen moves (`planned → approved → open → filled`, `frozen`
and `abolished` as above; `abolished` is final; `filled` can go back to `open` or `approved` when the holder leaves). The
facts a move needs: `open` needs a linked requisition, `filled` needs a linked holder, a held position is not made vacant or abolished
until its holder is unlinked (so abolishing one settles what happens to the worker first, WPM-D104), and a frozen position may keep
its holder. A position is one seat: **budget 1 to 100 FTE percent**. Funding is the post-funding kinds with the same end-date rule,
and `programme` names a `thing:` URN. Per position, vacancy and over-establishment are `max(budget - held, 0)` and
`max(held - budget, 0)`, and unknown when the holder's FTE is not recorded. Per cost centre they are **added over positions** and never
netted. Only `approved`, `open`, `filled` and `frozen` positions make the establishment; a planned one is a proposal and an abolished one
is gone; the vacancy that sits in frozen positions is shown apart. For a caller who may not see who holds what, a centre with
fewer positions than the floor withholds held, vacant and over (the budget and the count name no one). Records, the API and the
screen are WPM-T279.

## WPM-R153 — Current versus target organization

- **A diff, not a record.** The pure core (WPM-T280) takes two trees, the live reporting structure (the cycle-safe manager forest,
  WPM-T2) and a plan's target structure, and returns what changed: **span of control** by manager (direct reports, before and after),
  **layers** (depth of the tree), **roles added, removed and moved** (by role profile), and the **cost and skills** difference of the
  roles added and removed, as the plan cost and capability views already count them.
- **Aggregate by role, not by person.** The diff names roles and departments. Where it must say how many people sit in a role it gives
  a count, with the floor; it never lists the people a restructure would move (that is a decision for a person, with a consultation
  where the law requires one: [worker-unions.md](worker-unions.md)).
- **A screen beside the org chart**, read-only, closing the open "Workforce transformation" item of Phase 9.

## Design decisions

- **WPM-D104 — A position is a seat, not a person.** Positions hold budgeted FTE and a funding kind; the holder is a link. Moving a
  person does not change the position, and abolishing a position does not delete a worker. No per-person time allocation (WPM-D45,
  WPM-D59). Rejected: a position per worker ("headcount as positions") because it would make the establishment a copy of the staff
  list and lose the vacancy.
