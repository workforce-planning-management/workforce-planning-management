# Regulatory posture

> ⚠️ **Demo software.** Not a production HR/payroll system; no real
> personal data; statutory calculations are illustrative stubs.

## Observed by design

- **Data minimisation** — identities are URNs; WPM stores employment
  facts, not demographics; display names are refreshable caches.
- **Purpose limitation & consent** — the candidate pool is
  consent-bounded (`consent_until`); expired candidates leave search
  and are flagged for purge.
- **Access control** — ABAC personas + salary/review masking; the
  activation gate must be on before any real exposure.
- **Auditability** — mutations and sensitive reads audited (the
  substrate for GDPR accountability and payroll audit).
- **Synthetic data only** in seeds and tests.

## Newer data classes (WPM-T74–T89)

- **Emergency contacts** are other people's personal data: readable only by the
  person and HR, audited without detail, in the subject-access export, deleted
  on erasure (WPM-D30).
- **Expense claims** are financial and personal: seen by the claimant, their manager and HR only; never
  decided by the claimant; no amount or text in the audit or notifications; in the subject-access export;
  on erasure draft/submitted claims are cancelled, approved/reimbursed ones keep their amounts (statutory
  retention, WPM-D22) and every free-text field is scrubbed (WPM-D42).
- **Workplace health requirements** (WPM-D75) are health data. Off unless the deployer records a lawful basis. A record is a
  status (`up_to_date`, `exempt_recorded`, `declined`) and two dates: there is no column for a diagnosis, a product, a batch or
  the reason for an exemption. Written only by an occupational-health role (a token attribute), not by HR, a manager or the
  worker; the worker sees their own; a manager and HR see only cleared or not cleared; the detail is for occupational health and
  each read is audited; exported to the worker alone; erased with the worker; and never an input to a score (a test fails if
  other code names the table).
- **Equality monitoring answers** are special-category data (WPM-D74). Off unless the deployer records a lawful
  basis; WPM ships no classification (the deployer defines the categories); voluntary, with `prefer not to say`
  always available; readable by the worker only, never by a manager, HR, payroll or administrator; exported to the
  worker alone; output only as an aggregate with small groups withheld; audited without values; erased with the
  worker; and never an input to any score or decision (a test fails if other code names the table).
- A worker's **home address, telephone numbers and personal e-mail** are kept by the worker themself
  (WPM-R124): readable by the worker, HR and payroll only (a manager cannot), the previous value is not kept, the
  audit says they changed and never what they were, an HR or payroll read is audited, and they are in the
  subject-access export and deleted on erasure.
- A worker's **pay band and step** is a salary: the person and HR only, audited and
  notified without any figure, in the subject-access export, deleted on erasure
  (WPM-D41).
- A worker's **job level** is high-sensitivity career data (it tracks pay): the
  person and HR only, audited without the level, in the subject-access export,
  deleted on erasure (WPM-D40).
- **Directory** and **announcements** carry no sensitive field; being *away* is
  shown, never why (WPM-D29). **Read receipts** are the reader's data (exported,
  erased) and editors see only a count (WPM-D35).
- **Skill aspirations** are used only in the owner's own gap view and never in
  workforce roll-ups (WPM-D32).
- **Joiner/leaver records and handover actions** are in the subject-access
  export; erasure scrubs notes and unassigns tasks but **keeps the handover
  audit rows** (the trail of who gave what to whom is an accountability record,
  keyed to a pid that no longer identifies anyone).
- The retention sweep list (`rules::privacy::SOFT_DELETED_TABLES`, 55 tables)
  covers every soft-deleting table, new ones included.

## Proposed data classes (Phase 24, WPM-T309)

- **Date of birth** (WPM-D114) is held for pay and pension only: the person, HR and payroll; aggregate only as age bands with the floor;
  never an input to a score, a match or a ranking (a test fails if other code names the column); in the export; erased.
- **Bank details and the national insurance number** are not held (WPM-D115). Tax code, NI category and the other payroll details
  are highest sensitivity: payroll, HR and the person.
- **Sickness absence** stays reasonless (WPM-D73). A **fit note** is its dates and one yes/no; an **occupational health referral** is
  status and consent, and the advice stays with occupational health (WPM-D123). Work-related **stress** is assessed in the work, never
  recorded as a label (WPM-D121). These are health-adjacent: high sensitivity, in the export, erased or scrubbed with the worker.
- **Disability** is held nowhere except voluntary equality monitoring (WPM-D74, WPM-D125). The adjustments passport and "working with
  me" are the person's own, shared only by them (WPM-D126); candidate adjustments are kept from scoring and deleted with the
  application.

## AI Act duties of an employer that deploys a high-risk system (WPM-R148, WPM-D103)
 that deploys a high-risk system (WPM-R148, WPM-D103)

WPM scores people in a few places (the scoring register, [equality-impact/register.md](equality-impact/register.md), WPM-R99).
Where a deployer uses such a feature as an AI system in employment (Annex III point 4 of Regulation (EU) 2024/1689: recruitment,
decisions on work relationships, task allocation, monitoring and evaluation of workers; confirm the point's exact wording in the
Official Journal), Article 26 puts duties on the **deployer**. The register records them; it does not discharge them.

- **Human oversight** is assigned to natural persons with the competence, training and authority (Article 26(2)). The register
  holds the **overseer role** per feature; a feature with none recorded shows `unknown`, never "overseen" (WPM-D103).
- **Logs** the system generates are kept for a period fitting its purpose and **at least six months**, unless other law says
  otherwise, data protection law in particular (Article 26(6)). The register holds the **retention period**; WPM shows the
  period, and whether its own audit trail is the log is a statement the deployer makes, not one WPM infers.
- **Workers are told.** An employer, before putting a high-risk system into service or using it at the workplace, informs the workers'
  representatives and the affected workers that they will be subject to it (Article 26(7)): an **information** duty, not a
  consultation. The register holds the **date informed**, and to whom (the recognised body, [worker-unions.md](worker-unions.md),
  and the affected group), not a copy of the notice.
- **A person subject to a decision** is told they are subject to the system when it makes or helps make a decision about them
  (Article 26(11)), and may ask for an explanation (Article 86; *not read in this round*). WPM offers a **per-decision
  explanation the person concerned can read** (the factors and their weights, as the scoring rules state them), through the
  person's own record.
- **When it applies.** Annex III obligations were due from 2 August 2026 and were moved to **2 December 2027** (Annex I systems to
  2 August 2028) by the Digital Omnibus on AI, entered into force on 27 July 2026 (the amended Article 113 as the Commission's AI Act
  Service Desk shows it; the amending act's number was not read). The register therefore says **"applies from" with the date as
  configuration**, not a fixed date in code.
- **Not done.** WPM does not classify a system as high-risk (a deployer does, with legal advice), does not certify conformity (that
  is the provider's CE marking), and a filled-in register is not evidence of compliance (WPM-D103). Article 4 (AI literacy) and
  Article 27 (a fundamental-rights impact assessment, for some deployers) are outside this requirement.

### What is built (WPM-T274)

- `GET /api/ai-register` (any signed-in caller: it is a notice of how people are scored) lists the eight features of the scoring
  register, S1 to S8 (`rules::ai_register::FEATURES`; S9, a risk-of-loss score, is deliberately not built), each with what is
  *made of* in a sentence a worker can read and what the deployer recorded. `PUT /api/ai-register/{feature}` (HR and
  administrators) replaces the record, audited in the same transaction (WPM-D107).
- Recorded per feature: the Annex III category (`recruitment_selection`, `work_relationship_decisions`, `task_allocation`,
  `monitoring_evaluation`, or `not_high_risk`, which is the deployer's statement and not WPM's), the **overseer role** (a role; an
  address is refused), the date representatives were informed, the date workers were informed (neither in the future), and the log
  retention in months. A retention under six months needs a note naming the other law, and is flagged. A field with nothing
  recorded is `unknown`. The date the duties apply is `WPM_AI_ACT_APPLIES_FROM`.
- **Not built:** the per-decision explanation a person can read (Article 86; each score's factors for the person concerned); the
  link to the recognised body (Phase 18 is not built, so informing representatives is a date, not a link); `rules::scoring::REGISTER`
  of WPM-T166 is still not in code, so the features are listed in `rules::ai_register` and no test yet fails when a new scoring
  function is not registered.

## Production would additionally require

- UK GDPR / DPA 2018 lawful-basis mapping per record class;
  statutory **retention schedules** (payroll and right-to-work
  records carry multi-year duties; candidate data the opposite);
  subject-access and erasure flows coordinated with the identity
  services; jurisdiction-correct payroll/tax engines; equality-law
  review of any scoring (application screening, succession
  readiness); works-council/union consultation where applicable.
  Tracked as production gates in [tasks.md](tasks.md).

## UK regulators of sector employers

The target is worldwide ([worldwide.md](worldwide.md), WPM-R214); the UK is one region here, and other regions are *not yet read* (WPM-T370).

- **MHRA** (Medicines and Healthcare products Regulatory Agency): regulates medicines, medical devices and blood components for
  transfusion in the UK; an executive agency of the Department of Health and Social Care. Source:
  <https://www.gov.uk/government/organisations/medicines-and-healthcare-products-regulatory-agency>, read 2026-10-28. WPM holds no
  licence, product, device or adverse-event data. What the MHRA requires of an employer's staff is **not yet read** ([mhra.md](mhra.md)).
- **ISO 14971** (*Medical devices — Application of risk management to medical devices*), the international standard for a manufacturer's
  risk management of a medical device. Recorded from the author's recollection, **not read**: the standard is paid-for and its text was not
  available, and the edition in force (believed to be the 2019 third edition) is unchecked. WPM holds no device, hazard, risk-file or
  adverse-event data and claims no conformance. Whether it bears on workforce planning at all (for example competence of the people who
  carry out risk management) is **not established** ([mhra.md](mhra.md)).
