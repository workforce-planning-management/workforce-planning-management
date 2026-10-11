# For the Chief Legal Officer

**The question you are likely asking:** what legal duties does this software create or help with, and what must
we decide before real data goes in?

> Nothing here is legal advice. The specification states a posture, not a conclusion about any jurisdiction.

## What you get

| Topic | What exists | Read |
| --- | --- | --- |
| Posture | UK GDPR and employment-records posture; demo status; what production would additionally need | [regulatory.md](../../../../spec/regulatory.md) |
| Impact assessment | A data protection impact assessment template, completed for the demo, with the parts you must complete marked | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Record of processing and data flow | Adaptable documents | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Retention | A schedule per record kind, with a sweep for some | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Subject rights | Subject access export, erasure, and an erasure ledger | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Special-category data | Equality monitoring and workplace health requirements are off until you record a lawful basis, then bounded: voluntary, self-only or role-limited, aggregates above a floor, never used in a decision | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Statutory rights | Flexible working requests with a decision period and reasons you configure; resignation notice you configure | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Equality duties | An equality impact template and a register of every score | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Accessibility and language | An accessibility statement and an official-language statement | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Collective bodies | Research and proposal for recognised bodies and consultation, never individual membership | [worker-unions.md](../../../../spec/worker-unions.md) |
| Contractors | Engagement basis, the supplier, route and masked rate are recorded. Whether an engagement is lawful or correctly classified is not something the software decides | [engagements.md](../../../../spec/engagements.md) |
| Licences | Licence files, a dependency policy and a software bill of materials on release | [production-readiness.md](../../../../spec/production-readiness.md) |

## Defaults that are not law

Several settings are defaults, not legal advice: the resignation notice, the flexible working decision period and
appeal period, and the refusal reasons. The deployment guide says to **confirm them** for your jurisdiction.
Statutory calculations such as payroll tax are not implemented.

## Decisions that are yours

Lawful basis for each processing purpose, the data protection impact assessment and its sign-off, processor
agreements, the retention schedule you adopt, and whether to turn on any special-category feature.

See also [Chief People Officer](../chief-people-officer/index.md) and [Chief Risk Officer](../chief-risk-officer/index.md).

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Figures that change
(tests, migrations, tasks done) are not copied here; read them in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software
with synthetic data: this page says what it does and does not do, and is not a claim that it is fit for your
organisation. Nothing here is legal advice.
