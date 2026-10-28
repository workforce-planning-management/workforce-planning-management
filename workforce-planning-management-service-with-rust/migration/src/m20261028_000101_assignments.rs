//! Migration: **assignments** (WPM-R161, WPM-D112). An assignment is a contract: a job title, a department, a location, the FTE the
//! contract says, optional categories (WPM-R162; free codes until the vocabularies land, and `NULL` is *unknown*), and the days it
//! runs. One **primary** assignment is open at a time per worker. Every existing worker gets one primary assignment from the
//! columns the worker row already has, so the worker row stays the record of the primary one and its readers keep working.
//! An ended assignment is kept; a change ends one and starts another. Two triggers keep the older writers of the worker row working:
//! a new worker gets a primary assignment, and an edit to the title, department, location or FTE of the worker row corrects the open
//! primary **in place** (history is made only by a change), as does a termination date, which ends what is open.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `assignments`, back-fill one primary assignment per worker, and add the two triggers.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS assignments (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 job_title VARCHAR NOT NULL CHECK (length(btrim(job_title)) BETWEEN 1 AND 120),
                 department VARCHAR NOT NULL CHECK (length(btrim(department)) BETWEEN 1 AND 120),
                 location VARCHAR CHECK (location IS NULL OR length(btrim(location)) BETWEEN 1 AND 120),
                 site VARCHAR CHECK (site IS NULL OR length(btrim(site)) BETWEEN 1 AND 60),
                 area_of_work VARCHAR CHECK (area_of_work IS NULL OR length(btrim(area_of_work)) BETWEEN 1 AND 60),
                 job_role VARCHAR CHECK (job_role IS NULL OR length(btrim(job_role)) BETWEEN 1 AND 60),
                 staff_group VARCHAR CHECK (staff_group IS NULL OR length(btrim(staff_group)) BETWEEN 1 AND 60),
                 occupation_code VARCHAR CHECK (occupation_code IS NULL OR length(btrim(occupation_code)) BETWEEN 1 AND 60),
                 fte_percent INTEGER NOT NULL CHECK (fte_percent BETWEEN 1 AND 100),
                 is_primary BOOLEAN NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE,
                 recorded_by VARCHAR,
                 CHECK (ends_on IS NULL OR ends_on >= starts_on)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS assignments_one_open_primary ON assignments (worker_pid) WHERE is_primary AND ends_on IS NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS assignments_by_worker ON assignments (worker_pid, starts_on)",
        )
        .await?;
        // One primary per worker from the worker's own columns. A worker already left keeps an assignment that ended with them.
        db.execute_unprepared(
            "INSERT INTO assignments (pid, worker_pid, job_title, department, location, fte_percent, is_primary, starts_on, ends_on, recorded_by)
             SELECT gen_random_uuid(), w.pid, w.job_title, w.department, w.location, GREATEST(LEAST(w.fte_percent, 100), 1), TRUE,
                    w.hired_on, w.terminated_on, 'migration'
               FROM workers w
              WHERE w.deleted_at IS NULL
                AND length(btrim(w.job_title)) BETWEEN 1 AND 120
                AND length(btrim(w.department)) BETWEEN 1 AND 120
                AND NOT EXISTS (SELECT 1 FROM assignments a WHERE a.worker_pid = w.pid)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE OR REPLACE FUNCTION workers_assignment_sync() RETURNS trigger AS $$
             BEGIN
                 IF TG_OP = 'INSERT' THEN
                     INSERT INTO assignments (pid, worker_pid, job_title, department, location, fte_percent, is_primary, starts_on, ends_on, recorded_by)
                     VALUES (gen_random_uuid(), NEW.pid,
                             COALESCE(NULLIF(left(btrim(NEW.job_title), 120), ''), 'unknown'),
                             COALESCE(NULLIF(left(btrim(NEW.department), 120), ''), 'unknown'),
                             NULLIF(left(btrim(COALESCE(NEW.location, '')), 120), ''),
                             GREATEST(LEAST(NEW.fte_percent, 100), 1), TRUE, NEW.hired_on, NEW.terminated_on, 'worker row');
                 ELSE
                     UPDATE assignments
                        SET job_title = COALESCE(NULLIF(left(btrim(NEW.job_title), 120), ''), job_title),
                            department = COALESCE(NULLIF(left(btrim(NEW.department), 120), ''), department),
                            location = NULLIF(left(btrim(COALESCE(NEW.location, '')), 120), ''),
                            fte_percent = GREATEST(LEAST(NEW.fte_percent, 100), 1),
                            updated_at = CURRENT_TIMESTAMP
                      WHERE worker_pid = NEW.pid AND is_primary AND ends_on IS NULL
                        AND (OLD.job_title, OLD.department, OLD.location, OLD.fte_percent)
                            IS DISTINCT FROM (NEW.job_title, NEW.department, NEW.location, NEW.fte_percent);
                     IF NEW.terminated_on IS NOT NULL AND NEW.terminated_on IS DISTINCT FROM OLD.terminated_on THEN
                         UPDATE assignments SET ends_on = GREATEST(NEW.terminated_on, starts_on), updated_at = CURRENT_TIMESTAMP
                          WHERE worker_pid = NEW.pid AND ends_on IS NULL;
                     END IF;
                 END IF;
                 RETURN NEW;
             END;
             $$ LANGUAGE plpgsql",
        )
        .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS workers_assignment_insert ON workers")
            .await?;
        db.execute_unprepared(
            "CREATE TRIGGER workers_assignment_insert AFTER INSERT ON workers
             FOR EACH ROW EXECUTE FUNCTION workers_assignment_sync()",
        )
        .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS workers_assignment_update ON workers")
            .await?;
        db.execute_unprepared(
            "CREATE TRIGGER workers_assignment_update AFTER UPDATE OF job_title, department, location, fte_percent, terminated_on ON workers
             FOR EACH ROW EXECUTE FUNCTION workers_assignment_sync()",
        )
        .await?;
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("DROP TRIGGER IF EXISTS workers_assignment_insert ON workers")
            .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS workers_assignment_update ON workers")
            .await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS workers_assignment_sync()")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS assignments")
            .await?;
        Ok(())
    }
}
