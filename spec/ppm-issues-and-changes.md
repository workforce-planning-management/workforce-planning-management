# Issues register and change requests, for project-portfolio-management (WPM-R95–R97)

Status: **proposed 2026-10-10**. Nothing here is built. These are project-control capabilities,
so they belong to the sibling project-portfolio-management (PPM) service under
[WPM-D55](design.md) and [WPM-D65](plan.md); this file is the requirement text to carry over
there, plus the small part WPM itself supplies (WPM-T157). PPM numbers the issues register
**FR-14**; WPM refers to it by that name and does not renumber it.

> ⚠️ **Demo software.** Synthetic data only. See [regulatory.md](regulatory.md).

## WPM-R95 — Issues register (PPM FR-14)

*As a programme manager I can log, own, grade and close the issues that threaten a programme,
so that nothing a director needs to decide hides in a status report.*

- **An issue** has: a title and description; the programme (and optionally a stage or a task) it
  affects; a **raiser**, an **owner** (one accountable person) and a **due date**; a **severity**
  (`low`, `medium`, `high`, `critical`) with a stated reason; a **status** (`open`, `in_progress`,
  `blocked`, `resolved`, `closed`, `cancelled`); a **resolution** note when closed; and optional
  links to related **risks**, **change requests** and other issues.
- **Lifecycle** rules in a pure core: only the owner or a delegate moves an issue to `resolved`;
  `closed` needs a resolution note and someone other than the owner to confirm (an owner never
  closes their own critical issue); a `critical` issue cannot sit `open` without an owner and a due
  date; a due date in the past marks it **overdue** (derived, never stored).
- **Escalation**: an issue that is `critical`, or `high` and overdue, is reported to the programme
  sponsor within 48 hours of reaching that state (the rule in
  [production-readiness plan](plan.md) section C: late discovery is a measure), and the register
  records when it first reached the state and when it was acknowledged.
- **Views**: by programme, owner, severity, age and status; an aging chart (open issues by age
  band); a count that names no one. Free-text fields never go into an audit entry or notification.
- **Audit**: every create, move, reassign and close is audited with who and when, not with the text.

## WPM-R96 — Change requests

*As a sponsor I can see what a proposed change would do to scope, schedule, cost and people
before I decide, and afterwards what the baseline became.*

- **A change request** has: a title and rationale; the programme it changes; a **raiser**; the
  **impact** on scope, schedule (calendar days moved), cost (minor units and currency) and risk,
  each with a basis; the **people impact** (WPM-R97); a **status** (`draft`, `submitted`,
  `assessed`, `approved`, `rejected`, `withdrawn`, `implemented`); a **decision** with a reason, who
  decided and when.
- **Decision rules** in a pure core: the decider is never the raiser; a change above a configured
  threshold (cost, schedule or both) needs the next level of approver; an `approved` request
  records the **baseline before and after** (dates and budget) so the register shows what moved;
  only an approved request can be `implemented`; a rejected or withdrawn one is kept, not deleted.
- **Link to issues**: a change request can resolve one or more issues; closing the request offers
  those issues for closing, and a person confirms (WPM-D50: a suggestion, then a decision).
- **Audit and privacy**: as for issues; the rationale and decision text are never copied into an
  audit entry or a notification.

## WPM-R97 — People impact of an issue or a change request

*As a director I can see what a proposed change or an open issue does to the people who deliver
the programme, before it is approved.*

- WPM supplies, for a programme and a set of proposed demand changes (FTE by pool by month), a
  **what-if capacity answer**: which pool-months become over-committed, which stop being, the
  constraint pool before and after, and the earliest start that fits (the start check of
  proposed WPM-R60). It is computed on request and **stores nothing** (WPM-T158).
- The same answer lists the **contingent workforce** effect where a change lengthens or shortens
  an engagement: engagements that would end before the new end date and would need an extension
  (proposed WPM-R81, R86), as counts per pool with no names.
- PPM shows this answer on an issue or a change request by calling WPM with the programme's
  `EntityRef` URN and the proposed deltas. WPM never stores the issue or the request, and PPM never
  stores a person.
- If the capacity model (proposed WPM-R56–R60) is not yet built, the answer is `unknown`, not
  "no impact".

## Decision

- **WPM-D65 Issues and change requests are PPM's; WPM answers "what does this do to our people?"**
  They are project-control records, like the critical chain and earned value (WPM-D55). WPM keeps
  no issue, no change request, no baseline and no schedule; it holds capacity and answers questions
  about it. This also keeps a record about a project from becoming a record about a person.
  (WPM's existing *change initiatives*, for automation and AI effects on roles and skills, are a
  different thing and stay in WPM.)

## Acceptance (for the PPM side, to copy there)

1. An issue cannot be closed by its owner when critical; an overdue critical issue is escalated
   once, within 48 hours, and the escalation is acknowledged.
2. A change request cannot be decided by its raiser; an above-threshold one needs the next approver;
   an approved one records the baseline before and after.
3. The what-if call returns `unknown` for a programme with no demand, never zero.
4. No audit entry, notification or aggregate contains an issue's or a request's text, and no
   aggregate names a person.
