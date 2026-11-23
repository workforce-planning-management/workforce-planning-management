#!/bin/sh
# A nextest wrapper (WPM-T646, a prototype): run one test against its own database, cloned from a migrated template, so
# tests can run in parallel. Needs the `postgres` and a migrated `${WPM_TEMPLATE_DB:-wpm_template}` database beside it;
# DATABASE_URL names any database on the server (its path is replaced). The clone is dropped afterwards, pass or fail.
set -u
base=${DATABASE_URL%/*}
template=${WPM_TEMPLATE_DB:-wpm_template}
name="wpm_t_$$_$(date +%s)"
psql "$base/postgres" -qAt -v ON_ERROR_STOP=1 -c "CREATE DATABASE \"$name\" TEMPLATE \"$template\"" >/dev/null || exit 1
DATABASE_URL="$base/$name" "$@"
status=$?
psql "$base/postgres" -qAt -c "DROP DATABASE IF EXISTS \"$name\" WITH (FORCE)" >/dev/null 2>&1
exit $status
