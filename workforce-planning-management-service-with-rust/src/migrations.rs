//! **The schema migrations** (WPM-D193): one plain SQL file per migration in `migrations/`, applied in order by `sqlx`, each in
//! its own transaction. They are the 133 `SeaORM` migrations they replace, statement for statement, under the same names, so a
//! fresh database ends up with the same schema and rows either way.
//!
//! A database a `SeaORM` release migrated is adopted, not migrated again: [`up`] reads `seaql_migrations` once, when `sqlx` has
//! recorded nothing, and records those migrations as applied ([`sqlx::migrate::Migrator::skip`]), so only what is newer runs.
//! `SeaORM` applies its migrations in order, so what it recorded is always every migration up to some point; anything else (an
//! unknown name, or a gap) stops the boot with the name, because guessing would run a migration twice or not at all.

use axa::prelude::*;
use sqlx::PgPool;
use sqlx::migrate::Migrator;

/// Every migration, built into the binary.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// The migration that keeps a telephone number three ways (WPM-D152 question 2, WPM-T382), as it runs.
pub const PHONE_FORMS_SQL: &str = include_str!("../migrations/20261031000114_phone_forms.sql");

/// The statements of [`PHONE_FORMS_SQL`] that fill the derived columns from what is stored: the two digit updates and the two
/// E.164 updates. A request test runs them over rows of its own and compares them with `rules::phone`; running them again
/// changes nothing.
#[must_use]
pub fn phone_backfill() -> Vec<&'static str> {
    PHONE_FORMS_SQL
        .split(";\n\n")
        .map(str::trim)
        .map(|s| s.trim_end_matches(';'))
        .filter(|s| s.starts_with("UPDATE ") || s.starts_with("WITH plans"))
        .collect()
}

/// The `sqlx` version of a `SeaORM` migration name: `m20220101_000001_workers` is `20220101000001`, the file
/// `20220101000001_workers.sql`. `None` for a name not of that form. Pure, so the mapping is tested without a database.
#[must_use]
pub fn seaorm_version(name: &str) -> Option<i64> {
    let rest = name.strip_prefix('m')?;
    let (date, rest) = rest.split_once('_')?;
    let (sequence, _) = rest.split_once('_')?;
    if date.len() != 8 || sequence.len() != 6 {
        return None;
    }
    format!("{date}{sequence}").parse().ok()
}

/// The newest migration to record as applied for a `SeaORM` history: the last of `applied`, provided `applied` is exactly the
/// first migrations of `known`, in order. Pure, so a gap or an unknown name is tested without a database.
///
/// # Errors
/// The name that is unknown or out of place.
pub fn adopted_through(
    applied: &[String],
    known: &[i64],
) -> std::result::Result<Option<i64>, String> {
    let mut last = None;
    for (at, name) in applied.iter().enumerate() {
        let version = seaorm_version(name).ok_or_else(|| {
            format!("seaql_migrations names `{name}`, which is not a migration name")
        })?;
        match known.get(at) {
            Some(expected) if *expected == version => last = Some(version),
            Some(_) => {
                return Err(format!(
                    "seaql_migrations records `{name}` out of order or beside a gap; migrate with the release that wrote it first"
                ));
            }
            None => {
                return Err(format!(
                    "seaql_migrations records `{name}`, which this release does not know; it is newer than this binary"
                ));
            }
        }
    }
    Ok(last)
}

/// Record a `SeaORM` release's migrations as applied, once, when `sqlx` has recorded none. A no-op on a new database and on one
/// `sqlx` already migrated.
async fn adopt_seaorm_history(pool: &PgPool) -> Result<()> {
    let has = |table: &'static str| async move {
        sqlx::query_scalar::<_, bool>("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(pool)
            .await
    };
    if !has("seaql_migrations").await? {
        return Ok(());
    }
    if has("_sqlx_migrations").await?
        && sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM _sqlx_migrations)")
            .fetch_one(pool)
            .await?
    {
        return Ok(());
    }
    let applied: Vec<String> =
        sqlx::query_scalar("SELECT version FROM seaql_migrations ORDER BY version")
            .fetch_all(pool)
            .await?;
    let known: Vec<i64> = MIGRATOR.iter().map(|m| m.version).collect();
    let Some(through) = adopted_through(&applied, &known).map_err(|e| Error::string(&e))? else {
        return Ok(());
    };
    MIGRATOR
        .skip(pool, Some(through))
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    tracing::info!(
        adopted = applied.len(),
        through,
        "recorded the SeaORM release's migrations as applied"
    );
    Ok(())
}

/// Apply what is pending, after adopting a `SeaORM` history. `sqlx` holds a lock while it migrates, so instances booting together
/// take turns.
///
/// # Errors
/// An unknown or broken `SeaORM` history, a failed migration, or an applied migration whose file has changed.
pub async fn up(pool: &PgPool) -> Result<()> {
    adopt_seaorm_history(pool).await?;
    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| Error::string(&e.to_string()))
}

/// Each migration and whether it has run, oldest first.
///
/// # Errors
/// A database error.
pub async fn status(pool: &PgPool) -> Result<Vec<axa::sqlx::MigrationStatus>> {
    Ok(axa::sqlx::status(pool, &MIGRATOR).await?)
}

/// Drop every table and enum type, then migrate from nothing: a test's empty schema. **This destroys all data.**
///
/// # Errors
/// A database error or a failed migration.
pub async fn fresh(pool: &PgPool) -> Result<()> {
    axa::sqlx::dangerously_recreate(pool, &MIGRATOR)
        .await
        .map_err(|e| Error::string(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seaorm_name_maps_to_its_file_version() {
        assert_eq!(
            seaorm_version("m20220101_000001_workers"),
            Some(20_220_101_000_001)
        );
        assert_eq!(
            seaorm_version("m20261105_000133_course_requests"),
            Some(20_261_105_000_133)
        );
        for bad in [
            "20220101_000001_workers",
            "m2022010_000001_x",
            "m20220101_00001_x",
            "mxxxxxxxx_000001_x",
            "m",
        ] {
            assert_eq!(seaorm_version(bad), None, "{bad}");
        }
    }

    #[test]
    fn every_migration_file_has_a_seaorm_shaped_version() {
        assert_eq!(MIGRATOR.iter().count(), 133);
        for m in MIGRATOR.iter() {
            let name = format!(
                "m{}_{}_x",
                &m.version.to_string()[..8],
                &m.version.to_string()[8..]
            );
            assert_eq!(seaorm_version(&name), Some(m.version));
        }
    }

    #[test]
    fn the_phone_backfill_is_its_four_updates() {
        let backfill = phone_backfill();
        assert_eq!(backfill.len(), 4);
        assert!(backfill[0].starts_with("UPDATE worker_contact_details SET phone_mobile_digits"));
        assert!(backfill[3].starts_with("UPDATE emergency_contacts c SET phone_e164"));
    }

    #[test]
    fn only_an_unbroken_prefix_is_adopted() {
        let known = [20_220_101_000_001, 20_220_101_000_002, 20_220_101_000_003];
        let names = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(adopted_through(&[], &known), Ok(None));
        assert_eq!(
            adopted_through(
                &names(&["m20220101_000001_a", "m20220101_000002_b"]),
                &known
            ),
            Ok(Some(20_220_101_000_002))
        );
        // A gap, an unknown name, and one newer than the binary are each refused, never guessed at.
        assert!(
            adopted_through(
                &names(&["m20220101_000001_a", "m20220101_000003_c"]),
                &known
            )
            .is_err()
        );
        assert!(adopted_through(&names(&["not_a_migration"]), &known).is_err());
        let all = names(&[
            "m20220101_000001_a",
            "m20220101_000002_b",
            "m20220101_000003_c",
            "m20991231_000001_z",
        ]);
        assert!(
            adopted_through(&all, &known)
                .unwrap_err()
                .contains("does not know")
        );
    }
}
