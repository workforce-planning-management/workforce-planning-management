//! `cargo run -- task verify_audit_chain` — check the audit trail's hash chain (WPM-R115).
//!
//! Prints one JSON line: whether the chain holds, how many entries there are, the head hash and
//! the first entry that does not match. **Record the head hash somewhere the database cannot
//! reach** (a log service, a ticket) and compare it after a restore: a chain proves entries were
//! not changed or removed from the middle, and only an outside record proves none were cut from
//! the end. Exits with an error when the chain is broken, so a schedule can alert on it.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};

use crate::controllers::audits::verify_chain;

/// The audit-chain verification task.
pub struct VerifyAuditChain;

#[async_trait]
impl Task for VerifyAuditChain {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "verify_audit_chain".to_string(),
            detail: "Check the audit trail's hash chain; prints the head hash; fails when broken"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, _vars: &Vars) -> Result<()> {
        let report = verify_chain(ctx).await?;
        println!(
            "{}",
            serde_json::to_string(&report).map_err(|e| Error::string(&e.to_string()))?
        );
        if report.ok {
            Ok(())
        } else {
            Err(Error::string(&format!(
                "the audit chain is broken at entry {}",
                report.first_break.unwrap_or(0)
            )))
        }
    }
}
