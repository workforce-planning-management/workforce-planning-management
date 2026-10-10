# Scoring register and equality impact assessments, completed for the demo (WPM-R99)

Every score WPM derives about people, with its assessment. The register is the list; the template
is [template.md](template.md). **These assessments are first drafts written by the project's AI
assistant from the code and the spec; no equality lead or lawyer has reviewed them.** They record
what the software does and what it avoids; they cannot record impact measured on real people,
because the demo has none. A deployer must replace "not yet measurable" with their own evidence.

> ⚠️ Not legal advice. See the caveat in the template.

> **Equality monitoring data (WPM-D74)** is not a score and no score may read it. It exists only to produce the
> aggregates that feed the adverse-impact check of WPM-R100, as a grouping the deployer chooses. A test
> (`rules::equality_monitoring::tests::no_other_code_reads_the_declarations`) fails if any other code names the
> table.

## The register

| Id | Score | Kind | Computed by | Informs | Sees it | Code |
| --- | --- | --- | --- | --- | --- | --- |
| S1 | Succession readiness (`ready_now`, `ready_1y`, `ready_2y`) | rating | a person | succession shortlist | HR and involved parties | `rules::talent::READINESS` |
| S2 | Talent pipeline stage (`identified` to `placed`) | stage | a person | development offers, placement | HR and the pipeline owner | `rules::talent::PIPELINE_STAGES` |
| S3 | Assessment score band (`low` to `high`, by percentile) | band | rules (norm-referenced) | selection, development | HR and the person | `rules::assessment::band_for_percentile` |
| S4 | Review rating and competency mean | rating, mean | a person; rules for the mean | pay review, development | the person, their manager and HR | `rules::appraisal::competency_mean`, review lifecycle |
| S5 | Internal-mobility fit (critical requirements met of total) | ratio | rules | the **worker's own** ordering of roles | the worker | `rules::mobility::fit_score` |
| S6 | Skill-gap priority | order of gaps | rules | training spend and hiring | HR and planners, as aggregates | `rules::skill_gap::priority`, `rank` |
| S7 | Change readiness (skills meeting a bar) | counts | rules | reskilling plans | planners, as aggregates | `rules::change::skill_readiness` |
| S8 | Workforce insight flags (turnover, span of control, time to fill) | thresholds | rules | leadership attention | leaders, as aggregates | `rules::insights` |
| S9 | **Risk of loss** (flight risk) | not built | | | | |

**S9 is deliberately not built.** An individual risk-of-loss score predicts who might leave, so it
quietly decides who gets attention, retention pay and development. It is exposed to the worst
proxy problems (age, caring, sickness, part-time, pay gaps, a recent adjustment) and to use as a
surveillance tool. WPM offers **department-level attrition** (S8, with a floor on group size) and
no individual measure. A deployer who wants one must complete the template in full, including
section 6 evidence, and have it approved before it is built (WPM-D67); the recommended
alternative is to measure *why* people leave (exit data, pulse) in aggregate.

## S1 Succession readiness

- **Purpose and decision.** A person (a manager or HR) rates a candidate for a succession plan as
  ready now, in one year or in two. It informs a shortlist; it does not select. It must not be used
  for redundancy selection or pay. A succession plan is restricted to HR and the involved people
  (`spec/auth.md`).
- **Inputs.** None are computed: the rating is a judgement. Raters should not use absence, leave,
  working pattern or adjustments. **The software does not say so on the form and does not enforce
  it**; adding that guidance next to the rating is a task (WPM-T167).
- **Method.** Three ordered values; a human enters them; readiness may regress.
- **Who is affected.** Everyone nominated; the nominating pattern itself (who is nominated) is the
  larger fairness risk than the rating.
- **Evidence.** Not yet measurable here. Check: the rate of `ready_now` by group among those
  nominated, **and the rate of nomination** by group.
- **Proxies.** Part-time and flexible workers and those returning from leave are less visible to
  raters; managers rate people they know. Mitigation: require a second rater and a written basis;
  review nominations as well as ratings.
- **Adjustments.** None are inputs.
- **Oversight.** A person decides every outcome (WPM-D28). Whether the candidate may see their own
  readiness, and the challenge route, are the deployer's to decide; check the access rules in
  `spec/auth.md` before promising either.
- **Decision (draft).** *Approve with conditions*: a written basis for each rating, a second
  rater, and a yearly disparity check on nomination and rating. **Status: not approved by anyone.**

## S2 Talent pipeline stage

- **Purpose.** Tracks development of people towards a role. Stage moves are by a person; `ready`
  can step back. It informs development offers and placement.
- **Risks and mitigations.** As S1; plus the stage becomes a gate to opportunity. Condition: the
  basis for entering the pipeline is written, and the entry rate is checked by group.
- **Decision (draft).** *Approve with conditions.* **Not approved by anyone.**

## S3 Assessment score band

- **Purpose.** Turns a raw score into a percentile band for aptitude, personality and similar
  tests. It informs selection and development; the spec says an **equality-law review is a
  deployment duty before any selection use** (`spec/talent-development.md`).
- **Inputs.** Raw test scores only; the norm group is the deployer's.
- **Method.** Fixed percentile cut points (bottom tenth `low`, next fifth `below_average`, middle
  two fifths `average`, next fifth `above_average`, top tenth `high`).
- **Risks.** Tests can have group differences unrelated to the job; norm-referenced bands force a
  fixed share into `low` whatever the group's real ability; timed tests disadvantage some
  disabilities; personality tests carry the heaviest risk. Mitigations: validation evidence for each
  test against the job, adjusted time and format, never use a band as the sole basis, never use
  personality bands to select.
- **Evidence.** Not measurable here. Check pass rates by group for every test used in selection.
- **Decision (draft).** *Reject for selection use* until validation and group evidence exist;
  *approve with conditions* for the person's own development use. **Not approved by anyone.**

## S4 Review rating and competency mean

- **Purpose.** A rating by a manager, calibrated, and a mean of competency scores. It informs pay
  review and development.
- **Risks.** The best-documented bias source: who rates whom, leniency differences, recency. A
  mean of ordinal scores is a convenience, not a measurement. Mitigations: calibration across
  managers, the mean shown with its count (`rules::appraisal::competency_mean` returns the count),
  and a disparity check on ratings by group.
- **Decision (draft).** *Approve with conditions.* **Not approved by anyone.**

## S5 Internal-mobility fit

- **Purpose.** For one worker, orders roles by how many of the role's critical requirements the
  worker's **own declared** skills meet. It ranks roles for the worker; it never ranks people
  (`rules::mobility`).
- **Risks.** Low: self-declared skills are an input, and under-declaring is more common in some
  groups, so the fit can understate. Mitigation: the worker sees their fit and the declared skills
  behind it, and can add evidence.
- **Decision (draft).** *Approve.* **Not approved by anyone.**

## S6 Skill-gap priority, S7 change readiness

- **Purpose.** Aggregate views of where skills are short; they rank skills and departments, not
  people, and withhold groups below the floor.
- **Risks.** Small-group re-identification (mitigated by the floor) and use of an aggregate to
  justify individual decisions (forbidden by WPM-D28).
- **Decision (draft).** *Approve.* **Not approved by anyone.**

## S8 Workforce insight flags

- **Purpose.** Flags a department's turnover, span of control or time to fill against thresholds
  stated in the payload. Aggregate only.
- **Risks.** A high-turnover flag can be read as a verdict on a manager or a group; it states its
  threshold and says it is a prompt to look. **Not approved by anyone.**

## What every assessment still needs from the deployer

Their own groups and data for section 6, their own approver, their own review date, and their own
consultation (section 11).
