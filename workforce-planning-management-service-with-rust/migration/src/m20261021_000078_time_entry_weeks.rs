//! Migration: **recorded minutes per worker per week** (WPM-R142, WPM-D96). A rollup of
//! `time_entries` kept by triggers, so the working-time view can rule out most workers from one
//! row per worker-week instead of reading every entry.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `time_entry_weeks`, backfill it, and keep it current with triggers.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            // week_start is the Monday of the ISO week of worked_on; only live (not deleted) entries count.
            "CREATE TABLE IF NOT EXISTS time_entry_weeks (
                 worker_pid UUID NOT NULL,
                 week_start DATE NOT NULL,
                 minutes BIGINT NOT NULL CHECK (minutes >= 0),
                 PRIMARY KEY (worker_pid, week_start)
             )",
            "CREATE INDEX IF NOT EXISTS time_entry_weeks_week ON time_entry_weeks (week_start)",
            // The working-time view looks first at weeks heavy enough that a total over the ceiling must contain
            // one (WPM-D96): 48 h x 17 weeks over 18 calendar weeks is 2,720 minutes (rules::working_time::HEAVY_WEEK_MINUTES).
            "CREATE INDEX IF NOT EXISTS time_entry_weeks_heavy
                 ON time_entry_weeks (week_start, worker_pid) WHERE minutes >= 2720",
            "TRUNCATE time_entry_weeks",
            "INSERT INTO time_entry_weeks (worker_pid, week_start, minutes)
                 SELECT worker_pid, date_trunc('week', worked_on)::date, SUM(minutes)
                 FROM time_entries WHERE deleted_at IS NULL
                 GROUP BY worker_pid, date_trunc('week', worked_on)::date",
            // Subtract the old row (if it was live), add the new one (if it is live).
            "CREATE OR REPLACE FUNCTION time_entry_weeks_sync() RETURNS trigger AS $$
             BEGIN
                 IF TG_OP IN ('UPDATE', 'DELETE') AND OLD.deleted_at IS NULL THEN
                     UPDATE time_entry_weeks SET minutes = minutes - OLD.minutes
                      WHERE worker_pid = OLD.worker_pid
                        AND week_start = date_trunc('week', OLD.worked_on)::date;
                     DELETE FROM time_entry_weeks
                      WHERE worker_pid = OLD.worker_pid
                        AND week_start = date_trunc('week', OLD.worked_on)::date AND minutes = 0;
                 END IF;
                 IF TG_OP IN ('INSERT', 'UPDATE') AND NEW.deleted_at IS NULL THEN
                     INSERT INTO time_entry_weeks (worker_pid, week_start, minutes)
                     VALUES (NEW.worker_pid, date_trunc('week', NEW.worked_on)::date, NEW.minutes)
                     ON CONFLICT (worker_pid, week_start)
                     DO UPDATE SET minutes = time_entry_weeks.minutes + EXCLUDED.minutes;
                 END IF;
                 RETURN NULL;
             END;
             $$ LANGUAGE plpgsql",
            "DROP TRIGGER IF EXISTS time_entry_weeks_sync ON time_entries",
            "CREATE TRIGGER time_entry_weeks_sync
                 AFTER INSERT OR UPDATE OF worker_pid, worked_on, minutes, deleted_at OR DELETE
                 ON time_entries FOR EACH ROW EXECUTE FUNCTION time_entry_weeks_sync()",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drop the trigger, the function and the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "DROP TRIGGER IF EXISTS time_entry_weeks_sync ON time_entries",
            "DROP FUNCTION IF EXISTS time_entry_weeks_sync()",
            "DROP TABLE IF EXISTS time_entry_weeks",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}
