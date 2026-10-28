//! `cargo loco task import_sfiaplus file:<path> version:<n> licence_checked_on:<YYYY-MM-DD>
//! [licence_holder:<text>] [licence_reference:<text>]` — load the deployer's **licensed** `SFIAplus` detail, saved as CSV to the
//! documented (provisional) contract in `rules::sfia`, as development **pointers** on SFIA skills at levels (WPM-R112).
//!
//! WPM ships none of it, and books or buys nothing from it. The task refuses a file with no recorded licence check, refuses a
//! resource for a SFIA skill or level that is not in the imported catalogue (nothing is written then), records the framework
//! as `internal_only`, and is idempotent: a resource a newer file drops is marked retired, not deleted.

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, DatabaseConnection, TransactionTrait};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

use super::import_sfia::{Licence, check_licence};
use crate::models::_entities::{
    capability_frameworks, sfia_skill_levels, sfia_skills, sfiaplus_resources,
};
use crate::rules::framework::Redistribution;
use crate::rules::sfia as rules;

/// The framework slug the `SFIAplus` licence is recorded under.
pub const SFIAPLUS_SLUG: &str = "sfiaplus";

/// The `SFIAplus` import task.
pub struct ImportSfiaplus;

#[async_trait]
impl Task for ImportSfiaplus {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "import_sfiaplus".to_string(),
            detail: "Load the licensed SFIAplus CSV (file:<path> version:<n> licence_checked_on:<date> [licence_holder:] [licence_reference:])"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let file = vars
            .cli_arg("file")
            .map_err(|_| Error::string("file:<path to the SFIAplus CSV> is required"))?;
        let licence = Licence {
            version: vars.cli_arg("version").unwrap_or("").to_string(),
            checked_on: vars.cli_arg("licence_checked_on").unwrap_or("").to_string(),
            holder: vars.cli_arg("licence_holder").ok().map(str::to_string),
            reference: vars.cli_arg("licence_reference").ok().map(str::to_string),
        };
        let text = std::fs::read_to_string(Path::new(file))
            .map_err(|e| Error::string(&format!("{file}: {e}")))?;
        let report = import_sfiaplus(&ctx.db, &text, &licence).await?;
        tracing::info!(?report, "SFIAplus import finished");
        println!("{report:#?}");
        Ok(())
    }
}

/// What an import did.
#[derive(Debug, Default)]
pub struct Report {
    /// Resources in the file.
    pub resources: usize,
    /// Resources in the catalogue but not in the file, now marked retired.
    pub retired: usize,
}

/// Load the contract text: upsert, then retire what is gone.
///
/// # Errors
/// A refusal naming the licence field, the file row, or the SFIA skill or level that is not in the catalogue; or a database error.
#[allow(clippy::too_many_lines)] // validate → upsert → retire → record the licence, linearly
pub async fn import_sfiaplus(
    db: &DatabaseConnection,
    text: &str,
    licence: &Licence,
) -> Result<Report> {
    let checked_on = check_licence(licence)?;
    let parsed = rules::parse_resources(text).map_err(|e| Error::string(&e))?;
    if parsed.is_empty() {
        return Err(Error::string("the file has a header and no resources"));
    }
    let version = licence.version.trim();
    let txn = db.begin().await?;
    let skills: BTreeSet<String> = sfia_skills::Entity::find()
        .filter(sfia_skills::Column::RetiredIn.is_null())
        .all(&txn)
        .await?
        .into_iter()
        .map(|s| s.code)
        .collect();
    let levels: BTreeSet<(String, i32)> = sfia_skill_levels::Entity::find()
        .filter(sfia_skill_levels::Column::RetiredIn.is_null())
        .all(&txn)
        .await?
        .into_iter()
        .map(|l| (l.code, l.level))
        .collect();
    for r in &parsed {
        if !skills.contains(&r.code) {
            return Err(Error::string(&format!(
                "SFIA skill {} is not in the imported catalogue: run import_sfia first",
                r.code
            )));
        }
        if !levels.contains(&(r.code.clone(), r.level)) {
            return Err(Error::string(&format!(
                "SFIA {} is not defined at level {}",
                r.code, r.level
            )));
        }
    }
    let known: BTreeMap<(String, i32, String, String), sfiaplus_resources::Model> =
        sfiaplus_resources::Entity::find()
            .all(&txn)
            .await?
            .into_iter()
            .map(|m| {
                (
                    (m.code.clone(), m.level, m.kind.clone(), m.title.clone()),
                    m,
                )
            })
            .collect();
    let mut seen = BTreeSet::new();
    for r in &parsed {
        let key = (r.code.clone(), r.level, r.kind.clone(), r.title.clone());
        seen.insert(key.clone());
        if let Some(row) = known.get(&key) {
            let mut a: sfiaplus_resources::ActiveModel = row.clone().into();
            a.provider = ActiveValue::set(r.provider.clone());
            a.url = ActiveValue::set(r.url.clone());
            a.retired_in = ActiveValue::set(None);
            a.update(&txn).await?;
        } else {
            sfiaplus_resources::ActiveModel {
                code: ActiveValue::set(r.code.clone()),
                level: ActiveValue::set(r.level),
                kind: ActiveValue::set(r.kind.clone()),
                title: ActiveValue::set(r.title.clone()),
                provider: ActiveValue::set(r.provider.clone()),
                url: ActiveValue::set(r.url.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }
    }
    let mut retired = 0;
    for (key, row) in &known {
        if !seen.contains(key) && row.retired_in.is_none() {
            let mut a: sfiaplus_resources::ActiveModel = row.clone().into();
            a.retired_in = ActiveValue::set(Some(version.to_string()));
            a.update(&txn).await?;
            retired += 1;
        }
    }

    // The framework row: the deployer's BCS licence, internal use only.
    let existing = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(SFIAPLUS_SLUG))
        .one(&txn)
        .await?;
    let mut a: capability_frameworks::ActiveModel = match existing {
        Some(row) => row.into(),
        None => capability_frameworks::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            slug: ActiveValue::set(SFIAPLUS_SLUG.to_string()),
            ..Default::default()
        },
    };
    a.name = ActiveValue::set("SFIAplus (BCS, The Chartered Institute for IT)".to_string());
    a.source_url = ActiveValue::set(Some("https://www.bcs.org/".to_string()));
    a.licence = ActiveValue::set(Some(
        "BCS SFIAplus licence held by the deployer (internal use)".to_string(),
    ));
    a.scale_max = ActiveValue::set(rules::SFIA_SCALE_MAX);
    a.imported_on = ActiveValue::set(chrono::Utc::now().date_naive());
    a.note = ActiveValue::set(Some(
        "Loaded by the licensee from their own copy. Shown only as pointers: nothing is booked or bought from it. \
         WPM ships none of it; it must not be published or redistributed."
            .to_string(),
    ));
    a.version = ActiveValue::set(Some(version.to_string()));
    a.redistribution = ActiveValue::set(Redistribution::InternalOnly.token().to_string());
    a.licence_holder = ActiveValue::set(licence.holder.clone());
    a.licence_reference = ActiveValue::set(licence.reference.clone());
    a.licence_checked_on = ActiveValue::set(Some(checked_on));
    if a.id.is_not_set() {
        a.insert(&txn).await?;
    } else {
        a.update(&txn).await?;
    }
    txn.commit().await?;
    Ok(Report {
        resources: parsed.len(),
        retired,
    })
}
