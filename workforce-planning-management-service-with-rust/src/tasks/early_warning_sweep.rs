//! `cargo loco task early_warning_sweep [as_of:YYYY-MM-DD]` — look at every organization with a skill
//! pool or a recorded oversight position, open a warning for each condition newly true, close those
//! that have stopped, and tell the HR administrators (WPM-R68). Run it daily: the date a condition is
//! first true is the date of the first sweep that saw it. Idempotent within a day.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};

use crate::controllers::early_warning::sweep;
use crate::models::_entities::{oversight_positions, skill_pools};

/// The sweep task.
pub struct EarlyWarningSweep;

#[async_trait]
impl Task for EarlyWarningSweep {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "early_warning_sweep".to_string(),
            detail: "Open and close early warnings for over-committed pools and measures in breach, and \
                     tell HR (optional as_of:YYYY-MM-DD; run daily)"
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
        let mut orgs: BTreeSet<String> = BTreeSet::new();
        for p in skill_pools::Entity::find()
            .filter(skill_pools::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
        {
            orgs.insert(p.organization_ref);
        }
        for p in oversight_positions::Entity::find().all(&ctx.db).await? {
            orgs.insert(p.organization_ref);
        }
        for org in &orgs {
            let r = sweep(ctx, org, as_of).await?;
            tracing::info!(%org, %as_of, opened = r.opened, cleared = r.cleared, told = r.told, "early warning sweep");
        }
        Ok(())
    }
}
