# Backup and restore (WPM-R89)

> ⚠️ **Demo software.** These are the backup arrangements the repository can prove.
> A deployer sets their own targets, owns the storage, and rehearses the restore.

## Targets

| | What the scripts here achieve | What a deployer can reach by adding WAL archiving |
| --- | --- | --- |
| **Recovery point objective (RPO)**: the most data you can lose | **24 hours** with one nightly backup. Run it more often to shorten this; the erasure ledger (below) is exported every few minutes whatever the backup interval. | **15 minutes**, with continuous WAL archiving and point-in-time recovery. Documented below, **not scripted or tested here**. |
| **Recovery time objective (RTO)**: how long until the service is back | **4 hours** target for a database of this size (a seeded demo restores in well under a minute; the target allows for fetching the backup, restoring, replaying, and checking). The drill prints the time the restore step took, so the figure can be compared. | The same, plus the time to replay WAL. |

A target is a statement of intent. The only measured number is what the restore
drill prints each run, and the drill runs on a tiny database. **A deployer must
rehearse with a copy of their own data** and replace these figures with measured
ones.

## What is backed up

The whole PostgreSQL database: every record, the audit log, the erasure ledger and the
migration history. Nothing else holds state: the service keeps no files, and the UI
holds no data. **Secrets are not in the dump** (the token keys, `JWT_SECRET`, SMTP
credentials are environment): keep them in the deployer's own secret store.

## The scripts

```sh
# Nightly (cron or a timer). Output off the database host, encrypted at rest.
DATABASE_URL=postgres://… scripts/backup.sh /path/to/backups

# Every few minutes: copy the erasure ledger out of the database.
DATABASE_URL=postgres://… scripts/export-erasure-ledger.sh /path/off-host/erasure-ledger.csv

# After a loss: restore into an EMPTY database, replay erasures, then start.
WPM_BIN=/path/to/workforce-planning-management-service \
  scripts/restore.sh /path/to/backups/wpm-…dump postgres://…/new_database \
  /path/off-host/erasure-ledger.csv
```

- `backup.sh`: custom-format `pg_dump` (no owners or grants, so it restores under any
  role), owner-only permissions, a `.sha256` file, and **backups older than
  `BACKUP_RETENTION_DAYS` (default 35 calendar days) are deleted**, so that a person's
  erased data does not live on in old backups indefinitely.
- `restore.sh`: verifies the checksum, refuses a target that already has tables,
  restores, then **replays erasures** (it requires `WPM_BIN` for that, or an explicit
  `--skip-replay`, which prints a loud warning).
- `restore-drill.sh`: the whole loop on scratch databases; see below.

## Why erasures need replaying (WPM-D63)

A person erased on Tuesday must not reappear because the service was restored from
Monday's backup. The ledger records the pid and time of each erasure and is part of the
database, so a restore brings back the ledger *as of the backup*, which lacks Tuesday.
So:

1. `export-erasure-ledger.sh` copies the ledger to a file **outside** the database every
   few minutes (it holds only a pid and a time per erasure; the pid no longer identifies
   anyone).
2. After a restore, `task replay_erasures file:LEDGER.csv` adds those entries and erases
   each person again, using the same statements as the live erasure. It is idempotent and
   must run **before the service accepts traffic**.
3. With WAL archiving and point-in-time recovery to the moment of failure, the restored
   ledger is already complete and the file is a second check.

Backups are deleted after 35 calendar days by default; keep the exported ledger for at
least as long as the longest-kept backup.

## The restore drill

`scripts/restore-drill.sh` and the `restore-drill` CI job (every push). It:

1. migrates and seeds a scratch database;
2. backs it up and records a row checksum for **every table**;
3. starts the service, then terminates and **erases a worker after the backup**, and
   exports the ledger as the cron job would;
4. restores the backup into an empty database;
5. checks every restored table's checksum equals the backup's, that all migrations are
   recorded, and that the erased person is back (the problem being solved);
6. replays the exported ledger and checks the person is erased again;
7. starts the real service on the restored database and checks it answers, with the
   erased person a `404`.

**What it does not prove**: that the *full* request test suite passes on restored data
(that suite truncates its database when it boots, so it cannot run against a restore);
that a deployer's own backup storage, encryption and access controls work; WAL archiving
or point-in-time recovery; or restore time at the deployer's data size.

## WAL archiving (optional; not scripted here)

For an RPO under the nightly dump, enable continuous archiving on the database
(`archive_mode = on`, an `archive_command` to off-host storage, regular base backups) and
restore to a point in time with `recovery_target_time`. Test that restore before relying
on it; this repository does not.

## After a restore

1. Replay erasures (above), before traffic.
2. Check `GET /_posture` says `auth_enforced: true` in production.
3. Check `GET /_health`, then a read of a known record.
4. Run the scheduled tasks' next cycle (`snapshot_headcount` cannot be backfilled: a
   missed day is a gap in the headcount history, which the forecast reports as
   `insufficient_history` rather than guessing).
5. Record the incident and the time taken in the runbook's log.
