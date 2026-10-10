//! **Need-to-know on reads** (WPM-R114, WPM-D71): the middleware that applies
//! [`crate::rules::access`] to every `GET` under `/api`.
//!
//! It runs *inside* the sign-in guard, so a token has already been verified. It loads only
//! what the table needs: for a route about one worker, that worker; for a review, the review's
//! worker; for a privileged-only route, nothing but the caller's rights. Everything it decides
//! from comes from the verified token and the database, never from the request.
//!
//! A denial is `403` and is logged as a warning with the audience class and no identifier.

use authentication_verifier::Claims;
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{reviews, workers};
use crate::models::{confederations, memberships};
use crate::rules::access::{self, Audience, Facts, MAX_MANAGER_DEPTH, Relation};
use crate::rules::org_access;

/// Roles within an organization that carry HR or payroll duties.
const PRIVILEGED_ROLES: &[&str] = &["hr_admin", "payroll_admin"];

/// Whether the verified token itself carries HR, payroll, service or administrator rights.
#[must_use]
pub fn has_privileged_attrs(claims: &Claims) -> bool {
    let truthy = |key: &str| {
        claims
            .attrs
            .get(key)
            .is_some_and(|values| values.iter().any(|v| v == "true"))
    };
    truthy("hr")
        || truthy("payroll")
        || truthy("svc")
        || claims
            .attrs
            .get("access")
            .is_some_and(|values| values.iter().any(|v| v == "admin"))
}

fn denied(audience: &str) -> Response {
    tracing::warn!(audience, "need-to-know: read refused");
    (
        StatusCode::FORBIDDEN,
        "not permitted: this record is need-to-know",
    )
        .into_response()
}

/// Whether the caller holds an HR or payroll role in `organization_ref` (or in an organization
/// that contains it), or anywhere when `organization_ref` is `None`.
async fn privileged_by_membership(
    ctx: &AppContext,
    claims: &Claims,
    organization_ref: Option<&str>,
) -> Result<bool> {
    let held = memberships::caller_memberships(&ctx.db, Some(claims)).await?;
    let held: Vec<_> = held
        .into_iter()
        .filter(|m| PRIVILEGED_ROLES.contains(&m.role.as_str()))
        .collect();
    let Some(target) = organization_ref else {
        return Ok(!held.is_empty());
    };
    if held.is_empty() {
        return Ok(false);
    }
    let edges = confederations::live_edges(&ctx.db).await?;
    Ok(held.iter().any(|m| {
        m.organization_ref == target
            || org_access::descendants_of(&edges, &m.organization_ref)
                .iter()
                .any(|d| d == target)
    }))
}

/// The person id inside a `person:<id>` URN.
fn person_id(person_ref: &str) -> &str {
    person_ref.split_once(':').map_or(person_ref, |(_, id)| id)
}

/// Whether `caller_sub` is a line manager of `worker`, directly or higher up.
async fn is_line_manager(
    ctx: &AppContext,
    worker: &workers::Model,
    caller_sub: &str,
) -> Result<bool> {
    let mut next = worker.manager_pid;
    for _ in 0..MAX_MANAGER_DEPTH {
        let Some(pid) = next else {
            return Ok(false);
        };
        let Some(manager) = workers::Entity::find()
            .filter(workers::Column::Pid.eq(pid))
            .filter(workers::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
        else {
            return Ok(false);
        };
        if person_id(&manager.person_ref) == caller_sub {
            return Ok(true);
        }
        next = manager.manager_pid;
    }
    Ok(false)
}

/// What the caller is to this worker.
async fn facts_for(ctx: &AppContext, claims: &Claims, worker: &workers::Model) -> Result<Facts> {
    let privileged = has_privileged_attrs(claims)
        || privileged_by_membership(ctx, claims, Some(&worker.organization_ref)).await?;
    if person_id(&worker.person_ref) == claims.sub {
        return Ok(Facts {
            privileged,
            relation: Relation::Own,
        });
    }
    if privileged {
        // Nothing narrower is needed to read.
        return Ok(Facts {
            privileged,
            relation: Relation::Stranger,
        });
    }
    let relation = if is_line_manager(ctx, worker, &claims.sub).await? {
        Relation::Manager
    } else if memberships::caller_scope_refs(&ctx.db, Some(claims))
        .await?
        .iter()
        .any(|r| r == &worker.organization_ref)
    {
        Relation::InScope
    } else {
        Relation::Stranger
    };
    Ok(Facts {
        privileged,
        relation,
    })
}

async fn live_worker(ctx: &AppContext, pid: Uuid) -> Result<Option<workers::Model>> {
    Ok(workers::Entity::find()
        .filter(workers::Column::Pid.eq(pid))
        .filter(workers::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?)
}

/// The decision for one request: `Ok(true)` to let it through.
async fn allowed(
    ctx: &AppContext,
    claims: &Claims,
    audience: Audience,
    path: &str,
) -> Result<bool> {
    match audience {
        Audience::Open | Audience::ControllerDecides => Ok(true),
        Audience::Privileged => {
            Ok(has_privileged_attrs(claims) || privileged_by_membership(ctx, claims, None).await?)
        }
        Audience::Worker(level) => {
            if has_privileged_attrs(claims) {
                return Ok(true);
            }
            let Some(pid) = access::worker_pid_of(path).and_then(|p| Uuid::parse_str(p).ok())
            else {
                // Not a worker the controller can find either: it answers the error.
                return Ok(true);
            };
            let Some(worker) = live_worker(ctx, pid).await? else {
                return Ok(true);
            };
            Ok(access::permits(
                level,
                &facts_for(ctx, claims, &worker).await?,
            ))
        }
        Audience::ReviewRecord => {
            if has_privileged_attrs(claims) {
                return Ok(true);
            }
            let Some(pid) = path
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .and_then(|p| Uuid::parse_str(p).ok())
            else {
                return Ok(true);
            };
            let Some(review) = reviews::Entity::find()
                .filter(reviews::Column::Pid.eq(pid))
                .one(&ctx.db)
                .await?
            else {
                return Ok(true);
            };
            let Some(worker) = live_worker(ctx, review.worker_pid).await? else {
                return Ok(true);
            };
            Ok(access::permits(
                access::WorkerLevel::WithManagers,
                &facts_for(ctx, claims, &worker).await?,
            ))
        }
    }
}

/// The middleware. A no-op while sign-in is off (development only), for anything that is not a
/// `GET`/`HEAD` under `/api`, and for routes the controller decides.
pub async fn need_to_know_mw(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    req: Request,
    next: Next,
) -> Response {
    if !auth::require_auth() || !matches!(*req.method(), Method::GET | Method::HEAD) {
        return next.run(req).await;
    }
    let path = req.uri().path().to_string();
    if !path.starts_with("/api/") {
        return next.run(req).await;
    }
    let Some(audience) = access::classify_read(&path) else {
        // Not a route this service has: the router answers with a 404.
        return next.run(req).await;
    };
    let Some(claims) = caller.claims() else {
        // The sign-in guard, which runs first, has already refused an anonymous caller.
        return next.run(req).await;
    };
    match allowed(&ctx, claims, audience, &path).await {
        Ok(true) => next.run(req).await,
        Ok(false) => denied(&format!("{audience:?}")),
        Err(error) => {
            tracing::error!(%error, "need-to-know: decision failed, refusing");
            (StatusCode::INTERNAL_SERVER_ERROR, "could not check access").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(attrs: &[(&str, &[&str])]) -> Claims {
        Claims {
            sub: "11111111-1111-1111-1111-111111111111".into(),
            email: "x@example.com".into(),
            name: "X".into(),
            iss: "i".into(),
            aud: "a".into(),
            exp: 0,
            iat: 0,
            nbf: None,
            sid: "s".into(),
            scope: vec![],
            roles: vec![],
            attrs: attrs
                .iter()
                .map(|(k, v)| {
                    (
                        (*k).to_string(),
                        v.iter().map(|s| (*s).to_string()).collect(),
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn privileged_attributes_are_hr_payroll_service_and_admin_only() {
        assert!(!has_privileged_attrs(&claims(&[])));
        assert!(has_privileged_attrs(&claims(&[("hr", &["true"])])));
        assert!(has_privileged_attrs(&claims(&[("payroll", &["true"])])));
        assert!(has_privileged_attrs(&claims(&[("svc", &["true"])])));
        assert!(has_privileged_attrs(&claims(&[("access", &["admin"])])));
        // Write access is not privilege; "false" is not privilege; an unknown value is inert.
        assert!(!has_privileged_attrs(&claims(&[("access", &["write"])])));
        assert!(!has_privileged_attrs(&claims(&[("hr", &["false"])])));
        assert!(!has_privileged_attrs(&claims(&[("hr", &["yes"])])));
        assert!(!has_privileged_attrs(&claims(&[("dept", &["cardiology"])])));
    }

    #[test]
    fn a_person_id_is_what_follows_the_scheme() {
        assert_eq!(person_id("person:abc"), "abc");
        assert_eq!(person_id("abc"), "abc");
    }
}
