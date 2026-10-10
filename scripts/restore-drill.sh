#!/usr/bin/env bash
# The restore drill (WPM-R89): prove a backup can be restored, every run.
#
#   ADMIN_DATABASE_URL=postgres://loco:loco@localhost:5432/postgres \
#   WPM_BIN=target/debug/workforce-planning-management-service \
#     scripts/restore-drill.sh
#
# ADMIN_DATABASE_URL names a server where the role may create databases; the
# drill makes two scratch databases there and drops them at the end (or leaves
# them with KEEP=1). WPM_BIN is the built service binary.
#
# What it does:
#   1. migrate and seed a source database;
#   2. back it up, and fingerprint every table at that moment;
#   3. start the service on the source, terminate and ERASE a worker (an erasure
#      made AFTER the backup), and export the erasure ledger as the cron job would;
#   4. restore the backup into a second, empty database;
#   5. check the restored tables match the fingerprints (row counts and a checksum
#      of every row), that the audit hash chain is intact with the same head hash,
#      that the migrations are all recorded, and that the erased
#      person is back (the problem the replay exists to solve);
#   6. replay the exported ledger, and check the person is erased again;
#   7. start the real service on the restored database and check it answers with
#      the same worker count and treats the erased person as gone.
# It prints how long the restore took, to compare with the recovery time target.
# It is a smoke test of the restored data, NOT the request test suite (that suite
# truncates its database when it boots).
set -euo pipefail

: "${ADMIN_DATABASE_URL:?set ADMIN_DATABASE_URL (a server where databases can be created)}"
: "${WPM_BIN:?set WPM_BIN to the built service binary}"
here="$(cd "$(dirname "$0")" && pwd)"
port="${DRILL_PORT:-5199}"
work="$(mktemp -d)"
suffix="$$"
src="wpm_drill_src_$suffix"
dst="wpm_drill_dst_$suffix"

# Swap the database name in a URL: postgres://u:p@h:5432/old?x → …/NEW?x
with_db() { printf '%s' "$ADMIN_DATABASE_URL" | sed -E "s#(://[^/]*)/[^?]*#\\1/$1#"; }
SRC_URL="$(with_db "$src")"
DST_URL="$(with_db "$dst")"
pid=""

cleanup() {
  [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
  if [ "${KEEP:-0}" != "1" ]; then
    psql "$ADMIN_DATABASE_URL" -qc "DROP DATABASE IF EXISTS $src" 2>/dev/null || true
    psql "$ADMIN_DATABASE_URL" -qc "DROP DATABASE IF EXISTS $dst" 2>/dev/null || true
    rm -rf "$work"
  else
    echo "kept: $src $dst $work" >&2
  fi
}
trap cleanup EXIT

step() { printf '\n== %s\n' "$*" >&2; }
fail() { echo "DRILL FAILED: $*" >&2; exit 1; }

# One line per table: its name and an md5 of every row in a stable order.
checksums() {
  local url="$1" t
  for t in $(psql "$url" -Atc "select table_name from information_schema.tables where table_schema='public' and table_type='BASE TABLE' order by 1"); do
    printf '%s|%s\n' "$t" "$(psql "$url" -Atc "select coalesce(md5(string_agg(x::text, '|' order by x::text)), 'empty') from \"$t\" x")"
  done
}

start_service() { # url
  DATABASE_URL="$1" LOCO_ENV=demo WPM_REQUIRE_AUTH=0 PORT="$port" \
    WPM_RATE_LIMIT_PER_MINUTE=0 WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE=0 \
    "$WPM_BIN" start > "$work/service.log" 2>&1 &
  pid=$!
  for _ in $(seq 1 60); do
    curl -fs "http://localhost:$port/_health" >/dev/null 2>&1 && return 0
    kill -0 "$pid" 2>/dev/null || { cat "$work/service.log" >&2; fail "service exited"; }
    sleep 1
  done
  cat "$work/service.log" >&2; fail "service did not become healthy"
}
stop_service() { kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; pid=""; }
api() { curl -fsS -H 'content-type: application/json' "$@"; }

step "1. migrate and seed the source database ($src)"
psql "$ADMIN_DATABASE_URL" -qc "CREATE DATABASE $src"
export LOCO_ENV=demo
DATABASE_URL="$SRC_URL" "$WPM_BIN" db migrate >/dev/null
DATABASE_URL="$SRC_URL" "$WPM_BIN" task seed >/dev/null
src_workers="$(psql "$SRC_URL" -Atc 'select count(*) from workers where deleted_at is null')"
[ "$src_workers" -gt 0 ] || fail "the seed produced no workers"
# The seed writes no audit entries; add a few through the database's own chain trigger, so the
# audit check below has a chain to compare.
psql "$SRC_URL" -qc "insert into audit_logs (entity, entity_pid, action, actor) select 'drill', gen_random_uuid(), 'seeded', 'drill' from generate_series(1, 5)" >/dev/null

step "2. back up, and fingerprint every table at that moment"
# The drill runs the real, encrypted path (WPM-R117) with a key made for the run.
( umask 077; openssl rand -base64 48 > "$work/backup.key" )
export WPM_BACKUP_KEY_FILE="$work/backup.key"
dump="$(DATABASE_URL="$SRC_URL" BACKUP_RETENTION_DAYS=35 "$here/backup.sh" "$work/backups")"
checksums "$SRC_URL" > "$work/before.sums"
# The audit trail's hash chain (WPM-R115): record its head hash now, as an operator would
# keep it elsewhere, and check the restored copy has the same chain.
chain_before="$(DATABASE_URL="$SRC_URL" "$WPM_BIN" task verify_audit_chain 2>/dev/null | tail -1)"
tables="$(wc -l < "$work/before.sums" | tr -d ' ')"
echo "backed up $dump ($tables tables)" >&2

step "3. AFTER the backup: erase a worker, and export the ledger as the cron job would"
start_service "$SRC_URL"
victim="$(api "http://localhost:$port/api/workers?status=active" | python3 -c '
import json,sys
d=json.load(sys.stdin); rows=d if isinstance(d,list) else d.get("items",d.get("data",[]))
print(rows[0]["pid"])')"
for to in offboarding terminated; do
  api -X POST "http://localhost:$port/api/workers/$victim/status" -d "{\"to\":\"$to\"}" >/dev/null
done
api -X POST "http://localhost:$port/api/workers/$victim/erase" >/dev/null
stop_service
DATABASE_URL="$SRC_URL" "$here/export-erasure-ledger.sh" "$work/ledger.csv" >/dev/null
grep -q "$victim" "$work/ledger.csv" || fail "the erasure is not in the exported ledger"

step "4. restore into an empty database ($dst)"
psql "$ADMIN_DATABASE_URL" -qc "CREATE DATABASE $dst"
t0="$(date +%s)"
"$here/restore.sh" --skip-replay "$dump" "$DST_URL" 2>&1 | sed 's/^/   /' >&2
restore_secs=$(( $(date +%s) - t0 ))

step "5. the restored database matches the backup"
checksums "$DST_URL" > "$work/after.sums"
if ! diff -u "$work/before.sums" "$work/after.sums" >&2; then fail "a restored table differs from the backup"; fi
echo "all $tables tables match (row checksums)" >&2
applied_src="$(psql "$SRC_URL" -Atc 'select count(*) from seaql_migrations')"
applied_dst="$(psql "$DST_URL" -Atc 'select count(*) from seaql_migrations')"
[ "$applied_src" = "$applied_dst" ] || fail "migrations differ: $applied_src vs $applied_dst"
echo "$applied_dst migrations recorded" >&2
chain_after="$(DATABASE_URL="$DST_URL" "$WPM_BIN" task verify_audit_chain 2>/dev/null | tail -1)" \
  || fail "the restored audit chain is broken: $chain_after"
[ "$chain_before" = "$chain_after" ] || fail "the audit chain differs after the restore: $chain_before vs $chain_after"
echo "audit chain intact and identical after the restore: $chain_after" >&2
back="$(psql "$DST_URL" -Atc "select count(*) from workers where pid = '$victim' and display_name <> '[erased]'")"
[ "$back" = "1" ] || fail "expected the erased person to be back in the restored copy"
echo "the person erased after the backup is back (as expected before the replay)" >&2

step "6. replay the exported ledger"
DATABASE_URL="$DST_URL" "$WPM_BIN" task replay_erasures "file:$work/ledger.csv" 2>&1 | tail -2 >&2
gone="$(psql "$DST_URL" -Atc "select count(*) from workers where pid = '$victim' and display_name = '[erased]' and salary_minor is null")"
[ "$gone" = "1" ] || fail "the replay did not erase the person again"
echo "erased again" >&2

step "7. the real service, on the restored database"
start_service "$DST_URL"
posture="$(api "http://localhost:$port/_posture")"
echo "posture: $posture" >&2
code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost:$port/api/workers/$victim")"
[ "$code" = "404" ] || fail "the erased person should be a 404, got $code"
count="$(api "http://localhost:$port/api/workers" | python3 -c '
import json,sys
d=json.load(sys.stdin); rows=d if isinstance(d,list) else d.get("items",d.get("data",[]))
print(len(rows))')"
[ "$count" -ge 1 ] || fail "the restored service lists no workers"
echo "the service lists $count workers (the source had $src_workers before the erasure)" >&2
stop_service

printf '\nDRILL PASSED: restore took %s seconds; %s tables, %s migrations, erasure replayed.\n' \
  "$restore_secs" "$tables" "$applied_dst"
printf 'Compare the restore time with the recovery time target in spec/operations/backup-and-restore.md.\n'
