# Glossary

| Term                | Meaning                                                                                          |
| ------------------- | ------------------------------------------------------------------------------------------------ |
| **360° / multi-rater** | Feedback on one subject from manager, peers, reports, and self; reported as group aggregates only |
| **ATS**             | Applicant Tracking System — the requisition/application pipeline                                 |
| **Adjustment (reasonable)** | A barrier-based change request (barrier / impact / change); no diagnosis required or storable |
| **Content locale** | A language-and-region the UI is written for, named `<language>-<region>` (`en-001`, `cy-001`, `en-gb`); its strings are content in `content/locales/<locale>/ui.json` ([locales](locales-for-global-sharing-with-svelte/index.md)) |
| **`-001` locale**  | A language's general-purpose locale (UN M.49 "world"): holds every UI key; `cy-001` is Welsh, `en-001` English, the source |
| **Regional locale** | A locale such as `en-gb` or `de-de` that holds only **overrides** of its language's `-001` locale and falls back to it |
| **Locale address** | The one URL prefix a locale is served under, its full code (`/cy-001/…`); a bare language (`/cy/`) is not a route, only a tag that matches its `-001` locale |
| **Accrual**         | Leave entitlement earned over time; v1 grants annual entitlements, accrual schedules are roadmap |
| **Pay scale**       | A banded public-sector pay and grading scale, transcribed from the government's yearly pay circular ([pay-scales.md](pay-scales.md)) |
| **Expense claim** | A worker's request to be repaid for money spent for work: dated, categorised items in one currency, decided by someone other than the claimant ([expense-claims.md](expense-claims.md)) |
| **Job level** | A rung on a published career ladder (e.g. Google's L3–L11); not a salary ([job-levels.md](job-levels.md)) |
| **Pay band / step** | A grade on a pay scale and one pay point within it (entry, intermediate, top), each with the years before eligibility to progress |
| **Benchmark**       | Recorded market pay data (min/median/max) for a job title                                        |
| **Calibration**     | HR moderation pass over submitted reviews before ratings are shared                              |
| **Candidate pool**  | Retained applicant profiles, consent-bounded                                                     |
| **Cognitive index** | An IQ-style per-scale reading (working memory, processing speed, …); no composite score exists   |
| **DSE**             | Display Screen Equipment — the UK workstation-assessment shape; WPM records equipment, never symptoms |
| **FTE**             | Full-Time Equivalent; `fte_percent` scales contracted hours and pay pro-rating                   |
| **Group floor**     | 360 disclosure rule: peer/report cells need ≥ 3 responses; manager/self disclose at 1            |
| **k-anonymity floor** | Pulse disclosure rule: any cell under k = 5 responses is suppressed, count withheld            |
| **Backup (cover)**  | A colleague a person names to cover when they are out; ranked, optionally dated (WPM-R41)         |
| **CEO dashboard**   | `/ceo`: six tiles on one screen, sized for an iPad (9th gen); no new numbers (WPM-R48)             |
| **Directory**       | The searchable, deliberately narrow list of who works where (WPM-R39)                            |
| <a id="employed"></a>**Employed (on a date)** | Hired on or before it and not terminated on or before it; `rules::metrics::is_employed_on` — the one definition every headcount uses |
| **Handover**        | What a leaver still holds on their last day, reassigned or closed with an audit trail (WPM-R46)   |
| <a id="metrics"></a>**Metrics layer** | The shared, named workforce counts and rates (headcount, starters, leavers, turnover, span of control, time-to-fill) — WPM-T44 |
| **Movement**        | A joiner or leaver record with a dated checklist (WPM-R45)                                       |
| **On-call rota**    | A rotation of workers where the duty passes every N calendar days; swaps and leave-aware skipping (WPM-R42) |
| **Priority (skill gap)** | Importance weight (critical 3, important 2, useful 1) × levels short (WPM-R43)              |
| **Read receipt**    | That a person read an announcement; the reader sees theirs, editors see a count (WPM-D35)         |
| **Unknown (skill)** | An undeclared skill: no shortfall, no priority — assess, do not train (WPM-D32)                   |
| **WPM**             | Workforce Planning Management — strategic workforce optimization on top of core HR               |
| **LMS**             | Learning Management System; here, enrollments over the family course registry                    |
| **Minor units**     | Money as integer cents/pence + ISO-4217 code (family posture)                                    |
| **Onboarding item** | One checklist obligation (contract, background check, …) gating activation                       |
| **Payslip**         | Per-employee output of a payroll run: gross, deduction lines, net                                |
| **Persona**         | An ABAC policy profile (employee / manager / HR / payroll), not a code role                      |
| **Pulse**           | A periodic anonymous 1–5 wellbeing survey; responses store no author (WPM-D20)                   |
| **Readiness**       | Succession rating: `ready_now` / `ready_1y` / `ready_2y`                                         |
| **Requisition**     | A funded job opening with headcount and a hiring pipeline                                        |
| **Retention horizon** | `WPM_RETENTION_DAYS` (default 365, floor 30) — soft-deletes past it are swept                  |
| **Right to work**   | Statutory employment-eligibility check recorded during onboarding                                |
| **Subject access**  | The one-document export of everything WPM holds for an employee, exclusions named                |
| **Tombstone URN**   | `person:00000000-…` — the valid-but-resolves-to-no-one ref an erased employee's identity becomes |
| **Working-time guardrail** | Advisory 48 h-average / 11 h-rest flag (UK WTR shape); flags, never blocks                |
