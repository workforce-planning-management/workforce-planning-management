//! `cargo run -- task import_framework dir:<path>` — import a **job-capability
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

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
        let report = import_pcf(
            crate::db::pool(ctx),
            Path::new(dir),
            mapping,
            overwrite_levels,
        )
        .await?;
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

/// Whether a live role profile already has `title`.
async fn title_in_use(tx: &mut sqlx::PgConnection, title: &str) -> Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM role_profiles WHERE job_title = $1 AND deleted_at IS NULL) AS "e!""#,
        title,
    )
    .fetch_one(tx)
    .await?)
}

/// Import the PCF from the clone at `dir`.
///
/// # Errors
/// A filesystem or database error.
#[allow(clippy::too_many_lines)] // read → plan titles → upsert framework → upsert levels
pub async fn import_pcf(
    pool: &sqlx::PgPool,
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
    // A later import changes only the day.
    sqlx::query!(
        "INSERT INTO capability_frameworks
             (pid, slug, name, source_url, licence, attribution, scale_max, scale_labels, imported_on, note)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (slug) DO UPDATE SET imported_on = EXCLUDED.imported_on",
        Uuid::new_v4(),
        PCF_SLUG,
        "UK Government Digital and Data Profession Capability Framework",
        "https://ddat-capability-framework.service.gov.uk/",
        "Open Government Licence v3.0",
        "Contains public sector information licensed under the Open Government \
         Licence v3.0. © Crown copyright. Adapted from the Government Digital and \
         Data Profession Capability Framework.",
        rules::PCF_SCALE_MAX,
        "1 Awareness; 2 Working; 3 Practitioner; 4 Expert",
        today,
        framework_note,
    )
    .execute(pool)
    .await?;

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

        let mut tx = pool.begin().await?;
        // The profile: by framework reference first, else claim an unused title.
        let by_ref = sqlx::query!(
            "SELECT pid, job_title FROM role_profiles
              WHERE framework_slug = $1 AND external_ref = $2 AND deleted_at IS NULL LIMIT 1",
            PCF_SLUG,
            item.slug,
        )
        .fetch_optional(&mut *tx)
        .await?;
        let profile_pid = if let Some(row) = by_ref {
            let title_free = row.job_title == title || !title_in_use(&mut tx, &title).await?;
            sqlx::query!(
                "UPDATE role_profiles
                    SET job_title = CASE WHEN $2 THEN $3 ELSE job_title END, description = $4, profession = $5,
                        role_name = $6, level_name = $7, level_order = $8, management_track = $9
                  WHERE pid = $1",
                row.pid,
                title_free,
                title,
                description,
                item.parsed.profession_label,
                item.parsed.role,
                item.parsed.level,
                item.parts.order,
                rules::is_management_track(&item.parsed.level),
            )
            .execute(&mut *tx)
            .await?;
            report.profiles_updated += 1;
            row.pid
        } else {
            if title_in_use(&mut tx, &title).await? {
                report.title_taken_skipped += 1;
                tx.rollback().await?;
                continue;
            }
            let pid = Uuid::new_v4();
            sqlx::query!(
                "INSERT INTO role_profiles
                     (pid, job_title, description, source_ref, framework_slug, external_ref, profession, role_name,
                      level_name, level_order, management_track)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                pid,
                title,
                description,
                format!("{PCF_SLUG}:{}", item.slug),
                PCF_SLUG,
                item.slug,
                item.parsed.profession_label,
                item.parsed.role,
                item.parsed.level,
                item.parts.order,
                rules::is_management_track(&item.parsed.level),
            )
            .execute(&mut *tx)
            .await?;
            report.profiles_created += 1;
            pid
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
            let by_ref = sqlx::query_scalar!(
                r#"SELECT skill_pid FROM skill_external_refs WHERE framework_slug = $1 AND "ref" = $2 LIMIT 1"#,
                PCF_SLUG,
                skill.name,
            )
            .fetch_optional(&mut *tx)
            .await?;
            let skill_row = match by_ref {
                Some(skill_pid) => {
                    sqlx::query!(
                        r#"SELECT pid, deleted_at IS NOT NULL AS "deleted!" FROM skills WHERE pid = $1 LIMIT 1"#,
                        skill_pid,
                    )
                    .fetch_optional(&mut *tx)
                    .await?
                    .map(|r| (r.pid, r.deleted))
                }
                None => sqlx::query!(
                    r#"SELECT pid, deleted_at IS NOT NULL AS "deleted!" FROM skills WHERE name = $1 LIMIT 1"#,
                    skill.name,
                )
                .fetch_optional(&mut *tx)
                .await?
                .map(|r| (r.pid, r.deleted)),
            };
            let skill_pid = if let Some((pid, deleted)) = skill_row {
                if deleted {
                    sqlx::query!("UPDATE skills SET deleted_at = NULL WHERE pid = $1", pid,)
                        .execute(&mut *tx)
                        .await?;
                }
                report.skills_reused += 1;
                pid
            } else {
                let pid = Uuid::new_v4();
                sqlx::query!(
                    "INSERT INTO skills (pid, name, category) VALUES ($1, $2, 'other')",
                    pid,
                    skill.name,
                )
                .execute(&mut *tx)
                .await?;
                report.skills_created += 1;
                pid
            };
            if by_ref.is_none() {
                let has_other_ref = sqlx::query_scalar!(
                    r#"SELECT EXISTS (SELECT 1 FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2) AS "e!""#,
                    skill_pid,
                    PCF_SLUG,
                )
                .fetch_one(&mut *tx)
                .await?;
                if !has_other_ref {
                    sqlx::query!(
                        r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label, version)
                           VALUES ($1, $2, $3, $4, $4, NULL)"#,
                        Uuid::new_v4(),
                        skill_pid,
                        PCF_SLUG,
                        skill.name,
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }
            let wording = skill.statements.join("\n");
            // A known requirement refreshes provenance and wording only: the planner owns the level.
            let refreshed = sqlx::query!(
                "UPDATE role_skill_requirements
                    SET source_framework = $3, source_level = $4, source_scale_max = $5, note = $6,
                        min_proficiency = CASE WHEN $7 THEN $8 ELSE min_proficiency END
                  WHERE role_profile_pid = $1 AND skill_pid = $2",
                profile_pid,
                skill_pid,
                PCF_SLUG,
                baseline,
                rules::PCF_SCALE_MAX,
                wording,
                overwrite_levels,
                mapped,
            )
            .execute(&mut *tx)
            .await?
            .rows_affected();
            if refreshed > 0 {
                report.requirements_refreshed += 1;
            } else {
                sqlx::query!(
                    "INSERT INTO role_skill_requirements
                         (pid, role_profile_pid, skill_pid, min_proficiency, importance, note, source_framework,
                          source_level, source_scale_max)
                     VALUES ($1, $2, $3, $4, 'important', $5, $6, $7, $8)",
                    Uuid::new_v4(),
                    profile_pid,
                    skill_pid,
                    mapped,
                    wording,
                    PCF_SLUG,
                    baseline,
                    rules::PCF_SCALE_MAX,
                )
                .execute(&mut *tx)
                .await?;
                report.requirements_created += 1;
            }
        }
        tx.commit().await?;
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
