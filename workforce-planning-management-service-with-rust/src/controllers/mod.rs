//! Axum controllers (loco `Routes`), one module per pillar, plus the
//! shared error helpers.

use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;

pub mod acquisition;
pub mod adjustments;
pub mod announcements;
pub mod appraisals;
pub mod assessments;
pub mod audits;
pub mod capacity;
pub mod career;
pub mod change;
pub mod contacts;
pub mod conversion_plans;
pub mod cpd;
pub mod cutover;
pub mod development;
pub mod directory;
pub mod docs;
pub mod duplicates;
pub mod early_warning;
pub mod engagements;
pub mod equality_monitoring;
pub mod ergonomics;
pub mod esco;
pub mod expenses;
pub mod flexible_working;
pub mod framework_roles;
pub mod funding;
pub mod gap_sql;
pub mod governance;
pub mod grades;
pub mod groups;
pub mod handover;
pub mod health_requirements;
pub mod hr_core;
pub mod intelligence;
pub mod job_levels;
pub mod key_person;
pub mod learning;
pub mod lms;
pub mod me;
pub mod metrics;
pub mod mobility;
pub mod movements;
pub mod notifications;
pub mod organizations;
pub mod oversight;
mod oversight_derived;
pub mod oversight_packs;
pub mod pay_positions;
pub mod pay_scales;
pub mod payroll;
pub mod pay_information;
pub mod pay_transparency;
pub mod planning;
pub mod privacy;
pub mod programmes;
pub mod reporting;
pub mod resignations;
pub mod roles;
pub mod rota_swaps;
pub mod rotas;
pub mod skill_gaps;
pub mod talent;
pub mod training_plan;
pub mod transfers;
pub mod wellbeing;
pub mod workforce;

/// Map a non-empty problem list to the family's `422 Unprocessable
/// Entity` validation error (every problem in one response).
///
/// # Errors
///
/// Returns the `422` error when `problems` is non-empty.
pub fn ensure_valid(problems: &[String]) -> Result<()> {
    if problems.is_empty() {
        return Ok(());
    }
    Err(Error::CustomError(
        StatusCode::UNPROCESSABLE_ENTITY,
        ErrorDetail::new("validation", problems.join("; ")),
    ))
}

/// Ids per `IN (…)` list. A statement carries at most 65,535 bind parameters, so a list of every worker
/// of a large organization (WPM-R142) fails outright; a list this size is well under it.
pub(crate) const IN_BATCH: usize = 5_000;

/// Every row of `select` whose `column` is one of `ids`, read in batches of [`IN_BATCH`] ids.
///
/// # Errors
///
/// Propagates a database error.
pub(crate) async fn find_in<E, C>(
    ctx: &AppContext,
    select: sea_orm::Select<E>,
    column: C,
    ids: &[uuid::Uuid],
) -> Result<Vec<E::Model>>
where
    E: sea_orm::EntityTrait,
    C: sea_orm::ColumnTrait,
{
    let mut rows = Vec::new();
    for chunk in ids.chunks(IN_BATCH) {
        rows.extend(
            select
                .clone()
                .filter(column.is_in(chunk.to_vec()))
                .all(&ctx.db)
                .await?,
        );
    }
    Ok(rows)
}

/// A `422` with a single message (state-machine refusals etc.).
#[must_use]
pub fn unprocessable(message: &str) -> Error {
    Error::CustomError(
        StatusCode::UNPROCESSABLE_ENTITY,
        ErrorDetail::new("unprocessable", message),
    )
}

/// Refuse a whole-organization read that would load more workers than the measured ceiling
/// (WPM-R120, WPM-D90). `scope` is the same `workers` query the view is about to load, so the count
/// is of exactly those rows. The check costs one indexed `COUNT`.
///
/// # Errors
///
/// A `422` naming the count and the ceiling, or a database error.
pub async fn ensure_scale(
    ctx: &AppContext,
    scope: sea_orm::Select<crate::models::_entities::workers::Entity>,
) -> Result<()> {
    use sea_orm::PaginatorTrait;
    let count = scope.count(&ctx.db).await?;
    let ceiling = crate::rules::scale::parse_ceiling(
        crate::compat::env_var("WPM_MAX_ORG_WORKERS").as_deref(),
    );
    crate::rules::scale::check(count, ceiling).map_err(|m| unprocessable(&m))
}

/// Map a record-level authorization rejection (`(status, reason)`) to a
/// loco error response: `403` = policy denied; `401` = fail-safe when
/// claims are missing behind the guard.
#[must_use]
pub fn record_rejection((status, reason): (StatusCode, String)) -> Error {
    let code = if status == StatusCode::FORBIDDEN {
        "forbidden"
    } else {
        "unauthorized"
    };
    Error::CustomError(status, ErrorDetail::new(code, &reason))
}

/// A `401` with a single message: the caller is not signed in (WPM-T259).
#[must_use]
pub fn unauthorized(message: &str) -> Error {
    record_rejection((StatusCode::UNAUTHORIZED, message.to_string()))
}

/// An internal invariant broke (a query that must return a row did not, a stored value is not
/// one the code writes). The cause is logged; the caller gets a `500` with no detail (WPM-T259).
#[must_use]
pub fn internal(what: &str) -> Error {
    tracing::error!(invariant = what, "internal invariant broken");
    Error::CustomError(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorDetail::new("internal", "internal error"),
    )
}

/// A `403` with a single message, for a refusal the caller's role earns (WPM-T251).
#[must_use]
pub fn forbidden(message: &str) -> Error {
    record_rejection((StatusCode::FORBIDDEN, message.to_string()))
}

/// A worker the caller may change: HR, or the worker themself.
///
/// # Errors
///
/// `404` for an unknown pid, `403` when the caller may not write the record.
pub(crate) async fn writable_worker(
    ctx: &AppContext,
    caller: &crate::auth::MaybeAuthUser,
    pid: &str,
) -> Result<crate::models::_entities::workers::Model> {
    let worker =
        crate::models::records::find_worker(&ctx.db, crate::models::records::parse_pid(pid)?)
            .await?;
    crate::auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &crate::auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

/// `404` for an organization outside the caller's scope, as for every scoped read.
///
/// # Errors
///
/// `404` when out of scope; a database error.
pub(crate) async fn ensure_org_in_scope(
    ctx: &AppContext,
    caller: &crate::auth::MaybeAuthUser,
    org: &str,
) -> Result<()> {
    if let Some(refs) =
        crate::models::memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

/// Render a `(numerator, denominator, value)` triple as a terms-carrying ratio object, or
/// `null` when there is nothing to divide.
pub(crate) fn ratio_json(terms: Option<(usize, usize, f64)>) -> serde_json::Value {
    terms.map_or(serde_json::Value::Null, |(numerator, denominator, value)| {
        serde_json::json!({ "numerator": numerator, "denominator": denominator, "value": value })
    })
}

/// A stored row as a response: the row's own fields without its internal `id` (a surrogate
/// key, never an identifier a client may use; the `pid` is) or its `deleted_at` (WPM-T252).
///
/// # Errors
///
/// When the row does not serialize to a JSON object.
pub(crate) fn row_json<T: serde::Serialize>(row: &T) -> Result<serde_json::Value> {
    let mut value = serde_json::to_value(row).map_err(|e| internal(&e.to_string()))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("id");
        object.remove("deleted_at");
    }
    Ok(value)
}

// ─── Pagination (WPM-T40; agents/share/restful.md) ──────────────────────────

/// Largest page any collection read will serve. A bigger `limit` is
/// **clamped** to this rather than refused.
pub const MAX_LIMIT: u64 = 500;

/// Largest accepted `offset`; past this a request is a `400` — an
/// unbounded offset would otherwise make the database materialise
/// arbitrarily many rows only to discard them (SEC-G7). Deep paging
/// past this bound wants a cursor, not a bigger number.
pub const MAX_OFFSET: u64 = 10_000;

/// `?limit=` / `?offset=` for a collection read.
///
/// Declared as its own type (not `#[serde(flatten)]`-ed onto a wider
/// query struct): a flattened struct deserializes from a string-keyed
/// map, so `limit=2` would arrive as the string `"2"` and fail to parse
/// as a `u64` — a spurious `400` on a valid request.
#[derive(Debug, Default, Clone, Copy, serde::Deserialize)]
pub struct Page {
    /// Page size; absent, zero, or unparseable ⇒ the endpoint default.
    #[serde(default)]
    pub limit: Option<u64>,
    /// Rows to skip; absent ⇒ 0.
    #[serde(default)]
    pub offset: Option<u64>,
}

impl Page {
    /// The clamped `(limit, offset)` this request will actually use. A
    /// zero `limit` falls back to `default_limit`: an empty page and an
    /// empty collection look identical to a client, and only one is an
    /// answer.
    #[must_use]
    pub fn resolve(self, default_limit: u64) -> (u64, u64) {
        let limit = self
            .limit
            .filter(|l| *l > 0)
            .unwrap_or(default_limit)
            .min(MAX_LIMIT);
        (limit, self.offset.unwrap_or(0))
    }

    /// Reject an out-of-bound offset before it reaches the database.
    ///
    /// # Errors
    ///
    /// Returns a `400` when the offset exceeds [`MAX_OFFSET`].
    pub fn check_offset(self) -> Result<()> {
        if self.offset.unwrap_or(0) > MAX_OFFSET {
            return Err(Error::CustomError(
                StatusCode::BAD_REQUEST,
                ErrorDetail::new(
                    "offset_too_large",
                    format!("offset must not exceed {MAX_OFFSET}; narrow the query instead"),
                ),
            ));
        }
        Ok(())
    }
}

/// Stamp `X-Total-Count` / `X-Limit` / `X-Offset` onto a response
/// (`agents/share/restful.md`).
#[must_use]
pub fn with_page_headers(mut response: Response, total: u64, limit: u64, offset: u64) -> Response {
    let headers = response.headers_mut();
    for (name, value) in [
        ("x-total-count", total),
        ("x-limit", limit),
        ("x-offset", offset),
    ] {
        if let Ok(value) = axum::http::HeaderValue::from_str(&value.to_string()) {
            headers.insert(name, value);
        }
    }
    response
}

/// Default page size of a whole-organization or whole-service list (WPM-T254). It matches the
/// ceiling those lists already had, so a client that sends no `limit` sees what it saw; past it,
/// `x-total-count` says how many there are.
pub(crate) const LIST_DEFAULT_LIMIT: u64 = 500;

/// The `(limit, offset)` of a list request, refusing an offset past [`MAX_OFFSET`].
///
/// # Errors
///
/// A `400` for an out-of-bound offset.
pub(crate) fn window(page: Page) -> Result<(u64, u64)> {
    page.check_offset()?;
    Ok(page.resolve(LIST_DEFAULT_LIMIT))
}

/// One page of an already-built list, with the paging headers. For lists whose rows are
/// filtered by a rule the database cannot apply.
///
/// # Errors
///
/// When the page does not serialize.
pub(crate) fn paged_vec<T: serde::Serialize>(
    items: Vec<T>,
    (limit, offset): (u64, u64),
) -> Result<Response> {
    let total = items.len() as u64;
    let page: Vec<T> = items
        .into_iter()
        .skip(usize::try_from(offset).unwrap_or(usize::MAX))
        .take(usize::try_from(limit).unwrap_or(usize::MAX))
        .collect();
    Ok(with_page_headers(format::json(page)?, total, limit, offset))
}

/// One page of `select`, counted and cut by the database, each row shown by `show`. `select`
/// must already be ordered, or a page is not stable.
///
/// # Errors
///
/// A database error.
pub(crate) async fn paged_select<E, V>(
    ctx: &AppContext,
    select: sea_orm::Select<E>,
    (limit, offset): (u64, u64),
    show: impl Fn(&E::Model) -> Result<V>,
) -> Result<Response>
where
    E: sea_orm::EntityTrait,
    E::Model: Sync,
    V: serde::Serialize,
{
    use sea_orm::{PaginatorTrait, QuerySelect};
    let total = select.clone().count(&ctx.db).await?;
    let rows = select.limit(limit).offset(offset).all(&ctx.db).await?;
    let shown = rows.iter().map(show).collect::<Result<Vec<V>>>()?;
    Ok(with_page_headers(
        format::json(shown)?,
        total,
        limit,
        offset,
    ))
}

#[cfg(test)]
mod tests {
    use super::row_json;

    /// The cause of a broken invariant stays in the log; the caller learns only that it was internal.
    #[test]
    fn an_internal_error_is_a_500_that_names_nothing() {
        use super::{Error, StatusCode, internal, unauthorized};
        let Error::CustomError(status, detail) = internal("secret table name") else {
            panic!("a custom error");
        };
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let text = format!("{detail:?}");
        assert!(
            text.contains("internal error") && !text.contains("secret"),
            "{text}"
        );
        let Error::CustomError(status, _) = unauthorized("sign in") else {
            panic!("a custom error");
        };
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn a_window_clamps_and_refuses_a_deep_offset() {
        use super::{Page, window};
        let page = |limit, offset| Page { limit, offset };
        assert_eq!(window(page(None, None)).unwrap(), (500, 0));
        assert_eq!(window(page(Some(0), Some(5))).unwrap(), (500, 5));
        assert_eq!(window(page(Some(9_999), None)).unwrap(), (500, 0));
        assert_eq!(window(page(Some(20), Some(40))).unwrap(), (20, 40));
        assert!(window(page(None, Some(u64::MAX))).is_err());
    }

    #[test]
    fn a_vec_page_is_cut_after_the_total_is_counted() {
        use axum::http::header::HeaderName;
        let response = super::paged_vec(vec![1, 2, 3, 4, 5], (2, 1)).unwrap();
        let header = |n: &'static str| {
            response.headers()[HeaderName::from_static(n)]
                .to_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(header("x-total-count"), "5");
        assert_eq!(header("x-limit"), "2");
        assert_eq!(header("x-offset"), "1");
    }

    #[test]
    fn a_row_answers_without_its_surrogate_key_or_deletion_mark() {
        let row = serde_json::json!({ "id": 7, "pid": "p", "deleted_at": null, "note": "n" });
        assert_eq!(
            row_json(&row).unwrap(),
            serde_json::json!({ "pid": "p", "note": "n" })
        );
    }

    /// No handler answers with a bare stored row (WPM-T252): it goes through [`row_json`].
    #[test]
    fn no_handler_answers_with_a_bare_row() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/controllers");
        let mut found = Vec::new();
        let mut stack = vec![dir];
        while let Some(d) = stack.pop() {
            for entry in std::fs::read_dir(d).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let source = std::fs::read_to_string(&path).unwrap();
                    for (n, line) in source.lines().enumerate() {
                        let t = line.trim();
                        if t == "format::json(row)"
                            || t == "format::json(updated)"
                            || t == "format::json(saved)"
                        {
                            found.push(format!("{}:{}", path.display(), n + 1));
                        }
                    }
                }
            }
        }
        assert!(
            found.is_empty(),
            "bare rows answered:\n{}",
            found.join("\n")
        );
    }
}
