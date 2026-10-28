//! `cargo loco task import_sfia file:<path> [levels_file:<path>] version:<8|9> licence_checked_on:<YYYY-MM-DD>
//! [licence_holder:<text>] [licence_reference:<text>]` — load the deployer's
//! **licensed** SFIA download, saved as CSV to the documented contract
//! (`rules::sfia`), into the local catalogue (WPM-R109, WPM-R113, WPM-D69).
//!
//! WPM ships no SFIA content. The task refuses a file with no recorded
//! licence check, records the framework as `internal_only`, and is
//! idempotent: skills and levels a newer version drops are marked retired,
//! not deleted. A catalogue skill is linked to a SFIA skill only on an exact
//! normalised name match, never by guess.

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, DatabaseConnection, TransactionTrait};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

use crate::models::_entities::{
    capability_frameworks, job_level_frameworks, job_level_levels, sfia_skill_levels, sfia_skills,
    skill_external_refs, skills,
};
use crate::rules::esco::normalise_label;
use crate::rules::framework::Redistribution;
use crate::rules::job_levels::Attribute;
use crate::rules::sfia::{self as rules, Skill};

/// The framework slug SFIA is recorded under.
pub const SFIA_SLUG: &str = "sfia";

/// The SFIA import task.
pub struct ImportSfia;

#[async_trait]
impl Task for ImportSfia {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "import_sfia".to_string(),
            detail: "Load the licensed SFIA CSVs (file:<skills> levels_file:<levels> version:<n> licence_checked_on:<date> [licence_holder:] [licence_reference:])"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let file = vars.cli_arg("file").ok();
        let levels_file = vars.cli_arg("levels_file").ok();
        if file.is_none() && levels_file.is_none() {
            return Err(Error::string(
                "file:<skills CSV> or levels_file:<levels of responsibility CSV> (or both) is required",
            ));
        }
        let licence = Licence {
            version: vars.cli_arg("version").unwrap_or("").to_string(),
            checked_on: vars.cli_arg("licence_checked_on").unwrap_or("").to_string(),
            holder: vars.cli_arg("licence_holder").ok().map(str::to_string),
            reference: vars.cli_arg("licence_reference").ok().map(str::to_string),
        };
        let read = |f: &str| {
            std::fs::read_to_string(Path::new(f)).map_err(|e| Error::string(&format!("{f}: {e}")))
        };
        if let Some(f) = file {
            let report = import_sfia(&ctx.db, &read(f)?, &licence).await?;
            tracing::info!(?report, "SFIA skills import finished");
            println!("{report:#?}");
        }
        if let Some(f) = levels_file {
            let n = import_sfia_levels(&ctx.db, &read(f)?, &licence).await?;
            tracing::info!(levels = n, "SFIA levels import finished");
            println!("levels of responsibility: {n}");
        }
        Ok(())
    }
}

/// The deployer's licence record for this import.
#[derive(Debug)]
pub struct Licence {
    /// The SFIA version, e.g. `9`. Required.
    pub version: String,
    /// When the deployer last checked the licence, `YYYY-MM-DD`. Required.
    pub checked_on: String,
    /// The licence holder, if the deployer states it.
    pub holder: Option<String>,
    /// The licence reference, if the deployer states it.
    pub reference: Option<String>,
}

/// What an import did.
#[derive(Debug, Default)]
pub struct Report {
    /// Skills in the file.
    pub skills: usize,
    /// Skill levels in the file.
    pub levels: usize,
    /// Skills in the catalogue but not in the file, now marked retired.
    pub skills_retired: usize,
    /// Levels in the catalogue but not in the file, now marked retired.
    pub levels_retired: usize,
    /// WPM skills newly linked to a SFIA code by exact name.
    pub catalogue_linked: usize,
}

/// The licence record must be complete before anything is read: a version, and the day the licence was checked.
pub(crate) fn check_licence(licence: &Licence) -> Result<chrono::NaiveDate> {
    if licence.version.trim().is_empty() {
        return Err(Error::string("version:<SFIA version> is required"));
    }
    chrono::NaiveDate::parse_from_str(licence.checked_on.trim(), "%Y-%m-%d").map_err(|_| {
        Error::string(
            "licence_checked_on:<YYYY-MM-DD> is required: record when you checked your SFIA licence",
        )
    })
}

/// The slug the seven levels of responsibility are held under as a job-level framework.
pub const SFIA_LEVELS_SLUG: &str = "sfia-levels";

/// Load the levels-of-responsibility contract as a job-level framework (WPM-R110): the seven levels with the five
/// attributes, `internal_only`, replaced in place. Returns the number of levels written.
///
/// # Errors
/// A refusal naming the licence field or the file row, or a database error.
pub async fn import_sfia_levels(
    db: &DatabaseConnection,
    text: &str,
    licence: &Licence,
) -> Result<usize> {
    check_licence(licence)?;
    let levels = rules::parse_levels(text).map_err(|e| Error::string(&e))?;
    let version = licence.version.trim();
    let txn = db.begin().await?;
    let existing = job_level_frameworks::Entity::find()
        .filter(job_level_frameworks::Column::Slug.eq(SFIA_LEVELS_SLUG))
        .one(&txn)
        .await?;
    let mut a: job_level_frameworks::ActiveModel = match existing {
        Some(row) => row.into(),
        None => job_level_frameworks::ActiveModel {
            slug: ActiveValue::set(SFIA_LEVELS_SLUG.to_string()),
            ..Default::default()
        },
    };
    a.name = ActiveValue::set("SFIA levels of responsibility".to_string());
    a.organization = ActiveValue::set("SFIA Foundation".to_string());
    a.track = ActiveValue::set("skills".to_string());
    a.source = ActiveValue::set(
        "Loaded by the licensee from their own SFIA download; not for publication or redistribution."
            .to_string(),
    );
    a.version = ActiveValue::set(Some(version.to_string()));
    a.redistribution = ActiveValue::set(Redistribution::InternalOnly.token().to_string());
    if a.id.is_not_set() {
        a.insert(&txn).await?;
    } else {
        a.update(&txn).await?;
    }
    for l in &levels {
        let attributes: Vec<Attribute> = l
            .attributes
            .iter()
            .map(|(name, text)| Attribute {
                name: name.clone(),
                text: text.clone(),
            })
            .collect();
        let attributes =
            serde_json::to_string(&attributes).map_err(|e| Error::string(&e.to_string()))?;
        let code = format!("L{}", l.level);
        let row = job_level_levels::Entity::find()
            .filter(job_level_levels::Column::Framework.eq(SFIA_LEVELS_SLUG))
            .filter(job_level_levels::Column::Number.eq(l.level))
            .one(&txn)
            .await?;
        let mut a: job_level_levels::ActiveModel = match row {
            Some(r) => r.into(),
            None => job_level_levels::ActiveModel {
                framework: ActiveValue::set(SFIA_LEVELS_SLUG.to_string()),
                number: ActiveValue::set(l.level),
                ..Default::default()
            },
        };
        a.code = ActiveValue::set(code);
        a.title = ActiveValue::set(l.title.clone());
        a.summary = ActiveValue::set(l.essence.clone());
        a.attributes = ActiveValue::set(Some(attributes));
        if a.id.is_not_set() {
            a.insert(&txn).await?;
        } else {
            a.update(&txn).await?;
        }
    }
    txn.commit().await?;
    Ok(levels.len())
}

/// Load the contract text, replacing nothing: upsert, then retire what is gone.
///
/// # Errors
/// A refusal naming the licence field or the file row, or a database error.
#[allow(clippy::too_many_lines)] // validate → upsert → retire → link, linearly
pub async fn import_sfia(db: &DatabaseConnection, text: &str, licence: &Licence) -> Result<Report> {
    let checked_on = check_licence(licence)?;
    let parsed: Vec<Skill> = rules::parse_skills(text).map_err(|e| Error::string(&e))?;
    if parsed.is_empty() {
        return Err(Error::string("the file has a header and no skills"));
    }
    let version = licence.version.trim();
    let mut report = Report {
        skills: parsed.len(),
        levels: parsed.iter().map(|s| s.levels.len()).sum(),
        ..Report::default()
    };

    let txn = db.begin().await?;
    let known: BTreeMap<String, sfia_skills::Model> = sfia_skills::Entity::find()
        .all(&txn)
        .await?
        .into_iter()
        .map(|s| (s.code.clone(), s))
        .collect();
    let known_levels: BTreeMap<(String, i32), sfia_skill_levels::Model> =
        sfia_skill_levels::Entity::find()
            .all(&txn)
            .await?
            .into_iter()
            .map(|l| ((l.code.clone(), l.level), l))
            .collect();
    let mut seen_skills: BTreeSet<&str> = BTreeSet::new();
    let mut seen_levels: BTreeSet<(&str, i32)> = BTreeSet::new();
    for skill in &parsed {
        seen_skills.insert(&skill.code);
        let fields = (
            ActiveValue::set(skill.name.clone()),
            ActiveValue::set(skill.category.clone()),
            ActiveValue::set(skill.subcategory.clone()),
            ActiveValue::set(skill.description.clone()),
            ActiveValue::set(None::<String>),
        );
        if let Some(row) = known.get(&skill.code) {
            let mut a: sfia_skills::ActiveModel = row.clone().into();
            (
                a.name,
                a.category,
                a.subcategory,
                a.description,
                a.retired_in,
            ) = fields;
            a.update(&txn).await?;
        } else {
            sfia_skills::ActiveModel {
                code: ActiveValue::set(skill.code.clone()),
                name: fields.0,
                category: fields.1,
                subcategory: fields.2,
                description: fields.3,
                retired_in: fields.4,
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }
        for level in &skill.levels {
            seen_levels.insert((&skill.code, level.level));
            if let Some(row) = known_levels.get(&(skill.code.clone(), level.level)) {
                let mut a: sfia_skill_levels::ActiveModel = row.clone().into();
                a.description = ActiveValue::set(level.description.clone());
                a.attributes = ActiveValue::set(level.attributes.clone());
                a.retired_in = ActiveValue::set(None);
                a.update(&txn).await?;
            } else {
                sfia_skill_levels::ActiveModel {
                    code: ActiveValue::set(skill.code.clone()),
                    level: ActiveValue::set(level.level),
                    description: ActiveValue::set(level.description.clone()),
                    attributes: ActiveValue::set(level.attributes.clone()),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
            }
        }
    }
    // What a newer version drops stays, marked, so declared skills never dangle.
    for (code, row) in &known {
        if !seen_skills.contains(code.as_str()) && row.retired_in.is_none() {
            let mut a: sfia_skills::ActiveModel = row.clone().into();
            a.retired_in = ActiveValue::set(Some(version.to_string()));
            a.update(&txn).await?;
            report.skills_retired += 1;
        }
    }
    for ((code, level), row) in &known_levels {
        if !seen_levels.contains(&(code.as_str(), *level)) && row.retired_in.is_none() {
            let mut a: sfia_skill_levels::ActiveModel = row.clone().into();
            a.retired_in = ActiveValue::set(Some(version.to_string()));
            a.update(&txn).await?;
            report.levels_retired += 1;
        }
    }

    // The framework row: internal use only, whatever it said before.
    let existing = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(SFIA_SLUG))
        .one(&txn)
        .await?;
    let today = chrono::Utc::now().date_naive();
    let note = "Loaded by the licensee from their own download. WPM ships no SFIA content; \
                it must not be published or redistributed.";
    let mut a: capability_frameworks::ActiveModel = match existing {
        Some(row) => row.into(),
        None => capability_frameworks::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            slug: ActiveValue::set(SFIA_SLUG.to_string()),
            ..Default::default()
        },
    };
    a.name = ActiveValue::set("SFIA — Skills Framework for the Information Age".to_string());
    a.source_url = ActiveValue::set(Some("https://sfia-online.org/".to_string()));
    a.licence = ActiveValue::set(Some(
        "SFIA Foundation corporate licence (internal use)".to_string(),
    ));
    a.scale_max = ActiveValue::set(rules::SFIA_SCALE_MAX);
    a.imported_on = ActiveValue::set(today);
    a.note = ActiveValue::set(Some(note.to_string()));
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

    // Link by exact normalised name; one reference per skill and per code.
    let mut by_name: BTreeMap<String, Vec<&Skill>> = BTreeMap::new();
    for s in &parsed {
        by_name.entry(normalise_label(&s.name)).or_default().push(s);
    }
    let refs = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(SFIA_SLUG))
        .all(&txn)
        .await?;
    let mut taken: BTreeSet<String> = refs.iter().map(|r| r.reference.clone()).collect();
    let linked: BTreeSet<Uuid> = refs.iter().map(|r| r.skill_pid).collect();
    let catalogue = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(&txn)
        .await?;
    for skill in &catalogue {
        if linked.contains(&skill.pid) {
            continue;
        }
        if let Some([only]) = by_name
            .get(&normalise_label(&skill.name))
            .map(Vec::as_slice)
            && taken.insert(only.code.clone())
        {
            skill_external_refs::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                skill_pid: ActiveValue::set(skill.pid),
                framework_slug: ActiveValue::set(SFIA_SLUG.to_string()),
                reference: ActiveValue::set(only.code.clone()),
                label: ActiveValue::set(Some(only.name.clone())),
                version: ActiveValue::set(Some(version.to_string())),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
            report.catalogue_linked += 1;
        }
    }
    txn.commit().await?;
    Ok(report)
}
