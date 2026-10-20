//! Migration: **a CPD entry remembers its course** (WPM-R138). An entry that came from the learning
//! system now carries the course it was for, so a link to the course is built from data and not
//! from a sentence. Entries already there are filled from the sentence the sync wrote.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `course_ref` and fill it for earlier learning-system entries.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "ALTER TABLE cpd_entries ADD COLUMN IF NOT EXISTS course_ref VARCHAR",
            "UPDATE cpd_entries SET course_ref = substring(activity FROM 'LMS completion: (.*)$')
               WHERE source = 'lms' AND course_ref IS NULL AND activity LIKE 'LMS completion: %'",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drop the column; the sentence in `activity` still names the course.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("ALTER TABLE cpd_entries DROP COLUMN IF EXISTS course_ref")
            .await?;
        Ok(())
    }
}
