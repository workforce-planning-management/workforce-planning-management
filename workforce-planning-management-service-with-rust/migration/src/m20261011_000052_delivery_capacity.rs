//! Migration: **delivery capacity** (WPM-R56–R58, WPM-R60). Skill pools and their
//! members, programme demand, partner commitments, the organization's
//! work-in-progress limit and the recorded start decisions.
//!
//! None of these tables holds personal data (WPM-D45): a pool is a group of role
//! profiles and skills, demand and commitments are aggregate FTE, and programmes and
//! partners are `EntityRef` URNs owned by other services. FTE is stored in
//! hundredths. A month is stored as its first day.

use sea_orm_migration::prelude::*;

/// The delivery-capacity migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the six tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS skill_pools (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 operations_reservation_bp INTEGER NOT NULL DEFAULT 0,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (operations_reservation_bp BETWEEN 0 AND 10000)
             )",
            "CREATE UNIQUE INDEX IF NOT EXISTS skill_pools_name_live
                 ON skill_pools (organization_ref, lower(name)) WHERE deleted_at IS NULL",
            "CREATE TABLE IF NOT EXISTS skill_pool_members (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 pool_pid UUID NOT NULL,
                 role_profile_pid UUID NULL,
                 skill_pid UUID NULL,
                 min_proficiency INTEGER NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK ((role_profile_pid IS NOT NULL) <> (skill_pid IS NOT NULL)),
                 CHECK (min_proficiency IS NULL OR min_proficiency BETWEEN 1 AND 5)
             )",
            "CREATE INDEX IF NOT EXISTS skill_pool_members_pool ON skill_pool_members (pool_pid)",
            "CREATE TABLE IF NOT EXISTS programme_demands (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 programme_ref VARCHAR NOT NULL,
                 pool_pid UUID NOT NULL,
                 month DATE NOT NULL,
                 fte_centi BIGINT NOT NULL,
                 status VARCHAR NOT NULL DEFAULT 'proposed',
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (fte_centi > 0)
             )",
            "CREATE UNIQUE INDEX IF NOT EXISTS programme_demands_live
                 ON programme_demands (programme_ref, pool_pid, month) WHERE deleted_at IS NULL",
            "CREATE TABLE IF NOT EXISTS partner_commitments (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 partner_ref VARCHAR NOT NULL,
                 pool_pid UUID NOT NULL,
                 month DATE NOT NULL,
                 fte_centi BIGINT NOT NULL,
                 status VARCHAR NOT NULL DEFAULT 'requested',
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (fte_centi > 0)
             )",
            "CREATE UNIQUE INDEX IF NOT EXISTS partner_commitments_live
                 ON partner_commitments (partner_ref, pool_pid, month) WHERE deleted_at IS NULL",
            "CREATE TABLE IF NOT EXISTS capacity_settings (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL UNIQUE,
                 wip_limit INTEGER NULL,
                 CHECK (wip_limit IS NULL OR wip_limit >= 0)
             )",
            "CREATE TABLE IF NOT EXISTS start_decisions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 programme_ref VARCHAR NOT NULL,
                 decision VARCHAR NOT NULL,
                 reason VARCHAR NOT NULL,
                 fit VARCHAR NOT NULL,
                 earliest_shift_months INTEGER NULL,
                 wip_exceeds BOOLEAN NULL,
                 decided_by VARCHAR NULL
             )",
            "CREATE INDEX IF NOT EXISTS start_decisions_programme ON start_decisions (programme_ref)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for table in [
            "start_decisions",
            "capacity_settings",
            "partner_commitments",
            "programme_demands",
            "skill_pool_members",
            "skill_pools",
        ] {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
