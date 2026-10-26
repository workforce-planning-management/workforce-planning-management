//! Migration: the **consultation register** (WPM-R105, WPM-D68). One row per proposal that may trigger a collective duty: the body consulted,
//! the proposal as counts and dates (never names), the topics, a reference to the documents shared, when consultation began and ended, and the
//! outcome. **No attendee, representative or minute is held**: a consultation is with a *body*, and what was said is not stored.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `consultations`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS consultations (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     organization_ref VARCHAR NOT NULL,
                     body_pid UUID NOT NULL,
                     plan_pid UUID,
                     establishment VARCHAR NOT NULL CHECK (length(btrim(establishment)) BETWEEN 1 AND 80),
                     profile VARCHAR NOT NULL CHECK (profile IN ('great_britain', 'eu_directive_98_59')),
                     establishment_size INTEGER NOT NULL CHECK (establishment_size >= 0),
                     dismissals INTEGER NOT NULL CHECK (dismissals >= 1),
                     earliest_termination_on DATE NOT NULL,
                     topics VARCHAR NOT NULL CHECK (length(topics) BETWEEN 1 AND 200),
                     document_ref VARCHAR CHECK (document_ref IS NULL OR length(btrim(document_ref)) BETWEEN 1 AND 200),
                     started_on DATE,
                     ended_on DATE,
                     outcome VARCHAR NOT NULL DEFAULT 'open' CHECK (outcome IN ('open', 'agreed', 'no_agreement', 'withdrawn')),
                     agreement_pid UUID,
                     recorded_by VARCHAR,
                     CHECK ((outcome = 'open') = (ended_on IS NULL)),
                     CHECK (ended_on IS NULL OR started_on IS NULL OR ended_on >= started_on),
                     CHECK (agreement_pid IS NULL OR outcome = 'agreed')
                 )",
            )
            .await?;
        m.get_connection()
            .execute_unprepared("CREATE INDEX IF NOT EXISTS consultations_by_org ON consultations (organization_ref, establishment)")
            .await?;
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS consultations")
            .await?;
        Ok(())
    }
}
