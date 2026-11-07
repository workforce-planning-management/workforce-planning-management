//! The directory searches, orders and pages in the database (WPM-D90): the same answers as the
//! in-memory rule, with the work the size of a page.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(
    request: &axa::TestServer,
    org: &str,
    number: &str,
    name: &str,
    manager: Option<&str>,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": number,
            "display_name": name, "employment_type": "permanent", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2026-01-05", "manager_pid": manager,
        }))
        .await
        .json();
    created["pid"].as_str().expect("worker pid").to_string()
}

fn names(rows: &Value) -> Vec<&str> {
    rows.as_array()
        .expect("a list")
        .iter()
        .map(|r| r["display_name"].as_str().expect("name"))
        .collect()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn pages_are_in_name_order_ignoring_case_and_add_up_to_the_total() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        for (i, name) in ["delta", "alpha", "Charlie", "Bravo", "Echo"]
            .iter()
            .enumerate()
        {
            hire(
                &request,
                &org,
                &format!("{i}"),
                &format!("{name} {tag}"),
                None,
            )
            .await;
        }
        let mut seen = Vec::new();
        for offset in [0, 2, 4] {
            let page = request
                .get(&format!("/api/directory?q={tag}&limit=2&offset={offset}"))
                .await;
            assert_eq!(page.header("x-total-count"), "5");
            let rows: Value = page.json();
            seen.extend(names(&rows).iter().map(ToString::to_string));
        }
        let expected: Vec<String> = ["alpha", "Bravo", "Charlie", "delta", "Echo"]
            .iter()
            .map(|n| format!("{n} {tag}"))
            .collect();
        assert_eq!(seen, expected);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_search_term_is_text_not_a_pattern_and_a_managers_name_is_searchable() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let boss = hire(&request, &org, "1", &format!("Quentin Boss {tag}"), None).await;
        hire(
            &request,
            &org,
            "2",
            &format!("Report One {tag}"),
            Some(&boss),
        )
        .await;
        hire(&request, &org, "3", &format!("Stranger {tag}"), None).await;

        // `%` and `_` match themselves only, never "anything"
        for pattern in ["%25", "_", "%25%25"] {
            let rows: Value = request
                .get(&format!("/api/directory?q={tag}%20{pattern}"))
                .await
                .json();
            assert_eq!(names(&rows), Vec::<&str>::new(), "pattern {pattern}");
        }
        // both terms must match, one of them in the manager's name
        let rows: Value = request
            .get(&format!("/api/directory?q=report%20quentin%20{tag}"))
            .await
            .json();
        assert_eq!(names(&rows), [format!("Report One {tag}")]);
        let rows: Value = request
            .get(&format!("/api/directory?q=report%20{tag}"))
            .await
            .json();
        assert_eq!(rows[0]["manager_name"], format!("Quentin Boss {tag}"));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_page_past_the_end_is_empty_and_still_counts_every_match() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        hire(&request, &org, "1", &format!("One {tag}"), None).await;
        hire(&request, &org, "2", &format!("Two {tag}"), None).await;
        let page = request
            .get(&format!("/api/directory?q={tag}&limit=2&offset=50"))
            .await;
        assert_eq!(page.header("x-total-count"), "2");
        assert_eq!(names(&page.json()), Vec::<&str>::new());
    })
    .await;
}

/// The search is one statement over an array of terms, so each of these is a term that the worker
/// must match by some other route than the term before it. A worker here has no location and no
/// manager: a test on either is NULL, which must fail the term and not pass it.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn every_term_must_match_and_a_missing_value_matches_nothing() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let leeds: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org, "worker_number": "1",
                "display_name": format!("Located {tag}"), "employment_type": "permanent",
                "department": "Engineering", "job_title": "Plumber", "hired_on": "2026-01-05",
            }))
            .await
            .json();
        // the hiring route takes no location
        let pid = leeds["pid"].as_str().expect("worker pid");
        crate::requests::ddl(
            &ctx,
            &format!("UPDATE workers SET location = 'Leeds' WHERE pid = '{pid}'"),
        )
        .await;
        let unplaced = format!("Unplaced {tag}");
        hire(&request, &org, "2", &unplaced, None).await;
        let found = |q: String| {
            let request = &request;
            async move {
                let rows: Value = request
                    .get("/api/directory")
                    .add_query_param("q", q)
                    .await
                    .json();
                names(&rows)
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            }
        };
        assert_eq!(found(tag.clone()).await.len(), 2);
        // name, title, department and location, each in a term of its own and in any case
        let located = [format!("Located {tag}")];
        assert_eq!(
            found(format!("{tag} PLUMBER engineering LEEDS")).await,
            located
        );
        // one term that matches nothing is enough, whichever columns are NULL
        assert!(found(format!("{tag} nowhere-at-all")).await.is_empty());
        assert!(
            found(format!("{tag} plumber nowhere-at-all"))
                .await
                .is_empty()
        );
        // the worker with no location is not found by a location
        assert_eq!(found(format!("{tag} leeds")).await, located);
        assert_eq!(found(format!("{tag} unplaced")).await, [unplaced]);
        // only spaces is no term at all
        let rows: Value = request.get("/api/directory?q=%20%20&limit=1").await.json();
        assert_eq!(rows.as_array().map(Vec::len), Some(1));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_rota_a_worker_is_on_call_for_is_searchable_beside_another_term() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut members = Vec::new();
        for (number, name) in [("1", "Oncall"), ("2", "Offcall")] {
            let pid = hire(&request, &org, number, &format!("{name} {tag}"), None).await;
            request
                .post(&format!("/api/workers/{pid}/status"))
                .json(&json!({ "to": "active" }))
                .await
                .assert_status_ok();
            members.push(pid);
        }
        let today = chrono::Utc::now().date_naive();
        request
            .post("/api/rotas")
            .json(
                &json!({ "organization_ref": org, "name": format!("Night Desk {tag}"),
                           "period_days": 7, "starts_on": today.to_string(),
                           "members": members }),
            )
            .await
            .assert_status_ok();
        let oncall = [format!("Oncall {tag}")];
        let rows: Value = request
            .get(&format!("/api/directory?q=night%20desk%20{tag}"))
            .await
            .json();
        assert_eq!(names(&rows), oncall, "only whoever is on call today");
        let rows: Value = request
            .get(&format!("/api/directory?q=oncall%20night%20{tag}"))
            .await
            .json();
        assert_eq!(names(&rows), oncall);
        let rows: Value = request
            .get(&format!("/api/directory?q=offcall%20night%20{tag}"))
            .await
            .json();
        assert_eq!(names(&rows), Vec::<&str>::new());
    })
    .await;
}
