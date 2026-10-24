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

    /// Entries after `after_seq` in chain order, oldest first, at most `limit`: one page of the export for a
    /// security monitoring system, which resumes from the last `chain_seq` it saw.
    ///
    /// # Errors
    ///
    /// When the query fails.
    pub async fn export_after(
        db: &DatabaseConnection,
        after_seq: i64,
        limit: u64,
    ) -> ModelResult<Vec<Self>> {
        let rows = audit_logs::Entity::find()
            .filter(audit_logs::Column::ChainSeq.gt(after_seq))
            .order_by_asc(audit_logs::Column::ChainSeq)
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

/// A source guard (WPM-T250): an audit row commits with the change it records, so every
/// `Audit::record` outside a transaction must be on [`READ_AUDITS`], the audits of reads.
#[cfg(test)]
mod tests {
    /// `(file under src/, action)`: audits of a read, which have no write to share a transaction with.
    const READ_AUDITS: &[(&str, &str)] = &[
        ("controllers/engagements.rs", "rate_read"),
        ("controllers/pay_transparency.rs", "pay_gap_report_read"),
        ("controllers/human_capital.rs", "human_capital_report_read"),
        ("controllers/pay_range.rs", "pay_range_report_read"),
        ("controllers/policies.rs", "policy_acknowledgements_read"),
        ("controllers/mcp.rs", "mcp_tool_called"),
        ("controllers/pay_assessment.rs", "joint_assessment_read"),
        (
            "controllers/pay_information.rs",
            "pay_information_answer_read",
        ),
        ("controllers/pay_transparency.rs", "pay_gap_report_exported"),
        ("controllers/adjustments.rs", "adjustments_read"),
        ("controllers/development.rs", "review_content_read"),
        ("controllers/development.rs", "succession_read"),
        ("controllers/conversion_plans.rs", "conversion_plans_read"),
        ("controllers/conversion_plans.rs", "conversion_plans_listed"),
        ("controllers/me.rs", "contact_details_read"),
        ("controllers/appraisals.rs", "report_read"),
        ("controllers/funding.rs", "contingent_list_read"),
        ("controllers/funding.rs", "contingent_workforce_read"),
        ("controllers/key_person.rs", "key_person_read"),
        ("controllers/hr_core.rs", "salary_read"),
        ("controllers/health_requirements.rs", "health_record_read"),
        ("controllers/privacy.rs", "subject_access_exported"),
        ("controllers/assessments.rs", "results_read"),
        (
            "controllers/equality_monitoring.rs",
            "equality_monitoring_summary_read",
        ),
        ("controllers/programmes.rs", "programme_rollup_read"),
        ("controllers/payroll.rs", "payslips_read"),
        ("controllers/payroll.rs", "comparison_read"),
    ];

    /// The connection arguments that are, or are handed, a transaction.
    const IN_TRANSACTION: &[&str] = &["&txn", "txn", "db"];

    /// The top-level arguments of the call that opens at `open` (the index after its `(`).
    fn arguments(source: &str, open: usize) -> Vec<String> {
        let mut out = vec![String::new()];
        let mut depth = 0usize;
        let mut in_string = false;
        let mut previous = '\0';
        for c in source[open..].chars() {
            if in_string {
                out.last_mut().unwrap().push(c);
                if c == '"' && previous != '\\' {
                    in_string = false;
                }
            } else {
                match c {
                    '"' => {
                        in_string = true;
                        out.last_mut().unwrap().push(c);
                    }
                    '(' | '[' | '{' => {
                        depth += 1;
                        out.last_mut().unwrap().push(c);
                    }
                    ')' | ']' | '}' if depth == 0 => break,
                    ')' | ']' | '}' => {
                        depth -= 1;
                        out.last_mut().unwrap().push(c);
                    }
                    ',' if depth == 0 => out.push(String::new()),
                    _ => out.last_mut().unwrap().push(c),
                }
            }
            previous = c;
        }
        out.into_iter().map(|a| a.trim().to_string()).collect()
    }

    fn sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn every_audit_of_a_change_is_in_its_transaction() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        sources(&root, &mut files);
        let mut outside = Vec::new();
        let mut seen_reads = Vec::new();
        for path in files {
            let file = path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if file == "models/audit_logs.rs" {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for (at, _) in source.match_indices("Audit::record(") {
                let args = arguments(&source, at + "Audit::record(".len());
                let action = args.get(3).map_or("", |a| a.trim_matches('"')).to_string();
                if IN_TRANSACTION.contains(&args[0].as_str()) {
                    continue;
                }
                if READ_AUDITS.contains(&(file.as_str(), action.as_str())) {
                    seen_reads.push((file.clone(), action));
                } else {
                    let line = source[..at].lines().count();
                    outside.push(format!("{file}:{line} {} {action}", args[0]));
                }
            }
        }
        assert!(
            outside.is_empty(),
            "audit rows written outside the change's transaction:\n{}",
            outside.join("\n")
        );
        for (file, action) in READ_AUDITS {
            assert!(
                seen_reads.iter().any(|(f, a)| f == file && a == action),
                "READ_AUDITS lists {file} {action}, which no longer exists"
            );
        }
    }

    #[test]
    fn arguments_split_at_the_top_level_only() {
        let source = r#"Audit::record(&ctx.db, "worker", w.pid, "moved, twice", caller.actor(), Some(json!({ "a": [1, 2] }))).await"#;
        let args = arguments(source, "Audit::record(".len());
        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "&ctx.db");
        assert_eq!(args[3], "\"moved, twice\"");
        assert_eq!(args[5], r#"Some(json!({ "a": [1, 2] }))"#);
    }
}
