# Governance, Risk, Compliance

**The questions you are likely asking:** who is accountable for what, what could go wrong and what stops it, and
what evidence can I show an assessor? This page gathers the governance, risk and compliance material in one place.
The role pages for the [Chief Risk Officer](../chief-risk-officer/index.md) and [Chief Legal Officer](../chief-legal-officer/index.md) cover the
same ground by role.

## Governance: who decides, and how it is recorded

| Topic | What exists | Read |
| --- | --- | --- |
| How the work is governed | Spec first: a requirement, a decision, a task, code and tests land together; each task says what was **not** verified | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Project governance | Governance, maintainers, an RFC process and a code-owners file | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Decisions about people | A person decides; scores advise. Conversions, resignations and flexible working requests are recorded as a person's decision | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Access and roles | Roles and attributes decide access; a classified table decides need to know on every read route | [auth.md](../../../../spec/auth.md) |
| AI-assisted authorship | A statement of how AI was used | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risk: what could go wrong

| Topic | What exists | Read |
| --- | --- | --- |
| Verdict | **Not ready** for a high-assurance deployment; one blocker (reads were not need-to-know) found by probing and fixed | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Threats | A threat model: assets, actors, boundaries, the control as built, and what is **not assessed** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Tamper evidence | An append-only, hash-chained audit trail with a verify endpoint; entries hold no reason text or value | [audit.md](../../../../spec/audit.md) |
| Fail-safe start | Production refuses to start with sign-in off, no token keys, a plaintext database connection to another host, or a plaintext remote Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Recovery | Encrypted backups, a restore drill that CI runs, an erasure ledger so a restore does not bring an erased person back | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidents | A runbook of first actions; the deployer adds contacts and escalation paths | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Bias in scoring | An equality impact template and a register of every score, each with a draft assessment | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Open risks | Supply-chain pinning and signing, load evidence, log checks for personal data, shared rate limits, a least-privilege database role set, token revocation guidance | [readiness-assessment.md](../../../../spec/readiness-assessment.md), section 3 |

## Compliance: what you can show an assessor

| Topic | What exists | Read |
| --- | --- | --- |
| Control self-assessment | A checklist where each answer is met, partly met, not met, or for the deployer to answer | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Data protection impact assessment | A template, completed for the demo, with the parts you must complete marked | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Record of processing | An adaptable record | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Data flow | Where personal data moves | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Retention | A schedule per record kind | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Subject rights | Subject access export and erasure | [regulatory.md](../../../../spec/regulatory.md) |
| Special-category data | Equality monitoring and workplace health requirements are off until a lawful basis is recorded, then bounded | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Accessibility and language | An accessibility statement and an official-language statement | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licenses and dependencies | License files, a dependency policy checked by `cargo deny`, and a software bill of materials on release | [production-readiness.md](../../../../spec/production-readiness.md) |

## What only people can do

These are gates, not code, and the repository does not mark them done: an independent penetration test, an
independent accessibility audit, your data protection impact assessment signed off by your officer, a restore drill
on your platform, a professional review of the translations, and your approval of the equality impact assessments.

## Limits of this page

The tables point at documents. Whether each document is true of your deployment is for you and your assessor.
The author's tests are not an independent review, and nothing here is legal advice or a certification.

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Counts that change are
in [implementation-status.md](../../../../spec/implementation-status.md), which a script generates. This is demonstration software
with synthetic data.
