# For the Chief Security Officer

**The question you are likely asking:** how is this protected, how would we know if it were attacked or misused, and
what has nobody tested?

## What you get

| Concern | What exists | Read |
| --- | --- | --- |
| Threats | A threat model: assets, actors, trust boundaries, the control as built, and what is **not assessed** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Authentication | Sign-in on by default; production refuses to start with it off or with no token key source; enterprise directory sign-in (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Authorization | The policy decides by action only; a classified table decides need to know on every read route, and a source-scan test fails on an unclassified route | [auth.md](../../../../spec/auth.md) |
| Audit | An append-only, hash-chained trail with a verify endpoint; entries carry no reason text and no value | [audit.md](../../../../spec/audit.md) |
| Transport and browser | Security headers on every API response, an explicit CORS allow-list that fails boot when unset, `no-store` on `/api` | WPM-R73 in [production-readiness.md](../../../../spec/production-readiness.md) |
| Abuse | Rate limits per caller, stricter for erase, import, export and token routes | WPM-R74, same file |
| Insecure configuration | Boot refuses a plaintext database connection or plaintext Redis to another host, and a missing CORS origin | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Data at rest | Encrypted backups; the shared cache holds non-personal lookups only and fails open | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Supply chain | Licence and ban checks with `cargo deny`, vendored shared crates, a software bill of materials on release | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidents | A runbook of first actions; you add contacts and notification duties | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Self-assessment | A control checklist: met, partly met, not met, deployer to answer | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Known gaps, as recorded

- No independent penetration test has been done. The tests were written by the author.
- Actions are pinned by tag, not by digest, and images are not signed.
- Rate limits are per instance.
- The application uses whatever database role it is given; no least-privilege role set is provided.
- Token revocation and maximum token age are your identity provider's.
- The content manager's path loads a script from a public CDN without a pinned version or an integrity hash.
- Logs have not been checked for personal data, and the audit trail is not exported to a monitoring system.
- A managed Redis (cluster, sentinel, access-control users) has not been tried.

## What only people can do

Commission the independent penetration test, review the threat model for your environment, run a restore drill on
your platform, and set branch protection on the hosting platform. See the
[readiness assessment](../../../../spec/readiness-assessment.md), whose verdict is **not ready** for a high-assurance deployment.

See also the [Chief Risk Officer](../chief-risk-officer/index.md) and
[Chief Information Officer](../chief-information-officer/index.md) pages.

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Counts that change are in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software with
synthetic data. Nothing here is legal advice.
