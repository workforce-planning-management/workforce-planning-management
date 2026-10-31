//! Public holidays (WPM-R5): generated for an organization and region from a region's rules, so no one types them in one by one,
//! plus any day added by hand for what the rules cannot know. The shipped rule packs are **unverified examples** (see
//! `rules::holiday`); the listing says so. Reading is open to a signed-in caller in scope; generating and adding are HR's.

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::holidays;
use crate::models::audit_logs::Model as Audit;
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
    let mut find =
        holidays::Entity::find().filter(holidays::Column::OrganizationRef.eq(&q.organization));
    if let Some(region) = &q.region {
        ensure_valid(&rules::region_problems(region))?;
        find = find.filter(holidays::Column::Region.eq(region));
    }
    if let Some(year) = q.year {
        find = find.filter(holidays::Column::Year.eq(year));
    }
    let rows = find
        .order_by_asc(holidays::Column::ObservedOn)
        .order_by_asc(holidays::Column::Name)
        .all(&ctx.db)
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
    let txn = ctx.db.begin().await?;
    for h in rules::generate(pack.rules, p.year, &weekend) {
        let existing = holidays::Entity::find()
            .filter(holidays::Column::OrganizationRef.eq(&p.organization_ref))
            .filter(holidays::Column::Region.eq(&p.region))
            .filter(holidays::Column::NaturalOn.eq(h.natural_on))
            .filter(holidays::Column::Name.eq(h.name))
            .one(&txn)
            .await?;
        match existing {
            None => {
                holidays::ActiveModel {
                    pid: ActiveValue::set(Uuid::new_v4()),
                    organization_ref: ActiveValue::set(p.organization_ref.clone()),
                    region: ActiveValue::set(p.region.clone()),
                    year: ActiveValue::set(p.year),
                    name: ActiveValue::set(h.name.to_string()),
                    natural_on: ActiveValue::set(h.natural_on),
                    observed_on: ActiveValue::set(h.observed_on),
                    source: ActiveValue::set("generated".to_string()),
                    recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
                created += 1;
            }
            Some(row) if row.source == "manual" => kept_manual += 1,
            Some(row) if row.observed_on == h.observed_on => unchanged += 1,
            Some(row) => {
                let mut active: holidays::ActiveModel = row.into();
                active.observed_on = ActiveValue::set(h.observed_on);
                active.update(&txn).await?;
                updated += 1;
            }
        }
    }
    Audit::record(
        &txn,
        "holiday_calendar",
        Uuid::new_v4(),
        "holidays_generated",
        caller.actor(),
        Some(json!({ "region": p.region, "year": p.year, "created": created })),
    )
    .await?;
    txn.commit().await?;
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
    let clash = holidays::Entity::find()
        .filter(holidays::Column::OrganizationRef.eq(&p.organization_ref))
        .filter(holidays::Column::Region.eq(&p.region))
        .filter(holidays::Column::NaturalOn.eq(p.on))
        .filter(holidays::Column::Name.eq(&name))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable("that holiday is already in the calendar"));
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    holidays::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(p.organization_ref),
        region: ActiveValue::set(p.region),
        year: ActiveValue::set(year),
        name: ActiveValue::set(name),
        natural_on: ActiveValue::set(p.on),
        observed_on: ActiveValue::set(p.on),
        source: ActiveValue::set("manual".to_string()),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "holiday", pid, "holiday_added", caller.actor(), None).await?;
    txn.commit().await?;
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
