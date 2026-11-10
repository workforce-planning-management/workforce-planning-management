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
        crate::models::memberships::scope_organization_refs(ctx, caller.claims()).await?
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

// ─── Pagination (WPM-T40, WPM-T478; agents/share/restful.md) ─────────────────

/// Largest page any collection read will serve. A bigger `limit` is a `422`, not clamped.
pub const MAX_LIMIT: u64 = 500;

/// The key that signs every list's cursors (WPM-D189): `WPM_CURSOR_KEY`, or one made for this process.
static CURSOR_KEY: std::sync::OnceLock<CursorKey> = std::sync::OnceLock::new();

/// A key of 32 random bytes, for a process given none. Two v4 ids are 244 random bits from the operating system's
/// generator, which is ample for a key that only signs positions.
fn random_cursor_key() -> CursorKey {
    let mut secret = [0u8; 32];
    secret[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    secret[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    CursorKey::new(&secret).expect("32 bytes is long enough")
}

/// The cursor key from `WPM_CURSOR_KEY`'s value: base64url of at least 32 bytes. `None` means one is made for this process.
///
/// # Errors
///
/// A value that is not base64url, or is shorter than 32 bytes, so a deployment that meant to set a key does not start
/// without it. The message never quotes the value.
pub fn cursor_key_from(value: Option<&str>) -> std::result::Result<Option<CursorKey>, String> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let secret = URL_SAFE_NO_PAD
        .decode(value.trim_end_matches('='))
        .map_err(|_| "WPM_CURSOR_KEY is not base64url".to_string())?;
    CursorKey::new(&secret).map(Some).map_err(|short| {
        format!(
            "WPM_CURSOR_KEY is {} bytes; it needs at least 32",
            short.len
        )
    })
}

/// Hold the cursor key for this process: the one given, or a random one. A later call changes nothing.
pub fn install_cursor_key(key: Option<CursorKey>) {
    let _ = CURSOR_KEY.set(key.unwrap_or_else(random_cursor_key));
}

fn cursor_key() -> &'static CursorKey {
    CURSOR_KEY.get_or_init(random_cursor_key)
}

/// A list request's paging, from its URI: `?limit=` and the signed `?cursor=` of `axa::cursor_pagination`, read by
/// [`resolve`] (WPM-T478). It is the whole URI because a cursor is signed over the path and every other parameter, so a
/// cursor fits only the list and filters it came from.
#[derive(Debug, Clone)]
pub struct Page {
    uri: axum::http::Uri,
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for Page {
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> impl std::future::Future<Output = std::result::Result<Self, Self::Rejection>> + Send {
        // The URI as the client sent it, so the `Link` it is given names the path it can call.
        let uri = parts
            .extensions
            .get::<axum::extract::OriginalUri>()
            .map_or_else(|| parts.uri.clone(), |original| original.0.clone());
        std::future::ready(Ok(Self { uri }))
    }
}

impl Page {
    /// The paging of a request to `uri`, for a handler or test that has no extractor.
    #[must_use]
    pub fn of(uri: axum::http::Uri) -> Self {
        Self { uri }
    }
}

/// The page a request resolved to: [`Window::bounds`] gives `LIMIT` (one more than the page, to see whether more follow)
/// and `OFFSET` as `PostgreSQL` takes them, and [`Window::respond`] answers a bare array with `Link: <...>; rel="next"`
/// while more rows follow. There is no total, because a total would count rows the caller may not see.
pub use axa::cursor_pagination::CursorPage as Window;
use axa::cursor_pagination::{CursorKey, CursorLimits};

/// The page `page` asks for, with `default_limit` when it names none.
///
/// # Errors
///
/// A `422` for a `limit` that is zero, not a number, given twice or over [`MAX_LIMIT`]; for a cursor this service did
/// not sign for this list and these filters; and for an `offset`, which is no longer taken, so that a client still
/// paging by offset is told rather than given the first page again.
pub fn resolve(page: &Page, default_limit: u64) -> Result<Window> {
    let offset = page
        .uri
        .query()
        .unwrap_or("")
        .split('&')
        .any(|part| part.split_once('=').map_or(part, |(name, _)| name) == "offset");
    if offset {
        // The same `code` as `axa`'s refusal of a bad limit or cursor, so a client handles every paging refusal alike.
        return Err(Error::CustomError(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorDetail::new(
                "unprocessable_entity",
                "offset is not taken; follow the Link header's next page",
            ),
        ));
    }
    Ok(Window::new(
        cursor_key(),
        CursorLimits {
            default_limit,
            max_limit: MAX_LIMIT,
        },
        &page.uri,
    )?)
}

/// Default page size of a whole-organization or whole-service list (WPM-T254). It matches the
/// ceiling those lists already had, so a client that sends no `limit` sees what it saw; past it,
/// the `Link` header points to the next page.
pub(crate) const LIST_DEFAULT_LIMIT: u64 = 500;

/// The page of a list request, at [`LIST_DEFAULT_LIMIT`] unless it names a limit.
///
/// # Errors
///
/// A `422` for a bad limit, cursor or an offset ([`resolve`]).
pub(crate) fn window(page: &Page) -> Result<Window> {
    resolve(page, LIST_DEFAULT_LIMIT)
}

/// One page of an already-built list. For lists whose rows are filtered by a rule the database cannot apply.
pub(crate) fn paged_vec<T: serde::Serialize>(items: Vec<T>, window: &Window) -> Response {
    let page: Vec<T> = items
        .into_iter()
        .skip(usize::try_from(window.offset()).unwrap_or(usize::MAX))
        .take(
            usize::try_from(window.limit())
                .unwrap_or(usize::MAX)
                .saturating_add(1),
        )
        .collect();
    window.respond(page)
}

/// `response` with the `Link` to the next page when `more` rows follow: for a list whose body is not a bare array.
pub(crate) fn with_next(mut response: Response, window: &Window, more: bool) -> Response {
    if more && let Some(link) = window.next_link() {
        response
            .headers_mut()
            .insert(axum::http::header::LINK, link);
    }
    response
}

/// One page of `select`, cut by the database, each row shown by `show`. `select` must already be ordered by something
/// that ends in a unique column, or a page is not stable.
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
    use sea_orm::QuerySelect;
    let (limit, offset) = window.bounds();
    let rows = select
        .limit(limit.unsigned_abs())
        .offset(offset.unsigned_abs())
        .all(&ctx.db)
        .await?;
    let shown = rows.iter().map(show).collect::<Result<Vec<V>>>()?;
    Ok(window.respond(shown))
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

    fn page(uri: &str) -> super::Page {
        super::Page::of(uri.parse().unwrap())
    }

    /// The `cursor` in a response's `Link`, and the target it names.
    fn next(response: &axum::response::Response) -> Option<(String, String)> {
        let link = response
            .headers()
            .get("link")?
            .to_str()
            .unwrap()
            .to_string();
        let target = link.strip_prefix('<')?.split_once('>')?.0.to_string();
        let cursor = target
            .split_once("cursor=")?
            .1
            .split('&')
            .next()?
            .to_string();
        Some((target, cursor))
    }

    fn unprocessable<T: std::fmt::Debug>(result: super::Result<T>) -> String {
        let Err(error) = result else {
            panic!("refused: {result:?}");
        };
        let response = axum::response::IntoResponse::into_response(error);
        assert_eq!(response.status(), super::StatusCode::UNPROCESSABLE_ENTITY);
        format!("{response:?}")
    }

    #[test]
    fn a_limit_is_the_default_when_absent_and_refused_when_bad() {
        use super::{resolve, window};
        let w = window(&page("/api/things")).unwrap();
        assert_eq!((w.limit(), w.offset()), (500, 0));
        assert_eq!(resolve(&page("/api/things"), 50).unwrap().limit(), 50);
        assert_eq!(window(&page("/api/things?limit=20")).unwrap().limit(), 20);
        assert_eq!(window(&page("/api/things?limit=500")).unwrap().limit(), 500);
        // Refused, never clamped or defaulted (WPM-T478).
        for bad in ["0", "501", "many", "-1"] {
            unprocessable(window(&page(&format!("/api/things?limit={bad}"))));
        }
        unprocessable(window(&page("/api/things?limit=2&limit=3")));
    }

    #[test]
    fn an_offset_is_refused_so_an_old_client_is_told() {
        let text = unprocessable(super::window(&page("/api/things?offset=500")));
        assert!(!text.contains("500"), "{text}");
        unprocessable(super::window(&page("/api/things?status=open&offset")));
        // A parameter whose name only contains "offset" is a filter like any other.
        super::window(&page("/api/things?utc_offset=1")).unwrap();
    }

    #[test]
    fn a_page_links_to_the_next_while_more_follow_and_the_cursor_moves_on() {
        let first = super::window(&page("/api/things?status=open&limit=2")).unwrap();
        assert_eq!(
            first.bounds(),
            (3, 0),
            "one more row than the page, to see if more follow"
        );
        let response = super::paged_vec(vec![1, 2, 3, 4, 5], &first);
        let (target, cursor) = next(&response).expect("more follow");
        assert!(
            target.starts_with("/api/things?status=open&limit=2&cursor="),
            "{target}"
        );
        for header in ["x-total-count", "x-limit", "x-offset"] {
            assert!(response.headers().get(header).is_none(), "no {header}");
        }
        let second = super::window(&page(&target)).unwrap();
        assert_eq!(second.offset(), 2);
        let last = super::window(&page(&format!(
            "/api/things?status=open&limit=2&cursor={cursor}"
        )))
        .unwrap();
        let response = super::paged_vec(vec![1, 2, 3, 4], &last);
        assert!(next(&response).is_none(), "the last page has no Link");
    }

    #[test]
    fn a_cursor_fits_only_its_own_list_and_filters() {
        let first = super::window(&page("/api/things?status=open&limit=1")).unwrap();
        let (_, cursor) = next(&super::paged_vec(vec![1, 2], &first)).unwrap();
        unprocessable(super::window(&page(&format!(
            "/api/things?status=closed&cursor={cursor}"
        ))));
        unprocessable(super::window(&page(&format!(
            "/api/other?status=open&cursor={cursor}"
        ))));
        unprocessable(super::window(&page("/api/things?status=open&cursor=AAAA")));
        // The limit may change between pages.
        super::window(&page(&format!(
            "/api/things?status=open&limit=7&cursor={cursor}"
        )))
        .unwrap();
    }

    #[test]
    fn a_cursor_key_is_base64url_of_32_bytes_and_never_quoted() {
        use super::cursor_key_from;
        assert!(cursor_key_from(None).unwrap().is_none());
        assert!(cursor_key_from(Some("  ")).unwrap().is_none());
        let good = "QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE";
        assert!(cursor_key_from(Some(good)).unwrap().is_some());
        assert!(
            cursor_key_from(Some(&format!("{good}=")))
                .unwrap()
                .is_some(),
            "padding is allowed"
        );
        let short = cursor_key_from(Some("c2hvcnQta2V5")).unwrap_err();
        assert!(
            short.contains("9 bytes") && !short.contains("c2hvcnQ"),
            "{short}"
        );
        let bad = cursor_key_from(Some("not base64!")).unwrap_err();
        assert!(!bad.contains("not base64!"), "{bad}");
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
