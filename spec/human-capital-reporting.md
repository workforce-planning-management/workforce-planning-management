# Human capital reporting

A dated, read-only report built from measures WPM already holds, named so that it can be laid beside two reporting
frameworks: **ISO 30414** (human capital reporting) and **ESRS S1** (the EU sustainability standard for a company's own
workforce) (WPM-R147, WPM-D100). Proposed in [plan.md](plan.md) section N; the standards' status was checked in the spec round
(WPM-T268, 2026-10-24).

> ⚠️ **A mapping, not a disclosure.** WPM says which of its measures corresponds to which named metric and how far it
> corresponds. It does not produce a sustainability statement, an audit-ready ISO report or a CSRD filing, and it holds none
> of either standard's text.

## Where the two frameworks stand

- **ISO 30414:2025** is the second edition, published 24 August 2025. ISO says it adds auditable *requirements* beside the
  earlier recommendations, a baseline set of required metrics (ISO states 14), and new areas (human rights, labour relations,
  productivity, ethics, wellbeing). The standard is paywalled; the metric lists below are WPM's own names and **were not
  compared with the 2025 edition's text** (a deployer holding the licence confirms the mapping; WPM-D100).
- **ESRS S1 was revised and simplified** by Commission Delegated Regulation (EU) 2026/1563 of 3 July 2026, which applies to
  financial years starting on or after 1 January 2027 (per the Official Journal publication reported on 21 September 2026; the
  regulation text itself was read only as the EFRAG knowledge hub's rendering). **The disclosure requirement numbers changed**
  from the 2023 set, so a mapping that quotes a number must say which set.

  | Topic | 2023 set (Delegated Regulation 2023/2772) | Revised set (2026/1563) |
  | --- | --- | --- |
  | Characteristics of employees | S1-6 | **S1-5** |
  | Characteristics of non-employees | S1-7 | **S1-6** |
  | Collective bargaining coverage and social dialogue | S1-8 | **S1-7** |
  | Training and skills development | S1-13 | **S1-12** |
  | Health and safety | S1-14 | **S1-13** |
  | Remuneration metrics (including the pay gap) | S1-16 | **S1-15** |

  The 2023 numbers are from the standard as most guides quote it and were not re-read; the revised numbers and titles are from
  the EFRAG knowledge hub's list of the delegated act's sixteen S1 requirements. Whether a datapoint became optional or
  phased in the revision was **not** confirmed.

## WPM-R147 — The report

- **What it is.** `GET /api/reports/human-capital?organization=&as_of=` (WPM-T273): one dated report, read-only, for HR
  and for the roles the organization has configured. Nothing in it is typed in by hand; each figure is read from a measure
  WPM already works out, and the report is not stored as a record of anything (a frozen pack, like the oversight packs, is a
  later choice).
- **Every figure carries** its metric name, its definition (one sentence of WPM's own), its **source** (the measure and the
  table it reads), the **period** and the **exclusions** (for example a contractor not on payroll, or a worker who declared no
  sex). A figure WPM cannot compute is **`unknown`, never zero** (WPM-D100, WPM-D46).
- **The mapping** (WPM's measures against the two frameworks; "gap" means WPM cannot give it today):

  | WPM measure | ISO 30414 area (2018 names; unconfirmed for 2025) | ESRS S1 topic |
  | --- | --- | --- |
  | Headcount and FTE by contract type ([workforce-measures.md](workforce-measures.md)) | Workforce | Characteristics of employees |
  | Departures, turnover (leavers over the period) | Recruitment, mobility and turnover | Characteristics of employees |
  | Collective-agreement coverage ([worker-unions.md](worker-unions.md)) | Organizational culture, and the workforce | Collective bargaining coverage |
  | Training hours and completion (training enrolments, CPD) | Skills and capabilities | Training and skills development |
  | Time to fill (requisition opened to application hired) | Recruitment, mobility and turnover | gap (not an ESRS S1 datapoint) |
  | Workforce cost (payroll cost, where visible) | Costs | gap (not an ESRS S1 datapoint) |
  | Pay gap indicators ([pay-transparency.md](pay-transparency.md)) | Diversity | Remuneration metrics |
  | Succession coverage (critical roles with a ready successor) | Succession planning | gap |
  | Health and safety, work-life balance, adequate wages, social protection, disability, discrimination incidents | | **gap**: WPM holds no such record, or is built not to |

- **A gap is shown as a gap.** The report lists the named metrics WPM does not cover, so an omission is visible rather than silent.
- **Aggregates only.** Every figure is a count or a rate over a group at or above the floor (WPM-D32); no table in the report
  names a person, and a cell under the floor is withheld. Built (WPM-T273): a count of 1 to 9 is withheld and **zero is shown** (it
  says nobody, which names no one); the FTE of a withheld group is withheld too, since it would give the count back; a rate is
  withheld when the count it is made from is, or when the mean headcount is under the floor.
- **Built figures** (WPM-T273): headcount, and headcount and FTE (in hundredths) by employment type; starters, leavers, turnover
  rate (basis points); time to fill, median and mean (whole calendar days, halves up); training completion; succession coverage
  (criticality 4 or 5, a successor ready now, plans whose incumbent is employed in the organization). **Unknown with a reason:**
  training hours (no source), collective agreement coverage (no record yet), and workforce cost and the pay gap (behind a stricter
  read; the pay gap has `/api/pay-transparency/gap-report`).

## Design decisions

- **WPM-D100 — Standards are mapped, not copied.** A figure carries the standard's metric name or requirement identifier and
  WPM's own definition. The repository holds no ISO text and no copy of the ESRS beyond identifiers and titles, as with SFIA
  ([sfia.md](sfia.md)). A deployer with the licence reads its own copy. Rejected: shipping a metric dictionary because it is
  convenient (a licence problem), and computing a figure WPM has no data for so that the table looks complete (an invented number).

## What was checked, and what was not

Checked: that ISO 30414 has a 2025 second edition published in August 2025, from ISO's committee page and trade press; that the
revised ESRS was adopted as Delegated Regulation (EU) 2026/1563 and applies from 1 January 2027; and the revised S1 requirement
numbers and titles, from the EFRAG knowledge hub. **Not checked:** ISO's own metric titles (paywalled); the 2023 S1 numbers
against the Official Journal; the revised standard's datapoint-level detail, optionality and phase-in; and whether the report's
definitions match what an auditor would accept.

## Sources

- ISO, [ISO 30414:2025 announcement](https://committee.iso.org/sites/tc260/home/news/content-left-area/news-and-updates/iso-30414-2025-strengthening-hum.html).
- Commission Delegated Regulation (EU) 2026/1563, [EUR-Lex](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32026R1563).
- EFRAG, [ESRS S1 (simplified)](https://knowledgehub.efrag.org/eng/interactive/simplified-esrs/esrs-s1/delegated-act).
