# Record of processing activities (WPM-R92)

One row per purpose, in the form a record of processing needs. The controller's name,
contact and data protection officer, and each row's **lawful basis**, are the deployer's to
complete; the other columns describe what this software does.

> ⚠️ A template. Check it against your own processing and with your data protection
> officer. The lawful basis column is intentionally empty.

| # | Purpose | Data subjects | Categories of data | Special category? | Recipients | Retention kind | Lawful basis |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Recruiting and onboarding | Candidates, applicants | Name, email, source, stage, interview notes, consent date | No | Recruiters, hiring managers, HR | `recruitment` | deployer to complete |
| 2 | Employment administration | Workers | Identity URN, job title, department, dates, status, manager, working pattern | No | HR, the worker, managers (own team) | `employment` | deployer to complete |
| 3 | Payroll and benefits | Workers | Salary, payslips, deductions, benefit enrollment, expense claims | No (financial) | Payroll, the worker | `pay` | deployer to complete |
| 4 | Time, attendance and leave | Workers | Hours, overtime, leave kind and balance, shifts, on-call rota | Leave kind can imply health (sick leave) | Managers, HR, the worker | `time_and_leave` | deployer to complete |
| 5 | Performance and talent | Workers | Reviews, 360 feedback, goals, development plans, succession readiness | No | HR, the worker, reviewers | `performance` | deployer to complete |
| 6 | Learning and skills | Workers | Skills and proficiency, training, assessments, CPD, registrations, mentoring | No | HR, the worker, managers | `learning` | deployer to complete |
| 7 | Wellbeing, ergonomics and adjustments | Workers | Prompts acknowledged, workstation assessments, adjustment requests (barrier and change, never a diagnosis), anonymous pulse | Yes: health-adjacent | The worker and HR only; the pulse is anonymous and aggregate | `wellbeing` | deployer to complete (and an Article 9 condition) |
| 8 | Directory, cover and emergency contact | Workers; their emergency contacts | Name, contact preferences, backups, emergency contact details | Contacts are third parties | The worker, HR, colleagues (directory only) | `employment` | deployer to complete |
| 9 | Workforce planning | Workers (as aggregates) | Headcount history, plans, demand lines, forecast, gaps | No (aggregates, no names) | Planners, leadership | `planning` | deployer to complete |
| 10 | Joiners, leavers and handover | Workers | Dated checklists, last-day handover, notes | No | HR, managers | `employment` | deployer to complete |
| 11 | Security, audit and accountability | All users | Who did what and when; no sensitive values in audit entries | No | Auditors, administrators | none (not swept; deployer sets a rule) | deployer to complete |
| 12 | Subject rights | Workers | The subject-access export; the erasure ledger (a pid and a time) | No | The requester, HR | not swept | Legal obligation |

**Transfers outside the controller's country**: none made by the software; follow the
hosting, identity-provider and backup choices ([data-flow](data-flow.md)).
**Automated decisions with legal or similarly significant effect**: none. Forecasts and
gap analyses state their assumptions, and the levers they list are suggestions a person
decides (WPM-D28, D50); the tool never ranks or selects individuals for an outcome.
**Processors**: the hosting provider, the identity provider, the backup store, and any mail
relay: deployer to list, with their contracts.
**Technical and organizational measures**: see the [impact assessment](dpia.md) section 5
and the [assessment checklist](assessment-checklist.md).
