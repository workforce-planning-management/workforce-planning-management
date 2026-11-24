# Leadership continuity, cover and reorganization

Proposed 2026-11-24, from [the lessons of an independent review](lessons/20261124/index.md) (the owner asked on 2026-11-24 that
they be recorded in the plan, the tasks and the spec). Requirements **WPM-R288–R298**, decisions **WPM-D231–D235**, tasks
**WPM-T648–T659** ([tasks.md](tasks.md), [plan.md](plan.md) section Z). **Nothing here is built.** The numbers start above the
mobile-app branch's (WPM-R287, WPM-D230, WPM-T645) so the two never share an id.

The review described a loop: leaders changed often, posts stayed empty or were covered for a while, a long reorganization was
undecided, authority moved upward, people stopped speaking, and less got done. WPM can show the **structural** half of that loop
from facts about positions and dates, and never judges anyone. This file holds what WPM would add, in the order the lessons
advise. The source is anonymous on purpose: **no requirement, decision, task, test, demo record or commit that comes from it
names an organization, a person, a place, a programme or a document, or quotes or gives a figure from it.**

| Id | What it adds | Builds on | Tier |
| --- | --- | --- | --- |
| WPM-R288 | State history of a position; vacancy age | WPM-R152 | First |
| WPM-R289 | A freeze has a reason, a decider and a review date | WPM-R152, R288 | First |
| WPM-R290 | Holder history of a position; continuity in critical posts | WPM-R152, R161, R197 | First |
| WPM-R291 | Temporary cover with a whole length: acting, secondment, interim | WPM-R79, R203–R205, WPM-D131 | First |
| WPM-R292 | Requisition approval steps; funded and waiting | WPM-R1, WPM-R152 | First |
| WPM-R293 | A reorganization has a clock, and an end that releases what it holds | WPM-R104, R105, R108 | First |
| WPM-R294 | Pulse themes get an answer | WPM-R28, WPM-D20 | Second |
| WPM-R295 | Sustained, not only done | WPM-R66, R67 | Second |
| WPM-R296 | Authority stated beside accountability | WPM-R201, R202 | Second |
| WPM-R297 | A minimum offer for every manager | WPM-R189, R29 | Second |
| WPM-R298 | Where to raise a concern: a directory and counts | None | Third, with care |

Succession that is ready to use (the lessons' idea 8) adds no requirement: it is the argument for building
[WPM-R197–R200](succession-deputies-and-acting-up.md) sooner, with two additions recorded on WPM-T338 (below). The ideas the
lessons place last (protected time for professional leads, early leaving, change load, plans seen by those who deliver them,
decisions explained, recognition) are on the [roadmap](roadmap.md) until a deployer asks.

## Requirements

### WPM-R288 — State history of a position; vacancy age

- **Append-only state history** of a position (WPM-R152): from, to, the date, the actor. Today a status move keeps from and to in
  the audit trail only, so no date says since when a post is vacant.
- **Vacancy age** is worked out from it for every open vacancy, not only for the filled ones (time to fill counts only the
  requisitions that were filled, which leaves out the worst cases).
- A post vacant before the history began shows `vacant, start unknown`, **never zero**.
- A report by unit, band and critical role gives vacancies by age, with the number frozen and the number with no review date
  (R289). **Abolished posts are shown beside it**, because a vacancy age can be cut by abolishing the post.
- **Accept when:** ages count open vacancies; an unknown start is shown as unknown; abolished posts appear beside the ages; no
  holder is named; a cut under the floor is withheld.

### WPM-R289 — A freeze has a reason, a decider and a review date

- A move to `frozen` asks for a **reason** from a closed list (`reorganization`, `funding`, `demand_review`, `other`), the role
  that decided, and a **review date**. A freeze with no review date is refused.
- A review date that passes is a **finding**. The reason `reorganization` links to the change initiative (R293).
- **Accept when:** the three are required; a passed review date is listed; the move is audited.

### WPM-R290 — Holder history of a position; continuity in critical posts (confirmed by the owner 2026-11-24)

- **Append-only holder history** (start, end, kind of cover) kept under the staff-record retention rule. Setting a position's
  holder overwrites it today, and the audit row keeps only that a link was made, so the number of holders over time is lost.
- For **critical posts** (WPM-R197, WPM-D47): holders in the last 36 months, months empty, and months covered temporarily, by unit
  and band.
- **Guardrails.** A senior group is small: any cut under the floor is withheld, with its complement (`suppression.rs`). A view of
  one post is for HR and the post's line only. **No person's record shows a count of posts they left.** Turnover and sickness of a
  senior group are not reported when the floor would be breached.
- **Accept when:** a history row is added by every holder change and never changed; a small cut is withheld and says so; no
  endpoint returns "posts left" for a person.

### WPM-R291 — Temporary cover with a whole length

- Build the **acting assignments** as specified (WPM-R203–R205) and widen their kind to `acting`, `secondment` and `interim`:
  all three are dated cover of a post somebody else holds or nobody does. WPM-D131 (acting cover does not fill a vacancy) applies
  to all three.
- Report **time covered in total** for a post, across renewals and across the people covering it, beside the current end date.
- Each arrangement carries the **plan that ends it**: `return`, `recruit`, `abolish` or `not decided`.
- **Guardrail.** The figure describes a post. It never shows a person as a poor performer for having been kept in cover.
- **Accept when:** three renewals of three months read as nine months for the post; the people covering are not ranked.

### WPM-R292 — Requisition approval steps; funded and waiting

- Between `draft` and `open` a requisition may pass through `submitted`, `approved` and `returned`, each with a date and a role,
  a `decide_by`, and the number of returns with reasons.
- One measure: **funded, unfilled, and waiting for approval**, as posts, FTE and days, **by the step now holding them**.
- **The step is a setting, off by default.** A deployment that wants no approval step keeps today's behaviour. The record adds a
  step only where a deployer had one.
- **Guardrail.** Days are for a step or a role, never for a person (no list of slow approvers). State calendar or business days
  each time.
- **Accept when:** with the setting off, nothing changes; with it on, the measure counts by step.

### WPM-R293 — A reorganization has a clock, and an end

- On a change initiative of kind `restructure`: the date it was first announced; the **promised decision date**, with an
  append-only history of moves and reasons; the posts affected; and the positions frozen because of it (R289).
- Derived: days since people were told, how many times the date moved, posts waiting. A finding covers an initiative with no
  date, or one whose date moved more than once.
- **Ending it** (complete or cancel) asks for an answer to **each linked hold** (`released`, or `kept, because`) and for a plain
  message to those affected that says it has *ended* and not paused.
- **Before the decision** the record asks for at least two options, **including keeping the current structure**, with what each
  does to local leadership, to functions that cross units, and to the cost of disruption (numbers are the scenario work,
  WPM-R261–R263).
- **Guardrail.** No `at risk` mark on a worker. It is sensitive and easily wrong; posts carry what is needed. Where the law
  requires individual notices, the consultation register (WPM-R105) counts them.
- **Accept when:** cancelling an initiative with a linked freeze is refused until each hold has an answer.

### WPM-R294 — Pulse themes get an answer

- After a pulse closes: the **themes**, written by a person; for each an **answer** (`we will`, `we will not, because`, `we do not
  know yet, and will say by`), an owner role, a date and a link to what was done; an announcement when an answer changes.
- Answered, unanswered and overdue themes are shown. A later pulse may repeat a question, so the same score is compared over
  time, with the floor of five applied at each point.
- **Opening a new pulse shows the last one's unanswered themes** to whoever opens it. A prompt, not a block.
- **Guardrail.** No free text from respondents is stored; themes name nobody. A trend must not let a small group be found by
  comparing two dates or two cuts.

### WPM-R295 — Sustained, not only done

- A change initiative, a pulse answer or a development programme can name **one workforce measure**, a starting value and later
  dates to look. Show `held`, `not held` or `not measured` beside `completed`. A completed initiative with no measure says so.
  **No success rate is computed.** The rule is the one the oversight criteria already use (WPM-R66, R67): several periods in a row,
  unknown is not met.

### WPM-R296 — Authority stated beside accountability

- On a position or role: what it **may decide**, from a short list (`recruit within establishment`, `approve leave`, `approve
  spend to a limit`, `change rosters`), with a limit and the date it was confirmed. Report posts with people to manage and **no**
  stated authority, and decisions WPM records that were taken above the stated level. Deputies (WPM-R201, R202) use the same list
  as their scopes: one vocabulary.
- **Guardrail.** A statement here **grants nothing**. What a caller may do is decided by the access rules, and the two are not
  confused in the code or the screens.

### WPM-R297 — A minimum offer for every manager

- A deployer-set list of what a manager is offered and expected to have (learning, an appraisal, a 360 within a period, a
  mentor), reusing mandatory learning and role profiles. The audience "has direct reports" is specified and not built (WPM-R189,
  WPM-T331). Report coverage by unit as current, overdue and unknown, and new managers with nothing arranged within a set number
  of calendar days. Offer the 360 (WPM-R29) to the most senior roles too, keeping its floor of three.
- **Guardrail.** It measures what the organization provided, not how good a manager is.

### WPM-R298 — Where to raise a concern: a directory and counts

- WPM **holds no concerns** (WPM-D232). It holds a **directory of routes** (what each is for, whether it is confidential or
  anonymous, who receives it, when to expect a reply), in every locale and reachable without signing in if the deployer chooses;
  a **pulse question** on whether people would raise a concern and whether one led to anything; and **counts entered by the route's
  owner** per quarter (received, answered in time, led to a change), with the floor, and `unknown` where nothing was entered.
- **Not built, ever:** detecting retaliation from pay, rota or appraisal data; linking a concern to a worker; a psychological
  safety score for a team or a manager.

## Design decisions (proposed; each is the owner's)

- **WPM-D231 No score for trust, culture, morale or leadership**, for anyone, in any form. Tracking leaders' visits is not built
  (record the follow-up promised to a team instead, R294). Conduct and behaviour stay a yes or no (WPM-D120); no free-text manager
  notes (WPM-D124); no promotion-fairness inference beyond the voluntary, floored counts of WPM-R200 and R205; **no ranking of
  units** by vacancy, cover or approval time. Figures are for the unit and for those who can remove the cause.
- **WPM-D232 WPM holds no concerns.** WPM is HR's system, and many concerns are about managers and HR; a record kept where the
  subject's colleagues administer access endangers the person who raised it. Doing it half well is worse than leaving it out (R298).
- **WPM-D233 Structure is recorded as history, never overwritten** (holder history confirmed by the owner on 2026-11-24, for critical posts). Position states (R288) and holders (R290) are append-only;
  each month without them loses history that cannot be rebuilt, which is why they come first.
- **WPM-D234 An unknown start is unknown.** A vacancy, a cover or a reorganization whose start was not recorded says so; it is
  never zero (the stance of RCA-IN-6 applied to dates).
- **WPM-D235 A measure can be gamed, so its opposite sits beside it.** Vacancy age beside abolished posts; approval time beside
  returns; cover time beside the plan that ends it. Where a figure is about few people, it is withheld and says that it is
  withheld.

## Open questions for the owner

Each has the advised answer in [the lessons](lessons/20261124/index.md#questions-for-the-owner); none is decided.

1. ~~May WPM keep who held a position and when, for how long, and for which roles?~~ **Decided by the owner on 2026-11-24: yes**, as
   advised: for critical posts, under the staff-record retention rule. WPM-R290 and WPM-D233 are confirmed on that scope; widening
   it to every post would need a new decision.
2. Do `secondment` and `interim` join the acting assignment while it is unbuilt? *Advised: yes.*
3. Is an approval step on requisitions wanted by default, or as a setting that is off? *Advised: a setting, off.*
4. Is the directory-and-counts approach to speaking up enough? *Advised: yes; it is the most WPM should do.*

## Sequencing

1. **State and holder history** (R288, R290): first, because nothing can be recovered later.
2. Vacancy age, hold reasons and review dates (R288, R289); then requisition approval dates (R292).
3. Acting, secondment and interim together, on the specified acting-up work (R291); then the reorganization clock (R293).
4. Answered feedback and the sustained rule (R294, R295).
5. Authority, succession and the manager offer (R296, WPM-T338, R297); then R298.

## Also checked, and recorded

- **A read to confirm (WPM-T659).** `GET /api/workforce-intelligence/overview` returns each manager's `display_name` with their
  number of direct reports. Read on 2026-11-24: the handler takes no caller and applies no `mask`, and the reference policy's last
  rule lets any signed-in caller read with the `mask` obligation, so **any signed-in caller can read it**. A manager's name and
  team size are the kind of fact a staff directory holds, but this is a decision to make, not a default: either say in the
  specification that the span measure is for every signed-in reader, or narrow the read.
- **A correction to the lessons.** They cite WPM-D211 for the refusal of scores, rankings and league tables of people. WPM-D211 is
  the per-person allocation decision; the refusal is in
  [workforce-planning-and-scenarios.md](workforce-planning-and-scenarios.md) ("No score"), in WPM-D163 and in WPM-D28.

## When built, also update

[`domain-model.md`](domain-model.md) (the history tables), the sensitivity map in [`auth.md`](auth.md) (holder history, review
dates, authority statements, concern routes), [`audit.md`](audit.md), [`regulatory.md`](regulatory.md) (retention of holder
history), [`glossary.md`](glossary.md) (vacancy age, cover total, hold, review date), [`testing.md`](testing.md).
