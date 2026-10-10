#!/usr/bin/env bash
# Back the database up (WPM-R89): a custom-format pg_dump, owner-only, with a
# checksum, and old backups pruned.
#
#   DATABASE_URL=postgres://… scripts/backup.sh [OUTPUT_DIR]
#
# OUTPUT_DIR defaults to ./backups. Keep the output off the host that holds the
# database, and encrypt it at rest (see spec/operations/backup-and-restore.md).
# BACKUP_RETENTION_DAYS (default 35 calendar days) is how long a backup is kept:
# older ones matching wpm-*.dump in OUTPUT_DIR are deleted, so erased data does
# not live on in backups indefinitely (WPM-D63).
set -euo pipefail

: "${DATABASE_URL:?set DATABASE_URL to the database to back up}"
out="${1:-./backups}"
keep="${BACKUP_RETENTION_DAYS:-35}"
case "$keep" in '' | *[!0-9]*) echo "BACKUP_RETENTION_DAYS must be a whole number of calendar days" >&2; exit 2 ;; esac

mkdir -p "$out"
umask 077
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
name="$(psql "$DATABASE_URL" -Atc 'select current_database()')"
file="$out/wpm-${name}-${stamp}.dump"

pg_dump --format=custom --no-owner --no-privileges --file "$file" "$DATABASE_URL"
chmod 600 "$file"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$out" && sha256sum "$(basename "$file")" > "$(basename "$file").sha256")
else
  (cd "$out" && shasum -a 256 "$(basename "$file")" > "$(basename "$file").sha256")
fi
chmod 600 "$file.sha256"

# Prune: only files this script names, and only older than the retention.
find "$out" -maxdepth 1 -type f \( -name 'wpm-*.dump' -o -name 'wpm-*.dump.sha256' \) \
  -mtime "+$keep" -print -delete | sed 's/^/pruned: /' >&2 || true

echo "$file"
