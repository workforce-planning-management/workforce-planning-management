# Operations pack, information governance pack, OIDC sign-in, and every page in every locale (WPM-R89–R94)

Status: **specified and built 2026-10-10** except where [tasks.md](tasks.md) (WPM-T146–T155)
says otherwise: the professional translation review (WPM-T155) is not done. Generic by design:
no real organization, government or health service is named; a deployer's own
names, regulators and targets go in the templates.

> ⚠️ **Demo software.** The packs below are templates a deployer adapts and has
> reviewed by their own data protection officer, security lead and clinical or
> legal advisers. Nothing here is legal advice, and nothing here certifies
> compliance. See [regulatory.md](regulatory.md).

## WPM-R89 — Backup and restore, tested automatically

*As an operator I can back the database up, restore it, and know the restore
works, because CI restores one every time.*

- **Targets**, written down in [operations/backup-and-restore.md](operations/backup-and-restore.md):
  a recovery point objective (RPO) and a recovery time objective (RTO), each in two
  tiers — what the **scripts in this repository achieve** (a nightly logical dump:
  RPO 24 hours) and what a deployer who adds **WAL archiving** (documented, not
  scripted here) can reach (RPO 15 minutes) — and an RTO of 4 hours for a database
  of this size. A target is a statement, not a measurement: the drill records the
  restore time it actually took.
- **Scripts**: `scripts/backup.sh` (a custom-format `pg_dump`, written with a
  timestamp, owner-only permissions, and a checksum file), `scripts/restore.sh`
  (restore into a named, empty database, then replay erasures — below), and
  `scripts/restore-drill.sh` (the whole loop, against a scratch database).
- **The drill** seeds a database, dumps it, restores into a second database, and
  checks: every migration is recorded as applied, every table's row count and a
  per-table checksum match, and the **real service started against the restored
  database** answers its read endpoints with the same counts. A CI job runs it on
  every push. It is a smoke test of the restored data, **not** the full request
  suite (that suite truncates its database on boot).
- **Retention of backups** is a stated number of calendar days (default 35), after
  which a backup is deleted, so that erased data does not live on indefinitely
  (see WPM-D53).

## WPM-R90 — Deployment, upgrade, and a runbook

- [operations/deployment.md](operations/deployment.md): the compose file, the
  required environment (each variable, whether it is required, its default), the
  container demo versus a real deployment, health and readiness probes, TLS and
  proxy settings, and the scheduled tasks (`snapshot_headcount`, `rota_reminders`,
  `pay_progression_reminders`, the retention sweep).
- [operations/upgrade-and-migration.md](operations/upgrade-and-migration.md):
  migrations are forward-only (the one-way ones say so), taken after a backup;
  order of rollout; what to check afterwards; how to roll back (restore).
- [operations/runbook.md](operations/runbook.md): symptoms and first actions
  (service will not start, `401` for everyone, `429`, database full, a restore, a
  suspected breach, an erasure request), with who to tell.

## WPM-R91 — A retention schedule per kind of record

*As a data protection officer I can set, and read back, how long each kind of
record is kept after it is deleted.*

- Every soft-deleting table belongs to exactly one **record kind** (recruitment,
  employment, pay, time and leave, performance, learning, wellbeing, planning and
  reference). Each kind has a default horizon in calendar days and a floor of 30
  calendar days (WPM-D22). A kind is overridden with
  `WPM_RETENTION_<KIND>_DAYS`; the older single `WPM_RETENTION_DAYS` still applies
  to every kind with no override of its own.
- `GET /api/retention/schedule` lists each kind, its tables, its horizon, and
  whether the horizon is the default or an override. The retention report and the
  sweep use each table's own horizon. Both state the horizon per table.
- The defaults are **starting points to adapt**, chosen to be cautious and
  explained in [governance/retention-schedule.md](governance/retention-schedule.md)
  with the reasoning a deployer must replace with their own legal basis.

## WPM-R92 — An information governance pack a buyer can adapt

All under `spec/governance/`, written so a deployer's information governance lead
can complete their own assessment from it:

- [threat-model.md](governance/threat-model.md): a STRIDE threat model with the control that answers each threat as
  built and the gap where there is none (added by the readiness work, WPM-T193).
- [dpia.md](governance/dpia.md): a data protection impact assessment template,
  completed for the demo as an example of the depth expected, with the parts only
  the deployer can answer marked **deployer to complete**.
- [data-flow.md](governance/data-flow.md): a data-flow diagram (Mermaid) and the
  table behind it: what enters, where it is stored, who can read it, where it goes.
- [record-of-processing.md](governance/record-of-processing.md): a record of
  processing activities, one row per purpose.
- [retention-schedule.md](governance/retention-schedule.md): R91.
- [assessment-checklist.md](governance/assessment-checklist.md): a digital
  technology assessment checklist, modelled on the structure of the published
  DTAC-style criteria (company, clinical or domain safety, data protection,
  technical security, interoperability, usability and accessibility), each question
  answered **met**, **partly met**, **not met** or **deployer to answer**, with the
  evidence (a file, a test, a requirement id) or the gap. An answer without
  evidence is not "met" (WPM-D61).
- [accessibility-statement.md](governance/accessibility-statement.md): a statement
  against WCAG 2.2 AA that says what was tested and what was not.
- [official-language-statement.md](governance/official-language-statement.md): the
  languages the interface is offered in, how they were produced, and how they were
  reviewed.

## WPM-R93 — Sign-in with Microsoft Entra ID, directly or through Keycloak

- **Direct**: the API's `keycloak` (OIDC) backend verifies an Entra v2 access token
  against the tenant's published keys. It now accepts Entra's top-level `roles`
  claim (app roles) as well as Keycloak's `realm_access.roles`, and can take the
  subject from `oid` (`WPM_OIDC_SUBJECT_CLAIM=oid`) because Entra's `sub` is
  different for every application. Settings:
  `WPM_KEYCLOAK_JWKS_URL=https://login.microsoftonline.com/<tenant>/discovery/v2.0/keys`,
  `WPM_KEYCLOAK_ISSUER=https://login.microsoftonline.com/<tenant>/v2.0`,
  `WPM_KEYCLOAK_AUDIENCE=<application (client) id>`.
- **Brokered**: Keycloak federates to Entra as an identity provider and issues the
  tokens; the existing Keycloak path is unchanged.
- [operations/entra-sign-in.md](operations/entra-sign-in.md) documents both: the
  app registration, app roles to ABAC personas (`wpm-hr`, `wpm-payroll`, `wpm-svc`,
  `wpm-admin`), the optional claims, and the limits (group claims are object ids,
  not names).
- **Tested** end to end against an Entra-compatible provider: a test starts a local
  OIDC provider (discovery document, key set, authorization-code token endpoint),
  signs in with the authorization-code exchange, and calls the API with the token it
  receives. The browser redirect itself is the identity broker's job (the
  authentication service or Keycloak) and is **not** exercised by this repository.

## WPM-R94 — Every page in every locale

- **No visible text outside the catalogue.** Every heading, label, button, placeholder, title,
  alt text, aria-label and message a page can show comes from
  `content/locales/<locale>/ui.json`, read with `t()`, `tf()` (with `{name}` values) or `tv()`
  (a closed-vocabulary token such as a status, under `values.<token>`); the per-route page titles
  are `titles.*`.
- A CI test (`tests/unit/untranslated.test.ts`) fails when a template holds literal visible text,
  when a non-English locale repeats the English string for a key not on a short, reasoned
  allow-list, and when a token in the server's closed vocabularies has no `values.*` entry. The
  existing parity test still pins that every locale holds the same keys.
- The translations are **AI-written until a person reviews them**. Each locale has a row in
  `content/locales/REVIEW.md` (reviewer, date, scope, outcome); a locale is "reviewed" only when
  its row says so. A professional review is a task for people, recorded as not done until it is.
- **Not covered**, and said so in the language statement: the server's own error messages, text a
  person typed, tokens the catalogue does not know yet (shown humanized, in English), dates and
  numbers (shown in ISO form), plural forms, and the third-party content manager.

## Decisions

- **WPM-D60 Retention is per kind, set by the deployer.** One horizon for every
  record cannot be right: a payslip and a recruitment note have different reasons
  to be kept. The defaults are cautious starting points, never legal advice; the
  floor stays 30 calendar days.
- **WPM-D61 An assessment answer cites evidence or says it has none.** "Met" needs
  a requirement, a test or a document; "partly met" says what is missing; a
  question only the deployer can answer says so. The pack must not look more
  compliant than the software is.
- **WPM-D62 The API verifies tokens; the broker signs people in.** WPM never holds
  a client secret or an authorization code. The OIDC test proves the verification
  and the persona mapping, not the interactive login.
- **WPM-D63 A restore does not undo an erasure** (this is the WPM-D53 proposed in
  [plan.md](plan.md), built here). Erasures are recorded in a ledger of the erased
  pid and the date only; after a restore the ledger is replayed before traffic is
  accepted.
- **WPM-D64 Translated text is content, and an unreviewed translation says so.**
  The repository never presents a machine translation as reviewed.
