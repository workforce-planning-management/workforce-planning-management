//! Model layer: `SeaORM` entities (`_entities/`) plus helpers.
//!
//! WPM owns a **normalized relational schema** (WPM-D2) — not the
//! matcher-DTO-as-JSONB shape of the entity registries — because its
//! value is constraints and lifecycles. Controllers query the tables
//! with `sqlx` (WPM-D184); [`records`] provides the shared pid parsing
//! and worker read; [`audit_logs`] and [`event_outbox`] are the audit /
//! durable-event side.

pub mod _entities;
pub mod audit_logs;
pub mod confederations;
pub mod event_outbox;
pub mod link_tokens;
pub mod memberships;
pub mod records;
pub mod skill_history;
