#!/bin/sh
#
# Enable the Postgres extensions the test suites need.
#
# Mounted read-only into the test-database container (see
# compose.test.yaml). The official image runs everything in
# /docker-entrypoint-initdb.d exactly once, on first initialisation of an
# empty data directory.
#
# `template1` is done first and deliberately: Postgres copies it for every
# subsequently created database, so any database created later inherits
# these extensions without needing superuser DDL of its own.
set -eu

for db in template1 "${POSTGRES_DB}"; do
  psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname "${db}" <<-'SQL'
	CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
	CREATE EXTENSION IF NOT EXISTS citext;
	CREATE EXTENSION IF NOT EXISTS unaccent;
	CREATE EXTENSION IF NOT EXISTS pg_trgm;
	CREATE EXTENSION IF NOT EXISTS pgcrypto;
SQL
done

echo "test-db: extensions ready in template1 and ${POSTGRES_DB}"
