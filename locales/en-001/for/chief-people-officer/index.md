# For the Chief People Officer

**Perspective:** does this support the whole employee lifecycle, and can I use it without
holding more about people than I should?

## What you get

| Stage | What WPM holds | Read |
| --- | --- | --- |
| Hiring | Requisitions, an applicant pipeline, a candidate pool, onboarding checklists | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| The record | One employment record per worker, the org chart, reporting lines, groups, transfers between organizations | [hr-core.md](../../../../spec/hr-core.md) |
| Self-service | A worker sees and updates their own details, requests flexible working, gives notice, and sees their time off | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Working life | Leave, time, shifts, benefits, wellbeing prompts, reasonable adjustments, ergonomic assessments, the anonymous pulse | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Development | Reviews, 360 appraisals, courses through the course registry, assessments, mentorships, development plans, succession, early careers | [talent-development.md](../../../../spec/talent-development.md) |
| Pay | Payroll runs, payslips, salary benchmarking, a national pay scale lookup, job levels | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Non-permanent staff | End dates by basis, extensions, status assessments, conversion plans decided by a person, funding beside the contract end | [engagements.md](../../../../spec/engagements.md) |
| Leaving | Resignation with a notice period, a last-day handover, retention per record kind, erasure | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## How it protects people

- **Need to know on reads.** A manager sees their own reports, a worker sees themselves, HR sees what HR needs.
  A signed-in stranger sees nothing about a colleague ([auth.md](../../../../spec/auth.md)).
- **No diagnosis is stored.** Sick leave records a kind, not a reason.
- **Special-category features are off by default.** Equality monitoring and workplace health requirements need a
  recorded lawful basis, are voluntary or held by the occupational health role, and show only groups above a floor.
  They are never an input to any score or decision about a person.
- **Scores advise.** Succession readiness, skill gaps and forecasts suggest; a person decides, and decisions such
  as converting a contractor are recorded as a person's decision, never an automatic outcome.
- **The subject can see and erase.** Subject access export and erasure cover the records above, with a ledger so
  a restore does not bring an erased person back.

## Limits

- Not yet built: a user interface for conversion plans and funding, the contingent trend, and the worker's
  "my details" page. Check [tasks.md](../../../../spec/tasks.md) for the current state.
- Translations are machine-assisted; a professional review is an open gate.
- Collective agreements and recognized bodies are research and proposal only ([worker-unions.md](../../../../spec/worker-unions.md)).
- Payroll tax is a stub.

<Footer>
