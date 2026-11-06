//! Public holidays (WPM-R5): generated for an organization and region from a region's rules, so no one types them in one by one,
//! plus any day added by hand for what the rules cannot know. The shipped rule packs are **unverified examples** (see
//! `rules::holiday`); the listing says so. Reading is open to a signed-in caller in scope; generating and adding are HR's.

use axa::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::holidays;
use crate::rules::holiday as rules;

/// The years a calendar can hold.
const YEARS: std::ops::RangeInclusive<i32> = 1970..=2100;

/// `GET /api/holiday-regions` — the regions that have a rule pack, each marked unverified until someone checks it.
#[debug_handler]
async fn regions() -> Result<Response> {
    let packs: Vec<_> = rules::PACKS
        .iter()
        .map(|p| {
            json!({
                "region": p.region,
                "name": p.name,
                "rules": p.rules.len(),
                "verified": p.verified,
                "weekend_days": rules::weekend_to_iso(p.weekend),
            })
        })
        .collect();
    format::json(json!({
        "regions": packs,
        "note": "These packs are worked examples, not an official source. Confirm a region against its government's published list; add a local day by hand. A pack's weekend_days are used only where the organization's profile sets no holidays.weekend_days for the region. A day of the tabular Islamic calendar can differ by a day or two from an announced one.",
    }))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: String,
    region: Option<String>,
    year: Option<i32>,
}

/// `GET /api/holidays?organization=&region=&year=` — the holidays, by the day they are taken.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&q.organization))?;
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    if let Some(region) = &q.region {
        ensure_valid(&rules::region_problems(region))?;
    }
    let rows = sqlx::query_as!(
        holidays::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, region, year, name,
                  natural_on, observed_on, source, recorded_by
           FROM holidays
           WHERE organization_ref = $1 AND ($2::text IS NULL OR region = $2) AND ($3::int IS NULL OR year = $3)
           ORDER BY observed_on, name"#,
        q.organization,
        q.region,
        q.year,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let mut out = Vec::new();
    for row in &rows {
        let mut v = row_json(row)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
        }
        out.push(v);
    }
    format::json(json!({ "holidays": out }))
}

async fn hr_only(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<()> {
    ensure_valid(&organization_problems(organization))?;
    ensure_org_in_scope(ctx, caller, organization).await?;
    if !super::policies::privileged(ctx, caller, organization).await? {
        return Err(forbidden("the holiday calendar is kept by HR"));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct GeneratePayload {
    organization_ref: String,
    region: String,
    year: i32,
}

/// `POST /api/holidays/generate` — generate a region's holidays for a year. Safe to repeat: a row it made is brought up to date, a row
/// someone added by hand is never changed. HR.
#[debug_handler]
async fn generate(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<GeneratePayload>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &p.organization_ref).await?;
    ensure_valid(&rules::region_problems(&p.region))?;
    let Some(pack) = rules::pack(&p.region) else {
        return Err(unprocessable(
            "no rule pack for that region; add its days by hand, or ask for a pack",
        ));
    };
    if !YEARS.contains(&p.year) {
        return Err(unprocessable("year must be between 1970 and 2100"));
    }
    // The weekend is the organization's `holidays.weekend_days` for the region (WPM-D144, WPM-T379), resolved as of 1 January of
    // the year; where the profile has none, the pack's own example weekend, and the answer says which.
    let profile =
        super::jurisdictions::profiles_for(&ctx.db, std::slice::from_ref(&p.organization_ref))
            .await?
            .remove(&p.organization_ref)
            .unwrap_or_default();
    let on = NaiveDate::from_ymd_opt(p.year, 1, 1).unwrap_or_default();
    let resolved = profile.resolve("holidays.weekend_days", Some(&p.region), on);
    let configured = match &resolved {
        crate::rules::jurisdiction::Resolution::Resolved { value, .. } => value
            .as_weekdays()
            .and_then(|days| rules::weekend_from_iso(&days)),
        _ => None,
    };
    let (weekend, weekend_from) = match configured {
        Some(days) => (days, serde_json::to_value(&resolved).unwrap_or_default()),
        None => (
            pack.weekend.to_vec(),
            json!({ "state": "pack_example", "region": pack.region, "verified": pack.verified }),
        ),
    };
    let (mut created, mut updated, mut unchanged, mut kept_manual) = (0, 0, 0, 0);
    let mut tx = pool(&ctx).begin().await?;
    for h in rules::generate(pack.rules, p.year, &weekend) {
        let existing = sqlx::query!(
            "SELECT id, source, observed_on FROM holidays
             WHERE organization_ref = $1 AND region = $2 AND natural_on = $3 AND name = $4 LIMIT 1",
            p.organization_ref,
            p.region,
            h.natural_on,
            h.name,
        )
        .fetch_optional(&mut *tx)
        .await?;
        match existing {
            None => {
                sqlx::query!(
                    "INSERT INTO holidays (pid, organization_ref, region, year, name, natural_on, observed_on, source, recorded_by)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, 'generated', $8)",
                    Uuid::new_v4(),
                    p.organization_ref,
                    p.region,
                    p.year,
                    h.name,
                    h.natural_on,
                    h.observed_on,
                    caller.actor(),
                )
                .execute(&mut *tx)
                .await?;
                created += 1;
            }
            Some(row) if row.source == "manual" => kept_manual += 1,
            Some(row) if row.observed_on == h.observed_on => unchanged += 1,
            Some(row) => {
                sqlx::query!(
                    "UPDATE holidays SET observed_on = $2 WHERE id = $1",
                    row.id,
                    h.observed_on,
                )
                .execute(&mut *tx)
                .await?;
                updated += 1;
            }
        }
    }
    db::audit(
        &mut tx,
        "holiday_calendar",
        Uuid::new_v4(),
        "holidays_generated",
        caller.actor(),
        Some(json!({ "region": p.region, "year": p.year, "created": created })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "region": p.region,
        "year": p.year,
        "verified": pack.verified,
        "weekend_days": rules::weekend_to_iso(&weekend),
        "weekend_from": weekend_from,
        "created": created,
        "updated": updated,
        "unchanged": unchanged,
        "kept_manual": kept_manual,
    }))
}

#[derive(Debug, Deserialize)]
struct ManualPayload {
    organization_ref: String,
    region: String,
    name: String,
    on: NaiveDate,
}

/// `POST /api/holidays` — add a day by hand (a local or one-off holiday). HR.
#[debug_handler]
async fn add(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<ManualPayload>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &p.organization_ref).await?;
    let mut problems = rules::region_problems(&p.region);
    problems.extend(rules::manual_problems(&p.name));
    ensure_valid(&problems)?;
    let year = chrono::Datelike::year(&p.on);
    if !YEARS.contains(&year) {
        return Err(unprocessable("the date must be between 1970 and 2100"));
    }
    let name = p.name.trim().to_string();
    let clash = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM holidays WHERE organization_ref = $1 AND region = $2 AND natural_on = $3 AND name = $4
           ) AS "clash!""#,
        p.organization_ref,
        p.region,
        p.on,
        name,
    )
    .fetch_one(pool(&ctx))
    .await?;
    if clash {
        return Err(unprocessable("that holiday is already in the calendar"));
    }
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO holidays (pid, organization_ref, region, year, name, natural_on, observed_on, source, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'manual', $8)",
        pid,
        p.organization_ref,
        p.region,
        year,
        name,
        p.on,
        p.on,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "holiday",
        pid,
        "holiday_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/holiday-regions", get(regions))
        .add("/holidays", get(list))
        .add("/holidays", post(add))
        .add("/holidays/generate", post(generate))
}
