//! `cargo run -- task import_esco dir:<path> [lang:en] [version:v1.2.1]` —
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

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

use crate::rules::esco as rules;

/// The framework slug ESCO is recorded under.
pub const ESCO_SLUG: &str = "esco";

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
        let lang = vars.cli_arg("lang").map_or("en", String::as_str);
        let version = vars
            .cli_arg("version")
            .map_or("unspecified", String::as_str);
        let report = import_esco(crate::db::pool(ctx), Path::new(dir), lang, version).await?;
        // The catalogue changed, so searches cached from the old one are orphaned. With the Redis
        // provider this reaches every instance; with the in-memory one they expire within the hour.
        let cache =
            crate::shared_cache::Cache::from_env().map_err(|cause| axa::Error::string(&cause))?;
        crate::shared_cache::install(std::sync::Arc::new(cache));
        crate::shared_cache::invalidate(crate::shared_cache::Namespace::EscoOccupations).await;
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
    pool: &sqlx::PgPool,
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

    let mut tx = pool.begin().await?;
    // Replace wholesale, so the copy is exactly one ESCO version. Each table is one insert of arrays, so no statement
    // carries more than four binds however large the file.
    sqlx::query!("DELETE FROM esco_occupation_skills")
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM esco_occupations")
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM esco_skills")
        .execute(&mut *tx)
        .await?;
    let column = |values: Vec<&str>| {
        values
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<String>>()
    };
    sqlx::query!(
        "INSERT INTO esco_skills (uri, label, skill_type, reuse_level)
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])",
        &column(skills_csv.iter().map(|s| s.uri.as_str()).collect()),
        &column(skills_csv.iter().map(|s| s.label.as_str()).collect()),
        &skills_csv
            .iter()
            .map(|s| s.skill_type.clone())
            .collect::<Vec<_>>() as &[Option<String>],
        &skills_csv
            .iter()
            .map(|s| s.reuse_level.clone())
            .collect::<Vec<_>>() as &[Option<String>],
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO esco_occupations (uri, label, isco_code, description)
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])",
        &column(occupations_csv.iter().map(|o| o.uri.as_str()).collect()),
        &column(occupations_csv.iter().map(|o| o.label.as_str()).collect()),
        &occupations_csv
            .iter()
            .map(|o| o.isco_code.clone())
            .collect::<Vec<_>>() as &[Option<String>],
        &occupations_csv
            .iter()
            .map(|o| o.description.clone())
            .collect::<Vec<_>>() as &[Option<String>],
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO esco_occupation_skills (occupation_uri, skill_uri, relation)
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])",
        &column(
            relations
                .iter()
                .map(|r| r.occupation_uri.as_str())
                .collect()
        ),
        &column(relations.iter().map(|r| r.skill_uri.as_str()).collect()),
        &column(relations.iter().map(|r| r.relation.as_str()).collect()),
    )
    .execute(&mut *tx)
    .await?;

    // The framework row: attribution and the version pinned. A later import changes only the day and the pinned version.
    let note = "ESCO has no proficiency scale: it says which skills an occupation needs \
                and whether each is essential or optional. Required levels are set by the \
                planner on WPM's own 1–5 scale.";
    let attribution = "© European Union. ESCO (European Skills, Competences, Qualifications \
                       and Occupations), reused under Commission Decision 2011/833/EU with \
                       the source acknowledged.";
    let today = chrono::Utc::now().date_naive();
    sqlx::query!(
        "INSERT INTO capability_frameworks
             (pid, slug, name, source_url, licence, attribution, scale_max, scale_labels, imported_on, note)
         VALUES ($1, $2, $3, $4, $5, $6, 5, NULL, $7, $8)
         ON CONFLICT (slug) DO UPDATE SET imported_on = EXCLUDED.imported_on, note = EXCLUDED.note",
        Uuid::new_v4(),
        ESCO_SLUG,
        "ESCO — European Skills, Competences, Qualifications and Occupations",
        "https://esco.ec.europa.eu/",
        "Reuse under Commission Decision 2011/833/EU (attribution required)",
        attribution,
        today,
        format!("{note} Pinned version: {version}."),
    )
    .execute(&mut *tx)
    .await?;

    // Link catalogue skills by exact normalised label, never by guess.
    let mut by_label: BTreeMap<String, Vec<&rules::EscoSkill>> = BTreeMap::new();
    for skill in &skills_csv {
        by_label
            .entry(rules::normalise_label(&skill.label))
            .or_default()
            .push(skill);
    }
    let refs = sqlx::query!(
        r#"SELECT ref AS "reference", skill_pid FROM skill_external_refs WHERE framework_slug = $1"#,
        ESCO_SLUG,
    )
    .fetch_all(&mut *tx)
    .await?;
    let already_linked: BTreeSet<Uuid> = refs.iter().map(|r| r.skill_pid).collect();
    let mut taken: BTreeSet<String> = refs.into_iter().map(|r| r.reference).collect();
    let catalogue =
        sqlx::query!("SELECT pid, name FROM skills WHERE deleted_at IS NULL ORDER BY id")
            .fetch_all(&mut *tx)
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
                sqlx::query!(
                    r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label, version)
                       VALUES ($1, $2, $3, $4, $5, $6)"#,
                    Uuid::new_v4(),
                    skill.pid,
                    ESCO_SLUG,
                    only.uri,
                    only.label,
                    version,
                )
                .execute(&mut *tx)
                .await?;
                report.catalogue_linked += 1;
            }
            [_, _, ..] => report.catalogue_ambiguous += 1,
            _ => {}
        }
    }
    tx.commit().await?;
    Ok(report)
}
