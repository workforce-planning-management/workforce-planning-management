# Learning, upskilling and economic viability

> **Status: built, except the bridge to the project portfolio.** Requirements WPM-R227 to WPM-R230 and decisions WPM-D172 to WPM-D174 were confirmed by the product owner on 2026-11-05. Tasks WPM-T446 to WPM-T451 and WPM-T453 are done; WPM-T452 waits for a contract with the project portfolio service. The sister file in the project portfolio repository is `spec/learning-economics.md` there; the two share a vocabulary, not code. Every figure is an estimate on synthetic data; none has been compared with a real payroll or a finance system.

## Where WPM stood on 2026-11-04

WPM was strong on **what people need to learn** and weak on **what learning costs and returns**. The second table is what this work set out to close.

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

## Decisions (confirmed 2026-11-05)

- **WPM-D172.** Unknown is never zero: an absent fee, pay, recruitment cost or benefit leaves the figure unresolved and says which input is missing.
- **WPM-D173.** A benefit is a stated estimate with a named source. WPM never turns a proficiency change, a completion or a score into money.
- **WPM-D174.** NPV, payback and the verdict follow the same definitions in this project and the project portfolio one (`viability`): flows by period, a discount rate in basis points per period echoed in the answer, half-away-from-zero rounding at each step, a series capped at 600 periods, and `unmeasured` where either side is absent.

## What the build chose (WPM-T446 to WPM-T453)

The requirements left these open; the choices are recorded here so a reader can disagree with them.

- **Pay is masked, so the money follows the mask.** The reference policy gives HR a masked read and payroll an unmasked one. An hourly cost would give a masked salary straight back, so a masked reader of a plan gets the fees and hours and `pay_withheld`, and a masked reader of train or hire gets the inputs that do not rest on the candidate's pay, no totals, and `unknown`. A deployer who wants HR to use train or hire must give that reader an unmasked read. An unmasked read of someone else's figures records a `salary_read`.
- **"The plan's owner" is the plan's worker.** A plan has no other owner. HR in the worker's organization can act too; a line manager cannot, because the cost of hours rests on pay. The worker writes through `/api/me`, because the reference policy lets only HR write.
- **The hourly rate.** `workers.salary_minor` is a full-time-equivalent figure (payroll scales it by FTE), so the rate is salary plus `learning.on_cost_bp` over `learning.full_time_annual_hours`, at any FTE. The total is worked from the salary, not from the rounded rate.
- **Hours on a step that is not a course** are the skill's per-level figure, marked as estimates. A step of method `course` with no catalogue row counts as a course with no fee. A step that is not a course needs no fee. Abandoned steps are left out.
- **Viability is worked twice:** `on_fees`, and `with_hours`, so the fees and the hours are never merged into one figure a reader cannot separate. A course with no fee leaves the investment unknown, so neither is worked. Year 0 holds the investment and `year` 1 is the first year after it. Unlike the project portfolio's, there is no longest accepted payback, so its `too_slow` verdict cannot arise and is left out. The arithmetic is pinned by the same worked examples.
- **Train or hire adds cash and time** because each side needs one total, against WPM-R228's rule for a plan. Every part is shown beside the total.
  - The seat's skills are its demand line's role profile, and its cost is its title's benchmark, or the average of the department named by its cost centre for a cohort of at least five. A seat records no pay and no place of work, so parameters resolve at the candidate's.
  - The vacancy is the organization's median days to fill, over all filled requisitions, at a 365th of the annual cost. The pay difference is one year of the seat's cost less the candidate's, assuming the candidate keeps their present pay.
  - Within 10 percent of the larger total is `similar`, echoed. A candidate with an undeclared skill needs assessing first, so training is unknown.
  - There are two ramp-ups, `learning.ramp_up_days` and `recruitment.ramp_up_days`, and `recruitment.currency`, because a cost with no currency cannot be compared.
- **Parameters are by place of work, with no default,** as every parameter is: `learning.on_cost_bp`, `learning.full_time_annual_hours`, `learning.discount_rate_bp`, `learning.ramp_up_days`, `recruitment.cost_minor`, `recruitment.currency`, `recruitment.ramp_up_days`.
- **Spend** is a completed enrollment in the calendar year, at its course's *current* catalogue fee, in the person's *current* department. A fee that has changed since, or a person who has moved, is therefore not as it was; no historic fee is kept. A course reference whose catalogue rows disagree counts as no fee.

## Tasks

See [tasks.md](tasks.md): WPM-T446 (course fee) to WPM-T453 (screens).

## Not taken

- Valuing a skill from market pay or a model. Nothing here is a number WPM could defend.
- Attributing a business result to a course.
- Ranking people by return on their training. The aggregates name nobody, as in WPM-R43.
