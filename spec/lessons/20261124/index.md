# Lessons from an independent review of a large organization (WPM, proposed 2026-11-24)

> **Status: proposed, not built.** This file is an assessment and advice. Nothing in it changes how WPM behaves today. On
> 2026-11-24 the owner asked that it be recorded: the ideas became proposed requirements **WPM-R288–R298**, decisions
> **WPM-D231–D235** and tasks **WPM-T648–T659** in
> [leadership-continuity-and-reorganization.md](../../leadership-continuity-and-reorganization.md), [plan.md](../../plan.md)
> section Z and [tasks.md](../../tasks.md). Each is still the owner's to confirm, and two of the four questions below are open (the first two were answered yes on 2026-11-24).

## Where the ideas come from, and what is left out

The source is a published independent review of how a large organization sets direction, takes decisions, leads its people,
and follows through. **The source is anonymous here on purpose.** This file names no organization, person, place, sector,
programme, or document. It uses no quotation and no figure from the source, and it describes patterns in general terms. Keep
it that way in any requirement, decision, task, test, demo data, or commit message that comes from it.

**Evidence base.** The whole review was read. What WPM has was checked on 2026-11-24 against `spec/`, `AGENTS/`, the service
code, the domain crate, and the migrations, by reading. No test was run and no service was started. "None" below means a
search found nothing, not that its absence is proved. Statements about "specified, not built" rely on the specification's own
status lines and on a search of the code.

## Advice, in one paragraph

Most of the review concerns people and leadership, so WPM has more to gain than its siblings and more to get wrong. The
pattern was a loop: leaders changed often, posts stayed empty or were covered temporarily while a long reorganization was
undecided, authority moved upward, people stopped speaking, and less got done. WPM can show the **structural** half of that
loop from facts about positions and dates, with no judgement of anyone: how long posts have been empty and why (1), how long
cover has lasted (2), how long approval takes (3), and how long a reorganization has left people waiting (4). **Build those
four first.** Then build what answers staff (5), and finish the succession and acting-up work already specified (2 and 8).
Keep away from measuring trust, courage, or behaviour in named people or small teams.

## Summary

| # | Idea | Pattern it answers | What WPM has | Priority | Size |
| --- | --- | --- | --- | --- | --- |
| 1 | Vacancy age and hold reasons | Posts empty for a long time, some frozen | Position states, vacancy FTE; no dates | First | M |
| 2 | Temporary cover with a whole length | Short renewals that add up to years | Extension counts; acting up specified (WPM-R203 to R205) | First | M |
| 3 | Funded and waiting for approval | Funded posts stuck in approval | Requisitions go `draft` to `open`, with no approval step | First | M |
| 4 | Reorganization clock and release | Years of uncertainty | Change initiatives, consultation windows | First | M |
| 5 | Answered feedback | Listened to, never answered | Anonymous single-question pulse | Second | M |
| 6 | Continuity in key posts | Leaders change, priorities restart | Tenure and turnover overall; holder overwritten | Second | M |
| 7 | Authority stated beside accountability | Accountable without authority | Deputies specified (WPM-R201, R202) | Second | M |
| 8 | Succession that is ready to use | No pipeline for critical roles | Plans, bench, gaps; positions specified (WPM-R197 to R200) | Second | in plan |
| 9 | A minimum offer for every manager | Managers without preparation | Appraisal, 360, skills; no minimum | Second | M |
| 10 | Sustained, not only done | Action closed, measure unchanged | Consecutive-period rule for oversight | Second | S to M |
| 11 | Protected time for professional leads | Lead roles with no time | Groups; no protected time | Third | M |
| 12 | Early leaving and why | New arrivals leave disappointed | Resignation reasons with a floor | Third | S |
| 13 | Change load on a department | Change on top of change | Initiatives, role impacts | Third | S |
| 14 | Plans that were seen by those who deliver | Top-down targets | Plans go `draft` to `active` in one step | Third | S |
| 15 | Decisions explained | News with no reasons or bad news | Announcements with counted receipts | Third | S |
| 16 | Where to raise a concern | Routes unclear, outcomes unseen | None | Third, with care | S |
| 17 | Recognition that is open to all | Good work unnoticed | None | Later | S |

Sizes are estimates: S is days, M is one to two weeks, and L is more, with screens and locales counted.

## Build first

### 1. Vacancy age, and why a post is held

**Pattern.** Senior posts stayed empty for a long time. Some were held deliberately while a new structure was discussed, with
no date on which the hold would be looked at again.

**WPM has.** Positions with `planned`, `approved`, `open`, `filled`, `frozen`, and `abolished` (WPM-R152), and frozen vacancy
apart from other vacancy. `POST /api/positions/{pid}/status` keeps from and to in the audit trail only. There is no date since
which a post is vacant, and no reason for a freeze. Time to fill counts only the requisitions that were filled, which leaves
out the worst cases.

**Add.** An append-only position state history (from, to, date, actor). From it, the age of each vacancy. A `frozen` move asks
for a reason from a closed list (`reorganization`, `funding`, `demand_review`, `other`), who decided, and a **review date**; a
review date that passes is a finding. A report by unit, band, and critical role shows vacancies by age, with the number frozen
and the number with no review date.

**Accept when:** ages count open vacancies and not only filled ones; a post vacant before the history began shows
`vacant, start unknown` and not zero; a freeze with no review date is refused; and no holder is named.

### 2. The whole length of temporary cover

**Pattern.** Interim, seconded, and acting arrangements ran on through short renewals. Each renewal looked small. The total
did not.

**WPM has.** Dated extensions with `extension_count` for fixed-term, contractor, and intern workers (WPM-R79, R80), and
conversion plans. Acting up is specified, with extension history and a configured maximum, and is not built (WPM-R203 to R205,
[succession, deputies, and acting up](../../succession-deputies-and-acting-up.md)). Interim appointments and secondment are not
in the specification at all.

**Add.** Build the acting assignments as specified, and widen the kind to `acting`, `secondment`, and `interim` while they are
unbuilt, since all three are dated cover of a post that somebody else holds or nobody does. Report **time covered in total** for
the post, across renewals and across the people covering it, beside the current end date. Ask each arrangement for the plan
that ends it: `return`, `recruit`, `abolish`, or `not decided`. WPM-D131 (acting cover does not fill a vacancy) already gives
the right count, and it should apply to all three kinds.

**Guardrail.** The figure describes a post. It never shows a person as a poor performer for having been kept in cover.

### 3. Funded, and waiting for approval

**Pattern.** Posts with money behind them stayed empty while approvals went through several layers, and it was not clear who
could say yes.

**WPM has.** A requisition goes from `draft` to `open` with nothing between. Post funding is recorded. Approval chains are a
roadmap line for pay.

**Add.** `submitted`, `approved`, and `returned` between `draft` and `open`, each with a date and a role; a `decide_by`; and
the number of returns, with reasons. Then one measure: **funded, unfilled, and waiting for approval**, as posts, FTE, and days,
by the step now holding them. A deployment that wants no approval step keeps today's behaviour.

**Guardrail.** Days are for a step or role. WPM-D163, WPM-D28 and the "No score" rule of the planning spec refuse scores, rankings and league tables of people, and a list of slow approvers
would be one. State calendar or business days each time.

### 4. A reorganization has a clock, and an end

**Pattern.** A proposed restructuring took years, its dates moved, people did not know whether their roles were safe, and
posts stayed empty because of it. When the plan is stopped, nothing releases what it was holding.

**WPM has.** Change initiatives with kind `restructure`, role impacts, aggregate readiness, and `cancelled`
(`crates/workforce-planning-management-domain/src/change.rs`); a consultation register with windows (WPM-R104, R105); and
advisory decision gates (WPM-R108). Nothing dates the uncertainty, and cancelling releases nothing.

**Add.** On an initiative: the date it was first announced; the promised decision date with an append-only history of changes
and reasons; the number of posts affected; and the positions frozen because of it (linked from 1). Derived: days since people
were told, how many times the date moved, and posts waiting. **Ending it, by completing or cancelling, asks for an answer to
each linked hold** (`released`, `kept, because`), and for a plain message to the people affected that says it has ended and
not paused. A finding covers an initiative with no date, or with a date moved more than once.

Before the decision, the record asks for at least two options, **including keeping the current structure**, with what each
does to local leadership, to functions that cross units, and to the cost of disruption. Scenario comparison (WPM-R261 to R263)
is the place for the numbers.

**Guardrail.** An `at risk` mark on a worker is not advised. It is sensitive, it is easily wrong, and posts carry what is
needed. Where the law requires individual notices, the consultation register counts them.

## Then

### 5. Feedback that gets an answer

**Pattern.** People were asked often and rarely told what was done. Asking again then lowered trust further.

**WPM has.** An anonymous one-question pulse with no author column and a floor of five (WPM-R28, D20). It has no trend over
years, theme, action, or reply.

**Add.** After a pulse closes: the themes, written by a person; for each, an answer (`we will`, `we will not, because`,
`we do not know yet, and will say by`); an owner role, a date, and a link to what was done; and an announcement when an answer
changes. Show answered, unanswered, and overdue themes. Let a later pulse repeat the question, so the same score can be
compared over time, with the floor applied at each point. **A new pulse shows the unanswered themes of the last one to whoever
is about to open it.** It is a prompt and not a block.

**Guardrail.** No free text from respondents is stored, and themes name nobody. The trend keeps the k=5 floor and must not
allow a small group to be found by comparing two dates or two cuts.

### 6. Continuity in key posts

**Pattern.** Repeated changes of leader became a cause of the trouble and not only a symptom.

**WPM has.** Turnover for the organization, tenure buckets, and dated assignments per worker. Setting a position's holder
overwrites it, and the audit row keeps only `linked: true` or `false`, so the number of holders over time is lost.

**Add.** An append-only holder history (start, end, kind of cover), kept under the staff-record retention rule. Report for
**critical posts**: holders in the last 36 months, months empty, and months covered temporarily, by unit and band.

**Guardrail.** A senior group is small. Any cut under the floor is withheld, with its complement (`suppression.rs`). A view
of one post is for HR and the post's line only, and nobody's record shows a count of posts they left. Turnover and sickness
of a senior group are **not** reported when the floor would be breached, and in most organizations it would be.

### 7. Authority stated beside accountability

**Pattern.** Managers answered for results and could not decide on people or money. Delegation existed on paper.

**Add.** On a position or role: what it may decide, from a short list (`recruit within establishment`, `approve leave`,
`approve spend to a limit`, `change rosters`), with a limit and the date it was confirmed. Report posts with people to manage
and **no** stated authority, and decisions WPM records that were taken above the stated level. Deputies (WPM-R201, R202) use
the same list as their scopes, so build one vocabulary.

**Guardrail.** A statement here grants nothing. What a caller may do is decided by the access rules, and the two must not be
confused in code or in the UI.

### 8. Succession that is ready to use

WPM-R197 to R200 already describe succession on positions, with evidence beside a person's judgement. The review is an argument
for building them sooner. Two additions: **coverage of posts now under temporary cover** (a vacant critical post with no ready
successor is the first line to read), and a count, with a floor, of reasons internal people give for **not** wanting a role,
from a closed list they choose from and may skip. Readiness stays a person's judgement (WPM-D128).

### 9. A minimum offer for every manager

**Pattern.** Preparation for management varied by place, and there was no shared minimum.

**Add.** A deployer-set list of what a manager is offered and expected to have (learning, an appraisal, a 360 within a period, a
mentor), reusing mandatory learning and role profiles. It needs the audience "has direct reports", which is specified and not
built (WPM-R189, T331). Report coverage by unit, as current, overdue, and unknown. Report new managers with nothing arranged
within a set number of calendar days. Offer 360 feedback (WPM-R29) to the most senior roles too, keeping its floor of three.

**Guardrail.** This measures what the organization provided, and not how good a manager is.

### 10. Sustained, not only done

WPM's oversight criteria already need several periods in a row and treat unknown as not met (WPM-R66, R67). Use the same rule
elsewhere: a change initiative, a pulse answer, or a development programme can name one workforce measure, a starting value,
and later dates to look. Show `held`, `not held`, and `not measured` beside `completed`. A completed initiative with no measure
says so. No success rate is computed.

## Later

11. **Protected time for professional leads.** A lead role with no time is a title. Record planned hours for the role on the
    assignment and the hours released, as two facts, and report roles with none planned. Networks across units already exist
    as groups (`domain/src/groups.rs`); let a group name its lead roles and the decisions it contributes to.
12. **Early leaving.** Leavers within a set time of joining, by unit, with the floor, beside the resignation reasons
    (WPM-R127). Not joined to a person's record beyond what is kept now.
13. **Change load.** Active initiatives that reach one department in one month. A number and a list, with no threshold
    claiming to be evidence.
14. **Plans seen by those who deliver them.** Before a plan becomes `active`, record which units were asked, when, and what
    changed. An activation without it is a finding. Show workforce, money, and service demand for a plan together as WPM-R258
    and R264 arrive.
15. **Decisions explained.** A template for an announcement: what was decided, why, what it changes, what stays the same, what
    is not yet known, and when that will be said. Good news does not replace bad news. Receipts stay as counts (WPM-D35).
16. **Where to raise a concern.** See the next section.
17. **Recognition.** A thank-you from any worker to a team, public by choice, with no points, counts, or ranking.

## Speaking up: what WPM should and should not do

The review gives this more weight than any other cultural theme, so it deserves a plain answer. WPM has no grievance or
speaking-up module, and **should not hold concerns**. WPM is HR's system, and many concerns are about managers and HR. A
record kept where the subject's colleagues administer access is a risk to the person who raised it. Doing this half well is
worse than leaving it out.

Safe and useful:

- a **directory of routes**: what each is for, whether it is confidential or anonymous, who receives it, and when to expect a
  reply, in every locale, reachable without signing in if the deployer chooses;
- a **pulse question** on whether people would raise a concern and whether one led to anything, with the floor as it is;
- **counts entered by the route's owner** per quarter (received, answered in time, led to a change), with the floor, and with
  `unknown` where nothing was entered. WPM holds counts and no cases.

Not to build: detection of retaliation from pay, rota, or appraisal data; linking a concern to a worker; or a psychological
safety score for a team or manager.

## Not advised

- **A score for trust, culture, morale, or leadership**, in any form, for anyone.
- **Tracking leaders' visits.** It turns presence into a number to be met. If it is wanted, record the follow-up promised to a
  team (idea 5), and not the visit.
- **Conduct or behaviour records.** WPM-D120 keeps a formal process to yes or no. That stays.
- **Free-text manager notes about people** (WPM-D124).
- **Promotion fairness by inference.** The slate and acting counts already specified (WPM-R200, R205) use voluntary monitoring
  with a floor, and are enough to start.
- **Ranking of units** by vacancy, cover, or approval time. The figures are for the unit and those who can remove the cause.

## Worth checking in what exists

- `overview` in `controllers/intelligence.rs` returns each manager's name with their number of direct reports. Confirm that
  its readers are the intended ones, and state this in the specification beside the span measure.
- The next-free-id note in [spec-driven delivery](../../../AGENTS/spec-driven-delivery.md) gave WPM-T608 while task entries ran to
  WPM-T647. **Fixed on 2026-11-24**: it now gives WPM-R299, WPM-D236 and WPM-T660.
- **Checked on 2026-11-24 and recorded as WPM-T659:** `overview` takes no caller and applies no `mask`, and the reference
  policy's last rule lets any signed-in caller read with that obligation, so any signed-in caller can read each manager's name
  with their number of direct reports.
- **A wrong cite, corrected.** Idea 3 cites WPM-D211 for the refusal of scores, rankings and league tables. WPM-D211 is the
  per-person allocation decision; the refusal is in [workforce-planning-and-scenarios.md](../../workforce-planning-and-scenarios.md)
  ("No score"), WPM-D163 and WPM-D28.

## Sequencing

1. **State and holder history** (1 and 6). Each month without it loses history that cannot be rebuilt.
2. Vacancy age, hold reasons, and review dates (1), then requisition approval dates (3).
3. Acting, secondment, and interim together, on the specified acting-up work (2), then the reorganization clock (4).
4. Answered feedback (5) and the sustained rule (10), which share the "done is not better" idea.
5. Authority, succession, and the manager offer (7, 8, and 9).
6. The rest when a deployer asks for it.

## Questions for the owner

1. May WPM keep who held a position and when, for how long, and for which roles? Idea 6 depends on the answer. **Answered yes on
   2026-11-24**, for critical posts, under the staff-record retention rule.
2. Should `secondment` and `interim` join the acting assignment while it is unbuilt? The advice is yes. **Answered yes on
   2026-11-24**, and tracked as WPM-T651.
3. Is an approval step on requisitions wanted as a default, or as a setting that is off?
4. Is the directory and counts approach to speaking up enough? The advice is that it is the most WPM should do.

## Risks

- **Measures become targets.** A vacancy age can be cut by abolishing the post. Show abolished posts beside it.
- **More approvals.** Idea 3 records a step. It must not add one where a deployer had none.
- **Small numbers.** Leadership data is about few people. When in doubt, withhold, and say that it is withheld.

## Limits of this advice

One review, of one organization, was read. It relied on testimony and documents and says so. Its recommendations were new
when it was read, so nothing shows that they work. No worker, manager, or union was asked. Every size is an estimate.
