//! Axum controllers (`axa` `Routes`), one module per pillar, plus the
//! shared error helpers.

use axa::error::ErrorDetail;
use axa::prelude::*;
use axum::http::StatusCode;

pub mod absence_categories;
pub mod acquisition;
pub mod adjustments;
pub mod ai_register;
pub mod announcements;
pub mod appraisals;
pub mod assessments;
pub mod assignments;
pub mod audits;
pub mod bargaining;
pub mod bridges;
pub mod capacity;
pub mod career;
pub mod category_vocabulary;
pub mod change;
pub mod consultations;
pub mod contacts;
pub mod conversion_plans;
pub mod cost_centres;
pub mod course_requests;
pub mod coverage_evidence;
pub mod cpd;
pub mod credentials;
pub mod cutover;
pub mod decision_gates;
pub mod development;
pub mod directory;
pub mod docs;
pub mod duplicates;
pub mod early_warning;
pub mod engagements;
pub mod equality_monitoring;
pub mod equipment;
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
pub mod holidays;
pub mod hr_core;
pub mod human_capital;
pub mod individual_goals;
pub mod injury_records;
pub mod intelligence;
pub mod job_levels;
pub mod jurisdictions;
pub mod key_person;
pub mod learning;
pub mod lms;
pub mod mandatory_learning;
pub mod mcp;
pub mod me;
pub mod metrics;
pub mod mobility;
pub mod movements;
pub mod my_payslips;
pub mod notifications;
pub mod organizations;
pub mod oversight;
mod oversight_derived;
pub mod oversight_packs;
pub mod padr;
pub mod pay_assessment;
pub mod pay_information;
pub mod pay_positions;
pub mod pay_range;
pub mod pay_scales;
pub mod pay_transparency;
pub mod payroll;
pub mod planning;
pub mod policies;
pub mod positions;
pub mod privacy;
pub mod programmes;
pub mod qualifications;
pub mod reporting;
pub mod resignations;
pub mod roles;
pub mod rota_swaps;
pub mod rotas;
pub mod separation;
pub mod sfia;
pub mod skill_gaps;
pub mod skill_mappings;
pub mod statutory_defaults;
pub mod talent;
pub mod training_economics;
pub mod training_plan;
pub mod transfers;
pub mod webhooks;
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
    ensure_within_ceiling(scope.count(&ctx.db).await?)
}

/// Refuse `count` workers over the measured ceiling (WPM-R120, WPM-D90): the rule of [`ensure_scale`], for a count already made.
///
/// # Errors
///
/// A `422` naming the count and the ceiling.
pub fn ensure_within_ceiling(count: u64) -> Result<()> {
    let ceiling = crate::rules::scale::parse_ceiling(
        crate::compat::env_var("WPM_MAX_ORG_WORKERS").as_deref(),
    );
    crate::rules::scale::check(count, ceiling).map_err(|m| unprocessable(&m))
}

/// Map a record-level authorization rejection (`(status, reason)`) to a
/// error response: `403` = policy denied; `401` = fail-safe when
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

/// `?limit=` / `?offset=` for a collection read: `axa::pagination`'s (WPM-T476). The values are text, so an absent, zero or
/// unparseable `limit` is the endpoint's default and an unparseable `offset` is `0`, rather than a refusal.
///
/// Declared as its own type (not `#[serde(flatten)]`-ed onto a wider query struct): a flattened struct deserializes from a
/// string-keyed map. A wider query that also pages builds one from its own `limit` and `offset`.
pub use axa::pagination::PageParams as Page;

/// The page a request resolved to: its `limit` and `offset`, which [`Window::bounds`] gives as `PostgreSQL` takes them and
/// [`Window::respond`] answers with `X-Total-Count`, `X-Limit` and `X-Offset`.
pub use axa::pagination::Page as Window;

/// The page `page` asks for, with `default_limit` when it names none and at most [`MAX_LIMIT`].
///
/// # Errors
///
/// A `400`, code `offset_too_large`, when the offset is past [`MAX_OFFSET`].
pub fn resolve(page: &Page, default_limit: u64) -> Result<Window> {
    page.resolve(axa::pagination::Limits {
        default_limit,
        max_limit: MAX_LIMIT,
        max_offset: MAX_OFFSET,
    })
    .map_err(|_| {
        Error::CustomError(
            StatusCode::BAD_REQUEST,
            ErrorDetail::new(
                "offset_too_large",
                format!("offset must not exceed {MAX_OFFSET}; narrow the query instead"),
            ),
        )
    })
}

/// Default page size of a whole-organization or whole-service list (WPM-T254). It matches the
/// ceiling those lists already had, so a client that sends no `limit` sees what it saw; past it,
/// `x-total-count` says how many there are.
pub(crate) const LIST_DEFAULT_LIMIT: u64 = 500;

/// The page of a list request, refusing an offset past [`MAX_OFFSET`].
///
/// # Errors
///
/// A `400` for an out-of-bound offset.
pub(crate) fn window(page: &Page) -> Result<Window> {
    resolve(page, LIST_DEFAULT_LIMIT)
}

/// One page of an already-built list, with the paging headers. For lists whose rows are
/// filtered by a rule the database cannot apply.
pub(crate) fn paged_vec<T: serde::Serialize>(items: Vec<T>, window: Window) -> Response {
    let total = i64::try_from(items.len()).unwrap_or(i64::MAX);
    let page: Vec<T> = items
        .into_iter()
        .skip(usize::try_from(window.offset).unwrap_or(usize::MAX))
        .take(usize::try_from(window.limit).unwrap_or(usize::MAX))
        .collect();
    window.respond(total, page)
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
    window: Window,
    show: impl Fn(&E::Model) -> Result<V>,
) -> Result<Response>
where
    E: sea_orm::EntityTrait,
    E::Model: Sync,
    V: serde::Serialize,
{
    use sea_orm::{PaginatorTrait, QuerySelect};
    let total = select.clone().count(&ctx.db).await?;
    let rows = select
        .limit(window.limit)
        .offset(window.offset)
        .all(&ctx.db)
        .await?;
    let shown = rows.iter().map(show).collect::<Result<Vec<V>>>()?;
    Ok(window.respond(i64::try_from(total).unwrap_or(i64::MAX), shown))
}

#[cfg(test)]
mod tests {
    use super::row_json;

    fn sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// Every handler reads its body, path and query with `axa::extract` (WPM-T475). Axum's own `Json`, `Path` and `Query`,
    /// which `axa::prelude` still exports, reject with text that quotes the caller's value. A file that extracts with one of
    /// them must import `axa`'s, or name it in full.
    #[test]
    fn every_extractor_is_axas() {
        let mut files = Vec::new();
        sources(std::path::Path::new("src"), &mut files);
        let mut wrong = Vec::new();
        for file in files {
            let source = std::fs::read_to_string(&file).unwrap();
            for name in ["Json", "Path", "Query"] {
                for axum in [format!("axum::extract::{name}<"), format!("axum::{name}<")] {
                    if source.contains(&axum) {
                        wrong.push(format!("{}: {axum}", file.display()));
                    }
                }
                let extracts = source.lines().any(|line| {
                    line.contains(&format!("): {name}<")) && !line.contains("axa::extract::")
                });
                let imported = source.lines().any(|line| {
                    line.starts_with("use axa::extract::")
                        && (line.contains(&format!("::{name};"))
                            || line.contains(&format!("{{{name}"))
                            || line.contains(&format!(" {name}"))
                            || line.contains(&format!("{name},")))
                });
                if extracts && !imported {
                    wrong.push(format!(
                        "{}: {name} without `use axa::extract`",
                        file.display()
                    ));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "use axa::extract's extractors: {wrong:#?}"
        );
    }

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
        use super::{Error, Page, StatusCode, Window, window};
        let page = |limit: Option<&str>, offset: Option<&str>| Page {
            limit: limit.map(str::to_string),
            offset: offset.map(str::to_string),
        };
        let at = |limit, offset| Window { limit, offset };
        assert_eq!(window(&page(None, None)).unwrap(), at(500, 0));
        assert_eq!(window(&page(Some("0"), Some("5"))).unwrap(), at(500, 5));
        assert_eq!(window(&page(Some("9999"), None)).unwrap(), at(500, 0));
        assert_eq!(window(&page(Some("20"), Some("40"))).unwrap(), at(20, 40));
        assert_eq!(
            window(&page(Some("many"), Some("-1"))).unwrap(),
            at(500, 0),
            "unreadable is the default"
        );
        assert_eq!(window(&page(None, Some("10000"))).unwrap(), at(500, 10_000));
        let Err(Error::CustomError(status, detail)) = window(&page(None, Some("10001"))) else {
            panic!("a deep offset is refused");
        };
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(detail.code.as_deref(), Some("offset_too_large"));
    }

    #[test]
    fn a_vec_page_is_cut_after_the_total_is_counted() {
        use axum::http::header::HeaderName;
        let window = super::Window {
            limit: 2,
            offset: 1,
        };
        let response = super::paged_vec(vec![1, 2, 3, 4, 5], window);
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
