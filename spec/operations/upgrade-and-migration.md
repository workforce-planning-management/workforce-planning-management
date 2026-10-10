# Upgrade and migration guide (WPM-R90)

## Principles

- **Migrations only go forward.** `down` exists for the early ones but is not a rollback
  plan; some migrations rewrite data (for example `m20261009_000050_generic_pay_scale_id`
  moves stored ids and its `down` does nothing). **The way back is a restore.**
- **Back up first, every time.** Run `scripts/backup.sh`, check it, then migrate.
- **Production does not migrate on start** (`auto_migrate: false`). Migrating is a step
  you take on purpose.
- Pin the version you deploy: a release tag and the image or commit it was built from
  (`.github/workflows/release.yml`).

## Steps

1. Read the release notes and `CHANGELOG.md` for **BREAKING** entries. For example, in
   the 2026-10 changes: sign-in enforcement became **on by default**; `/metrics.prom`
   now needs a token; `GET /api/retention` returns `horizons` per record kind in place of
   `horizon_days`; the pay scale id became `national-2026-27`.
2. Take and verify a backup: `scripts/backup.sh` (the `.sha256` file is checked on
   restore). Start the next restore drill on a copy if you can.
3. Stop traffic (or put the proxy in maintenance) if the release says migrations lock
   tables. None of the migrations so far take long on a database of this size, but test
   yours.
4. Deploy the new binary or image **without** starting traffic, then
   `workforce-planning-management-service db migrate`.
5. Start it. Check `GET /_health`, `GET /_posture` (`auth_enforced: true`), and a read.
6. Watch the logs for `WARN` lines: the service warns when sign-in is off and when it is
   on with no key source.
7. Keep the previous image and the backup until the new version has run a full
   scheduled-task cycle.

## Rolling back

There is no automatic rollback. If the new version is bad **before** it has written data
you need: stop it, restore the backup into an empty database with `scripts/restore.sh`
(replaying erasures), and start the old version. If it has been running a while, work out
what would be lost (the audit log shows what changed) before restoring.

## Migrations that change data

| Migration | What it does | Back out |
| --- | --- | --- |
| `…_000050_generic_pay_scale_id` | Moves stored pay-scale ids to `national-2026-27` | Restore |
| `…_000051_erasure_ledger` | Adds the erasure ledger | Drop the table (nothing depends on it before replay) |

New data-changing migrations add a row here in the change that introduces them.

## Compatibility

The API is versioned by header (`Accepts-version`); an unsupported version is a `406`.
Environment variables are renamed with a compatibility layer (`HCM_` to `WPM_`); a
deprecated name still works but logs. Check the changelog before relying on either.
