//! `cargo loco task oversight_pack_freeze [as_of:YYYY-MM-DD]` — freeze the monthly oversight pack
//! for each organization that has a recorded position (WPM-R69). Run it on the first days of the
//! month. **Idempotent:** an organization that already has a pack for the month being frozen is
//! skipped, so a re-run, or two overlapping schedules, never write a second one.

use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};

use crate::controllers::oversight_packs::freeze;
use crate::models::_entities::{oversight_packs, oversight_positions};

/// The freeze task.
pub struct OversightPackFreeze;

#[async_trait]
impl Task for OversightPackFreeze {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "oversight_pack_freeze".to_string(),
            detail: "Freeze the monthly oversight pack for each organization with a recorded position \
                     (optional as_of:YYYY-MM-DD; skips an organization that already has the month's pack)"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let as_of = match vars.cli_arg("as_of") {
            Ok(text) => text
                .parse::<NaiveDate>()
                .map_err(|e| Error::string(&format!("as_of must be YYYY-MM-DD: {e}")))?,
            Err(_) => chrono::Utc::now().date_naive(),
        };
        let frozen = freeze_all(ctx, as_of).await?;
        tracing::info!(%as_of, frozen, "oversight pack freeze done");
        Ok(())
    }
}

/// Freeze the pack for the month before `as_of` for each organization with a recorded position that
/// has none for that month. Returns how many were frozen.
///
/// # Errors
///
/// A database error.
pub async fn freeze_all(ctx: &AppContext, as_of: NaiveDate) -> Result<usize> {
    let this_month = as_of.with_day(1).unwrap_or(as_of);
    let month = this_month
        .checked_sub_months(chrono::Months::new(1))
        .unwrap_or(this_month);
    let orgs: BTreeSet<String> = oversight_positions::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| p.organization_ref)
        .collect();
    let mut frozen = 0;
    for org in &orgs {
        let have = oversight_packs::Entity::find()
            .filter(oversight_packs::Column::OrganizationRef.eq(org))
            .filter(oversight_packs::Column::Month.eq(month))
            .one(&ctx.db)
            .await?
            .is_some();
        if have {
            tracing::info!(%org, %month, "oversight pack already frozen; skipped");
            continue;
        }
        let pack = freeze(ctx, org, as_of).await?;
        frozen += 1;
        tracing::info!(%org, %month, pack = %pack.pid, "oversight pack frozen");
    }
    Ok(frozen)
}
