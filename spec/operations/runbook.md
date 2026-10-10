# Runbook (WPM-R90)

First actions for the things that go wrong. Write down what you did and when; a deployer
adds contacts and escalation paths for their own organization.

> This is demo software. A real deployment needs its own on-call arrangements, severity
> definitions and notification duties (including to a regulator, where one applies).

## The service will not start

| Message | Meaning | Action |
| --- | --- | --- |
| `WPM_REQUIRE_AUTH is off in the production environment…` | Someone set it to `0` | Remove the override. Production does not run with sign-in off |
| `no token key source is configured in the production environment…` | No keys | Set `WPM_PASETO_KEYS_URL`/`WPM_PASETO_KEYS` or `WPM_KEYCLOAK_JWKS_URL` |
| `environment variable `WPM_CORS_ORIGIN` not found` | CORS allow-list not set | Set it to the UI's origin |
| `environment variable `JWT_SECRET` not found` | Required by the framework's config | Set any strong value; WPM does not use it for sign-in |
| Database connection errors | Wrong `DATABASE_URL`, or the database is down | Check the database first; the service does not start without it |

## Everyone gets `401`

Sign-in is on by default and a request without a valid token is refused, including for a
person who was working an hour ago. Check, in order: the key source is reachable (the key
set is fetched at boot and refreshed every hour; a failed refresh keeps the old keys);
the clock on the service and on the identity provider agree (tokens have an expiry); the
issuer and audience settings equal the token's `iss` and `aud`; and the provider did not
rotate keys faster than the refresh (restart to force a fetch).

## Some callers get `429`

The rate limiter named them by address. Behind a proxy without `WPM_TRUST_FORWARDED`, all
callers share one bucket: raise `WPM_RATE_LIMIT_PER_MINUTE`, or set the variable and make
the proxy overwrite `X-Forwarded-For`. A burst of `429` on erase, sweep or import routes
is the stricter class working as designed.

## `GET /_posture` says `auth_enforced: false` in production

That must never happen (production refuses to start that way), so first suspect that you
are not looking at production. If you are: take the service out of rotation, find who
changed the configuration, treat any access since as a possible incident.

## The database is full or slow

Connections: `DB_MAX_CONNECTIONS` against the server's limit. Size: the audit log and the
headcount snapshots only grow; the sweep removes soft-deleted rows past their horizon, not
the audit. Do not delete from the database by hand.

## Restoring

Follow [backup-and-restore.md](backup-and-restore.md): empty database, checksum, restore,
**replay erasures before traffic**, check, then start. Tell the people who use it the data
is as of the backup time.

## A suspected breach

1. Contain: take the service out of rotation; revoke the tokens' signing keys or disable
   the affected accounts at the identity provider.
2. Preserve: keep the logs and the audit log (`GET /api/audits/recent`), run `verify_audit_chain` and compare its
   head hash with the last one you recorded (a broken chain, or a head that has moved backwards, is itself a
   finding); take a backup of
   the database as it is, before changing anything.
3. Assess: what could the attacker read? The sensitivity map is in
   [auth.md](../auth.md); the reads of the highest tiers are audited.
4. Notify: the deployer's data protection officer decides duties and deadlines (in the UK
   a personal data breach that risks people's rights is reportable to the regulator within
   72 hours of becoming aware). **This repository cannot make that decision.**
5. Fix, then restore from a backup taken **before** the compromise if the data cannot be
   trusted, replaying erasures.

## A person asks for their data, or to be erased

- **Access**: `GET /api/workers/{pid}/subject-access` (HR). It names what it excludes.
- **Erasure**: `POST /api/workers/{pid}/erase`, refused while employment is open (the
  employment is the lawful basis). It anonymises; payroll rows stay under statutory
  retention. Remember the upstream identity services hold their own copies.
- Both are audited. An erasure is also written to the ledger, so a restore does not undo it.

## Log

Keep a dated log of incidents, restores and the time each took, and update the recovery
time figure in [backup-and-restore.md](backup-and-restore.md) with the measured one.
