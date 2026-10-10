//! `audit_logs` model — record and query the audit trail.
//!
//! Every mutation **and every sensitive read** (salary-bearing worker reads,
//! payslips, review content, succession plans) writes one row. The `snapshot` JSON carries the action
//! detail (old/new state, override reasons, the `department` where
//! relevant — which is what the department-scoped query filters on).

use loco_rs::prelude::*;
use sea_orm::{ConnectionTrait, QueryOrder, QuerySelect};
use uuid::Uuid;

pub use super::_entities::audit_logs::{self, ActiveModel, Entity, Model};

impl ActiveModelBehavior for super::_entities::audit_logs::ActiveModel {}

/// The result of checking the audit chain.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChainReport {
    /// Whether every entry matched its hash and its place.
    pub ok: bool,
    /// How many entries there are.
    pub entries: i64,
    /// The newest entry's hash: record it somewhere the database cannot reach, and compare it
    /// after a restore.
    pub head_hash: Option<String>,
    /// The `chain_seq` of the first entry that does not match, or `None`.
    pub first_break: Option<i64>,
}

impl Model {
    /// Record one audit entry. `actor` is the caller's `sub` (user
    /// `pid`) when a verified token was presented, else `None`.
    ///
    /// Generic over [`ConnectionTrait`] so it runs on the handler's
    /// `&DatabaseTransaction` — the audit row commits **with** the
    /// mutation it records (family invariant 8), never separately.
    ///
    /// # Errors
    ///
    /// When the insert fails.
    pub async fn record<C: ConnectionTrait>(
        db: &C,
        entity: &str,
        entity_pid: Uuid,
        action: &str,
        actor: Option<&str>,
        snapshot: Option<serde_json::Value>,
    ) -> ModelResult<Self> {
        let entry = audit_logs::ActiveModel {
            entity: ActiveValue::set(entity.to_string()),
            entity_pid: ActiveValue::set(entity_pid),
            action: ActiveValue::set(action.to_string()),
            actor: ActiveValue::set(actor.map(ToString::to_string)),
            snapshot: ActiveValue::set(snapshot),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok(entry)
    }

    /// Check the whole chain: every entry's hash is the hash of its content and the entry
    /// before it, in order. Reports the first place it breaks, the number of entries and the
    /// head hash to record somewhere the database cannot reach.
    ///
    /// # Errors
    ///
    /// When the query fails.
    pub async fn verify_chain(db: &DatabaseConnection) -> ModelResult<ChainReport> {
        use sea_orm::{DbBackend, Statement};
        let broken = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                "SELECT chain_seq FROM (
                     SELECT chain_seq, prev_hash, entry_hash,
                            audit_entry_hash(prev_hash, chain_seq, entity, entity_pid, action, actor, snapshot, created_at) AS recomputed,
                            lag(entry_hash) OVER (ORDER BY chain_seq) AS previous,
                            lag(chain_seq) OVER (ORDER BY chain_seq) AS previous_seq
                     FROM audit_logs
                 ) t
                 WHERE entry_hash <> recomputed
                    OR prev_hash <> COALESCE(previous, repeat('0', 64))
                    OR chain_seq <> COALESCE(previous_seq, 0) + 1
                 ORDER BY chain_seq LIMIT 1",
            ))
            .await?
            .map(|row| row.try_get::<i64>("", "chain_seq"))
            .transpose()?;
        let head = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                "SELECT count(*)::bigint AS entries,
                        (SELECT entry_hash FROM audit_logs ORDER BY chain_seq DESC LIMIT 1) AS head_hash
                 FROM audit_logs",
            ))
            .await?;
        let (entries, head_hash) = match head {
            Some(row) => (
                row.try_get::<i64>("", "entries")?,
                row.try_get::<Option<String>>("", "head_hash")?,
            ),
            None => (0, None),
        };
        Ok(ChainReport {
            ok: broken.is_none(),
            entries,
            head_hash,
            first_break: broken,
        })
    }

    /// Most-recent audit entries, capped at `limit`.
    ///
    /// # Errors
    ///
    /// When the query fails.
    pub async fn recent(db: &DatabaseConnection, limit: u64) -> ModelResult<Vec<Self>> {
        let rows = audit_logs::Entity::find()
            .order_by_desc(audit_logs::Column::Id)
            .limit(limit)
            .all(db)
            .await?;
        Ok(rows)
    }

    /// Audit entries for one record, most-recent first.
    ///
    /// # Errors
    ///
    /// When the query fails.
    pub async fn for_entity(db: &DatabaseConnection, entity_pid: Uuid) -> ModelResult<Vec<Self>> {
        let rows = audit_logs::Entity::find()
            .filter(audit_logs::Column::EntityPid.eq(entity_pid))
            .order_by_desc(audit_logs::Column::Id)
            .all(db)
            .await?;
        Ok(rows)
    }

    /// The department-scoped query: entries since `since` whose
    /// snapshot names this `department`, most-recent first, capped at
    /// `limit`. The filter is applied over the snapshot JSON in Rust
    /// (cross-backend; the candidate set is already time-bounded).
    ///
    /// # Errors
    ///
    /// When the query fails.
    pub async fn for_department_since(
        db: &DatabaseConnection,
        department: &str,
        since: chrono::DateTime<chrono::FixedOffset>,
        limit: u64,
    ) -> ModelResult<Vec<Self>> {
        let rows = audit_logs::Entity::find()
            .filter(audit_logs::Column::CreatedAt.gte(since))
            .order_by_desc(audit_logs::Column::Id)
            .limit(limit.saturating_mul(10)) // headroom before the in-app filter
            .all(db)
            .await?;
        let rows = rows
            .into_iter()
            .filter(|row| {
                row.snapshot
                    .as_ref()
                    .and_then(|s| s.get("department"))
                    .and_then(serde_json::Value::as_str)
                    == Some(department)
            })
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .collect();
        Ok(rows)
    }
}
