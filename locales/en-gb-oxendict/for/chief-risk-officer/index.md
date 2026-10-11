# For the Chief Risk Officer

**The question you are likely asking:** what could go wrong with this system, what stops it, and what has nobody
checked?

## The honest starting point

The [readiness assessment](../../../../spec/readiness-assessment.md) of 2026-10-11 gives the verdict **not ready** for a
high-assurance deployment. It found one blocker by probing the running service: any signed-in person could read a
colleague's sick leave and its free-text reason. That is fixed and tested. The gates only people can close are
still open.

## What you get

| Concern | What exists | Read |
| --- | --- | --- |
| Threats at each boundary | A threat model: assets, actors, boundaries, the control as built, and what is **not assessed** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Unauthorized reads | A need-to-know guard that classifies every read route; a test fails on an unclassified one | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Tampering and accountability | An append-only, hash-chained audit trail with a verify endpoint; entries hold no reason text or value | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Sensitive data | No diagnosis stored; special-category features off by default; a small-group floor | [regulatory.md](../../../../spec/regulatory.md) |
| Insecure paths | Production refuses to start with sign-in off, no token key source, a plaintext database connection to another host, or a plaintext remote Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Loss of data | Encrypted backups, a restore drill run in CI, an erasure ledger | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Bias in scoring | An equality impact template, and a register of every score with a draft assessment | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| A control self-assessment | A checklist with each answer met, partly met, not met, or for the deployer to answer | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Open risks, as recorded

Supply-chain pinning and signing, load evidence, log checks for personal data, shared rate limits across
instances, a least-privilege database role set, and token revocation guidance are open items in the readiness
assessment, each with a task. The tests were written by the author; none is an independent review.

## What only people can do

An independent penetration test, an independent accessibility audit, the deployer's impact assessment signed off,
a restore drill on your platform, and approval of equality impact assessments. The repository states these
as gates and does not mark them done.

See also [Chief Information Officer](../chief-information-officer/index.md) and [Chief Legal Officer](../chief-legal-officer/index.md).

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Figures that change
(tests, migrations, tasks done) are not copied here; read them in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software
with synthetic data: this page says what it does and does not do, and is not a claim that it is fit for your
organization. Nothing here is legal advice.
