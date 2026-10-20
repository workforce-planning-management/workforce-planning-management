#!/usr/bin/env bash
# Prove the least-privilege database roles (WPM-R123, WPM-T199): migrate as wpm_migrator, then run the service's request
# tests as wpm_app, which can read and write rows and cannot change the schema or the audit trail.
#
#   ADMIN_DATABASE_URL=postgres://loco:loco@localhost:5432/postgres scripts/verify-db-roles.sh
#
# Needs a PostgreSQL superuser URL (to create roles and databases), psql and cargo. The service's request tests reset their
# data by recreating the schema, which the runtime role must not be able to do, so each test here gets its own clone of a
# migrated template database (fast) and runs against it as wpm_app with the schema left alone.
set -euo pipefail

ADMIN_URL="${ADMIN_DATABASE_URL:-postgres://loco:loco@localhost:5432/postgres}"
MIGRATOR_PW="${WPM_MIGRATOR_PASSWORD:-verify_migrator_pw}"
APP_PW="${WPM_APP_PASSWORD:-verify_app_pw}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"
SERVICE="$HERE/workforce-planning-management-service-with-rust"
TEMPLATE=wpm_roles_template
RUN_DB=wpm_roles_run

# host:port from the admin URL, for the role URLs
HOSTPORT="$(printf '%s' "$ADMIN_URL" | sed -E 's#^[a-z]+://[^@]*@([^/]+)/.*#\1#')"
admin() { psql "$ADMIN_URL" -v ON_ERROR_STOP=1 -qAt "$@"; }
in_db() { local db="$1"; shift; psql "${ADMIN_URL%/*}/$db" -v ON_ERROR_STOP=1 -qAt "$@"; }
as() { local role="$1" pw="$2" db="$3"; shift 3; psql "postgres://$role:$pw@$HOSTPORT/$db" -v ON_ERROR_STOP=1 -qAt "$@"; }
roles_sql() { in_db "$1" -v migrator_password="$MIGRATOR_PW" -v runtime_password="$APP_PW" -f "$SERVICE/postgres-roles/roles.sql" >/dev/null; }

echo "== template: roles, migrations as wpm_migrator, grants"
admin -c "DROP DATABASE IF EXISTS $TEMPLATE WITH (FORCE)" -c "DROP DATABASE IF EXISTS $RUN_DB WITH (FORCE)" -c "CREATE DATABASE $TEMPLATE"
roles_sql "$TEMPLATE"
(cd "$SERVICE" && LOCO_ENV=test WPM_TEST_FRESH_SCHEMA=false WPM_TEST_TRUNCATE=false \
  DATABASE_URL="postgres://wpm_migrator:$MIGRATOR_PW@$HOSTPORT/$TEMPLATE" cargo run -q -- db migrate >/dev/null 2>&1)
roles_sql "$TEMPLATE"
echo "   $(in_db "$TEMPLATE" -c 'select count(*) from seaql_migrations') migrations applied"

echo "== what wpm_app must NOT be able to do"
fail=0
refused() {
  local what="$1" sql="$2"
  if as wpm_app "$APP_PW" "$TEMPLATE" -c "$sql" >/dev/null 2>&1; then echo "   NOT REFUSED: $what"; fail=1; else echo "   refused: $what"; fi
}
refused "create a table" "CREATE TABLE intruder (id int)"
refused "alter a table" "ALTER TABLE workers ADD COLUMN intruder int"
refused "drop a table" "DROP TABLE workers"
refused "truncate a table" "TRUNCATE workers"
refused "drop the schema" "DROP SCHEMA public CASCADE"
refused "update the audit trail" "UPDATE audit_logs SET actor = 'x'"
refused "delete from the audit trail" "DELETE FROM audit_logs"
refused "truncate the audit trail" "TRUNCATE audit_logs"
refused "create a role" "CREATE ROLE intruder"
echo "== what wpm_app must be able to do"
as wpm_app "$APP_PW" "$TEMPLATE" -c "SELECT count(*) FROM workers" >/dev/null && echo "   read rows"
[ "$fail" = 0 ] || { echo "FAILED: the runtime role can do more than it should"; exit 1; }

echo "== the request tests, one fresh clone each, as wpm_app"
cd "$SERVICE"
cargo test --test mod --no-run -q 2>/dev/null
TESTS="$(cargo test --test mod -- --ignored --list 2>/dev/null | sed -n 's/^\(.*\): test$/\1/p')"
# These two deliberately tamper with the audit trail through SQL. Run as wpm_app they must be refused by the database.
TAMPER="requests::audit_chain::removing_an_entry_from_the_middle_is_found_and_the_tail_is_not requests::audit_chain::the_audit_trail_cannot_be_changed_and_a_tamper_is_found"
passed=0; refused_ok=0; bad=0
for t in $TESTS; do
  admin -c "DROP DATABASE IF EXISTS $RUN_DB WITH (FORCE)" -c "CREATE DATABASE $RUN_DB TEMPLATE $TEMPLATE"
  out="$(LOCO_ENV=test DB_CONNECT_TIMEOUT=10000 WPM_TEST_FRESH_SCHEMA=false WPM_TEST_TRUNCATE=false WPM_TEST_REDIS_URL="${WPM_TEST_REDIS_URL:-}" \
    DATABASE_URL="postgres://wpm_app:$APP_PW@$HOSTPORT/$RUN_DB" \
    cargo test --test mod -q -- --ignored --exact "$t" --test-threads=1 2>&1)" && rc=0 || rc=$?
  if [[ " $TAMPER " == *" $t "* ]]; then
    if [ "$rc" != 0 ] && grep -qE "must be owner of table audit_logs|permission denied for table audit_logs|append-only" <<<"$out"; then
      refused_ok=$((refused_ok+1)); echo "   refused by the database, as it should be: $t"
    else bad=$((bad+1)); echo "   UNEXPECTED (tamper test): $t"; echo "$out" | tail -5; fi
  elif [ "$rc" = 0 ]; then passed=$((passed+1))
  else bad=$((bad+1)); echo "   FAILED as wpm_app: $t"; echo "$out" | grep -E "panicked|42501|permission denied|must be owner" | head -3; fi
done
admin -c "DROP DATABASE IF EXISTS $RUN_DB WITH (FORCE)" -c "DROP DATABASE IF EXISTS $TEMPLATE WITH (FORCE)"
echo "== $passed passed, $refused_ok tamper tests refused by the database, $bad unexpected"
[ "$bad" = 0 ]
