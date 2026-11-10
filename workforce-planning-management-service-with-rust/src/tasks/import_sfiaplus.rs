//! `cargo run -- task import_sfiaplus file:<path> version:<n> licence_checked_on:<YYYY-MM-DD>
//! [licence_holder:<text>] [licence_reference:<text>]` — load the deployer's **licensed** `SFIAplus` detail, saved as CSV to the
//! documented (provisional) contract in `rules::sfia`, as development **pointers** on SFIA skills at levels (WPM-R112).
//!
//! WPM ships none of it, and books or buys nothing from it. The task refuses a file with no recorded licence check, refuses a
//! resource for a SFIA skill or level that is not in the imported catalogue (nothing is written then), records the framework
//! as `internal_only`, and is idempotent: a resource a newer file drops is marked retired, not deleted.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

use super::import_sfia::{Licence, check_licence};
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
        let text = std::fs::read_to_string(Path::new(file))
            .map_err(|e| Error::string(&format!("{file}: {e}")))?;
        let report = import_sfiaplus(crate::db::pool(ctx), &text, &licence).await?;
        tracing::info!(?report, "SFIAplus import finished");
        println!("{report:#?}");
        Ok(())
    }
}

/// A resource's identity: SFIA code, level, kind and title.
type ResourceKey = (String, i32, String, String);

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
pub async fn import_sfiaplus(pool: &sqlx::PgPool, text: &str, licence: &Licence) -> Result<Report> {
    let checked_on = check_licence(licence)?;
    let parsed = rules::parse_resources(text).map_err(|e| Error::string(&e))?;
    if parsed.is_empty() {
        return Err(Error::string("the file has a header and no resources"));
    }
    let version = licence.version.trim();
    let mut tx = pool.begin().await?;
    let skills: BTreeSet<String> =
        sqlx::query_scalar!("SELECT code FROM sfia_skills WHERE retired_in IS NULL")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .collect();
    let levels: BTreeSet<(String, i32)> =
        sqlx::query!("SELECT code, level FROM sfia_skill_levels WHERE retired_in IS NULL")
            .fetch_all(&mut *tx)
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
    // Each resource is one row by (code, level, kind, title): a known one is refreshed and brought back, a new one added.
    let known: BTreeMap<ResourceKey, (i32, Option<String>)> =
        sqlx::query!("SELECT id, code, level, kind, title, retired_in FROM sfiaplus_resources")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|m| ((m.code, m.level, m.kind, m.title), (m.id, m.retired_in)))
            .collect();
    let mut seen = BTreeSet::new();
    for r in &parsed {
        seen.insert((r.code.clone(), r.level, r.kind.clone(), r.title.clone()));
        sqlx::query!(
            "INSERT INTO sfiaplus_resources (code, level, kind, title, provider, url) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (code, level, kind, title) DO UPDATE
                SET provider = EXCLUDED.provider, url = EXCLUDED.url, retired_in = NULL",
            r.code,
            r.level,
            r.kind,
            r.title,
            r.provider,
            r.url,
        )
        .execute(&mut *tx)
        .await?;
    }
    let mut retired = 0;
    for (key, (id, retired_in)) in &known {
        if !seen.contains(key) && retired_in.is_none() {
            sqlx::query!(
                "UPDATE sfiaplus_resources SET retired_in = $2 WHERE id = $1",
                id,
                version,
            )
            .execute(&mut *tx)
            .await?;
            retired += 1;
        }
    }

    // The framework row: the deployer's BCS licence, internal use only.
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
        SFIAPLUS_SLUG,
        "SFIAplus (BCS, The Chartered Institute for IT)",
        "https://www.bcs.org/",
        "BCS SFIAplus licence held by the deployer (internal use)",
        rules::SFIA_SCALE_MAX,
        chrono::Utc::now().date_naive(),
        "Loaded by the licensee from their own copy. Shown only as pointers: nothing is booked or bought from it. \
         WPM ships none of it; it must not be published or redistributed.",
        version,
        Redistribution::InternalOnly.token(),
        licence.holder,
        licence.reference,
        checked_on,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Report {
        resources: parsed.len(),
        retired,
    })
}
