//! The directory searches, orders and pages in the database (WPM-D90): the same answers as the
//! in-memory rule, with the work the size of a page.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(
    request: &loco_rs::TestServer,
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
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
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
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
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
