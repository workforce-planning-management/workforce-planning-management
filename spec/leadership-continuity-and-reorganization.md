# Leadership continuity, cover and reorganization

Proposed 2026-11-24, from [the lessons of an independent review](lessons/20261124/index.md) (the owner asked on 2026-11-24 that
they be recorded in the plan, the tasks and the spec). Requirements **WPM-R288–R298**, decisions **WPM-D231–D237**, tasks
**WPM-T648–T659** ([tasks.md](tasks.md), [plan.md](plan.md) section Z). **Built so far (2026-11-26): the position state history, vacancy age and freeze reasons (WPM-T648, part) , the holder history with one post's continuity (WPM-T649, part), and the critical marker and grade on a position with the vacancy age by band and for critical posts (WPM-T660); the rest is not built.** The numbers start above the
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
| WPM-R291 | Temporary cover with a whole length: three distinct categories, `acting_up`, `secondment`, `interim` | WPM-R79, R203–R205, WPM-D131 | First |
| WPM-R292 | Requisition status history: drafted to accepted, with the time in each stage | WPM-R1, WPM-R152 | First |
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

### WPM-R291 — Temporary cover with a whole length (confirmed by the owner 2026-11-24)

- Build the **acting assignments** as specified (WPM-R203–R205) with **three distinct categories**, `acting_up`, `secondment` and
  `interim` (the owner's decision, 2026-11-24). They share one record because all three are dated cover of a post somebody else
  holds or nobody does, but they are **never merged into one**: each has its own definition in the glossary, and a report shows
  each category separately beside the total.
  - **`acting_up`**: a person takes on the duties of a higher post for a while, and keeps their own substantive post (WPM-R203–R205).
  - **`secondment`**: a person is lent to another unit or organization for a fixed period, and their substantive post stays theirs.
  - **`interim`**: a stopgap holds a post while the permanent holder is being found, with no substantive claim to it. WPM-D131 (acting cover does not fill a vacancy) applies
  to all three.
- Report **time covered in total** for a post, across renewals and across the people covering it, beside the current end date.
- Each arrangement carries the **plan that ends it**: `return`, `recruit`, `abolish` or `not decided`.
- **Guardrail.** The figure describes a post. It never shows a person as a poor performer for having been kept in cover.
- **Accept when:** three renewals of three months read as nine months for the post; the people covering are not ranked.

### WPM-R292 — Requisition status history: drafted to accepted (confirmed by the owner 2026-11-24)

The owner asked that WPM **track the status changes of a requisition**: drafted, proposed, funded, advertising, interviewing,
evaluating, extending offers, accepted. This amends the lifecycle of WPM-R1 (`draft → open → interviewing → offer → filled |
cancelled`), which has no stage before `open` and none between `interviewing` and `offer`.

- **The stages**, in order, with the stored token and the words a person sees (the words are content, in every locale):

  | Stage | Token | Is it new? |
  | --- | --- | --- |
  | Drafted | `draft` | Today's |
  | Proposed (put forward for approval) | `proposed` | New |
  | Funded (approved, with money behind it) | `funded` | New |
  | Advertising | `open` | Today's `open`, shown as "advertising" |
  | Interviewing | `interviewing` | Today's |
  | Evaluating (the panel weighs the candidates) | `evaluating` | New |
  | Extending offers | `offer` | Today's `offer` |
  | Accepted (an offer is accepted, before the person starts) | `accepted` | New |
  | Filled (hired applications equal the headcount) | `filled` | Today's |

  `cancelled` is reachable from every stage before `filled`. A requisition may go back one stage (an offer declined goes back to
  `interviewing` or `evaluating`; a proposal sent back goes to `draft`), and each return keeps its reason from a closed list.
  **The tokens `draft`, `open`, `interviewing`, `offer` and `filled` keep their meaning**, so stored data and the API do not
  change; only the new tokens are added, and the words shown are labels.
- **A history of every change**, append-only: from, to, the date, the role that made it, and a reason on a return or a
  cancellation. From it, **the time in each stage** and the time from `draft` to `filled`, in calendar or business days (say
  which, each time).
- **The measures.** (a) **Funded, unfilled, and waiting**: posts, FTE and days in `funded` and `open` by the stage now holding
  them; (b) **proposed and waiting for a decision**, with a `decide_by` date, in posts, FTE and days; (c) returns by stage, with
  reasons. A requisition that began before the history began shows `stage start unknown`, **never zero** (WPM-D234).
- **Optional stages.** `proposed`, `funded` and `evaluating` are stages a deployer turns on. With them off (the default for an
  existing deployment), `draft → open` and `interviewing → offer` stay legal as today, and nothing changes for anyone. With them
  on, a requisition cannot reach `open` without passing `proposed` and `funded`. *Decided by the owner on 2026-11-24: a setting.* It is off by
  default for an existing deployment, so that WPM adds a stage only where a deployer already has one.
- **`funded` and the post.** A requisition for a post (WPM-R152) does not become `funded` unless the post has funding recorded
  (post funding, WPM-R152); one record of the money, not two. A requisition with no post records who confirmed the funding.
- **Guardrail.** Days are for a stage or a role, never for a person: no list of slow approvers, and no ranking of units (WPM-D231).
  A measure that can be gamed has its opposite beside it (WPM-D235): time in `funded` beside the number cancelled.
- **Accept when:** every change adds a history row and none is edited; an illegal jump is refused naming the current stage
  (as WPM-R1 does); with the optional stages off, every existing route and screen behaves as before; time in each stage adds up to
  the total; an unknown start is shown as unknown.

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

### WPM-R298 — Where to raise a concern: a directory and counts (confirmed by the owner 2026-11-24)

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
- **WPM-D236 An erasure cuts the person from a tenure and keeps its dates** (2026-11-26, WPM-T649). The holder history is about the
  post as much as the person: deleting an erased holder's tenures would turn a post that had three holders into one that had two and
  a gap, and the continuity figures would be wrong for everyone after. So erasure sets the worker of each tenure to nothing and ends
  an open one that day; the dates stay and name no one. The table is otherwise append-only: a trigger allows only closing an open
  tenure once and cutting a worker, and refuses any other change and every delete. Proposed with the build, like the decisions above:
  the owner may choose deletion instead.
- **WPM-D237 A post is critical because HR says so** (the owner decided on 2026-11-26 that positions carry a critical-role marker and a
  band). `critical` is a statement HR records on the position, audited when it changes; it is never derived from who holds the
  post, their pay or their skills, and it is the one list of critical posts that succession on positions (WPM-R197) reads, not a
  second one. The band is the post's grade (WPM-R255): a reference to a published pay scale, never an amount, and a post with no
  grade recorded is `unknown` in a report by band, never placed in a band.

## Open questions for the owner

Each has the advised answer in [the lessons](lessons/20261124/index.md#questions-for-the-owner); none is decided.

1. ~~May WPM keep who held a position and when, for how long, and for which roles?~~ **Decided by the owner on 2026-11-24: yes**, as
   advised: for critical posts, under the staff-record retention rule. WPM-R290 and WPM-D233 are confirmed on that scope; widening
   it to every post would need a new decision.
2. ~~Do `secondment` and `interim` join the acting assignment while it is unbuilt?~~ **Decided by the owner on 2026-11-24: yes**, as
   three distinct categories (`acting_up`, `secondment`, `interim`), tracked: WPM-R291 is confirmed, and the acting-up tasks
   WPM-T338–T344 build the three together (WPM-T651).
3. ~~Track a requisition's status changes, and are the new stages on for everyone?~~ **Decided by the owner on 2026-11-24:** track
   the status changes (drafted, proposed, funded, advertising, interviewing, evaluating, extending offers, accepted), and the three
   new stages (`proposed`, `funded`, `evaluating`) are **a setting**, off for an existing deployment.
4. ~~Is the directory-and-counts approach to speaking up enough?~~ **Decided by the owner on 2026-11-24: yes, for now.** WPM-R298
   and WPM-D232 are confirmed on that scope; holding cases stays out.

## Sequencing

1. **State and holder history** (R288, R290): first, because nothing can be recovered later.
2. Vacancy age, hold reasons and review dates (R288, R289); then requisition status history (R292).
3. Acting up, secondment and interim together, as three categories, on the specified acting-up work (R291); then the reorganization clock (R293).
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
