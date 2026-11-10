//! Cursor paging (WPM-T478, WPM-D189; `agents/share/restful.md`): a list takes `?limit=` and a signed `?cursor=`, and
//! answers a bare array with `Link: <...>; rel="next"` while more rows follow. There is no total. A bad `limit`, a cursor
//! this service did not sign for this list and these filters, and an `offset` are each a `422`.
//!
//! `#[ignore]`d: needs PostgreSQL; run with `cargo test -- --ignored`.

use serde_json::Value;
use serial_test::serial;

use super::{an_org, seed_worker};

/// The target of a response's `rel="next"` link, as the client calls it, or `None` on the last page.
fn next(response: &axum_test::TestResponse) -> Option<String> {
    let link = response.headers().get("link")?.to_str().ok()?;
    let target = link.strip_prefix('<')?.split_once('>')?;
    assert_eq!(target.1, "; rel=\"next\"", "{link}");
    Some(target.0.to_string())
}

/// `cursor` with its last character changed.
fn edited(cursor: &str) -> String {
    let (head, last) = cursor.split_at(cursor.len() - 1);
    format!("{head}{}", if last == "A" { "B" } else { "A" })
}

fn no_totals(response: &axum_test::TestResponse, what: &str) {
    for header in ["x-total-count", "x-limit", "x-offset"] {
        assert!(
            response.headers().get(header).is_none(),
            "{what}: no {header}"
        );
    }
}

/// Every page of `first`, following `Link` to the end: the rows, and how many pages there were.
async fn walk(request: &axa_test::TestServer, first: &str) -> (Vec<Value>, usize) {
    let mut rows = Vec::new();
    let mut pages = 0;
    let mut at = Some(first.to_string());
    while let Some(path) = at {
        let page = request.get(&path).await;
        assert_eq!(page.status_code(), 200, "{path}");
        no_totals(&page, &path);
        rows.extend(
            page.json::<Value>()
                .as_array()
                .expect("a bare array")
                .clone(),
        );
        pages += 1;
        at = next(&page);
        assert!(pages < 1000, "the pages end");
    }
    (rows, pages)
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn worker_list_pages_by_cursor_to_the_end_with_each_row_once() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let mut seeded = Vec::new();
        for i in 0..5 {
            seeded.push(seed_worker!(&request, org.clone(), format!("EMP-PAGE-{i}"), None).await);
        }

        let (whole, pages) = walk(&request, "/api/workers").await;
        let (paged, small_pages) = walk(&request, "/api/workers?limit=2").await;
        assert_eq!(pages, 1, "a list under the default limit is one page");
        let pids = |rows: &[Value]| {
            rows.iter()
                .map(|r| r["pid"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            pids(&paged),
            pids(&whole),
            "the same rows in the same order, each once"
        );
        assert_eq!(small_pages, whole.len().div_ceil(2));
        for pid in &seeded {
            assert_eq!(
                pids(&paged).iter().filter(|p| *p == pid).count(),
                1,
                "{pid} once"
            );
        }

        // The `Link` names the path the client called, keeps its filters and `limit`, and the limit may change between pages.
        let first = request
            .get("/api/workers?department=engineering&limit=2")
            .await;
        let target = next(&first).expect("more than two engineers");
        assert!(
            target.starts_with("/api/workers?department=engineering&limit=2&cursor="),
            "{target}"
        );
        let cursor = target.rsplit_once("cursor=").unwrap().1.to_string();
        let wider = request
            .get(&format!(
                "/api/workers?department=engineering&limit=3&cursor={cursor}"
            ))
            .await;
        assert_eq!(wider.status_code(), 200);
        assert_eq!(wider.json::<Value>().as_array().unwrap().len(), 3);

        // A cursor fits only its list and filters, and cannot be edited or made up.
        for path in [
            format!("/api/workers?department=finance&limit=2&cursor={cursor}"),
            format!("/api/workers?limit=2&cursor={cursor}"),
            format!("/api/benefit-plans?department=engineering&limit=2&cursor={cursor}"),
            format!(
                "/api/workers?department=engineering&limit=2&cursor={}",
                edited(&cursor)
            ),
            "/api/workers?cursor=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .to_string(),
        ] {
            assert_eq!(request.get(&path).await.status_code(), 422, "{path}");
        }

        // A limit is refused, not clamped or defaulted; an offset is refused so that a client still paging by offset is told.
        for path in [
            "/api/workers?limit=501",
            "/api/workers?limit=0",
            "/api/workers?limit=many",
            "/api/workers?offset=2",
            "/api/workers?limit=2&offset=0",
        ] {
            let refused = request.get(path).await;
            assert_eq!(refused.status_code(), 422, "{path}");
            let body: Value = refused.json();
            assert_eq!(body["code"], "unprocessable_entity", "{path}: {body}");
            assert!(!body.to_string().contains("many"), "{path}: quotes nothing");
        }
        assert_eq!(
            request.get("/api/workers?limit=500").await.status_code(),
            200
        );
    })
    .await;
}

/// The row fetched past a page only says that more follow: it is not sent, so its salary is not recorded as read
/// (WPM-D147).
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_page_records_a_salary_read_only_for_the_rows_it_sends() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let department = format!("paging-audit-{}", uuid::Uuid::new_v4().simple());
        let mut pids = Vec::new();
        for i in 0..3 {
            let created: Value = request
                .post("/api/workers")
                .json(&serde_json::json!({
                    "person_ref": crate::requests::a_person(),
                    "organization_ref": org,
                    "worker_number": format!("EMP-AUDIT-{i}"),
                    "display_name": format!("Audit Worker {i}"),
                    "employment_type": "permanent",
                    "department": department,
                    "job_title": "Engineer",
                    "salary_minor": 4_000_000,
                    "salary_currency": "GBP",
                    "hired_on": "2026-01-05",
                }))
                .await
                .json();
            pids.push(uuid::Uuid::parse_str(created["pid"].as_str().unwrap()).unwrap());
        }
        let reads = || async {
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM audit_logs WHERE action = 'salary_read' AND entity_pid = ANY($1)",
            )
            .bind(&pids)
            .fetch_one(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap()
        };
        let before = reads().await;
        let first = request.get(&format!("/api/workers?department={department}&limit=1")).await;
        assert_eq!(first.json::<Value>().as_array().unwrap().len(), 1);
        assert_eq!(reads().await - before, 1, "one row sent, one read recorded");
        let rest = request.get(&next(&first).expect("more follow")).await;
        assert_eq!(rest.json::<Value>().as_array().unwrap().len(), 1);
        assert_eq!(reads().await - before, 2);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn benefit_plan_list_pages_by_cursor() {
    crate::requests::request_open(|request, _ctx| async move {
        for i in 0..3 {
            let created = request
                .post("/api/benefit-plans")
                .json(&serde_json::json!({
                    "name": format!("Test Plan {i}"),
                    "kind": "pension",
                    "provider": "Test Provider",
                    "worker_cost_minor": 0,
                    "employer_cost_minor": 0,
                    "currency": "GBP",
                }))
                .await;
            assert_eq!(created.status_code(), 200, "plan {i} should create");
        }
        let (whole, _) = walk(&request, "/api/benefit-plans").await;
        let (paged, pages) = walk(&request, "/api/benefit-plans?limit=1").await;
        assert_eq!(paged, whole);
        assert!(pages >= 3, "{pages}");
        assert_eq!(
            request
                .get("/api/benefit-plans?offset=1")
                .await
                .status_code(),
            422
        );
    })
    .await;
}

/// The whole-service and whole-organization lists page the same way (WPM-T254, WPM-T478): a bare array, no totals,
/// and an offset refused.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn org_lists_page_by_cursor_and_refuse_an_offset() {
    crate::requests::request_open(|request, _ctx| async move {
        for path in [
            "/api/movements",
            "/api/requisitions",
            "/api/payroll-runs",
            "/api/resignations",
            "/api/expense-claims",
            "/api/flexible-working",
            "/api/conversion-plans",
        ] {
            let page = request.get(&format!("{path}?limit=1")).await;
            assert_eq!(page.status_code(), 200, "{path}");
            assert!(
                page.json::<Value>().is_array(),
                "{path} stays a plain array"
            );
            no_totals(&page, path);
            if let Some(target) = next(&page) {
                assert!(
                    target.starts_with(&format!("{path}?limit=1&cursor=")),
                    "{target}"
                );
            }
            let offset = request.get(&format!("{path}?offset=1")).await;
            assert_eq!(offset.status_code(), 422, "{path}: an offset is refused");
            let deep = request.get(&format!("{path}?limit=501")).await;
            assert_eq!(deep.status_code(), 422, "{path}: past MAX_LIMIT");
        }
    })
    .await;
}
