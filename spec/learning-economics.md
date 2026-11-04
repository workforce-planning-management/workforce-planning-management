# Learning, upskilling and economic viability (proposed 2026-11-04)

> **Status: proposed, not built.** Requirements WPM-R227 to WPM-R230, decisions WPM-D172 to WPM-D174 (proposed, the owner's to confirm), tasks WPM-T446 to WPM-T453. The sister file in the project portfolio repository is `spec/learning-economics.md` there; the two share a vocabulary, not code.

## Where WPM stands

WPM is strong on **what people need to learn** and weak on **what learning costs and returns**.

| Holds today | Source |
|---|---|
| Skills gaps per person and per workforce, ranked, nobody named in the aggregate | [skills-and-training.md](skills-and-training.md) (WPM-R43) |
| Training time: courses, hours per level, a schedule at a weekly pace | WPM-R44 |
| Upskill and reskill plans with declared and verified progress | [talent-development.md](talent-development.md) |
| Mandatory learning, audiences, due-soon windows, CPD | WPM-T434 and the learning rules |
| A cost model for hiring: unit cost, on-cost, annual cost against a budget | `rules::cost` |
| Time to fill | the fill-time summary |

| Missing | Consequence |
|---|---|
| A course has hours but no fee | A training plan cannot say what it costs |
| No cost of the hours themselves | The cheapest-hours course may be the dearest in pay |
| No comparison of training with hiring | "Reskill or recruit?" is answered by instinct |
| No stated benefit, so no return | Nobody can say whether a learning plan is worth doing |
| No link to a project's business case | The cost of building capability a project needs is invisible to the portfolio |

## Requirements

- **WPM-R227 — A course can carry a fee.** An optional fee in minor units with its currency. **No fee recorded is unknown, never free.** A plan's direct cost is the sum of its recorded fees per currency, and the answer says how many courses have no fee.
- **WPM-R228 — The hours cost money too.** For a plan or a gap, the hours at the worker's loaded hourly cost (pay with the organization's on-cost, over the contracted hours). Shown **beside** the fees, never added to them, because one is cash out and the other is time. No pay recorded gives `unresolved`, never zero. Currencies are never mixed.
- **WPM-R229 — Train or hire.** For a seat's skill gap, compare an internal candidate's training cost (R227 and R228, plus a ramp-up the deployer sets) with the cost of an external hire (a recruitment cost the deployer sets, the vacancy days from the fill-time summary at the seat's daily cost, a ramp-up, and any pay difference). The answer is `train`, `hire`, `similar` or `unknown`, with both totals and every input shown. A side with a missing input makes the answer `unknown`, not the other side's win. It is labelled an estimate; WPM decides nothing.
- **WPM-R230 — Is a learning plan worth it?** The plan's owner may state an expected benefit per year with its source (for example the external hire it avoids, or a rate of work moved in house). WPM returns payback and net present value at the organization's discount rate, echoed. **No stated benefit is `unmeasured`, never `not viable`.** Proficiency gained is evidence of learning, never a benefit in money, so WPM does not infer one.

## Decisions (proposed)

- **WPM-D172.** Unknown is never zero: an absent fee, pay, recruitment cost or benefit leaves the figure unresolved and says which input is missing.
- **WPM-D173.** A benefit is a stated estimate with a named source. WPM never turns a proficiency change, a completion or a score into money.
- **WPM-D174.** NPV, payback and the verdict follow the same definitions in this project and the project portfolio one (`viability`): flows by period, a discount rate in basis points per period echoed in the answer, half-away-from-zero rounding at each step, a series capped at 600 periods, and `unmeasured` where either side is absent.

## Tasks

See [tasks.md](tasks.md): WPM-T446 (course fee) to WPM-T453 (screens).

## Not taken

- Valuing a skill from market pay or a model. Nothing here is a number WPM could defend.
- Attributing a business result to a course.
- Ranking people by return on their training. The aggregates name nobody, as in WPM-R43.
