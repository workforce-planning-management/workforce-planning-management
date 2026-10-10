#!/usr/bin/env bash
# Restore a backup into an EMPTY database, then replay erasures (WPM-R89).
#
#   WPM_BIN=/path/to/workforce-planning-management-service \
#     scripts/restore.sh DUMP TARGET_DATABASE_URL [LEDGER_FILE]
#
# - Refuses a target that already has tables.
# - Verifies DUMP.sha256 when it is there.
# - Replays erasures before you start the service. WPM_BIN is required for that;
#   pass --skip-replay as the first argument to restore without it (the script
#   then says loudly that erased people may have come back).
# - LEDGER_FILE is an export from scripts/export-erasure-ledger.sh; without it
#   only the ledger inside the dump is replayed.
set -euo pipefail

skip=0
if [ "${1:-}" = "--skip-replay" ]; then skip=1; shift; fi
dump="${1:?usage: restore.sh [--skip-replay] DUMP TARGET_DATABASE_URL [LEDGER_FILE]}"
target="${2:?usage: restore.sh [--skip-replay] DUMP TARGET_DATABASE_URL [LEDGER_FILE]}"
ledger="${3:-}"

# Decide about the replay first, so a missing binary stops us before anything is restored.
if [ "$skip" != "1" ]; then
  : "${WPM_BIN:?set WPM_BIN to the service binary so erasures can be replayed (or pass --skip-replay)}"
fi

if [ -f "$dump.sha256" ]; then
  dir="$(dirname "$dump")"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$dir" && sha256sum --check "$(basename "$dump").sha256" >&2)
  else
    (cd "$dir" && shasum -a 256 --check "$(basename "$dump").sha256" >&2)
  fi
else
  echo "warning: no $dump.sha256; the dump's integrity is not checked" >&2
fi

tables="$(psql "$target" -Atc "select count(*) from information_schema.tables where table_schema = 'public'")"
if [ "$tables" != "0" ]; then
  echo "refusing: the target database already has $tables tables; restore into an empty database" >&2
  exit 3
fi

start="$(date +%s)"
pg_restore --no-owner --no-privileges --exit-on-error --dbname "$target" "$dump"
echo "restored in $(( $(date +%s) - start )) seconds" >&2

if [ "$skip" = "1" ]; then
  echo "WARNING: erasures were NOT replayed. Anyone erased after this backup is back." >&2
  echo "Run: DATABASE_URL=… \$WPM_BIN task replay_erasures [file:LEDGER_FILE] before starting the service." >&2
  exit 0
fi
args=()
[ -n "$ledger" ] && args+=("file:$ledger")
DATABASE_URL="$target" LOCO_ENV="${LOCO_ENV:-demo}" "$WPM_BIN" task replay_erasures ${args[@]+"${args[@]}"}
echo "erasures replayed" >&2
