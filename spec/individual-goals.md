# Individual goals tied to reviews

Status: **built.** The product owner confirmed WPM-R224 to WPM-R226 and WPM-D163 to WPM-D166 on 2026-11-05, and asked for the PADR link and the goals screens the same day. Tasks WPM-T428 to WPM-T433, WPM-T458 and WPM-T459 in [tasks.md](tasks.md) Phase 27. Plan: section U of
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

- **WPM-R232 The other person is told.** When a goal waits for someone's agreement, that person gets an in-app notification: the person, for a manager's proposal; their direct manager, for theirs. The same applies to a PADR: the person is told when one is opened with them, and each signer when the other has signed. The notification says what is waiting and carries pids and nothing else: no title, no text, no measure (WPM-D23). A person with no manager on record tells nobody, and that stays visible on the goal.

- **WPM-R233 From a gap to a goal.** A person looking at their own skill gaps can make a gap a proposed goal in one step: it names the skill, aims at the level the gap needs and ends when the training plan says the skill would be reached. It goes through the same proposal and agreement as any goal. A gap with an open goal for that skill says so and offers no second one. A skill the person has not declared gets no such offer, because an unknown needs assessing first (WPM-R44).
- **WPM-R234 Evidence beside the claim.** A goal may carry a target level for its skill. It then shows the person's declared level against that target: reached, short, or not declared, which is unknown and not short. A goal marked achieved while the declared level is short says both, and neither overrides the other, as a development plan shows declared and verified progress side by side. This compares one person with their own target; it is no score and is never added up across people (WPM-D163).

- **WPM-R235 A goal names its courses.** A goal that is proposed, agreed or in progress and builds a skill lists the catalogue courses for that skill, each with its hours, its fee or "no fee recorded", and the person's standing on it where the reader may see that. The list opens on request and enrols no one (WPM-D181). A manager can reach a report's skill gaps from their goals and make one a proposal, which the person must agree (WPM-R233, WPM-D177).

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

- **WPM-D177 Changing a proposal makes it yours to propose.** Either party may change a goal until it is agreed. A change by the party who did not propose it makes them the proposer, so the other must agree the new words. Otherwise a manager could rewrite a person's goal and agree it alone, and the person would be held to something they never saw. (Chosen in the build, 2026-11-05.)

- **WPM-D180 Evidence does not move a goal.** Reaching the level does not achieve a goal and falling short does not fail one, because the two people agreed the goal and only they close it. Declared level is the person's own statement, so the evidence is as good as that and no better. (Chosen in the build, 2026-11-05, not yet confirmed by the owner.)

- **WPM-D181 A suggestion enrols no one.** A goal shows courses and their cost so the two people can talk about them. Starting one is a separate step and is the owner's to decide, because a course can cost money. Today enrolment is a write that only HR, an administrator or a service may make under the reference policy, through the API, and no screen calls it. (Chosen in the build, 2026-11-05, not yet confirmed by the owner.)

## Questions for the product owner

**Open, 2026-11-05: starting a course from a goal.** May a person enrol themselves, or does it need a manager's approval? A recorded fee of zero is free; a course with no fee recorded is unknown and must not be treated as free (WPM-R227). A completed enrolment counts against the learning budget at the catalogue fee (WPM-T451), so an enrolment with no approval is spend with no approval.

**Taken as proposed, 2026-11-05.** The owner said yes to building this without answering the three questions one by one, so the build follows each proposal below. Any of them can still be changed.

1. **Team goals.** Should one goal be shared by several people (a team goal each member sees), or is every goal one person's, with the
   alignment reference as the only link? The proposal is the second, which keeps one owner and one audience per goal.
2. **Who sets a goal.** The proposal is that the person or their manager may propose and the other agrees (`proposed` to `agreed`), with HR
   able to see but not agree.
3. **Measures.** Numeric only (name, unit, start, target, current), or also a yes/no milestone? The proposal is numeric plus a yes/no milestone,
   with no free-form score.

## What is built and what is not

Built on 2026-11-05: `rules::individual_goals`, the tables `worker_goals`, `goal_measures`, `goal_check_ins` and `review_goal_outcomes`, the API, `/me/goals` (with a form for each person below the reader, the links below, and a count of what waits for them), the goals of a review on the worker page, the roll-up on `/development`, the notifications of WPM-R232, and a PADR's objectives ([staff-records-and-pay.md](staff-records-and-pay.md), WPM-R172).

The requirements left these open; the choices are recorded so a reader can disagree.

- **`worker_goals`, not `goals`.** A table called `goals` already holds weighted lines inside one review (WPM-R10). Those are left as they are, so two things are called goals for now, and whether the older ones should move is a question for later.
- **"Signed" means `shared`.** A review has no signature; its last status is `shared` (`draft`, `submitted`, `calibrated`, `shared`). The snapshot is taken in the transaction that shares it, and the reviewer's and reviewee's outcomes stop changing then.
- **Every write is under `/api/me`**, because the reference policy lets only HR write and neither party is HR. HR has no write route at all, so HR cannot agree a goal. A person with no manager on record cannot get their own goal agreed, and it stays `proposed`.
- **A read is the person, a manager in their chain or HR with a seat**, not payroll, which is narrower than the need-to-know guard's privileged callers. The reviewer named on a review may read that review's goals.
- **Measures are whole numbers**, with the unit chosen to suit, or a yes-or-no milestone. A goal is editable until it is agreed; after that it can be moved on, checked in or withdrawn. An agreed goal cannot be edited, so a change is a withdrawal and a new proposal.
- **A check-in says** `on_track`, `at_risk` or `off_track`, and a note, and may set one measure's current value. Only an agreed or in-progress goal takes one.
- **The roll-up's floor is five people**, counted as people and not goals, with the next smallest department withheld when exactly one is.

- **A PADR's objectives are the person's live goals whose period overlaps the PADR's**, leaving out withdrawn ones. The first signature freezes them on that version; a goal added later is not an objective of a signed version, and a correction takes the list afresh when it is signed.
- **The links are checked.** A skill must be in the catalogue and a plan item must be on one of this person's own plans. `aligns_to` is checked for shape and never resolved (WPM-D166). A goal shows the skill's name and the plan step's skill.
- **A manager's list holds every person below them**, with or without a goal, so that a first goal has somewhere to go.

Built later on 2026-11-05 (WPM-T462): `worker_goals.skill_target_level`, `rules::individual_goals::skill_evidence`, the `skill_evidence` a goal carries, "Make this a goal" on the person's own skill gaps on `/me`, and the evidence line on a goal. The form has no field for a target, so only a goal made from a gap or through the API has one, and an edit from the form keeps it. A manager viewing someone's gaps on the worker page gets no such action.

Not built: a form with more than one measure (the API takes several, and a change keeps the ones it does not show); closing a period does nothing automatically.
