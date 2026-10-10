//! `cargo loco task replay_erasures [since:YYYY-MM-DD] [file:PATH]` — after
//! restoring a database from a backup, erase again everyone who was erased after
//! the backup was taken (WPM-R89, WPM-D63).
//!
//! The erasure ledger is a table, so a restore brings back the ledger *as of the
//! backup*, and an erasure made after it is missing from it. Keep a copy of the
//! ledger outside the database (`scripts/export-erasure-ledger.sh`, run every few
//! minutes) and pass it as `file:PATH`: its entries are added to the ledger, and
//! then everything is replayed. Without a file, the task replays only what the
//! restored ledger already holds, which is complete only when the database was
//! restored to a point after the last erasure (for example from WAL archiving).
//!
//! Run it **before the service accepts traffic**. `since` limits the replay to
//! entries on or after a date; omit it to replay everything, which is idempotent.
//! A malformed line in the file stops the task: a skipped erasure would be silent.

use chrono::{DateTime, NaiveDate};
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::ConnectionTrait;
use uuid::Uuid;

use crate::controllers::privacy::replay_erasures;

/// Parse an exported ledger: one `pid,RFC 3339 time` per line; blank lines and
/// `#` comments are skipped. Any other line is an error naming its number.
///
/// # Errors
///
/// The first malformed line.
pub fn parse_ledger(
    text: &str,
) -> std::result::Result<Vec<(Uuid, DateTime<chrono::FixedOffset>)>, String> {
    let mut out = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (pid, at) = line
            .split_once(',')
            .ok_or_else(|| format!("line {}: expected `pid,time`", number + 1))?;
        let pid = pid
            .trim()
            .parse::<Uuid>()
            .map_err(|e| format!("line {}: bad pid: {e}", number + 1))?;
        let at = DateTime::parse_from_rfc3339(at.trim())
            .map_err(|e| format!("line {}: bad time: {e}", number + 1))?;
        out.push((pid, at));
    }
    Ok(out)
}

/// The erasure-replay task.
pub struct ReplayErasures;

#[async_trait]
impl Task for ReplayErasures {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "replay_erasures".to_string(),
            detail: "After a restore, erase again everyone erased since the backup \
                     (optional since:YYYY-MM-DD and file:PATH of an exported ledger; \
                     idempotent)"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let since = match vars.cli_arg("since") {
            Ok(text) => Some(
                text.parse::<NaiveDate>()
                    .map_err(|e| Error::string(&format!("since must be YYYY-MM-DD: {e}")))?,
            ),
            Err(_) => None,
        };
        if let Ok(path) = vars.cli_arg("file") {
            let text = std::fs::read_to_string(path)
                .map_err(|e| Error::string(&format!("cannot read ledger file {path}: {e}")))?;
            let entries = parse_ledger(&text).map_err(|e| Error::string(&e))?;
            for (pid, at) in &entries {
                ctx.db
                    .execute_unprepared(&format!(
                        "INSERT INTO erasure_ledger (worker_pid, erased_at) \
                         VALUES ('{pid}', '{}') ON CONFLICT DO NOTHING",
                        at.to_rfc3339()
                    ))
                    .await?;
            }
            tracing::info!(entries = entries.len(), "erasure ledger file imported");
        }
        let report = replay_erasures(&ctx.db, since).await?;
        tracing::info!(
            considered = report.considered,
            reapplied = report.reapplied,
            missing = report.missing,
            "erasure ledger replayed"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ledger_file_parses_and_skips_blanks_and_comments() {
        let text = "# exported 2026-10-10\n\n\
                    11111111-1111-4111-8111-111111111111,2026-10-10T08:00:00+00:00\n\
                    22222222-2222-4222-8222-222222222222 , 2026-10-10T09:30:00Z\n";
        let entries = parse_ledger(text).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].0.to_string(),
            "11111111-1111-4111-8111-111111111111"
        );
        assert_eq!(entries[1].1.to_rfc3339(), "2026-10-10T09:30:00+00:00");
        assert_eq!(parse_ledger("").unwrap(), vec![]);
    }

    #[test]
    fn a_malformed_line_stops_the_import_and_names_its_number() {
        for (text, needle) in [
            ("not-a-line", "line 1: expected"),
            ("\nnot-a-uuid,2026-10-10T08:00:00Z", "line 2: bad pid"),
            (
                "11111111-1111-4111-8111-111111111111,yesterday",
                "line 1: bad time",
            ),
            // SQL in either field is a parse error, never executed.
            ("1'; DROP TABLE workers;--,2026-10-10T08:00:00Z", "bad pid"),
        ] {
            let error = parse_ledger(text).unwrap_err();
            assert!(error.contains(needle), "{text:?} -> {error}");
        }
    }
}
