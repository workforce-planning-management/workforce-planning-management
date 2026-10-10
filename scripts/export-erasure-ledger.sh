#!/usr/bin/env bash
# Copy the erasure ledger out of the database (WPM-R89, WPM-D63).
#
#   DATABASE_URL=postgres://… scripts/export-erasure-ledger.sh FILE
#
# The ledger holds only a pid and a time per erasure. A restore brings back the
# ledger as of the backup, so an erasure made after the backup is lost with the
# database unless the ledger is kept elsewhere. Run this every few minutes (cron
# or a timer), write FILE somewhere off the database host, and give it to
# `replay_erasures file:FILE` after a restore. The write is atomic.
set -euo pipefail

: "${DATABASE_URL:?set DATABASE_URL}"
file="${1:?usage: export-erasure-ledger.sh FILE}"
umask 077
tmp="$(mktemp "${file}.XXXXXX")"
trap 'rm -f "$tmp"' EXIT
{
  echo "# erasure ledger exported $(date -u +%Y-%m-%dT%H:%M:%SZ): pid,time"
  psql "$DATABASE_URL" -Atc "COPY (
      SELECT worker_pid,
             to_char(erased_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"+00:00\"')
      FROM erasure_ledger ORDER BY erased_at
    ) TO STDOUT WITH (FORMAT csv)"
} > "$tmp"
mv "$tmp" "$file"
trap - EXIT
echo "$file"
