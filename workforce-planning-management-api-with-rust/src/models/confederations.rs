//! Org confederation (WPM-Rxx): a parent organization that
//! transitively contains member organizations. This module owns the
//! DB fetch (the live edge list); the graph algorithms themselves
//! (walking descendants, detecting a cycle before granting a new edge)
//! are pure and live in [`crate::rules::org_access`], per this crate's
//! pure-core/DB-touching-model split.

use loco_rs::prelude::*;
use sea_orm::ConnectionTrait;

use super::_entities::organization_confederations;

/// Whether a confederation edge is currently live: not soft-deleted
/// and within its `starts_on`/`ends_on` window. Mirrors
/// `memberships::is_live`.
fn is_live(row: &organization_confederations::Model, today: chrono::NaiveDate) -> bool {
    row.deleted_at.is_none() && row.starts_on <= today && row.ends_on.is_none_or(|end| end >= today)
}

/// Every live confederation edge, as `(parent_organization_ref,
/// child_organization_ref)` pairs — the whole graph. Small enough to
/// hold in memory and walk in Rust at demo scale, matching this
/// crate's existing convention (e.g.
/// `memberships::caller_memberships`) rather than a recursive SQL
/// query.
///
/// # Errors
///
/// Any query error.
pub async fn live_edges<C: ConnectionTrait>(db: &C) -> Result<Vec<(String, String)>> {
    let today = chrono::Utc::now().date_naive();
    let rows = organization_confederations::Entity::find()
        .filter(organization_confederations::Column::DeletedAt.is_null())
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|r| is_live(r, today))
        .map(|r| (r.parent_organization_ref, r.child_organization_ref))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn an_edge(
        parent: &str,
        child: &str,
        starts_on: chrono::NaiveDate,
        ends_on: Option<chrono::NaiveDate>,
        deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> organization_confederations::Model {
        organization_confederations::Model {
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: uuid::Uuid::new_v4(),
            parent_organization_ref: parent.to_string(),
            child_organization_ref: child.to_string(),
            starts_on,
            ends_on,
            deleted_at,
        }
    }

    #[test]
    fn is_live_windows_on_dates_and_deletion() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let past = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let future = chrono::NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();

        assert!(is_live(&an_edge("a", "b", past, None, None), today));
        assert!(
            !is_live(&an_edge("a", "b", future, None, None), today),
            "not started yet"
        );
        assert!(
            !is_live(
                &an_edge("a", "b", past, None, Some(chrono::Utc::now().into())),
                today
            ),
            "soft-deleted"
        );
    }
}
