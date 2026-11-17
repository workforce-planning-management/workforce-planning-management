# Deliberate departures

Where WPM departs **on purpose** from a statute, a standard, or a sibling application's contract: what the rule says, what WPM
does instead, why, and where that is specified (WPM-T519, from lesson 10 of
[the Rust-rewrite lessons](lessons/integration-hub-rewrite-it-in-rust/index.md)). Each row is a summary; the file and ids in
"Where" are the specification, and they win where this table disagrees.

> ⚠️ **Not legal advice.** Each row restates what the topic file says, in that file's words where possible. Nobody has
> reviewed these departures against the law or the standard's text, and the topic files say where their own sources are
> unconfirmed. The last column is for the accountable person: it is empty until someone confirms the row, on a date, by a
> decision.

Compiled on 2026-11-17 by the AI assistant, from a search of `spec/` and a check of the quoted wording against the files.
A departure that is only "not built yet" is left out unless the specification frames it as a choice.

## Statutes and regulations

| # | Rule | What it says | What WPM does | Why | Where | Confirmed by, date, decision |
| --- | --- | --- | --- | --- | --- | --- |
| S1 | UK National Minimum Wage | Pay at least the age band's floor, averaged over the pay reference period, with the law's own counting of hours and pay | An advisory check (`below`, `near`, `ok`, `unknown`, `none_by_law`) over recorded time and payslip gross; no accommodation offsets, tips, piece work, apprentices' rules or which pay counts; never blocks a payroll run | Arithmetic over data WPM already holds; WPM checks and warns, it never decides an entitlement | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-R218, WPM-D154, WPM-D156 | |
| S2 | Minimum wage age bands | The rate depends on the worker's age | The date of birth places a worker in a band at the end of the period; only the band and the outcome are stored, never an age | The date of birth is held for pay alone | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-D156, WPM-D114 | |
| S3 | No statutory minimum (Sweden, Denmark) | Floors come from collective agreements | `none_by_law` with a source, apart from "not configured"; one agreement floor a deployer enters; the agreement itself is never read | An unset value must never read as no rule; reading agreements would be a union-rules engine | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-D160, WPM-D161 | |
| S4 | US FLSA and state overtime | A premium is due over the statutory thresholds | The measure is minutes beyond the contracted day, named **contractual** overtime; statutory bands are parameters | So nobody reads it as the legal premium | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-R219, WPM-D157 | |
| S5 | FLSA regular rate | Bonuses and other pay count in the regular rate | The base hourly rate, and it says so; not modelled: bonuses, rest days, public holidays, a monthly band | A person is never paid a guess | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-D169 | |
| S6 | FLSA white-collar exemption | Exemption turns on a salary floor and a duties test | Exempt status is a fact a person records on the post, with an advisory salary-floor check | The duties test is a legal judgement, so it stays outside WPM | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-D157 | |
| S7 | UK Working Time Regulations, EU Working Time Directive | Weekly and rest limits, with opt-outs, sector carve-outs and compensatory rest | Breaches are flagged, never refused; opt-outs, carve-outs and compensatory rest are not modelled; the average counts recorded time, not only approved | Blocking would encode a legal judgement WPM cannot make; a safety signal must not wait for an approval | [design.md](design.md) WPM-D19; [workforce-management.md](workforce-management.md); WPM-R27 | |
| S8 | Income tax and national insurance | The jurisdiction's full calculation | A stub of one allowance and one rate in one currency, with stub deductions; a run in another currency is refused | Payroll here is a derivation, not jurisdiction-complete tax law (a documented limitation) | [payroll-compensation.md](payroll-compensation.md), [worldwide.md](worldwide.md), [scope.md](scope.md); WPM-R13, WPM-D5 | |
| S9 | Employer social contributions (FUTA, EI, employer NI) | Rates, wage bases, credits and reductions | Labelled stub cost lines, never deducted, remitted or filed; no credit-reduction logic, no in-year rate change, no per-period cap | So workforce cost stops understating the real cost, without WPM computing or filing the tax | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-R223, WPM-D159, WPM-D170 | |
| S10 | Employer rates set by class (Korea, China, Spain, France, Germany, Japan) | The rate follows an industry, risk or establishment class | The deployer sets the class; WPM never picks one from the organization's name; no class costs nothing, and the payslip says so | An unknown is never a zero, and the nearest rate is never used | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-D162, WPM-D170 | |
| S11 | Workers' compensation, UK employers' liability insurance | Carry cover, display the certificate, handle claims | Coverage evidence with an expiry reminder; a display or retention duty is a reminder, not a copy of the document; claims and benefits are not WPM's | Left to the insurer | [statutory-pay-protections.md](statutory-pay-protections.md); WPM-R220, WPM-D158 | |
| S12 | Statutory injury reports (RIDDOR, OSHA 29 CFR 1904 and others) | The record describes what happened, and most name the injury or the body part | A row that proves a report was made: class, counts, recipient, reference; no text column, no injury, no body part | Health is special-category data; WPM's row holds the proof and a pointer | [injury-and-separation-duties.md](injury-and-separation-duties.md); WPM-R221, WPM-D179, WPM-D158 | |
| S13 | Separation documents (US state notices, P45, Arbeitsbescheinigung, certificado de empresa, 離職証明書 and others) | Country forms, timings and reason rules | A common core and a CSV export; WPM generates and submits no form, and the pay summary is a pointer, never an amount | The forms vary too widely; they stay outside WPM | [injury-and-separation-duties.md](injury-and-separation-duties.md); WPM-R222 | |
| S14 | Attachment of earnings, court orders | Statutory priority and protected earnings | Not done | A payroll system's job | [staff-records-and-pay.md](staff-records-and-pay.md); WPM-R175 | |
| S15 | Trade union check-off | Permitted with the worker's written authorisation | Not modelled: no subscription through payroll and no membership stored | A check-off deduction would record membership; a list of members is the artefact that was misused | [worker-unions.md](worker-unions.md), [staff-records-and-pay.md](staff-records-and-pay.md); WPM-D68, WPM-D119 | |
| S16 | EU Pay Transparency Directive 2023/970, Art. 3 and 9 | Pay includes any other consideration, in cash or in kind | Pay is annual full-time basic salary; indicators (a) to (f) are `unknown` (`variable_not_recorded`) and only (g) in basic pay is shown | WPM does not hold what it is not given; a zero would invent a gap | [pay-transparency.md](pay-transparency.md); WPM-R144, WPM-D99 | |
| S17 | Pay Transparency, Art. 9 | Report the indicators | A cell under the floor (default 10, at least 5) is withheld, even where the law asks for a figure | The Directive leaves the protection of personal data in place | [pay-transparency.md](pay-transparency.md); WPM-D99 | |
| S18 | Pay Transparency, Art. 10 joint pay assessment | Triggered by a category's average gap of 5% or more, unjustified and unremedied | Uses the category's gap in basic pay alone, and does not judge whether a justification is objective; a person records it | Variable pay is not held; the judgement belongs to people and their representatives | [pay-transparency.md](pay-transparency.md); WPM-R146 | |
| S19 | EU AI Act, Art. 26 deployer duties | Oversight, logs kept at least six months, workers informed, explanations | A register that records the duties and does not discharge them; WPM does not classify a system as high-risk or certify conformity | Classification is the deployer's, with legal advice; a filled-in register is not evidence of compliance | [regulatory.md](regulatory.md); WPM-R148, WPM-D103 | |
| S20 | NHS Agenda for Change pay circular | Bank workers are paid the same scales; progression follows the years on a step and the employer's conditions | No bank-worker concept; eligibility is the years on the step alone; screens say "eligible", never "will be paid" | A reminder promises nothing | [pay-scales.md](pay-scales.md); WPM-R51, WPM-R54, WPM-D41 | |

## Standards

| # | Standard | What it says | What WPM does | Why | Where | Confirmed by, date, decision |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | SCIM 2.0 (RFC 7643, 7644) with the Enterprise User extension | A full `User`, with name parts, emails, phones, addresses and `costCenter` | `userName` is the employee number; no `costCenter`, no name parts, emails, phone numbers or addresses | Provisioning sends employment facts only; contact details stay with the person service | [integrations.md](integrations.md); WPM-R150, WPM-D102, WPM-D106 | |
| T2 | RFC 9457 problem details | `type` names the problem type | `type` is always `about:blank`; the extension `code` is the stable reason; `/mcp` keeps JSON-RPC's own errors | One shape a client handles in one place; JSON-RPC has its own error shape | [production-readiness.md](production-readiness.md), [rust-axum-stack.md](rust-axum-stack.md); WPM-R238, WPM-D186 | |
| T3 | MCP Streamable HTTP (2025-11-25), RFC 9728 | Sessions, server-sent events, resources, prompts, sampling | JSON replies only, no session, no server-sent events (`GET` and `DELETE` answer `405`), tools only; WPM is not an authorization server | An agent sees what its caller sees, through tools over the `GET` routes | [integrations.md](integrations.md); WPM-R159, WPM-D101, WPM-D110 | |
| T4 | ESCO | Skills relevant to an occupation, essential or optional, with no proficiency scale | A pinned, optional reference, never a second skills model; the planner sets the level; essential is `important`, optional `useful`, never `critical`; linked by URI or exact label, never fuzzy | ESCO says a skill is relevant, not that an employer needs it; an automatic match is a guess about a person's job | [esco/index.md](esco/index.md) | |
| T5 | ESCO's download | Alternative labels, labels in 28 languages, the qualifications pillar | Alternative labels ignored, qualifications and EQF out of scope, other languages shown as text and never copied as truth | No need for them yet | [esco/index.md](esco/index.md) | |
| T6 | SFIA and SFIAplus | Seven levels of responsibility; licensed content that may not be redistributed | No SFIA content ships; a deployer imports it under their own licence; levels compared through a `linear` mapping; no SFIA assessment or membership data | The free licence excludes distribution, and this repository is public | [sfia.md](sfia.md); WPM-R109 to WPM-R113, WPM-D69, WPM-D70 | |
| T7 | UK Government Digital and Data Profession Capability Framework | Four levels, with no grading of importance | Levels 1 to 4 map onto WPM's 1 to 5 by `identity` by default; importance defaults to `important`; required levels are a draft for a person to edit | `linear` is stricter and surprising, so not the default; the source level is kept, so the choice is reversible | [uk-gdad-pcf/index.md](uk-gdad-pcf/index.md) | |
| T8 | ISO 30414:2025, ESRS S1 | Each standard's own metric definitions and datapoints | A mapping, not a disclosure: WPM's definitions under the standards' names, gaps shown as gaps; not compared with the 2025 text; a legal minimum wage is not ESRS S1's adequate wage | Standards are mapped, not copied (their licences); no invented numbers | [human-capital-reporting.md](human-capital-reporting.md); WPM-R147, WPM-D100 | |
| T9 | NHS England Workforce Disability Equality Standard | The published metrics | Mapped, not copied; metrics that need data WPM lacks (the staff survey, the board) are `unknown` | As T8 | [disability-and-adjustments.md](disability-and-adjustments.md); WPM-D100 | |
| T10 | Hijri calendar holidays | The day is announced or sighted | The tabular Hijri calendar, an arithmetic approximation that can be a day or two out; the deployer adds the announced day | Needs no table and no sighting | [worldwide.md](worldwide.md) | |

## Sibling applications' contracts

| # | Contract | What it says | What WPM does | Why | Where | Confirmed by, date, decision |
| --- | --- | --- | --- | --- | --- | --- |
| B1 | CRM's overlap check and teams import (CRM-D16) | CRM calls with one HR token the deployer holds, for every caller | Accepted as a recorded choice against WPM-D135 (each inbound call under its caller's own token); WPM cannot tell which CRM user asked | Confirmed by the product owner on 2026-10-31 | [cross-app-bridges.md](cross-app-bridges.md); WPM-D135, WPM-R208 | |
| B2 | CRM's active-workers import | Pages with `&offset=` | `GET /api/workers` refuses `?offset=` with `422` and pages by signed cursor; CRM's change, not made here | The owner chose it on 2026-11-07; a total and an offset leak hidden rows and allow costly scans | [cross-app-bridges.md](cross-app-bridges.md), [production-readiness.md](production-readiness.md); WPM-R241, WPM-D189 | |
| B3 | PPM's capacity model | Pools as text tags, FTE as a float, months as `YYYY-MM` | A pool link is a `peer_pool_ref` a person sets, never a name match; demand is PPM's and supply WPM's, never added together; a silent peer is `unknown` | A name is not an identity, and a wrong match would move real numbers | [cross-app-bridges.md](cross-app-bridges.md); WPM-R213, WPM-D138 to WPM-D141 | |
| B4 | The family contract, point 3 (PPM demand) | PPM sets and withdraws figures | A claim counts only after a WPM start decision; a withdrawal closes the claim and leaves the decision | A person's recorded decision is not the peer's to change | [cross-app-bridges.md](cross-app-bridges.md); WPM-R215, WPM-D146, WPM-D148 | |
| B5 | The family contract, point 4 (people impact) | PPM asks for impact by programme | WPM holds no end date for a programme, so engagements ending are `unknown` unless the caller passes `ends_on`; counts only, floor of 10 | No invented data, no differencing | [cross-app-bridges.md](cross-app-bridges.md); WPM-R211 | |
| B6 | PPM's what-if and scheduling | Not in the family contract | WPM builds its own half of the what-if, marked unverified, and stores no tasks, durations, dependencies or schedules | Coding against a guessed schema was rejected; a schedule kept in two services drifts | [cross-app-bridges.md](cross-app-bridges.md), [design.md](design.md); WPM-D55, WPM-D136, WPM-R209 | |

## To confirm: not clearly deliberate

The search found these, and the files do not say plainly whether each is a departure chosen on purpose. Each needs the
accountable person to say yes (and it moves into a table above) or no.

- **GDPR erasure and the injury record:** erasure takes the worker reference and keeps the row, which proves a duty was met.
  The file says the choice has had no legal review ([regulatory.md](regulatory.md), WPM-D179).
- **Pay Transparency, Art. 9(1)(g):** the per-category gap is the mean, with no median per category, and the Directive's text
  for the variable-pay basis was not read ([pay-transparency.md](pay-transparency.md), WPM-R144).
- **Pay Transparency, Art. 5:** the pay range is required before a requisition opens (switched off by
  `WPM_REQUIRE_PAY_RANGE`), because WPM publishes no advert itself ([pay-transparency.md](pay-transparency.md), WPM-R145).
- **SCIM operations:** no `PATCH` and no groups, and deactivation follows the termination date, which may be a day before the
  last day worked ([integrations.md](integrations.md)).
- **Open Badges 3.0:** compact JWS only, not the Data Integrity proof form
  ([skills-and-training.md](skills-and-training.md), WPM-R157, WPM-D109).
- **ISO 3166-1:** a deployer may add a code the standard lacks, such as `XK` ([worldwide.md](worldwide.md), WPM-D152).
- **PPM's skills route:** PPM expects `GET /api/workers/{id}/skills` as tags, and WPM has no such route
  ([cross-app-bridges.md](cross-app-bridges.md)).
- **The GB fit note after seven calendar days:** kept as configuration, not shipped as a default
  ([absence-and-line-managers.md](absence-and-line-managers.md), WPM-R179).
