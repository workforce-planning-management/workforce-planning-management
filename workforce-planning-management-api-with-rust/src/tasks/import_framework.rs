//! `cargo loco task import_framework dir:<path>` — import a **job-capability
//! framework** as role profiles: today the UK Government Digital and Data
//! (GDAD) Profession Capability Framework (PCF), from a local clone of the
//! community repository (`spec/uk-gdad-pcf/index.md`).
//!
//! Structure — profession, role, level, skills, and the framework's wording
//! — comes from the **role summaries** (Crown copyright, OGL v3.0; the
//! framework is attributed on the imported framework row). A skill's
//! required level comes from the **baseline** in the community competency
//! assessments (AI-assisted, human-reviewed — a draft for a human to edit);
//! a skill with no baseline is **skipped and counted**, never given an
//! invented level. Levels on the PCF's 1–4 scale are converted to WPM's 1–5
//! by [`crate::rules::framework::map_level`] (`scale:identity` — the default,
//! PCF 3 is WPM 3 — or `scale:linear`), with the source level kept.
//!
//! Idempotent. Re-importing refreshes provenance, titles, the framework's
//! wording, and the source level, but never overwrites a planner's
//! `min_proficiency` or `importance` (unless `overwrite_levels:true`, which
//! re-derives every minimum from the source level), and never removes a
//! requirement.

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, DatabaseConnection, TransactionTrait};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::models::_entities::{
    capability_frameworks, role_profiles, role_skill_requirements, skill_external_refs, skills,
};
use crate::rules::framework as rules;

/// The framework slug this task writes.
pub const PCF_SLUG: &str = "uk-gdad-pcf";

/// The import task.
pub struct ImportFramework;

#[async_trait]
impl Task for ImportFramework {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "import_framework".to_string(),
            detail: "Import the UK GDAD PCF as role profiles (dir:<path to uk-gdad/uk-gdad> [scale:identity|linear] [overwrite_levels:true])"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let dir = vars
            .cli_arg("dir")
            .map_err(|_| Error::string("dir:<path to the uk-gdad clone> is required"))?;
        let mapping = match vars.cli_arg("scale") {
            Ok(text) => rules::LevelMapping::parse(text)
                .ok_or_else(|| Error::string("scale must be `identity` or `linear`"))?,
            Err(_) => rules::LevelMapping::default(),
        };
        let overwrite_levels = vars.cli_arg("overwrite_levels").is_ok_and(|v| v == "true");
        let report = import_pcf(&ctx.db, Path::new(dir), mapping, overwrite_levels).await?;
        tracing::info!(?report, "framework import finished");
        println!("{report:#?}");
        Ok(())
    }
}

/// What an import did.
#[derive(Debug, Default)]
pub struct Report {
    /// Role levels read.
    pub levels_read: usize,
    /// Retired levels left out.
    pub retired_skipped: usize,
    /// Profiles created.
    pub profiles_created: usize,
    /// Profiles refreshed.
    pub profiles_updated: usize,
    /// Levels whose job title already belongs to a non-framework profile.
    pub title_taken_skipped: usize,
    /// Skills created in the catalogue.
    pub skills_created: usize,
    /// Skills that already existed (matched by exact name).
    pub skills_reused: usize,
    /// Requirements created.
    pub requirements_created: usize,
    /// Requirements refreshed (level and wording only).
    pub requirements_refreshed: usize,
    /// Skill lines left out because no baseline level is stated.
    pub requirements_skipped_no_baseline: usize,
    /// Distinct skill names with no baseline anywhere.
    pub skills_without_any_baseline: usize,
}

/// One role level ready to import.
struct Item {
    slug: String,
    parts: rules::SlugParts,
    parsed: rules::ParsedLevel,
    baselines: BTreeMap<String, i32>,
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| Error::string(&format!("cannot read {}: {e}", dir.display())))?;
        for entry in entries {
            let path = entry.map_err(|e| Error::string(&e.to_string()))?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn slug_of(root: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(root).ok()?.with_extension("");
    Some(rel.to_string_lossy().replace('\\', "/"))
}

/// Import the PCF from the clone at `dir`.
///
/// # Errors
/// A filesystem or database error.
#[allow(clippy::too_many_lines)] // read → plan titles → upsert framework → upsert levels
pub async fn import_pcf(
    db: &DatabaseConnection,
    dir: &Path,
    mapping: rules::LevelMapping,
    overwrite_levels: bool,
) -> Result<Report> {
    let summaries = dir.join("uk-gdad-pcf-role-summaries/roles");
    let assessments = dir.join("uk-gdad-pcf-competency-assessments-by-assessor/roles");
    if !summaries.is_dir() {
        return Err(Error::string(&format!(
            "{} not found — dir must be the uk-gdad clone",
            summaries.display()
        )));
    }
    let mut report = Report::default();

    let mut items: Vec<Item> = Vec::new();
    for file in markdown_files(&summaries)? {
        let Some(slug) = slug_of(&summaries, &file) else {
            continue;
        };
        let Some(parts) = rules::parse_slug(&slug) else {
            continue;
        };
        let text = std::fs::read_to_string(&file)
            .map_err(|e| Error::string(&format!("{}: {e}", file.display())))?;
        let parsed =
            rules::parse_role_summary(&text).map_err(|e| Error::string(&format!("{slug}: {e}")))?;
        report.levels_read += 1;
        if parsed.retired || parts.not_in_use {
            report.retired_skipped += 1;
            continue;
        }
        let baselines = std::fs::read_to_string(assessments.join(format!("{slug}.md")))
            .map(|t| rules::parse_baselines(&t))
            .unwrap_or_default();
        items.push(Item {
            slug,
            parts,
            parsed,
            baselines,
        });
    }

    // A level name used by more than one role is qualified with its role.
    let mut base_counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in &items {
        let base = rules::job_title(&item.parsed.level, &item.parsed.role, false);
        *base_counts.entry(base).or_default() += 1;
    }

    // Framework row.
    let today = chrono::Utc::now().date_naive();
    let framework_note = "Role structure and wording: the official framework role summaries. \
                          Required levels: baselines from the community competency assessments \
                          (AI-assisted, human-reviewed) — a draft for a human to edit.";
    let existing = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(PCF_SLUG))
        .one(db)
        .await?;
    match existing {
        Some(row) => {
            let mut active: capability_frameworks::ActiveModel = row.into();
            active.imported_on = ActiveValue::set(today);
            active.update(db).await?;
        }
        None => {
            capability_frameworks::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                slug: ActiveValue::set(PCF_SLUG.to_string()),
                name: ActiveValue::set(
                    "UK Government Digital and Data Profession Capability Framework".to_string(),
                ),
                source_url: ActiveValue::set(Some(
                    "https://ddat-capability-framework.service.gov.uk/".to_string(),
                )),
                licence: ActiveValue::set(Some("Open Government Licence v3.0".to_string())),
                attribution: ActiveValue::set(Some(
                    "Contains public sector information licensed under the Open Government \
                     Licence v3.0. © Crown copyright. Adapted from the Government Digital and \
                     Data Profession Capability Framework."
                        .to_string(),
                )),
                scale_max: ActiveValue::set(rules::PCF_SCALE_MAX),
                scale_labels: ActiveValue::set(Some(
                    "1 Awareness; 2 Working; 3 Practitioner; 4 Expert".to_string(),
                )),
                imported_on: ActiveValue::set(today),
                note: ActiveValue::set(Some(framework_note.to_string())),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
    }

    let mut no_baseline: BTreeSet<String> = BTreeSet::new();
    for item in &items {
        let collision = base_counts
            .get(&rules::job_title(
                &item.parsed.level,
                &item.parsed.role,
                false,
            ))
            .is_some_and(|n| *n > 1);
        let title = rules::job_title(&item.parsed.level, &item.parsed.role, collision);
        let description = [
            item.parsed.role_description.as_deref(),
            item.parsed.level_description.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n\n");

        let txn = db.begin().await?;
        // The profile: by framework reference first, else claim an unused title.
        let by_ref = role_profiles::Entity::find()
            .filter(role_profiles::Column::FrameworkSlug.eq(PCF_SLUG))
            .filter(role_profiles::Column::ExternalRef.eq(&item.slug))
            .filter(role_profiles::Column::DeletedAt.is_null())
            .one(&txn)
            .await?;
        let profile_pid = if let Some(row) = by_ref {
            let pid = row.pid;
            let title_free = row.job_title == title
                || role_profiles::Entity::find()
                    .filter(role_profiles::Column::JobTitle.eq(&title))
                    .filter(role_profiles::Column::DeletedAt.is_null())
                    .one(&txn)
                    .await?
                    .is_none();
            let mut active: role_profiles::ActiveModel = row.into();
            if title_free {
                active.job_title = ActiveValue::set(title.clone());
            }
            active.description = ActiveValue::set(Some(description));
            active.profession = ActiveValue::set(Some(item.parsed.profession_label.clone()));
            active.role_name = ActiveValue::set(Some(item.parsed.role.clone()));
            active.level_name = ActiveValue::set(Some(item.parsed.level.clone()));
            active.level_order = ActiveValue::set(item.parts.order);
            active.management_track =
                ActiveValue::set(rules::is_management_track(&item.parsed.level));
            active.update(&txn).await?;
            report.profiles_updated += 1;
            pid
        } else {
            let taken = role_profiles::Entity::find()
                .filter(role_profiles::Column::JobTitle.eq(&title))
                .filter(role_profiles::Column::DeletedAt.is_null())
                .one(&txn)
                .await?
                .is_some();
            if taken {
                report.title_taken_skipped += 1;
                txn.rollback().await?;
                continue;
            }
            let row = role_profiles::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                job_title: ActiveValue::set(title),
                description: ActiveValue::set(Some(description)),
                source_ref: ActiveValue::set(Some(format!("{PCF_SLUG}:{}", item.slug))),
                framework_slug: ActiveValue::set(Some(PCF_SLUG.to_string())),
                external_ref: ActiveValue::set(Some(item.slug.clone())),
                profession: ActiveValue::set(Some(item.parsed.profession_label.clone())),
                role_name: ActiveValue::set(Some(item.parsed.role.clone())),
                level_name: ActiveValue::set(Some(item.parsed.level.clone())),
                level_order: ActiveValue::set(item.parts.order),
                management_track: ActiveValue::set(rules::is_management_track(&item.parsed.level)),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
            report.profiles_created += 1;
            row.pid
        };

        for skill in &item.parsed.skills {
            let Some(&baseline) = item.baselines.get(&skill.name) else {
                report.requirements_skipped_no_baseline += 1;
                no_baseline.insert(skill.name.clone());
                continue;
            };
            let Some(mapped) = rules::map_level(
                mapping,
                baseline,
                rules::PCF_SCALE_MAX,
                rules::WPM_SCALE_MAX,
            ) else {
                report.requirements_skipped_no_baseline += 1;
                no_baseline.insert(skill.name.clone());
                continue;
            };
            // The skill: reuse by exact name (undeleting if needed), else create.
            // Match by the framework reference first (so a planner's rename
            // survives), then by exact name; record the reference.
            let by_ref = skill_external_refs::Entity::find()
                .filter(skill_external_refs::Column::FrameworkSlug.eq(PCF_SLUG))
                .filter(skill_external_refs::Column::Reference.eq(&skill.name))
                .one(&txn)
                .await?;
            let skill_row = match &by_ref {
                Some(r) => {
                    skills::Entity::find()
                        .filter(skills::Column::Pid.eq(r.skill_pid))
                        .one(&txn)
                        .await?
                }
                None => {
                    skills::Entity::find()
                        .filter(skills::Column::Name.eq(&skill.name))
                        .one(&txn)
                        .await?
                }
            };
            let skill_pid = if let Some(row) = skill_row {
                let pid = row.pid;
                if row.deleted_at.is_some() {
                    let mut active: skills::ActiveModel = row.into();
                    active.deleted_at = ActiveValue::set(None);
                    active.update(&txn).await?;
                }
                report.skills_reused += 1;
                pid
            } else {
                let row = skills::ActiveModel {
                    pid: ActiveValue::set(Uuid::new_v4()),
                    name: ActiveValue::set(skill.name.clone()),
                    category: ActiveValue::set("other".to_string()),
                    deleted_at: ActiveValue::set(None),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
                report.skills_created += 1;
                row.pid
            };
            if by_ref.is_none() {
                let has_other_ref = skill_external_refs::Entity::find()
                    .filter(skill_external_refs::Column::SkillPid.eq(skill_pid))
                    .filter(skill_external_refs::Column::FrameworkSlug.eq(PCF_SLUG))
                    .one(&txn)
                    .await?
                    .is_some();
                if !has_other_ref {
                    skill_external_refs::ActiveModel {
                        pid: ActiveValue::set(Uuid::new_v4()),
                        skill_pid: ActiveValue::set(skill_pid),
                        framework_slug: ActiveValue::set(PCF_SLUG.to_string()),
                        reference: ActiveValue::set(skill.name.clone()),
                        label: ActiveValue::set(Some(skill.name.clone())),
                        version: ActiveValue::set(None),
                        ..Default::default()
                    }
                    .insert(&txn)
                    .await?;
                }
            }
            let wording = skill.statements.join("\n");
            let existing = role_skill_requirements::Entity::find()
                .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile_pid))
                .filter(role_skill_requirements::Column::SkillPid.eq(skill_pid))
                .one(&txn)
                .await?;
            if let Some(row) = existing {
                // Refresh provenance and wording only: the planner owns the level.
                let mut active: role_skill_requirements::ActiveModel = row.into();
                active.source_level = ActiveValue::set(Some(baseline));
                active.source_scale_max = ActiveValue::set(Some(rules::PCF_SCALE_MAX));
                active.note = ActiveValue::set(Some(wording));
                if overwrite_levels {
                    active.min_proficiency = ActiveValue::set(mapped);
                }
                active.update(&txn).await?;
                report.requirements_refreshed += 1;
            } else {
                role_skill_requirements::ActiveModel {
                    pid: ActiveValue::set(Uuid::new_v4()),
                    role_profile_pid: ActiveValue::set(profile_pid),
                    skill_pid: ActiveValue::set(skill_pid),
                    min_proficiency: ActiveValue::set(mapped),
                    importance: ActiveValue::set("important".to_string()),
                    note: ActiveValue::set(Some(wording)),
                    source_level: ActiveValue::set(Some(baseline)),
                    source_scale_max: ActiveValue::set(Some(rules::PCF_SCALE_MAX)),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
                report.requirements_created += 1;
            }
        }
        txn.commit().await?;
    }
    // Names that never had a baseline on any level they appear in.
    let with_baseline: BTreeSet<&str> = items
        .iter()
        .flat_map(|i| i.baselines.keys().map(String::as_str))
        .collect();
    report.skills_without_any_baseline = no_baseline
        .iter()
        .filter(|n| !with_baseline.contains(n.as_str()))
        .count();
    Ok(report)
}
