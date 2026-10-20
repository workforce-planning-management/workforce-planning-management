-- Least-privilege database roles (WPM-R123, WPM-T199).
--
--   wpm_migrator  owns the schema and may change it (CREATE, ALTER, DROP). Used only to run
--                 migrations (`workforce-planning-management db migrate`), never by the running service.
--   wpm_app       what the running service connects as: read and write rows, nothing else. It cannot
--                 create, alter or drop a table, cannot truncate, and cannot change or delete the audit trail.
--
-- Run it as a superuser (or the database owner) against the WPM database, once before the first migration and again
-- after any migration (it is idempotent: it also grants on what the migration just created):
--
--   psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 \
--        -v migrator_password="$WPM_MIGRATOR_PASSWORD" -v runtime_password="$WPM_APP_PASSWORD" \
--        -f postgres-roles/roles.sql
--
-- then run migrations with DATABASE_URL set to wpm_migrator, and start the service with DATABASE_URL set to wpm_app and
-- `auto_migrate: false` (the production profile already has it). `scripts/verify-db-roles.sh` proves all of this.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE wpm_migrator LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS PASSWORD %L', :'migrator_password')
 WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'wpm_migrator') \gexec
SELECT format('CREATE ROLE wpm_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS PASSWORD %L', :'runtime_password')
 WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'wpm_app') \gexec
SELECT format('ALTER ROLE wpm_migrator PASSWORD %L', :'migrator_password') \gexec
SELECT format('ALTER ROLE wpm_app PASSWORD %L', :'runtime_password') \gexec

-- Connect, and nothing for everyone else.
SELECT current_database() AS dbname \gset
REVOKE ALL ON DATABASE :"dbname" FROM PUBLIC;
GRANT CONNECT ON DATABASE :"dbname" TO wpm_migrator, wpm_app;
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
GRANT USAGE, CREATE ON SCHEMA public TO wpm_migrator;
GRANT USAGE ON SCHEMA public TO wpm_app;

-- Tables and sequences the migrator creates from now on are usable by the service.
ALTER DEFAULT PRIVILEGES FOR ROLE wpm_migrator IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO wpm_app;
ALTER DEFAULT PRIVILEGES FOR ROLE wpm_migrator IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO wpm_app;

-- Anything that already exists (this script run after a migration).
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO wpm_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO wpm_app;

-- The audit trail is append-only for the service: it can add entries, not change or remove them. (The hash chain would show
-- a change; this makes the database refuse it.) Skipped before the first migration, when the table does not exist yet.
SELECT 'REVOKE UPDATE, DELETE, TRUNCATE ON audit_logs FROM wpm_app'
 WHERE to_regclass('public.audit_logs') IS NOT NULL \gexec
