//! Migration: **a peer event id is unique per claim key, not across every claim** (WPM-T397, WPM-D148).
//!
//! `programme_demands_peer_event` was `(source, peer_event_id)`, so an event id applied to one claim anywhere could not be applied to
//! another, and the route answered it as a duplicate of a claim the caller might not see. The repeat check now looks only among the
//! claims of the event's own key (pool, programme, month, source), and the index matches it: the same id for another key is a new event
//! there. Idempotency within a key is unchanged (one live claim per key is already `programme_demands_live`).

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Replace the global index with one per key.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "DROP INDEX IF EXISTS programme_demands_peer_event",
            "CREATE UNIQUE INDEX IF NOT EXISTS programme_demands_peer_event
                 ON programme_demands (pool_pid, programme_ref, month, source, peer_event_id) WHERE peer_event_id IS NOT NULL",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Put the global index back. An id used for more than one key is kept on its earliest claim and cleared on the others (they lose
    /// only the memory of their last event, so a replay of it there would be applied again), so the old index can be built.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "UPDATE programme_demands d SET peer_event_id = NULL
                 WHERE d.peer_event_id IS NOT NULL AND EXISTS (
                     SELECT 1 FROM programme_demands e
                     WHERE e.source = d.source AND e.peer_event_id = d.peer_event_id AND e.id < d.id)",
            "DROP INDEX IF EXISTS programme_demands_peer_event",
            "CREATE UNIQUE INDEX IF NOT EXISTS programme_demands_peer_event
                 ON programme_demands (source, peer_event_id) WHERE peer_event_id IS NOT NULL",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}
