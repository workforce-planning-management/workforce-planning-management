//! Migration: an **append-only, hash-chained audit trail** (WPM-R115, WPM-D72).
//!
//! The database itself refuses to change or remove an audit row, and each row carries a hash of
//! its own content and of the row before it, so removing or rewriting one shows up as a break in
//! the chain. Both are done by triggers, so every writer is covered, including a person with a SQL
//! prompt, and no application code path can forget to do it.
//!
//! - `chain_seq` is the row's place in the chain, assigned under a lock so the order is the order
//!   rows were inserted, whatever the transaction ids say.
//! - `audit_entry_hash(...)` is the one function that defines the hash; the trigger, the
//!   back-fill of existing rows and the verifier all call it.
//! - **What this does not do:** a database superuser can disable a trigger, and `TRUNCATE` is not
//!   a row change. Rewriting a row then re-chaining everything after it is detectable only against a
//!   head hash recorded somewhere else (`verify_audit_chain` prints it). A restore re-inserts rows
//!   and so re-chains them; compare the head hash from before the backup.
//! - **Cost:** inserts take a transaction-scoped advisory lock, so audit writes are serialised
//!   until their transaction ends. Two transactions that take row locks in opposite order around an
//!   audit write can deadlock; the database aborts one and the request fails and can be retried.

use sea_orm_migration::prelude::*;

/// The audit-chain migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

/// The advisory-lock key that orders chain inserts.
const LOCK_KEY: i64 = 7_242_001;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the chain columns, back-fill existing rows, and install the triggers.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        let statements = [
            "ALTER TABLE audit_logs
                 ADD COLUMN IF NOT EXISTS chain_seq BIGINT NULL,
                 ADD COLUMN IF NOT EXISTS prev_hash TEXT NULL,
                 ADD COLUMN IF NOT EXISTS entry_hash TEXT NULL"
                .to_string(),
            // The one definition of the hash. Text fields are length-prefixed so no value can
            // be read as two. The time is rendered in UTC so a session's time zone cannot change it.
            "CREATE OR REPLACE FUNCTION audit_entry_hash(
                 prev TEXT, seq BIGINT, p_entity TEXT, p_entity_pid UUID, p_action TEXT,
                 p_actor TEXT, p_snapshot JSONB, p_created TIMESTAMPTZ
             ) RETURNS TEXT AS $$
                 SELECT encode(sha256(convert_to(
                     prev || '|' || seq::text
                     || '|' || length(p_entity)::text || ':' || p_entity
                     || '|' || p_entity_pid::text
                     || '|' || length(p_action)::text || ':' || p_action
                     || '|' || length(COALESCE(p_actor, ''))::text || ':' || COALESCE(p_actor, '')
                     || '|' || length(COALESCE(p_snapshot::text, ''))::text || ':' || COALESCE(p_snapshot::text, '')
                     || '|' || to_char(p_created AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US'),
                     'UTF8')), 'hex')
             $$ LANGUAGE sql IMMUTABLE"
                .to_string(),
            // Chain the rows that already exist, oldest first.
            "DO $$
             DECLARE r RECORD; prev TEXT := repeat('0', 64); seq BIGINT := 0; h TEXT;
             BEGIN
                 FOR r IN SELECT * FROM audit_logs ORDER BY id LOOP
                     seq := seq + 1;
                     h := audit_entry_hash(prev, seq, r.entity, r.entity_pid, r.action, r.actor, r.snapshot, r.created_at);
                     UPDATE audit_logs SET chain_seq = seq, prev_hash = prev, entry_hash = h WHERE id = r.id;
                     prev := h;
                 END LOOP;
             END $$"
                .to_string(),
            "ALTER TABLE audit_logs
                 ALTER COLUMN chain_seq SET NOT NULL,
                 ALTER COLUMN prev_hash SET NOT NULL,
                 ALTER COLUMN entry_hash SET NOT NULL"
                .to_string(),
            "CREATE UNIQUE INDEX IF NOT EXISTS audit_logs_chain_seq ON audit_logs (chain_seq)"
                .to_string(),
            format!(
                "CREATE OR REPLACE FUNCTION audit_logs_chain_insert() RETURNS trigger AS $$
                 DECLARE head RECORD;
                 BEGIN
                     PERFORM pg_advisory_xact_lock({LOCK_KEY});
                     SELECT chain_seq, entry_hash INTO head FROM audit_logs ORDER BY chain_seq DESC LIMIT 1;
                     NEW.chain_seq := COALESCE(head.chain_seq, 0) + 1;
                     NEW.prev_hash := COALESCE(head.entry_hash, repeat('0', 64));
                     NEW.entry_hash := audit_entry_hash(NEW.prev_hash, NEW.chain_seq, NEW.entity,
                         NEW.entity_pid, NEW.action, NEW.actor, NEW.snapshot, NEW.created_at);
                     RETURN NEW;
                 END $$ LANGUAGE plpgsql"
            ),
            "DROP TRIGGER IF EXISTS audit_logs_chain ON audit_logs".to_string(),
            "CREATE TRIGGER audit_logs_chain BEFORE INSERT ON audit_logs
                 FOR EACH ROW EXECUTE FUNCTION audit_logs_chain_insert()"
                .to_string(),
            "CREATE OR REPLACE FUNCTION audit_logs_immutable() RETURNS trigger AS $$
                 BEGIN
                     RAISE EXCEPTION 'audit_logs is append-only: % is not allowed', TG_OP
                         USING ERRCODE = 'insufficient_privilege';
                 END $$ LANGUAGE plpgsql"
                .to_string(),
            "DROP TRIGGER IF EXISTS audit_logs_immutable ON audit_logs".to_string(),
            "CREATE TRIGGER audit_logs_immutable BEFORE UPDATE OR DELETE ON audit_logs
                 FOR EACH ROW EXECUTE FUNCTION audit_logs_immutable()"
                .to_string(),
        ];
        for statement in statements {
            db.execute_unprepared(&statement).await?;
        }
        Ok(())
    }

    /// Remove the triggers, the functions and the columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "DROP TRIGGER IF EXISTS audit_logs_immutable ON audit_logs",
            "DROP TRIGGER IF EXISTS audit_logs_chain ON audit_logs",
            "DROP FUNCTION IF EXISTS audit_logs_immutable()",
            "DROP FUNCTION IF EXISTS audit_logs_chain_insert()",
            "DROP INDEX IF EXISTS audit_logs_chain_seq",
            "DROP FUNCTION IF EXISTS audit_entry_hash(TEXT, BIGINT, TEXT, UUID, TEXT, TEXT, JSONB, TIMESTAMPTZ)",
            "ALTER TABLE audit_logs DROP COLUMN IF EXISTS entry_hash, DROP COLUMN IF EXISTS prev_hash, DROP COLUMN IF EXISTS chain_seq",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
