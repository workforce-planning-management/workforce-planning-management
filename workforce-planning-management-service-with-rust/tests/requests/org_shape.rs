//! The organization now and as a plan intends it (WPM-R153): the diff compares roles and counts, names no one, and withholds a
//! small count together with the change that would give it back.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(request: &loco_rs::TestServer, org: &str, number: &str, title: &str, dept: &str, manager: Option<&str>) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": number,
            "display_name": format!("Shape {number}"), "employment_type": "permanent",
            "department": dept, "job_title": title, "hired_on": "2020-01-05", "manager_pid": manager,
        }))
        .await
        .json();
    created["pid"].as_str().unwrap().to_string()
}

fn role<'a>(body: &'a Value, title: &str) -> &'a Value {
    body["diff"]["roles"].as_array().unwrap().iter().find(|r| r["title"] == title).unwrap_or_else(|| panic!("{title}"))
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn the_plan_is_compared_with_the_live_structure_by_role_and_names_no_one() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        // A head, two managers, ten engineers (five each).
        let head = hire(&request, &org, "S-H", "Head", "exec", None).await;
        let m1 = hire(&request, &org, "S-M1", "Manager", "eng", Some(&head)).await;
        let m2 = hire(&request, &org, "S-M2", "Manager", "eng", Some(&head)).await;
        for i in 0..5 {
            hire(&request, &org, &format!("S-A{i}"), "Engineer", "eng", Some(&m1)).await;
            hire(&request, &org, &format!("S-B{i}"), "Engineer", "eng", Some(&m2)).await;
        }
        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({ "name": "Flatten", "organization_ref": org, "horizon_start": "2026-01-01", "horizon_end": "2027-12-31" }))
            .await
            .json();
        let pid = plan["pid"].as_str().unwrap().to_string();
        let diff = format!("/api/workforce-plans/{pid}/organization-diff");
        let target = format!("/api/workforce-plans/{pid}/target-organization");

        // No target yet: it says so.
        let none: Value = request.get(&diff).await.json();
        assert_eq!(none["target_set"], false);

        // Faults are refused.
        let node = |key: &str, parent: Option<&str>, dept: &str, title: &str, n: i32| json!({ "key": key, "parent": parent, "department": dept, "title": title, "headcount": n });
        for bad in [
            json!({ "nodes": [] }),
            json!({ "nodes": [node("a", Some("zzz"), "eng", "T", 1)] }),
            json!({ "nodes": [node("a", Some("b"), "eng", "T", 1), node("b", Some("a"), "eng", "U", 1)] }),
            json!({ "nodes": [node("a", None, "eng", "T", 1), node("a", None, "eng", "U", 1)] }),
            json!({ "nodes": [node("a", None, "eng", "T", 0)] }),
        ] {
            assert_eq!(request.put(&target).json(&bad).await.status_code(), 422, "{bad}");
        }

        // A flatter target: one manager seat, twelve engineers, a new product lead; the head moves to a department.
        let put = request
            .put(&target)
            .json(&json!({ "nodes": [
                node("head", None, "leadership", "Head", 1),
                node("mgr", Some("head"), "eng", "Manager", 1),
                node("eng", Some("mgr"), "eng", "Engineer", 12),
                node("prod", Some("head"), "product", "Product Lead", 1),
            ]}))
            .await;
        assert_eq!(put.status_code(), 200, "{}", put.text());
        let got: Value = request.get(&target).await.json();
        assert_eq!(got["nodes"].as_array().unwrap().len(), 4);

        let d: Value = request.get(&diff).await.json();
        assert_eq!(d["target_set"], true);
        assert_eq!(d["diff"]["current"]["layers"], 3);
        assert_eq!(d["diff"]["target"]["layers"], 3);
        assert_eq!(d["diff"]["current"]["headcount"], 13);
        assert_eq!(d["diff"]["target"]["headcount"], 15);
        // Now: the head has 2 reports, each manager 5 (12 reports over 3 managers = 4.00). Target: the head 2, the manager seat 12 (14 over 2).
        assert_eq!(d["diff"]["current"]["span"], json!({ "managers": 3, "mean_hundredths": 400, "max": 5 }));
        assert_eq!(d["diff"]["target"]["span"], json!({ "managers": 2, "mean_hundredths": 700, "max": 12 }));
        // Ten engineers is at the floor and is shown; one head and two managers are under it and are withheld, with their change.
        assert_eq!(role(&d, "Engineer")["current"], json!({ "status": "shown", "value": 10 }));
        assert_eq!(role(&d, "Engineer")["change"], "larger");
        assert_eq!(role(&d, "Engineer")["delta"], 2);
        assert_eq!(role(&d, "Manager")["current"], json!({ "status": "withheld" }));
        assert_eq!(role(&d, "Manager")["delta"], Value::Null);
        assert_eq!(role(&d, "Manager")["change"], "smaller", "the direction is not withheld");
        assert_eq!(role(&d, "Head")["moved"], true, "exec now, leadership in the plan");
        assert_eq!(role(&d, "Product Lead")["change"], "added");
        assert_eq!(role(&d, "Product Lead")["current"], json!({ "status": "shown", "value": 0 }));

        // No person, no manager link and no pid of a worker in the answer.
        let text = d.to_string();
        for private in [&head, &m1, &m2] {
            assert!(!text.contains(private.as_str()), "{private}");
        }
        assert!(!text.contains("Shape S-") && !text.contains("S-A0"), "{text}");

        // Replacing the target replaces it, and an archived plan is not edited.
        request.put(&target).json(&json!({ "nodes": [node("only", None, "eng", "Engineer", 3)] })).await.assert_status_ok();
        let again: Value = request.get(&target).await.json();
        assert_eq!(again["nodes"].as_array().unwrap().len(), 1);
        request.post(&format!("/api/workforce-plans/{pid}/status")).json(&json!({ "to": "archived" })).await.assert_status_ok();
        assert_eq!(request.put(&target).json(&json!({ "nodes": [node("x", None, "eng", "T", 1)] })).await.status_code(), 422);
    })
    .await;
}
