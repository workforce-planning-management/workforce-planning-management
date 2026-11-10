//! `cargo run -- task import_sfia file:<path> [levels_file:<path>] version:<8|9> licence_checked_on:<YYYY-MM-DD>
//! [licence_holder:<text>] [licence_reference:<text>]` — load the deployer's
//! **licensed** SFIA download, saved as CSV to the documented contract
//! (`rules::sfia`), into the local catalogue (WPM-R109, WPM-R113, WPM-D69).
//!
//! WPM ships no SFIA content. The task refuses a file with no recorded
//! licence check, records the framework as `internal_only`, and is
//! idempotent: skills and levels a newer version drops are marked retired,
//! not deleted. A catalogue skill is linked to a SFIA skill only on an exact
//! normalised name match, never by guess.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

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
            version: vars
                .cli_arg("version")
                .map_or("", String::as_str)
                .to_string(),
            checked_on: vars
                .cli_arg("licence_checked_on")
                .map_or("", String::as_str)
                .to_string(),
            holder: vars.cli_arg("licence_holder").ok().cloned(),
            reference: vars.cli_arg("licence_reference").ok().cloned(),
        };
        let read = |f: &str| {
            std::fs::read_to_string(Path::new(f)).map_err(|e| Error::string(&format!("{f}: {e}")))
        };
        if let Some(f) = file {
            let report = import_sfia(crate::db::pool(ctx), &read(f)?, &licence).await?;
            tracing::info!(?report, "SFIA skills import finished");
            println!("{report:#?}");
        }
        if let Some(f) = levels_file {
            let n = import_sfia_levels(crate::db::pool(ctx), &read(f)?, &licence).await?;
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
    pool: &sqlx::PgPool,
    text: &str,
    licence: &Licence,
) -> Result<usize> {
    check_licence(licence)?;
    let levels = rules::parse_levels(text).map_err(|e| Error::string(&e))?;
    let version = licence.version.trim();
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "INSERT INTO job_level_frameworks (slug, name, organization, track, source, version, redistribution)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (slug) DO UPDATE
            SET name = EXCLUDED.name, organization = EXCLUDED.organization, track = EXCLUDED.track,
                source = EXCLUDED.source, version = EXCLUDED.version, redistribution = EXCLUDED.redistribution",
        SFIA_LEVELS_SLUG,
        "SFIA levels of responsibility",
        "SFIA Foundation",
        "skills",
        "Loaded by the licensee from their own SFIA download; not for publication or redistribution.",
        version,
        Redistribution::InternalOnly.token(),
    )
    .execute(&mut *tx)
    .await?;
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
        sqlx::query!(
            "INSERT INTO job_level_levels (framework, number, code, title, summary, attributes)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (framework, number) DO UPDATE
                SET code = EXCLUDED.code, title = EXCLUDED.title, summary = EXCLUDED.summary,
                    attributes = EXCLUDED.attributes",
            SFIA_LEVELS_SLUG,
            l.level,
            format!("L{}", l.level),
            l.title,
            l.essence,
            attributes,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(levels.len())
}

/// Load the contract text, replacing nothing: upsert, then retire what is gone.
///
/// # Errors
/// A refusal naming the licence field or the file row, or a database error.
#[allow(clippy::too_many_lines)] // validate → upsert → retire → link, linearly
pub async fn import_sfia(pool: &sqlx::PgPool, text: &str, licence: &Licence) -> Result<Report> {
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

    let mut tx = pool.begin().await?;
    let known: BTreeMap<String, Option<String>> =
        sqlx::query!("SELECT code, retired_in FROM sfia_skills")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|s| (s.code, s.retired_in))
            .collect();
    let known_levels: BTreeMap<(String, i32), Option<String>> =
        sqlx::query!("SELECT code, level, retired_in FROM sfia_skill_levels")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|l| ((l.code, l.level), l.retired_in))
            .collect();
    let mut seen_skills: BTreeSet<&str> = BTreeSet::new();
    let mut seen_levels: BTreeSet<(&str, i32)> = BTreeSet::new();
    for skill in &parsed {
        seen_skills.insert(&skill.code);
        // A known skill is refreshed and brought back; a new one is added.
        sqlx::query!(
            "INSERT INTO sfia_skills (code, name, category, subcategory, description) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (code) DO UPDATE
                SET name = EXCLUDED.name, category = EXCLUDED.category, subcategory = EXCLUDED.subcategory,
                    description = EXCLUDED.description, retired_in = NULL",
            skill.code,
            skill.name,
            skill.category,
            skill.subcategory,
            skill.description,
        )
        .execute(&mut *tx)
        .await?;
        for level in &skill.levels {
            seen_levels.insert((&skill.code, level.level));
            sqlx::query!(
                "INSERT INTO sfia_skill_levels (code, level, description, attributes) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (code, level) DO UPDATE
                    SET description = EXCLUDED.description, attributes = EXCLUDED.attributes, retired_in = NULL",
                skill.code,
                level.level,
                level.description,
                level.attributes,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    // What a newer version drops stays, marked, so declared skills never dangle.
    for (code, retired_in) in &known {
        if !seen_skills.contains(code.as_str()) && retired_in.is_none() {
            sqlx::query!(
                "UPDATE sfia_skills SET retired_in = $2 WHERE code = $1",
                code,
                version,
            )
            .execute(&mut *tx)
            .await?;
            report.skills_retired += 1;
        }
    }
    for ((code, level), retired_in) in &known_levels {
        if !seen_levels.contains(&(code.as_str(), *level)) && retired_in.is_none() {
            sqlx::query!(
                "UPDATE sfia_skill_levels SET retired_in = $3 WHERE code = $1 AND level = $2",
                code,
                level,
                version,
            )
            .execute(&mut *tx)
            .await?;
            report.levels_retired += 1;
        }
    }

    // The framework row: internal use only, whatever it said before.
    let today = chrono::Utc::now().date_naive();
    let note = "Loaded by the licensee from their own download. WPM ships no SFIA content; \
                it must not be published or redistributed.";
    sqlx::query!(
        "INSERT INTO capability_frameworks
             (pid, slug, name, source_url, licence, scale_max, imported_on, note, version, redistribution, licence_holder,
              licence_reference, licence_checked_on)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         ON CONFLICT (slug) DO UPDATE
            SET name = EXCLUDED.name, source_url = EXCLUDED.source_url, licence = EXCLUDED.licence,
                scale_max = EXCLUDED.scale_max, imported_on = EXCLUDED.imported_on, note = EXCLUDED.note,
                version = EXCLUDED.version, redistribution = EXCLUDED.redistribution,
                licence_holder = EXCLUDED.licence_holder, licence_reference = EXCLUDED.licence_reference,
                licence_checked_on = EXCLUDED.licence_checked_on",
        Uuid::new_v4(),
        SFIA_SLUG,
        "SFIA — Skills Framework for the Information Age",
        "https://sfia-online.org/",
        "SFIA Foundation corporate licence (internal use)",
        rules::SFIA_SCALE_MAX,
        today,
        note,
        version,
        Redistribution::InternalOnly.token(),
        licence.holder,
        licence.reference,
        checked_on,
    )
    .execute(&mut *tx)
    .await?;

    // Link by exact normalised name; one reference per skill and per code.
    let mut by_name: BTreeMap<String, Vec<&Skill>> = BTreeMap::new();
    for s in &parsed {
        by_name.entry(normalise_label(&s.name)).or_default().push(s);
    }
    let refs = sqlx::query!(
        r#"SELECT ref AS "reference", skill_pid FROM skill_external_refs WHERE framework_slug = $1"#,
        SFIA_SLUG,
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut taken: BTreeSet<String> = refs.iter().map(|r| r.reference.clone()).collect();
    let linked: BTreeSet<Uuid> = refs.iter().map(|r| r.skill_pid).collect();
    let catalogue =
        sqlx::query!("SELECT pid, name FROM skills WHERE deleted_at IS NULL ORDER BY id")
            .fetch_all(&mut *tx)
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
            sqlx::query!(
                r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label, version)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
                Uuid::new_v4(),
                skill.pid,
                SFIA_SLUG,
                only.code,
                only.name,
                version,
            )
            .execute(&mut *tx)
            .await?;
            report.catalogue_linked += 1;
        }
    }
    tx.commit().await?;
    Ok(report)
}
