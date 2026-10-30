//! Migration: **a default checklist item is stored by key** (WPM-T386).
//!
//! `ergonomic_items.item_key` holds the stable key of a default workstation item (`chair`, `lighting`, …), which a locale translates;
//! `name` keeps the English text for an API reader, and a custom item has no key. Rows written before this migration hold only the
//! English sentence: those whose `name` is exactly one of the eight default sentences get its key, so they show in the reader's language
//! too; any other row is left as typed. The pairs are the service's `rules::ergonomics::DSE_ITEMS`, and a unit test there fails if they
//! differ.

use sea_orm_migration::prelude::*;

/// The default items' keys and English text, as `rules::ergonomics::DSE_ITEMS` holds them.
const PAIRS: [(&str, &str); 8] = [
    (
        "screen",
        "Screen at a comfortable height and distance, free of flicker",
    ),
    ("chair", "Chair adjustable, with lower-back support"),
    (
        "keyboard_mouse",
        "Keyboard and mouse positioned to keep wrists straight",
    ),
    (
        "desk_space",
        "Enough desk space; documents holdable beside the screen",
    ),
    (
        "lighting",
        "Lighting adequate; screen free of glare and reflections",
    ),
    ("leg_room", "Leg room clear; feet supported"),
    (
        "breaks",
        "Work pattern allows breaks or changes of activity",
    ),
    ("software", "Software and display legible and responsive"),
];

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `item_key` and give the existing default rows theirs.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE ergonomic_items ADD COLUMN IF NOT EXISTS item_key VARCHAR NULL
                 CONSTRAINT ergonomic_items_item_key_shape CHECK (item_key ~ '^[a-z][a-z_]*$')",
        )
        .await?;
        for (key, text) in PAIRS {
            // both are constants above, with no quote in them
            db.execute_unprepared(&format!(
                "UPDATE ergonomic_items SET item_key = '{key}' WHERE item_key IS NULL AND name = '{text}'"
            ))
            .await?;
        }
        Ok(())
    }

    /// Drop the key; the English text in `name` is untouched, so nothing is lost.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("ALTER TABLE ergonomic_items DROP COLUMN IF EXISTS item_key")
            .await?;
        Ok(())
    }
}
