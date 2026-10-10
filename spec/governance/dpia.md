# Data protection impact assessment: template, completed for the demo (WPM-R92)

> ⚠️ **A template.** It is completed here for the **demo** software with **synthetic**
> data, to show the depth an assessment needs. A deployer must carry out their own
> assessment for their own processing, take their data protection officer's advice, consult
> the people affected, and sign it off. Parts only the deployer can answer are marked
> **deployer to complete**. Nothing here is legal advice.

## 1. Why an assessment is needed

The software systematically evaluates people who are employees (reviews, 360 feedback,
succession readiness, forecasts), processes financial data (pay, expenses) and
health-adjacent data (adjustments, ergonomic assessments, sick leave), and monitors work
(time and attendance). Each is a recognized trigger. **Screening outcome: an assessment is
required** before any real use.

| | |
| --- | --- |
| Controller, contact, data protection officer | deployer to complete |
| Assessor and date | deployer to complete |
| Version of the software assessed | deployer to complete (record the release tag) |

## 2. Description of the processing

- **Nature**: collection, storage, use, masking, audit, export, anonymization and deletion
  of employment records by a web application and API; identities are referenced by URN, not
  copied (WPM-D11).
- **Scope**: the eight kinds of record in the [retention schedule](retention-schedule.md);
  purposes are in the [record of processing](record-of-processing.md); flows are in the
  [data-flow diagram](data-flow.md). Volume, number of people and geography: deployer to
  complete.
- **Context**: employer and employee have an unequal relationship, so consent is rarely a
  valid basis for employment data (a candidate's consent to stay in a talent pool is the
  exception the software models, with an expiry date). Staff may not expect pulse and 360
  data to be handled as described; the software's anonymity claims are in section 5.
- **Purposes**: administer employment; pay people; plan and develop the workforce; keep
  people safe and well; meet legal duties; account for who did what.

## 3. Consultation

- Data subjects or their representatives: deployer to complete (works council or union
  consultation where applicable).
- Data protection officer's advice and the controller's response: deployer to complete.
- Processors and their assurances: deployer to complete (hosting, identity provider,
  backup storage).

## 4. Necessity and proportionality

| Question | Position in the software | Deployer to complete |
| --- | --- | --- |
| Is there a lawful basis for each purpose? | The record of processing lists purposes with the basis left blank | yes |
| Is the data adequate, relevant and limited? | Yes by design: no demographics, URNs not copies, no column for a diagnosis, a symptom, a health cohort or a pulse author (WPM-D17, D20, D24, D25) | whether the organization needs each field |
| Is data accurate and kept up to date? | Display names are refreshable caches of the upstream record; people edit their own details | process for corrections |
| How long is it kept? | A [retention schedule per record kind](retention-schedule.md) with defaults to replace; a floor of 30 calendar days | the basis for each horizon |
| How are people informed? | The software provides no privacy notice | write and publish one |
| How are rights supported? | Access (subject-access export), erasure (anonymization, refused while employment is open), restriction and objection by process; erasure survives a restore (WPM-D63) | request handling, identity checks, upstream copies |
| Are processors bound? | Not applicable to the code | contracts |
| Are transfers safeguarded? | The software makes none | where the hosting and providers are |

## 5. Risks to people, and what reduces them

Likelihood and severity are the assessor's judgement for the demo with real data, before
the deployer's own measures. **Residual** assumes the measures in the third column work as
described and the deployer completes the final column.

| # | Risk to people | L / S | Measures in the software (evidence) | Residual | Deployer action |
| --- | --- | --- | --- | --- | --- |
| R1 | Unauthorized access to pay, reviews or sick leave | Med / High | Sign-in enforced by default and refused-off in production (WPM-D52); attribute policy; masking; tier-based audited reads (`spec/auth.md`, `spec/audit.md`); enforcement matrix tests | Low-medium | Configure the policy and keys; review access; penetration test |
| R2 | Staff identified from "anonymous" pulse or 360 results | Med / High | No author column on pulse responses; counts below a floor are withheld; rater-level 360 comments are not disclosed; but 360 anonymity is **procedural, not structural** and the design says so (WPM-D20, D21) | Medium for small teams | Set the floor; tell staff plainly what is and is not anonymous |
| R3 | Health information inferred from adjustments, ergonomics or leave | Med / High | Barrier and change recorded, never a diagnosis; no aggregate over adjustment requests; leave kinds are high-tier; adjustment words scrubbed on erasure (WPM-D24, D25) | Low-medium | Article 9 condition; restrict who may see sick leave |
| R4 | Unfair automated treatment (forecasts, readiness, rankings) | Low / High | No automated decision; levers are suggestions a person decides; gap views are aggregate and name no one (WPM-D27, D28, D50) | Low | Equality-law review of any scoring added |
| R5 | Excessive monitoring (time, on-call, patterns of absence) | Med / Med | Working-time guardrails advise and never block (WPM-D19); capacity is planned by pool, not by person; no per-person time allocation (WPM-D45, D59); being *away* is shown, never why | Low-medium | Say what is monitored and why; do not add timesheets |
| R6 | Data kept longer than needed | Med / Med | Retention per record kind, swept; backups expire after 35 calendar days; erased data does not return from a restore | Low-medium | Replace the defaults; give the audit log a rule |
| R7 | A restore brings back an erased person | Med / High | Erasure ledger exported off-host and replayed before traffic; the restore drill proves it every push (WPM-D63) | Low | Keep the exported ledger as long as the longest-kept backup |
| R8 | Loss or corruption of data | Low / High | Backup scripts, checksums, the restore drill; targets stated honestly | Low-medium | WAL archiving; rehearse on real data |
| R9 | Disclosure through a breach of the software (injection, scripting, abuse) | Low / High | Content security policy, headers, rate limiting, parameterized queries, dependency checks in CI | Medium until independently tested | Penetration test; patch process |
| R10 | Third-party data (emergency contacts) held without their knowledge | Med / Med | Readable by the person and HR only; deleted on erasure (WPM-D30) | Low-medium | Tell staff to inform their contacts |
| R11 | A person cannot understand or exercise their rights | Med / Med | Subject access names its exclusions; the interface is in 13 languages, machine-written and unreviewed | Medium | Plain-language notice; reviewed translations; accessible formats |
| R12 | The assessment itself is out of date | Med / Med | Counts and answers are generated (`spec/implementation-status.md`); a requirement that adds personal data must update this document | Low | Review at each release and at least yearly |

## 6. Outcome and sign-off

| | |
| --- | --- |
| Measures approved by | deployer to complete |
| Residual risks accepted by | deployer to complete |
| Data protection officer's advice | deployer to complete |
| Does the outcome need the regulator's prior consultation (high residual risk)? | deployer to complete |
| Review date | deployer to complete |

**Maintainers' note.** For the demo, with synthetic data only, no residual risk needs
accepting. For real data, risks R2, R9 and R11 are the ones the software alone cannot bring
down: they need staff communication, independent testing and reviewed translations.
