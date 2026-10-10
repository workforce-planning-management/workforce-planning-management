# Equality impact assessment: template for any scoring (WPM-R99)

Use this for **any** number, band, rating, ranking, flag or stage that is derived about a person
or that a person's outcome could depend on: succession readiness, a talent pipeline stage, an
assessment band, a review rating, an internal-mobility fit, and any future **risk-of-loss** (flight
risk) measure. Copy the template into `register.md` (or its own file) with one section per score,
fill every field, and have it **approved by someone other than the owner** before the score is
shown to anyone. A score with no approved assessment does not ship (WPM-D67).

> ⚠️ A template and a method, not legal advice. It does not make a score lawful or fair; it makes
> the reasoning visible and reviewable. Equality and data protection law differs by jurisdiction;
> the deployer's lawyers and equality lead decide what applies (for example, in the United Kingdom,
> the Equality Act 2010 and its public sector equality duty; in the European Union, the rules on
> automated decisions and, where an AI system is used, the AI Act's treatment of employment uses).
> Check the current text of whatever applies to you.

## 1. Identification

| Field | Entry |
| --- | --- |
| Score id (the same as in the scoring register) | |
| Name and what it outputs (number, band, rating, flag, stage, order) | |
| Owner (a role) and approver (a different role) | |
| Version of the score and of this assessment, and the date | |
| Is it computed by rules or entered by a person? | |

## 2. Purpose and the decision it informs

- What decision does this score inform (selection, promotion, development offer, redundancy
  selection, pay, targeting of retention effort)? Name each.
- What must it **not** be used for? State it, and say how the software prevents the misuse.
- Is a person always the decider (WPM-D28, D50)? If any step is automatic, say so and justify it.

## 3. Inputs

| Input | Why it is a fair indicator | Source and how fresh |
| --- | --- | --- |
| | | |

- **Inputs refused**, and why: every protected characteristic; anything standing in for one
  (postcode, name, school, age bands, nationality, marital or caring status, part-time status where
  it is a proxy for sex or caring); **sickness absence, leave patterns and reasonable adjustments**
  (using these penalizes disability, pregnancy and caring); and **trade union membership or
  activity, or acting as a representative** (never an input to anything: see
  [worker-unions.md](../worker-unions.md)).
- Is any input itself a score or a judgement by someone else? If so, assess that one too.

## 4. Method

- The rule, thresholds and weights, in plain words and in code (name the function). Who set them,
  on what evidence, and who can change them.
- Is it transparent? Can the person scored be shown what went in and how it was combined?
- What does it do with **missing** data? An unknown is not a zero (WPM-D32): say how it is shown.

## 5. Who is affected

- The population scored, and its size. Groups for which the score may behave differently: list the
  characteristics your equality law protects, and any others that matter locally (for example
  working pattern, grade, site, contract type).
- Who sees the score, and who is affected if it is wrong or leaks?

## 6. Evidence of impact

- **How disparity will be measured**: the rate at which each group is given a favourable outcome
  (rated ready, shortlisted, offered), compared with the best-treated group. A ratio below 0.8 is a
  conventional warning sign, not a legal test and not a safe harbour.
- **Where the group data comes from.** By default WPM stores no protected characteristic (WPM-D17); the
  bounded, off-by-default monitoring data of WPM-D74 is the one exception and gives aggregates only. The
  deployer supplies a grouping at the time of the check from their own, voluntarily given equality
  data held elsewhere; small groups are withheld (a count that points at someone is not shown).
- Results to date, with the date, the groups compared and the numbers. "Not yet measurable" is a
  valid entry and means the score is limited (section 10).

## 7. Indirect discrimination and proxies

Walk through: part-time and flexible workers; people on or returning from leave; career breaks;
age; disability and adjustments; shift and site differences; the effect of who rated whom. For each,
say whether the score could disadvantage the group, and what prevents it.

## 8. Reasonable adjustments

How a person who needs an adjustment is assessed without penalty; that an adjustment is **never an
input** to the score; how the score is recomputed or set aside when an adjustment applies.

## 9. Human oversight and challenge

Who reviews outcomes; how a person finds out a score exists and what it was; how they challenge it;
how long that takes; what happens to an outcome while it is challenged.

## 10. Mitigations, monitoring and limits

| Risk found | Mitigation | Owner | Check | Pause trigger |
| --- | --- | --- | --- | --- |
| | | | | |

- Review frequency and the trigger that suspends use of the score.
- **Limits**: what the score cannot support, stated where the score is shown.

## 11. Consultation

Who was consulted (staff, representatives, recognised bodies, the equality lead, the data
protection officer), when, and what changed as a result. Where the law or an agreement requires
consulting a recognised body or a works council before introducing the score, record it
([worker-unions.md](../worker-unions.md), decision gates).

## 12. Decision

| Decision (approve, approve with conditions, reject) | |
| --- | --- |
| Conditions | |
| Approved by (not the owner) and date | |
| Review due | |

## 13. Related assessments

The data protection impact assessment ([dpia.md](../governance/dpia.md)), and whether the score is
an automated decision with significant effect on a person (it must not be, in this software).
