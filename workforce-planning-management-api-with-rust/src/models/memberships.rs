//! Multi-organization membership scoping (WPM-Rxx): resolves a caller
//! to the set of organizations they currently belong to, for read
//! endpoints to filter by (§3 of the design) and for the grant/revoke
//! admin surface to gate against (§2 Step C). A WPM-local second pass
//! layered on top of the blanket ABAC guard — not a policy-engine
//! change, since the engine can't compare a resource attribute
//! against a dynamic, multi-valued subject claim today, and the
//! token-minting crate is an external sibling this deployment doesn't
//! own.

use authentication_verifier::Claims;
use loco_rs::prelude::*;
use sea_orm::ConnectionTrait;

use super::_entities::organization_memberships;

/// Today, as a plain date — memberships are windowed by
/// `starts_on`/`ends_on`, checked in Rust rather than SQL (the row
/// count per person is small; this matches the family's existing
/// date-window pattern elsewhere in this crate, e.g. the candidate
/// consent-expiry filter in `controllers::acquisition`).
fn is_live(row: &organization_memberships::Model, today: chrono::NaiveDate) -> bool {
    row.deleted_at.is_none() && row.starts_on <= today && row.ends_on.is_none_or(|end| end >= today)
}

/// This caller's live memberships, oldest first — the full rows, for
/// `GET /api/me/organizations` and for the grant/revoke gate below.
/// Empty (never an error) when `claims` is `None`.
///
/// # Errors
///
/// Any query error.
pub async fn caller_memberships<C: ConnectionTrait>(
    db: &C,
    claims: Option<&Claims>,
) -> Result<Vec<organization_memberships::Model>> {
    let Some(claims) = claims else {
        return Ok(Vec::new());
    };
    let person_ref = format!("person:{}", claims.sub);
    let today = chrono::Utc::now().date_naive();
    let rows = organization_memberships::Entity::find()
        .filter(organization_memberships::Column::PersonRef.eq(person_ref))
        .filter(organization_memberships::Column::DeletedAt.is_null())
        .order_by_asc(organization_memberships::Column::Id)
        .all(db)
        .await?;
    Ok(rows.into_iter().filter(|r| is_live(r, today)).collect())
}

/// This caller's live memberships' organizations, **expanded through
/// confederation**: each membership's own `organization_ref` plus
/// every transitive descendant org
/// ([`crate::rules::org_access::descendants_of`]) — a membership in a
/// parent confederation reads as membership in the whole tree beneath
/// it. Unconditional, not gated by `WPM_REQUIRE_AUTH` (unlike
/// [`scope_organization_refs`]), for the caller-facing "what can I
/// see" surface (`GET /api/me/organizations/scope`) as well as
/// internal use by that function. Empty when `claims` is `None` or the
/// caller has no live memberships.
///
/// # Errors
///
/// Any query error.
pub async fn caller_scope_refs<C: ConnectionTrait>(
    db: &C,
    claims: Option<&Claims>,
) -> Result<Vec<String>> {
    let memberships = caller_memberships(db, claims).await?;
    if memberships.is_empty() {
        return Ok(Vec::new());
    }
    let edges = crate::models::confederations::live_edges(db).await?;
    let mut refs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for m in &memberships {
        refs.insert(m.organization_ref.clone());
        refs.extend(crate::rules::org_access::descendants_of(
            &edges,
            &m.organization_ref,
        ));
    }
    Ok(refs.into_iter().collect())
}

/// The organization scope to filter a list/read query by: `None` when
/// blanket enforcement is off (today's demo default) — behave exactly
/// as before this feature existed, unscoped — `Some(refs)` otherwise,
/// which may be an **empty** vec for a signed-in caller with no live
/// memberships (correctly seeing nothing, not everything). See
/// [`caller_scope_refs`] for the confederation expansion this wraps.
///
/// # Errors
///
/// Any query error.
pub async fn scope_organization_refs<C: ConnectionTrait>(
    db: &C,
    claims: Option<&Claims>,
) -> Result<Option<Vec<String>>> {
    if !crate::auth::require_auth() {
        return Ok(None);
    }
    Ok(Some(caller_scope_refs(db, claims).await?))
}

/// Whether any of this caller's live memberships in `organization_ref`
/// carries one of `roles` — the grant/revoke admin gate (§2 Step C):
/// "is the caller privileged enough, *in this org*, to act on someone
/// else's membership in it".
///
/// # Errors
///
/// Any query error.
pub async fn has_role_in<C: ConnectionTrait>(
    db: &C,
    claims: Option<&Claims>,
    organization_ref: &str,
    roles: &[&str],
) -> Result<bool> {
    let memberships = caller_memberships(db, claims).await?;
    Ok(memberships
        .iter()
        .any(|m| m.organization_ref == organization_ref && roles.contains(&m.role.as_str())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_membership(
        organization_ref: &str,
        role: &str,
        starts_on: chrono::NaiveDate,
        ends_on: Option<chrono::NaiveDate>,
        deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> organization_memberships::Model {
        organization_memberships::Model {
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: uuid::Uuid::new_v4(),
            person_ref: "person:11111111-1111-1111-1111-111111111111".to_string(),
            organization_ref: organization_ref.to_string(),
            worker_pid: None,
            role: role.to_string(),
            starts_on,
            ends_on,
            deleted_at,
        }
    }

    #[test]
    fn is_live_windows_on_dates_and_deletion() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let past_start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let future_start = chrono::NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();
        let past_end = chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();

        assert!(is_live(
            &a_membership("organization:a", "member", past_start, None, None),
            today
        ));
        assert!(
            !is_live(
                &a_membership("organization:a", "member", future_start, None, None),
                today
            ),
            "not started yet"
        );
        assert!(
            !is_live(
                &a_membership("organization:a", "member", past_start, Some(past_end), None),
                today
            ),
            "already ended"
        );
        assert!(
            !is_live(
                &a_membership(
                    "organization:a",
                    "member",
                    past_start,
                    None,
                    Some(chrono::Utc::now().into())
                ),
                today
            ),
            "soft-deleted"
        );
    }
}
