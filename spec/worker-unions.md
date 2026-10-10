# Worker unions: the concept, and what a workforce planning tool should and should not do (WPM-R102–R108)

Status: **research and proposal, 2026-10-10.** Nothing here is built. Generic by design: no real
union, employer, government or scheme is modelled, and the law below is cited as examples a
deployer must check for their own jurisdiction. Where a statement comes from my own knowledge
rather than a source I read, it is marked *(unverified)*.

> ⚠️ **Not legal advice.** Industrial relations law differs by country and changes. A deployer's
> employment lawyers and their recognised bodies decide what applies.

## 1. What a union is, in the terms of this tool

- A **trade union** (labor union) is an organization of workers that represents their interests,
  above all in **collective bargaining** with an employer over pay and conditions. Two ILO
  fundamental conventions give the international frame: freedom of association and the right to
  organize (No. 87), and the right to organize and collective bargaining (No. 98), which protects
  workers and unions against anti-union discrimination and employer interference and promotes
  voluntary bargaining. Even a state that has not ratified them is bound to respect these
  principles by its ILO membership
  ([ILO Convention 98](https://tradefordecentwork.ilo.org/wp-content/uploads/2024/09/ILO-Convention-No.-98-Right-to-Organise-and-Collective-Bargaining-Convention.pdf)).
- **Recognition.** An employer *recognizes* a union for a defined group of workers, the
  **bargaining unit**. In Great Britain, recognition may be voluntary (the parties agree the unit,
  the scope of bargaining and facility time) or statutory, through the Central Arbitration
  Committee, which tests membership and support and, failing agreement, decides the unit
  ([Acas on voluntary recognition](https://www.acas.org.uk/collective-conciliation/getting-voluntary-trade-union-recognition);
  [recent reforms](https://www.osborneclarke.com/employment-law-reforms-horizon/trade-unions-industrial-action),
  whose commencement dates are not confirmed). The unit is defined by *roles and places*, not by
  who joined.
- **A collective agreement** is the outcome: terms (pay scales, hours, notice, procedures) that
  apply to the *unit*, whether or not each worker is a member. **Coverage is not membership.**
- **Works councils and similar bodies** are the other model: elected employee representatives with
  rights to *information, consultation and, in some countries, co-determination* over specified
  decisions, regardless of union membership. The EU frame is Directive 2002/14/EC on information
  and consultation
  ([text](https://www.legislation.gov.uk/eudr/2002/14)) and, for redundancies, Directive 98/59/EC
  ([text](https://www.legislation.gov.uk/eudr/1998/59)); Germany's Works Constitution Act gives
  works councils co-determination on social matters and consultation on economic changes
  ([overview](https://www.businesslocationcenter.de/en/labor-market/employment-law-and-collective-contracts-system/german-works-council-constitution-act)).
- **Representatives** (stewards, delegates, council members) are workers with legal protection
  against detriment and usually a right to paid time off for the role (**facility time**).
  *(unverified: the statutory detail varies; in Great Britain the source is the 1992 consolidation
  Act and an Acas code of practice, which I did not read.)*

## 2. Why this matters to workforce planning

A workforce plan is exactly what triggers collective duties:

| Planning act | Collective duty it may trigger | Example |
| --- | --- | --- |
| A plan that reduces headcount at one site | Collective **redundancy consultation**, with a minimum period before the first dismissal takes effect, and notice to a public authority | Great Britain: 20 or more at one establishment within a 90-calendar-day period; consultation at least 30 calendar days before the first dismissal for 20–99, and 45 for 100 or more; the clock runs to the termination date, and it is a floor, not a target ([government guidance](https://www.gov.uk/redundant-your-rights/consultation); [commentary](https://www.lewissilkin.com/insights/2026/04/07/collective-redundancies)). EU: Directive 98/59/EC with thresholds by establishment size, consultation "in good time" with a view to agreement, and notice to the authority |
| Restructuring, reorganizing work, changing contracts | **Information and consultation** (EU) or **co-determination** (for example Germany) | Directive 2002/14/EC; a social plan and a reconciliation of interests in Germany |
| Changing shift patterns, introducing time or performance monitoring, introducing a new scoring | Consultation or a works council's consent, in some systems | The Works Constitution Act's co-determination on social matters (monitoring and working-time rules) *(unverified detail)* |
| Setting pay | Collective bargaining where the unit is covered | A pay scale in WPM is already reference data from a circular; a collective agreement is the same shape |

So the useful things for the tool to do are about **obligations, calendars, reference data and
aggregate information**, not about individual workers' affiliations.

## 3. The line the tool must not cross

**Trade union membership is special category personal data** under the UK GDPR and the EU GDPR
(it is named in Article 9). An employer holding it needs a specific condition, and in practice it
arises for the **check-off** deduction of subscriptions through payroll, which requires the worker's
written authorization
([a legal summary](https://legal-island.com/employment-law-hub/q---a-is-it-lawful-for-an-employer-to-monitor-communications-between-a-trade-union-representative-and-trade-union-members);
[an industry guide to the GDPR](https://news.industriall-europe.eu/documents/upload/2023/7/638242381242430635_GDPR_EN7_FINAL.pdf)).
Beyond that, the history is a warning: in Great Britain a database of workers' union membership and
activity was used to vet job applicants, and the practice was banned by regulation (the
[Blacklists Regulations 2010](https://www.legislation.gov.uk/ukdsi/2010/9780111491041/note/data.htm)).
A list of who is a member, or a representative, is exactly the artefact that was misused.

**Therefore (WPM-D68): WPM stores no individual's union membership, no representative status, and
nothing from which either could be read** (no flag, no tag, no deduction label that names a union,
no time-off reason that names representative duties, no attendee list of a union meeting). This is
the same stance as for diagnoses and the pulse author: what must not be stored gets no column
(WPM-D17, D20, D24). Where a deployer must run check-off, it stays in their payroll process, with
the worker's written authorization, access limited to payroll, and it is recorded as special
category processing in their impact assessment; WPM does not model it.

## 4. Proposed capabilities (all about the organization, not the person)

- **WPM-R102 Recognised bodies and bargaining units.** Reference data about the *organization*: a
  recognised body (a name the deployer chooses; a kind: union, works council, staff forum), its
  recognition date and basis, a contact **role** (not a person), and the **bargaining units** it
  covers, each defined by rules over employment facts WPM already holds (department, role
  profile, location, employment basis, grade). The coverage count is an aggregate with a floor on
  group size; it names no one and says nothing about who belongs to the body.
- **WPM-R103 Collective agreements as reference data.** Like a pay scale: transcribed, dated,
  sourced, versioned, with the unit it covers, effective dates and the terms WPM can use (a pay
  scale reference, working-time limits, notice periods, a consultation procedure). Whether an
  agreement *applies to a worker* is derived from the unit rules (coverage), shown to HR and payroll
  because it affects pay, and is never presented as membership.
- **WPM-R104 Consultation obligations calendar.** A pure, configurable calculation: from a
  proposal in a workforce plan (a headcount reduction at an establishment, with a date), the
  jurisdiction's thresholds (configuration with a named source, not code) and any agreed longer
  procedure, say whether a collective duty **may** apply, the earliest date consultation must begin
  for a given earliest termination date, and the checklist of notices and information to give. It
  is a **suggestion with its evidence** (WPM-D50), never a verdict, and states that thresholds are
  the deployer's to confirm.
- **WPM-R105 Consultation register.** A record per proposal: the body consulted, the dates
  consultation began and ended, the topics, a reference to the documents shared, the outcome
  and any agreement. Roles, not names. Audited without text. It answers "what did we consult on,
  with whom and when?" for the people who must show it.
- **WPM-R106 Information pack for bargaining.** A monthly or on-request pack of **aggregates** for
  a recognised body about its unit: headcount and FTE by group, pay-band distribution, starters
  and leavers, vacancies, contingent share. Floors on group size, no names, no salaries of
  individuals; each disclosure is logged (who asked, what, when). It makes disclosure of
  information for bargaining routine and auditable.
- **WPM-R107 Facility time as an aggregate.** HR records time released for representative duties as
  **hours per body per period**, not per person, so a budget can be planned and agreed without
  creating a list of representatives. No time-entry kind exists for it.
- **WPM-R108 Decision gates.** A configurable list of changes that need consultation or consent in
  the deployer's jurisdiction or agreements (a redundancy proposal, a shift-pattern change, new
  monitoring, **a new or changed score**), each with the body to consult. When someone makes such a
  change in WPM, it points to the gate and asks for the consultation reference. It connects to the
  equality impact assessment for scores ([equality-impact/template.md](equality-impact/template.md),
  section 11).

## 5. What is deliberately left out

- **Individual membership, representative status, ballots, strike participation.** Industrial
  action affects pay and attendance; recording who took part is membership data by another route.
  A deployer's payroll handles it; WPM has no leave kind or marker for it.
- **Check-off.** Described in section 3; not modelled.
- **Grievance and disciplinary support.** In Great Britain a worker may be accompanied at certain
  hearings by a union representative or colleague *(unverified)*. WPM has no grievance module; if
  one is built, who accompanies must not be recorded as a union field.
- **Anything that scores, ranks or flags a person by attitude to unions**, and any input derived
  from union activity into any score (a hard rule in every assessment: see the template, section 3).

## 6. Sequencing

1. Decisions and the guard (WPM-D68, WPM-T178): the stance, and tests that fail if a column,
   export or label could reveal membership.
2. Reference data (R102, R103) and the obligations calendar (R104), because they are pure and
   useful alone.
3. The consultation register (R105) and decision gates (R108).
4. The information pack (R106) and facility time (R107).

## Sources

- [ILO Convention No. 98](https://tradefordecentwork.ilo.org/wp-content/uploads/2024/09/ILO-Convention-No.-98-Right-to-Organise-and-Collective-Bargaining-Convention.pdf)
- [Acas: getting voluntary trade union recognition](https://www.acas.org.uk/collective-conciliation/getting-voluntary-trade-union-recognition)
- [Osborne Clarke: trade union recognition reforms](https://www.osborneclarke.com/employment-law-reforms-horizon/trade-unions-industrial-action)
- [GOV.UK: redundancy consultation](https://www.gov.uk/redundant-your-rights/consultation) and [Lewis Silkin: collective redundancies](https://www.lewissilkin.com/insights/2026/04/07/collective-redundancies)
- [Directive 2002/14/EC](https://www.legislation.gov.uk/eudr/2002/14) and [Directive 98/59/EC](https://www.legislation.gov.uk/eudr/1998/59)
- [Germany: Works Constitution Act overview](https://www.businesslocationcenter.de/en/labor-market/employment-law-and-collective-contracts-system/german-works-council-constitution-act)
- [Blacklists Regulations 2010, explanatory note](https://www.legislation.gov.uk/ukdsi/2010/9780111491041/note/data.htm)
- [Industry guide to the GDPR and union data](https://news.industriall-europe.eu/documents/upload/2023/7/638242381242430635_GDPR_EN7_FINAL.pdf) and [a legal summary](https://legal-island.com/employment-law-hub/q---a-is-it-lawful-for-an-employer-to-monitor-communications-between-a-trade-union-representative-and-trade-union-members)
