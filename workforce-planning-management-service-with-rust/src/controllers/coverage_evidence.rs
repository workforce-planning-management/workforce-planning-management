//! Coverage evidence: an organization's workers' compensation or employer's liability cover, per jurisdiction (WPM-R220, WPM-D154,
//! WPM-T420). HR records the kind, the insurer and policy reference, the minimum cover, the dates and a *reference* to the certificate;
//! the list says where each record stands today. WPM records the fact and its expiry. It never decides that an organization is covered,
//! pays nothing and adjudicates nothing. The standing arithmetic is `rules::coverage_evidence`. Spec: `spec/statutory-pay-protections.md`.

use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::ensure_valid;
use super::mandatory_learning::ensure_hr;
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::coverage_evidence;
use crate::rules::coverage_evidence as rules;

/// A record as the API shows it, with its standing on `today`.
fn row_json(r: &coverage_evidence::Model, today: NaiveDate, window: i64) -> Value {
    let standing = rules::standing(r.valid_from, r.valid_to, today, window);
    json!({
        "pid": r.pid, "organization_ref": r.organization_ref, "jurisdiction": r.jurisdiction, "kind": r.kind,
        "insurer": r.insurer, "policy_reference": r.policy_reference,
        "minimum_cover_minor": r.minimum_cover_minor, "currency": r.currency,
        "valid_from": r.valid_from, "valid_to": r.valid_to, "certificate_ref": r.certificate_ref,
        "standing": standing.as_str(),
        "days_left": match standing { rules::Standing::EndingSoon(days) => Some(days), _ => None },
        "retired_on": r.retired_on,
    })
}

/// Query of the list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// The organization whose records are listed.
    organization: String,
    /// The reminder window in calendar days, 1 to 366; default 60.
    days: Option<i64>,
}

/// `GET /api/coverage-evidence?organization=` — the organization's records in force, each with its standing today. HR only: a policy
/// reference is the organization's business, not every colleague's.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let today = Utc::now().date_naive();
    let window = q
        .days
        .filter(|d| (1..=366).contains(d))
        .unwrap_or(rules::DEFAULT_WINDOW_DAYS);
    let rows = sqlx::query_as!(
        coverage_evidence::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, jurisdiction, kind,
                  insurer, policy_reference, minimum_cover_minor, currency, valid_from, valid_to, certificate_ref, recorded_by,
                  retired_on
           FROM coverage_evidence WHERE organization_ref = $1 ORDER BY jurisdiction, id"#,
        q.organization,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
        .filter(|r| r.retired_on.is_none_or(|d| d > today))
        .collect::<Vec<_>>();
    format::json(
        rows.iter()
            .map(|r| row_json(r, today, window))
            .collect::<Vec<_>>(),
    )
}

/// The body of a new record.
#[derive(Debug, Deserialize)]
struct CreatePayload {
    organization: String,
    jurisdiction: String,
    kind: String,
    insurer: Option<String>,
    policy_reference: Option<String>,
    minimum_cover_minor: Option<i64>,
    currency: Option<String>,
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
    /// Where the certificate is kept: a reference, never the document.
    certificate_ref: Option<String>,
}

/// Trims text, and treats blank as absent.
fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

/// `POST /api/coverage-evidence` — HR adds a record. It is checked for what its kind needs; a second record for the same jurisdiction is
/// allowed (a renewal, or two covers), and the old one is retired by HR when it no longer applies.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CreatePayload>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &p.organization).await?;
    let jurisdiction = p.jurisdiction.trim().to_string();
    let insurer = clean(p.insurer.as_deref());
    let policy = clean(p.policy_reference.as_deref());
    let currency = clean(p.currency.as_deref());
    let certificate = clean(p.certificate_ref.as_deref());
    let mut problems = rules::problems(
        &jurisdiction,
        &p.kind,
        insurer.as_deref(),
        policy.as_deref(),
        p.minimum_cover_minor,
        currency.as_deref(),
        p.valid_from,
        p.valid_to,
    );
    if insurer.as_ref().is_some_and(|s| s.chars().count() > 200) {
        problems.push("insurer must be at most 200 characters".to_string());
    }
    if policy.as_ref().is_some_and(|s| s.chars().count() > 100) {
        problems.push("policy_reference must be at most 100 characters".to_string());
    }
    if certificate
        .as_ref()
        .is_some_and(|s| s.chars().count() > 255)
    {
        problems.push("certificate_ref must be at most 255 characters".to_string());
    }
    ensure_valid(&problems)?;
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO coverage_evidence
             (pid, organization_ref, jurisdiction, kind, insurer, policy_reference, minimum_cover_minor, currency,
              valid_from, valid_to, certificate_ref, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        pid,
        p.organization,
        jurisdiction,
        p.kind,
        insurer,
        policy,
        p.minimum_cover_minor,
        currency,
        p.valid_from,
        p.valid_to,
        certificate,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "coverage_evidence",
        pid,
        "coverage_evidence_added",
        caller.actor(),
        Some(json!({ "kind": p.kind })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// `POST /api/coverage-evidence/{pid}/retire` — HR retires a record from today: it is no longer listed or reminded about. Retiring twice
/// keeps the first date.
#[debug_handler]
async fn retire(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let record = sqlx::query!(
        "SELECT pid, organization_ref, retired_on FROM coverage_evidence WHERE pid = $1",
        pid,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_hr(&ctx, &caller, &record.organization_ref).await?;
    let on = record.retired_on.unwrap_or_else(|| Utc::now().date_naive());
    if record.retired_on.is_none() {
        let mut tx = pool(&ctx).begin().await?;
        sqlx::query!(
            "UPDATE coverage_evidence SET retired_on = $2 WHERE pid = $1",
            record.pid,
            on,
        )
        .execute(&mut *tx)
        .await?;
        db::audit(
            &mut tx,
            "coverage_evidence",
            record.pid,
            "coverage_evidence_retired",
            caller.actor(),
            None,
        )
        .await?;
        tx.commit().await?;
    }
    format::json(json!({ "pid": record.pid, "retired_on": on }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/coverage-evidence", get(list))
        .add("/coverage-evidence", post(create))
        .add("/coverage-evidence/{pid}/retire", post(retire))
}
