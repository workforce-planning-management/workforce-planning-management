# The electronic staff record: assignments, coding, learning, reviews and pay deductions

Proposed 2026-10-27 ([plan.md](plan.md) section P; spec round WPM-T309, delivery Phase 24, WPM-T310–T325). Requirements
**WPM-R161–R178**, decisions **WPM-D112–D120**. Nothing here is built.

An **electronic staff record** is the one place an employer keeps everything about a person's employment: what they do, where,
on what terms, how it is coded in the ledger, what they must train in, what they hold, how they are reviewed and what comes off
their pay. Health and public-sector employers (the NHS's ESR is the best-known example) also follow a person **across every
role and employer**, because service, training and pay progression travel with them. WPM already holds much of this; this file
adds what is missing and says how the pieces assemble.

> ⚠️ Demo software ([regulatory.md](regulatory.md)). WPM is not a payroll system of record, and the deduction arithmetic below
> is dated configuration, checked against its source on the date given, not tax advice (WPM-D118).

## What already exists and is only referenced here

| Asked for | Where it is | What this file adds |
| --- | --- | --- |
| Contact info | WPM-R124, `worker_contact_details` ([self-service](self-service-and-worker-requests.md)) | A section of the staff record (R168) |
| Emergency contacts | WPM-R41, `emergency_contacts` ([people and cover](people-directory-and-cover.md)) | A section of the staff record (R168) |
| Demographics (protected characteristics) | WPM-R125, voluntary and off by default (WPM-D74) | Date of birth for pay and pension only (R169, WPM-D114) |
| Payroll runs, payslips | WPM-R13, [payroll-compensation.md](payroll-compensation.md) | Payroll information, deductions, pensions, student loans, my payslips (R174–R178) |
| Skills | WPM-R43, `worker_skills` and history ([skills-and-training.md](skills-and-training.md)) | Qualifications beside them (R171) |
| Positions, establishment | WPM-R152 ([positions.md](positions.md)) | A position number and a cost centre from the register (R163, R165) |
| Annual reviews | review cycles ([talent-development.md](talent-development.md)) | The PADR as a review kind, and when one is due (R172) |
| Band and step | WPM-R54, `worker_pay_positions` ([pay-scales.md](pay-scales.md)) | Readiness for the next step (R173) |
| Mandatory training | the compliance view over training enrolments ([talent-development.md](talent-development.md)) | Requirements by audience with refresher periods (R170) |

## Assignments and how they are categorised

### WPM-R161 — Assignments

- **A worker holds one or more assignments** in their organization. An assignment is a job: a position (optional), a job title,
  the categories of R162, a cost centre (R163), FTE percent, a start date and an optional end date. Exactly one current assignment
  is **primary**.
- **FTE is a contract fact, not time tracking** (WPM-D112). The FTE of a worker's current assignments adds up to at most 100; a
  write that would go over is a `422` naming the total. Across organizations the person-centric record (R167) shows the total and
  says when it is over 100, as the working-time guardrails do: it advises and does not refuse.
- **The worker row stays the record of the primary assignment.** `workers.department`, `job_title`, `location` and `fte_percent`
  are written through from the primary assignment, so the hundred-odd readers of those columns keep working. The migration creates
  one primary assignment per existing worker from those columns. Retiring the columns is a later decision, not this one.
- **A position held by an assignment** links through the assignment. `positions.holder` becomes the holding assignment, so one
  person can hold two seats part-time. The vacancy arithmetic of WPM-R152 is unchanged: held FTE is the assignment's FTE.
- **History.** An ended assignment is kept with its dates. A change of category, cost centre or FTE ends one assignment and starts
  another on the effective date (the same shape as a transfer), so the record answers "what did they do on that day".
- **Not done:** a costing split of one assignment across several cost centres; assignments in an organization the worker has no
  row in (that is a second worker row: R167).

### WPM-R162 — Assignment categories

- **Where you work** (site, and area of work), **what you do** (job role and staff group) and an optional **occupation code** are
  per-organization **vocabularies**: each entry is a code, a label and, where it comes from a national standard, the standard's
  name and its own code (WPM-D116).
- **National standards are imported, not shipped.** The NHS model is the worked example: an ESR occupation code is a **staff
  group, job role and area of work**, and the older three-character NHS occupation codes run beside them as a complementary
  scheme. A deployer imports the list it uses, as ESCO is imported; the demo seeds a short synthetic list labelled synthetic
  (WPM-D48 spirit).
- **A job role and staff group must agree** where the vocabulary says which staff group a job role belongs to; a mismatch is a
  `422`. An assignment with no category recorded reads `unknown`, never a default.
- **Views.** Headcount and FTE by staff group, job role and area of work, as aggregates with the floor (WPM-D35).

## Cost centres, financial coding and subjective coding

### WPM-R163 — Cost centres

- **A register per organization**: code, name, an optional parent, and the dates it is valid. A cost centre is never deleted;
  it is closed.
- **Assignments and positions point at a code in the register.** A new write naming a code that is not open on the effective date
  is a `422`. Existing free-text `positions.cost_centre` values are **listed as unmatched** for HR to resolve, never matched by a
  guess or created silently.
- **Views.** The establishment per cost centre (WPM-R152) rolls up the parent chain; workforce cost per cost centre follows the
  plan cost view's rule (cost shown only to callers who may read salaries).

### WPM-R164 — Financial coding

- **A coding structure** is configuration: an ordered list of segments (for example entity, cost centre, subjective, analysis),
  each with a name, a length and a source (`cost_centre`, `subjective`, a fixed value, or a vocabulary).
- **An assignment's full code is assembled in the pure core.** A segment with nothing to fill it makes the code **incomplete**,
  and the answer names the missing segment; it is never padded with zeros or a default (WPM-D116).
- **Not done:** posting to a ledger. WPM states the code an assignment's pay should carry; a finance system posts it.

### WPM-R165 — Subjective coding

- **A subjective code** is the ledger's expense category for pay (for example "nursing, band 5, basic pay"). It is derived from
  the assignment by **rules** a deployer configures: each rule matches some of staff group, job role, occupation code, pay scale
  band and position number, and gives a code.
- **The most specific rule wins**; two equally specific rules that match the same assignment are refused when the second is
  saved, so the derivation never depends on order. No rule matching is **uncoded**: listed for finance, never given a default
  code.
- **A position has a position number**: a code unique in its organization, given by HR, shown on the seat and usable in a rule.
- **The coding view** of an assignment shows its role, position number, occupation code, staff group, job role, area of work,
  band, cost centre, subjective code, full financial code and the person's **annual service** (R166), each with whether it is
  recorded, derived or unknown.

## Following a person across roles and organizations

### WPM-R166 — Annual service across all roles

- **Service is the union of a person's employment periods**: every worker row with the same `person_ref`, from hire date to
  termination date (or the as-of date for a current row), with overlaps counted once.
- **Breaks.** A gap no longer than a configured tolerance does not break **continuous** service; the tolerance is configuration,
  with no default shipped (employers and agreements differ). The answer gives total service, continuous service to the as-of
  date, whole years of each, and the periods it rests on.
- **Unknown, not zero.** A period with no recorded start makes the total `unknown` and names the period. The pure core takes the
  as-of date as an argument (clock-free).
- **Recognised prior service** (service with an employer WPM has no row for, accepted by HR) is a dated period with its basis
  stated, counted only when HR has recorded it, and shown apart.

### WPM-R167 — The person-centric record

- **One view of a person**: every worker row with their `person_ref` in every organization, each with its assignments, and their
  service (R166), qualifications (R171), professional registrations, and statutory and mandatory learning status (R170).
- **Who sees what** (WPM-D113). The person sees all of it at `/api/me/record`. A privileged caller sees in full the rows in
  organizations they can read; for any other organization they see **the service total and that another employment exists**,
  never the employer, the role or the dates. A manager sees their report's row only, as today.
- **The same person means the same `person_ref`.** Possible duplicates found by the worker matcher are shown to HR as possible,
  never merged into the view (the matcher's decision stays a person's).
- **Training that travels.** A statutory or mandatory completion recorded in another of the person's organizations counts toward
  R170 when the receiving organization's requirement says it accepts transferred completions; the view says which organization
  recorded it.

### WPM-R168 — The staff record screen

- **One screen per worker** (`/staff-record/{pid}`, and `/me/record` for oneself) assembles the record, assignments, coding,
  contact details, emergency contacts, payroll information, learning, qualifications, reviews and service. **It adds no data and
  no read**: each section calls the endpoint that owns it and is governed by that endpoint's rule.
- **Not visible is not empty.** A section the caller may not read says so; a section with nothing recorded says "none recorded";
  an unknown says "—". The three are never shown the same way.

### WPM-R169 — Date of birth, and demographics in aggregate

- **Date of birth** is held on the worker for pay and pension purposes only (pension eligibility and re-enrolment, R176)
  (WPM-D114). The person, HR and payroll read it; a manager does not. It is written by HR or payroll, or by the person through
  `/api/me` when the self-service allow-list includes it.
- **Never an input to a score, a match or a ranking** (age is a protected characteristic): a test fails if other code names the
  column, as for equality monitoring and health data.
- **Demographics in aggregate.** Age bands (configured edges) by organization and department, with the floor. Every other
  demographic (sex, ethnicity, disability, …) remains voluntary equality monitoring (WPM-R125, WPM-D74), unchanged.

## Statutory and mandatory learning, qualifications, and reviews

### WPM-R170 — Statutory and mandatory learning

- **A requirement** has a subject, a **basis**, either `statutory` (required by law or a regulator; the source is named) or
  `mandatory` (required by the organization), a refresher period in months (or none: once is enough), the course URNs that
  satisfy it, and an **audience**: the organization, or some of staff group, job role, area of work and position.
- **Status per person and requirement**: `compliant`, `due_soon` (inside a configured window, 90 calendar days as the existing
  compliance view), `expired`, `never_completed`, `not_applicable`, or **`unknown`** when the person's assignments do not record
  the category the audience is defined by (WPM-D117). Unknown is never counted as compliant or as missing.
- **Evidence** is a completed training enrolment for one of the courses, or a completion HR records with its date and source
  (for training done elsewhere). Transferred completions follow R167.
- **Views.** The person sees their own; a manager sees their reports by name (this is a work record, not a sensitive one); HR
  sees their organization. Roll-ups by department, staff group and requirement are counts and rates with the floor; a rate with
  nothing to divide by is `null`.
- **Frameworks are mapped, not copied.** A national framework (in England, the Core Skills Training Framework of Skills for
  Health: eleven subjects, each with a required refresher frequency) is imported as requirements a deployer chooses; the repo
  holds no framework text (WPM-D100).

### WPM-R171 — Qualifications

- **A qualification** held by a worker: kind (`academic`, `professional`, `vocational`, `licence`), title, awarding body, an
  optional level on a named framework (for example RQF or EQF level), awarded on and optional expiry.
- **Declared and verified are different facts.** A qualification the person adds is `declared`; HR may mark it `verified`, with
  who, when and an evidence reference. **No certificate is stored**: the evidence is a reference to where it is held.
- **Requirements.** A role profile may list qualifications it requires; a worker in that role without a verified match shows a
  qualification gap beside the skill gap (WPM-R43). A declared, unverified match is shown as such, not as met.
- **Who sees it.** The worker, their managers up the chain and HR (a qualification is not sensitive in itself). In the
  subject-access export; erased with the person.

### WPM-R172 — Annual reviews and the personal development review (PADR)

- **A review has a kind**: `performance` (the existing review) or `padr`. A PADR records objectives for the year ahead,
  development needs (each may link to a development plan item), a summary of the values and behaviours conversation, whether a
  wellbeing conversation was **offered** (yes or no; never what was said, WPM-D73 spirit), and two sign-offs.
- **Complete means both signed.** The reviewer and the reviewee each sign with a date; the reviewee may add a comment. Signed
  content is frozen; a correction is a new version with the previous kept.
- **When one is due.** Twelve months after the last completed PADR, or a configured number of months after the start of the
  primary assignment for someone who has never had one. The pure core takes the as-of date.
- **Views.** The person and their managers up the chain see the PADR (review content stays high sensitivity, ABAC-scoped and
  audited on read, as today). Managers see their team's due and overdue by name; HR's roll-up is counts by department with the
  floor.

### WPM-R173 — Readiness for the next pay step

- **On a worker's step date** (WPM-R54), the service reports whether the configured **progression standards** are met. The
  standards a deployer can switch on: a PADR completed within the last 12 months; statutory and mandatory learning compliant;
  **no formal process live**. In NHS England this is the Agenda for Change pay progression framework (Annex 23 of the handbook);
  employers word their standards differently, so each is configuration.
- **"No formal process live" is a yes or no** that HR records with a date; there is no column for what the process is or why
  (WPM-D120).
- **It reports; a person decides** (WPM-D49, WPM-D50). A standard that cannot be evaluated is `unknown`, and the report says
  which. The move to the next step is the existing pay position write, by HR, audited.

## Pay: payroll information, deductions, pensions, student loans and payslips

### WPM-R174 — Payroll information

- **Per worker**: payroll number, pay frequency (`weekly`, `fortnightly`, `four_weekly`, `monthly`), tax code, national insurance
  category letter, starter declaration, and a reference to the payee record in the payroll provider.
- **Bank details and the national insurance number get no column** (WPM-D115). The payroll provider holds them; WPM holds the
  provider's reference. A field that looks like either is refused in any free text this file adds.
- **Who.** Payroll and HR write; the person reads their own and cannot change a tax code (it comes from the tax authority's
  notice). Highest sensitivity: payroll, HR and the person. Every write audited without values.

### WPM-R175 — Deductions

- **A deduction instruction** for a worker: kind (`salary_sacrifice`, `payroll_giving`, `loan_repayment`, `overpayment_recovery`,
  `other`), a fixed amount in minor units or a rate in basis points, start and optional end, an optional cap per period, and a
  reference. No free-text reason (WPM-D119).
- **Order.** Statutory deductions (tax, social, student loan, pension) come first, then instructions in the order HR set. A
  voluntary deduction that would take net pay below a configured floor is **held for that period**, not taken in part, and the
  payslip says it was held. The arithmetic is pure core and overflow-checked, and net still equals gross minus deductions.
- **No union subscriptions through payroll** (WPM-D119). A check-off deduction would record union membership, which WPM does not
  hold ([worker-unions.md](worker-unions.md)).
- **Not done:** attachment of earnings and other court orders (statutory priority and protected-earnings rules are a payroll
  system's job); salary sacrifice's effect on pensionable pay beyond the scheme's stated rule.

### WPM-R176 — Pensions

- **A pension scheme** is configuration: name, kind (`defined_benefit`, `defined_contribution`, `hybrid`), the employer rate and
  **member contribution tiers** by annual pensionable pay, effective from a date. Worked example, checked 2026-10-27: the NHS
  Pension Scheme (England and Wales) from 1 April 2026 has six member tiers, 5.2% up to £13,259, 6.5%, 8.3%, 9.8%, 10.7%, and
  12.5% from £67,669, with an employer rate of 23.7% (NHSBSA). The demo seeds a synthetic scheme, labelled synthetic.
- **Membership** per worker and assignment: scheme, status (`member`, `opted_out`, `not_eligible`, `unknown`), joined, left and
  opted-out dates. An opt-out is recorded with its date and never a reason.
- **Contribution.** The tier is found from the assignment's **annualised pensionable pay**; the member contribution is that
  tier's rate on the period's pensionable pay. Unknown pay is unknown tier, never the lowest. This replaces the stub percentage
  for scheme members; a benefit-enrolment pension (the existing rule) still works for others.
- **Eligibility.** Where a deployer configures an automatic-enrolment rule (age and earnings bands), the service reports who is
  due to be assessed or re-enrolled; it needs the date of birth (R169), and with none recorded the answer is `unknown`.

### WPM-R177 — Student loan deductions

- **Plans held by a worker**: plan type, the date of the start notice and an optional stop date. Plan types are configuration by
  jurisdiction. For the UK, checked 2026-10-27 against GOV.UK for the 2026 to 2027 tax year: **Plan 1** £26,900 a year, **Plan 2**
  £29,385, **Plan 4** £33,795 and **Plan 5** £25,000, each at 9% of earnings above the threshold; **Postgraduate loan** £21,000 at
  6%.
- **Pure core.** The period threshold is the annual threshold for the pay frequency; the deduction is the rate on earnings above
  it. Only one of plans 1, 2, 4 and 5 is deducted at a time (GOV.UK: start on the plan with the lowest threshold until a start
  notice says otherwise); a postgraduate loan is deducted alongside it.
- **Unknown plan.** Where a worker has a loan and the plan is not known, GOV.UK's rule (deduct as Plan 5 until a start notice)
  is configuration the deployer turns on, and the payslip says the default was used.
- **Thresholds are dated configuration with their source** (WPM-D118); a run whose period has no thresholds configured refuses to
  calculate and names the missing year. **Not checked:** the rounding rule for each period and how employers treat a change of
  plan mid-period.

### WPM-R178 — Managing your payslips

- **`/api/me/payslips`**: the person's own payslips from **approved and paid** runs (never a draft or a calculated run), newest
  first; each with gross, every deduction line (including held deductions, student loan and pension tier), net and the run's
  period.
- **Year to date** per tax year (the start of the tax year is configuration) for gross, each deduction kind and net, derived from
  the payslips, never stored.
- **A printable payslip**: a print stylesheet on the screen, no PDF made by the server.
- **A payslip query**: the person may flag one payslip as queried, with a short comment, which payroll sees and closes. No
  amount is edited through a query.
- Every read is audited (the existing rule).

## Design decisions

- **WPM-D112 An assignment is a contract, not a timesheet.** An assignment's FTE is what the contract says, as `fte_percent`
  already is. It refines WPM-D45, D59 and D104: there is still no per-person allocation of time to pools, programmes or
  positions; a person who holds two jobs holds two assignments. Rejected: a percentage split of one worker across positions,
  which is the allocation those decisions refused.
- **WPM-D113 A person is followed by `person_ref`, and another employer's record stays with that employer.** The person sees
  everything; HR elsewhere sees the derived service total and that another employment exists. Service and transferred training
  are the facts that legitimately travel; the role, employer and dates do not.
- **WPM-D114 Date of birth is held for pay and pension, and never scores anyone.** It is a bounded exception to "never raw
  demographics" ([domain-model.md](domain-model.md)), like WPM-D74: purpose-limited, readable by the person, HR and payroll,
  aggregate only as age bands with the floor, and kept out of every score by a test. Rejected: reading it from the person service
  on each payroll run, as wellbeing entitlements read age today ([hr-core.md](hr-core.md)). A prompt can be re-evaluated; a
  payroll run and a pension assessment must be reproducible from what WPM held that day. Entitlements keep reading upstream.
- **WPM-D115 Bank details and the national insurance number get no column.** They are what payroll fraud and identity theft
  want, and WPM is not the payroll system of record. The provider holds them; WPM holds a reference.
- **WPM-D116 Codes are configuration, and uncoded is unknown.** Cost centres, subjective codes, occupation codes and coding
  segments come from the deployer. A missing code is reported and never filled with a default, a zero or a guess. National lists
  are imported with their source, as ESCO is; licensed text is mapped, not copied (WPM-D100).
- **WPM-D117 Statutory is not mandatory, and an unknown audience is not compliance.** Each requirement says which it is and,
  for statutory, the source. A person whose category is not recorded is `unknown` against a requirement aimed at that category,
  and is counted apart.
- **WPM-D118 Payroll arithmetic is dated configuration with its source.** Thresholds, rates and tiers are stored with an
  effective date and the source they were checked against; the demo seeds values labelled with that source and date, or labelled
  synthetic. A period with no configuration refuses rather than guessing.
- **WPM-D119 A deduction says nothing about why, and none reveals union membership.** Deduction instructions carry a kind and a
  reference, never a reason (an overpayment's cause, a loan's purpose). Union check-off is out of scope.
- **WPM-D120 Progression readiness reports evidence; a person moves the step.** A formal process is recorded as live or not with
  a date, never what it is.

## Order of delivery

Assignments and categories come first (R161–R162, WPM-T310–T311): coding, learning audiences and pensions all read them. Then
cost centres and coding (R163–R165, WPM-T312), then service and the person-centric record (R166–R167, WPM-T313), then statutory
and mandatory learning (R170, WPM-T314–T315), qualifications (R171, WPM-T316), the PADR (R172, WPM-T317) and step readiness
(R173, WPM-T318). Pay follows: date of birth (R169, WPM-T319), payroll information (R174, WPM-T320), deductions (R175, WPM-T321),
pensions (R176, WPM-T322), student loans (R177, WPM-T323) and my payslips (R178, WPM-T324). The staff record screen (R168,
WPM-T325) comes last, because it assembles the others.

## Spec round (WPM-T309): what was checked

| Item | Checked against | Result |
| --- | --- | --- |
| UK student loan thresholds and rates, 2026 to 2027 | GOV.UK, "Special rules for student loans" (fetched 2026-10-27) | Plan 1 £26,900, Plan 2 £29,385, Plan 4 £33,795, Plan 5 £25,000 at 9%; postgraduate £21,000 at 6%; lowest-threshold plan first; postgraduate alongside; default Plan 5 when unknown |
| NHS Pension Scheme member tiers from 1 April 2026 | NHSBSA employer hub, as quoted in search results, and NHS Employers for the employer rate | Six tiers as in R176; employer rate 23.7%. The Scottish scheme's bands were not read and may differ |
| ESR coding | NHS England, "A guide to the staff group, job role and area of work classifications used in ESR" (search excerpts) | The occupation code is staff group, job role and area of work; older occupation codes are complementary |
| Pay progression standards | NHS Employers' Annex 23 and employers' published policies (search excerpts) | Appraisal in the last 12 months, statutory and mandatory training, no formal capability process or live disciplinary sanction; wording varies by employer |
| Core Skills Training Framework | Skills for Health overview (search excerpts) | Eleven subjects in England, each with a required refresher frequency; per-subject periods not read |

**Not checked:** the student loan rounding rule per period; the NHS (Scotland) and (Northern Ireland) pension tiers; the
current NHS Data Dictionary staff group list (the excerpt read was the 2017 release); the CSTF's per-subject refresher periods;
the break in service that agreements disregard for continuous service (hence no default tolerance); and every non-UK
jurisdiction.
