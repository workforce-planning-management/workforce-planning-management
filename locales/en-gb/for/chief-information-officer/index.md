# For the Chief Information Officer

**The question you are likely asking:** how does this fit our estate, who can sign in, where does the data live,
and how do we run and recover it?

## What you get

| Concern | What exists | Read |
| --- | --- | --- |
| Shape | A JSON API and a separate browser client; PostgreSQL holds the data | [architecture.md](../../../../spec/architecture.md) |
| Identity | WPM holds no identities. A person, a worker and an organisation are references to other services, resolved by name only and cached | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Sign-in | On by default. Enterprise directory sign-in (OIDC, Entra-compatible, directly or through Keycloak); roles and attributes decide access | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Deployment | A container build and a compose file for a demonstration; for production a TLS-terminating proxy, required settings, and boot checks that refuse an insecure setup | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Shared cache | Optional Redis for non-personal lookups only; every call fails open | `REDIS_URL` in the deployment guide (WPM-D78) |
| Backup and recovery | Encrypted backups, a restore script, and a restore drill that CI runs | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Upgrade | Migrations and an upgrade guide | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| When it goes wrong | A runbook of first actions | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Data movement | A data flow document | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Things to know before you plan

- The UI is **not containerised**; you choose an adapter for your platform.
- Rate limits are per instance. A shared store is yours to provide.
- The application uses whatever database role `DATABASE_URL` names; a least-privilege role set is not yet provided.
- Token revocation and maximum token age are your identity provider's.
- Managed Redis (cluster, sentinel, access-control users) has not been tried.
- No load measurement exists, and some views read an organisation into memory.
- The image build and compose file were written but should be tried first in your environment.

See [readiness-assessment.md](../../../../spec/readiness-assessment.md) for the full list of gaps and the verdict, and the
[Chief Technology Officer](../chief-technology-officer/index.md) page for the engineering view.

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Figures that change
(tests, migrations, tasks done) are not copied here; read them in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software
with synthetic data: this page says what it does and does not do, and is not a claim that it is fit for your
organisation. Nothing here is legal advice.
