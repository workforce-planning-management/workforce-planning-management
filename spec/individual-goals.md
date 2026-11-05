# Individual goals tied to reviews

Status: **built, except the PADR link.** The product owner confirmed WPM-R224 to WPM-R226 and WPM-D163 to WPM-D166 on 2026-11-05. Tasks WPM-T428 to WPM-T433 in [tasks.md](tasks.md) Phase 27; WPM-T431 is part done, because the PADR record does not exist yet. Plan: section U of
[plan.md](plan.md).

## Why

WPM has reviews, 360° appraisals and, planned, the personal development review (PADR, [WPM-R172](staff-records-and-pay.md)), which "records objectives
for the year ahead" as text inside one review. Nothing records a goal on its own, follows it through the year, or lets the end-of-year review
look back at it. There is no objectives-and-key-results (OKR) or key-performance-indicator (KPI) record anywhere in the spec; the only KPI
mentions are the CEO dashboard's tiles. This topic adds **individual goals**, owned by the person and agreed with their manager, and ties them to
the reviews they feed.

## Requirements

- **WPM-R224 A goal.** A worker's goal has a title, an optional description, a period (start and end dates), a status (`proposed`, `agreed`,
  `in_progress`, `achieved`, `partly_achieved`, `not_achieved`, `withdrawn`), and zero or more **measures** (a key result: a name, a unit, a
  start value, a target and a current value; every number optional, and an unmeasured goal is allowed). **Check-ins** are dated entries on a goal
  (a status, an optional current value, a short note). A goal may link to a development-plan item or a skill (WPM-R43) it builds, and may carry an
  optional **alignment reference** to a team or organization objective held elsewhere (WPM-D166).
- **WPM-R225 Tied to reviews.** A PADR's objectives for the year ahead are goals in that PADR's period (WPM-R172 is changed to say so). A
  performance review lists the goals whose period overlaps its own and carries, for each, an outcome the reviewer and the reviewee each record. **A
  signed review freezes a snapshot** of each goal it lists (its pid, title, measures and status at signing); later edits to the goal never change a
  signed review, and a correction is a new review version, as WPM-R172 already says (WPM-D165).
- **WPM-R226 Who sees it, and what it is never used for.** The person and their managers up the chain see a goal; HR sees their organization's.
  Goals are as confidential as review content: restricted by ABAC, and every read audited (WPM-D164). **No score, rank or attainment percentage is
  computed across people, and no pay, progression or employment decision is made from a goal** (WPM-D163). HR's roll-up is counts by status and
  department with the floor; a rate with nothing to divide by is `null`, and a goal with no measure is never counted as met or missed by a number.
  In the subject-access export; erased with the person.

## Decisions (confirmed 2026-11-05)

- **WPM-D163 A goal is a record of intent and evidence, not a score.** There is no attainment percentage, no ranking and no rating derived from
  goals (the scoring guard test applies). A person's measure shows its own start, current and target values and nothing computed against other
  people. `unknown` is not zero: a goal with no current value shows "—".
- **WPM-D164 Goals follow review confidentiality.** They are high sensitivity: masked unless the caller is the person, a manager in their chain or
  HR of the organization, and each read is audited with the goal's pid and no content.
- **WPM-D165 A signed review freezes a snapshot of its goals.** What was agreed and what was reported at signing stays as it was; the live goal may
  go on changing.
- **WPM-D166 Team and company OKRs are referenced, never held.** Cascading objectives live in the portfolio app or the deployer's own tool. A goal
  here may carry an optional `aligns_to` reference (an EntityRef URN) that WPM never resolves or fetches, so WPM works with no sibling (the
  independence rule). WPM holds no company-wide OKR tree.

## Questions for the product owner

**Taken as proposed, 2026-11-05.** The owner said yes to building this without answering the three questions one by one, so the build follows each proposal below. Any of them can still be changed.

1. **Team goals.** Should one goal be shared by several people (a team goal each member sees), or is every goal one person's, with the
   alignment reference as the only link? The proposal is the second, which keeps one owner and one audience per goal.
2. **Who sets a goal.** The proposal is that the person or their manager may propose and the other agrees (`proposed` to `agreed`), with HR
   able to see but not agree.
3. **Measures.** Numeric only (name, unit, start, target, current), or also a yes/no milestone? The proposal is numeric plus a yes/no milestone,
   with no free-form score.

## What is built and what is not

Built on 2026-11-05: `rules::individual_goals`, the tables `worker_goals`, `goal_measures`, `goal_check_ins` and `review_goal_outcomes`, the API, `/me/goals`, the goals of a review on the worker page, and the roll-up on `/development`.

The requirements left these open; the choices are recorded so a reader can disagree.

- **`worker_goals`, not `goals`.** A table called `goals` already holds weighted lines inside one review (WPM-R10). Those are left as they are, so two things are called goals for now, and whether the older ones should move is a question for later.
- **"Signed" means `shared`.** A review has no signature; its last status is `shared` (`draft`, `submitted`, `calibrated`, `shared`). The snapshot is taken in the transaction that shares it, and the reviewer's and reviewee's outcomes stop changing then.
- **Every write is under `/api/me`**, because the reference policy lets only HR write and neither party is HR. HR has no write route at all, so HR cannot agree a goal. A person with no manager on record cannot get their own goal agreed, and it stays `proposed`.
- **A read is the person, a manager in their chain or HR with a seat**, not payroll, which is narrower than the need-to-know guard's privileged callers. The reviewer named on a review may read that review's goals.
- **Measures are whole numbers**, with the unit chosen to suit, or a yes-or-no milestone. A goal is editable until it is agreed; after that it can be moved on, checked in or withdrawn. An agreed goal cannot be edited, so a change is a withdrawal and a new proposal.
- **A check-in says** `on_track`, `at_risk` or `off_track`, and a note, and may set one measure's current value. Only an agreed or in-progress goal takes one.
- **The roll-up's floor is five people**, counted as people and not goals, with the next smallest department withheld when exactly one is.

Not built: the PADR's objectives as goals (WPM-R172 has a pure core only, so there is no PADR record to tie to); the link from a goal to a skill or plan item is stored but no screen sets it; `aligns_to` is checked for shape only and no screen sets it; no notification tells the other party a goal is waiting; a manager's proposal has an API route and no form; closing a period does nothing automatically.
