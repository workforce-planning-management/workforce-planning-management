//! `cargo loco task import_esco dir:<path> [lang:en] [version:v1.2.1]` —
//! load a pinned **ESCO** CSV download into local reference tables
//! (`spec/esco/index.md`), then link catalogue skills to ESCO skills by an
//! **exact** (normalised) label match.
//!
//! Expects `skills_<lang>.csv`, `occupations_<lang>.csv`, and
//! `occupationSkillRelations_<lang>.csv` from the ESCO classification
//! download (CSV), columns resolved by header name. The tables are
//! **replaced wholesale** so a given ESCO version is reproducible. ESCO is
//! reused under Commission Decision 2011/833/EU with attribution; the
//! `esco` row in `capability_frameworks` carries it and the version.
//!
//! Linking never guesses: a catalogue skill is linked only when exactly one
//! ESCO skill has the same normalised label, the ESCO skill is not already
//! referenced by another catalogue skill, and the skill has no ESCO
//! reference yet. Anything else is left for a planner (`POST
//! /api/skills/{pid}/refs`).

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, DatabaseConnection, TransactionTrait};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

use crate::models::_entities::{
    capability_frameworks, esco_occupation_skills, esco_occupations, esco_skills,
    skill_external_refs, skills,
};
use crate::rules::esco as rules;

/// The framework slug ESCO is recorded under.
pub const ESCO_SLUG: &str = "esco";

const BATCH: usize = 2000;

/// The ESCO import task.
pub struct ImportEsco;

#[async_trait]
impl Task for ImportEsco {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "import_esco".to_string(),
            detail: "Load a pinned ESCO CSV download (dir:<path> [lang:en] [version:v1.2.1]) and link catalogue skills by exact label"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let dir = vars
            .cli_arg("dir")
            .map_err(|_| Error::string("dir:<path to the ESCO CSV files> is required"))?;
        let lang = vars.cli_arg("lang").unwrap_or("en");
        let version = vars.cli_arg("version").unwrap_or("unspecified");
        let report = import_esco(&ctx.db, Path::new(dir), lang, version).await?;
        tracing::info!(?report, "ESCO import finished");
        println!("{report:#?}");
        Ok(())
    }
}

/// What an import did.
#[derive(Debug, Default)]
pub struct Report {
    /// ESCO skills loaded.
    pub skills: usize,
    /// ESCO occupations loaded.
    pub occupations: usize,
    /// Relations loaded.
    pub relations: usize,
    /// Relations dropped because their occupation or skill is not in the files.
    pub relations_dangling: usize,
    /// Catalogue skills newly linked by exact label.
    pub catalogue_linked: usize,
    /// Catalogue skills left unlinked because the label matched several ESCO skills.
    pub catalogue_ambiguous: usize,
}

fn read(dir: &Path, name: &str) -> Result<String> {
    let path = dir.join(name);
    std::fs::read_to_string(&path).map_err(|e| Error::string(&format!("{}: {e}", path.display())))
}

/// Load the ESCO CSVs in `dir`, replacing the local copy.
///
/// # Errors
/// A filesystem, parse, or database error.
#[allow(clippy::too_many_lines)] // read → parse → replace → link, linearly
pub async fn import_esco(
    db: &DatabaseConnection,
    dir: &Path,
    lang: &str,
    version: &str,
) -> Result<Report> {
    let skills_csv = rules::parse_skills(&read(dir, &format!("skills_{lang}.csv"))?)
        .map_err(|e| Error::string(&e))?;
    let occupations_csv = rules::parse_occupations(&read(dir, &format!("occupations_{lang}.csv"))?)
        .map_err(|e| Error::string(&e))?;
    let relations_csv =
        rules::parse_relations(&read(dir, &format!("occupationSkillRelations_{lang}.csv"))?)
            .map_err(|e| Error::string(&e))?;

    let skill_uris: BTreeSet<&str> = skills_csv.iter().map(|s| s.uri.as_str()).collect();
    let occupation_uris: BTreeSet<&str> = occupations_csv.iter().map(|o| o.uri.as_str()).collect();
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    let relations: Vec<&rules::EscoRelation> = relations_csv
        .iter()
        .filter(|r| {
            skill_uris.contains(r.skill_uri.as_str())
                && occupation_uris.contains(r.occupation_uri.as_str())
                && seen.insert((r.occupation_uri.as_str(), r.skill_uri.as_str()))
        })
        .collect();
    let mut report = Report {
        skills: skills_csv.len(),
        occupations: occupations_csv.len(),
        relations: relations.len(),
        relations_dangling: relations_csv.len() - relations.len(),
        ..Report::default()
    };

    let txn = db.begin().await?;
    // Replace wholesale, so the copy is exactly one ESCO version.
    esco_occupation_skills::Entity::delete_many()
        .exec(&txn)
        .await?;
    esco_occupations::Entity::delete_many().exec(&txn).await?;
    esco_skills::Entity::delete_many().exec(&txn).await?;
    for chunk in skills_csv.chunks(BATCH) {
        esco_skills::Entity::insert_many(chunk.iter().map(|s| esco_skills::ActiveModel {
            uri: ActiveValue::set(s.uri.clone()),
            label: ActiveValue::set(s.label.clone()),
            skill_type: ActiveValue::set(s.skill_type.clone()),
            reuse_level: ActiveValue::set(s.reuse_level.clone()),
            ..Default::default()
        }))
        .exec(&txn)
        .await?;
    }
    for chunk in occupations_csv.chunks(BATCH) {
        esco_occupations::Entity::insert_many(chunk.iter().map(|o| {
            esco_occupations::ActiveModel {
                uri: ActiveValue::set(o.uri.clone()),
                label: ActiveValue::set(o.label.clone()),
                isco_code: ActiveValue::set(o.isco_code.clone()),
                description: ActiveValue::set(o.description.clone()),
                ..Default::default()
            }
        }))
        .exec(&txn)
        .await?;
    }
    for chunk in relations.chunks(BATCH) {
        esco_occupation_skills::Entity::insert_many(chunk.iter().map(|r| {
            esco_occupation_skills::ActiveModel {
                occupation_uri: ActiveValue::set(r.occupation_uri.clone()),
                skill_uri: ActiveValue::set(r.skill_uri.clone()),
                relation: ActiveValue::set(r.relation.as_str().to_string()),
                ..Default::default()
            }
        }))
        .exec(&txn)
        .await?;
    }

    // The framework row: attribution and the version pinned.
    let note = "ESCO has no proficiency scale: it says which skills an occupation needs \
                and whether each is essential or optional. Required levels are set by the \
                planner on WPM's own 1–5 scale.";
    let attribution = "© European Union. ESCO (European Skills, Competences, Qualifications \
                       and Occupations), reused under Commission Decision 2011/833/EU with \
                       the source acknowledged.";
    let existing = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(ESCO_SLUG))
        .one(&txn)
        .await?;
    let today = chrono::Utc::now().date_naive();
    if let Some(row) = existing {
        let mut active: capability_frameworks::ActiveModel = row.into();
        active.imported_on = ActiveValue::set(today);
        active.note = ActiveValue::set(Some(format!("{note} Pinned version: {version}.")));
        active.update(&txn).await?;
    } else {
        capability_frameworks::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            slug: ActiveValue::set(ESCO_SLUG.to_string()),
            name: ActiveValue::set(
                "ESCO — European Skills, Competences, Qualifications and Occupations".to_string(),
            ),
            source_url: ActiveValue::set(Some("https://esco.ec.europa.eu/".to_string())),
            licence: ActiveValue::set(Some(
                "Reuse under Commission Decision 2011/833/EU (attribution required)".to_string(),
            )),
            attribution: ActiveValue::set(Some(attribution.to_string())),
            scale_max: ActiveValue::set(5),
            scale_labels: ActiveValue::set(None),
            imported_on: ActiveValue::set(today),
            note: ActiveValue::set(Some(format!("{note} Pinned version: {version}."))),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }

    // Link catalogue skills by exact normalised label, never by guess.
    let mut by_label: BTreeMap<String, Vec<&rules::EscoSkill>> = BTreeMap::new();
    for skill in &skills_csv {
        by_label
            .entry(rules::normalise_label(&skill.label))
            .or_default()
            .push(skill);
    }
    let referenced_uris: BTreeSet<String> = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
        .all(&txn)
        .await?
        .into_iter()
        .map(|r| r.reference)
        .collect();
    let already_linked: BTreeSet<Uuid> = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
        .all(&txn)
        .await?
        .into_iter()
        .map(|r| r.skill_pid)
        .collect();
    let mut taken = referenced_uris;
    let catalogue = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(&txn)
        .await?;
    for skill in &catalogue {
        if already_linked.contains(&skill.pid) {
            continue;
        }
        let Some(candidates) = by_label.get(&rules::normalise_label(&skill.name)) else {
            continue;
        };
        match candidates.as_slice() {
            [only] if !taken.contains(&only.uri) => {
                taken.insert(only.uri.clone());
                skill_external_refs::ActiveModel {
                    pid: ActiveValue::set(Uuid::new_v4()),
                    skill_pid: ActiveValue::set(skill.pid),
                    framework_slug: ActiveValue::set(ESCO_SLUG.to_string()),
                    reference: ActiveValue::set(only.uri.clone()),
                    label: ActiveValue::set(Some(only.label.clone())),
                    version: ActiveValue::set(Some(version.to_string())),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
                report.catalogue_linked += 1;
            }
            [_, _, ..] => report.catalogue_ambiguous += 1,
            _ => {}
        }
    }
    txn.commit().await?;
    Ok(report)
}
