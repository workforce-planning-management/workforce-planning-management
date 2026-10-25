//! Positions (WPM-R152, WPM-D104): seats with a budget, moves that need their facts, links that copy nothing, and an establishment
//! that never nets an over-establishment against a vacancy.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn worker(request: &loco_rs::TestServer, org: &str, number: &str, fte: i32) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": number,
            "display_name": format!("Seat {number}"), "employment_type": "permanent",
            "department": "engineering", "job_title": "Engineer", "hired_on": "2020-01-05", "fte_percent": fte,
        }))
        .await
        .json();
    created["pid"].as_str().unwrap().to_string()
}

async fn seat(request: &loco_rs::TestServer, org: &str, cost_centre: &str, budget: i32) -> String {
    let made = request
        .post("/api/positions")
        .json(&json!({ "organization_ref": org, "cost_centre": cost_centre, "title": "Engineer", "budget_fte_percent": budget, "funding_kind": "core" }))
        .await;
    assert_eq!(made.status_code(), 200, "{}", made.text());
    made.json::<Value>()["pid"].as_str().unwrap().to_string()
}

async fn go(request: &loco_rs::TestServer, pid: &str, to: &str) -> (u16, String) {
    let r = request
        .post(&format!("/api/positions/{pid}/status"))
        .json(&json!({ "to": to }))
        .await;
    (r.status_code().as_u16(), r.text())
}

fn centre<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["cost_centres"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["cost_centre"] == name)
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_seat_moves_with_its_facts_and_the_establishment_never_nets() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let other = an_org();

        // Faults are refused.
        let bad = |extra: Value| {
            let mut b = json!({ "organization_ref": org, "cost_centre": "CC-1", "title": "Engineer", "budget_fte_percent": 100, "funding_kind": "core" });
            b.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            b
        };
        for fault in [
            json!({ "budget_fte_percent": 0 }),
            json!({ "budget_fte_percent": 150 }),
            json!({ "funding_kind": "time_limited" }),
            json!({ "funding_kind": "programme" }),
            json!({ "title": "" }),
            json!({ "demand_line_pid": "11111111-2222-4333-8444-555555555555" }),
        ] {
            assert_eq!(request.post("/api/positions").json(&bad(fault.clone())).await.status_code(), 422, "{fault}");
        }

        // A seat starts planned and cannot be filled before its facts are there.
        let a = seat(&request, &org, "CC-1", 100).await;
        let (code, text) = go(&request, &a, "filled").await;
        assert_eq!(code, 422, "{text}");
        assert!(text.contains("planned"), "names the state it is in: {text}");
        assert_eq!(go(&request, &a, "approved").await.0, 200);
        let (code, text) = go(&request, &a, "open").await;
        assert_eq!(code, 422);
        assert!(text.contains("requisition"), "{text}");

        // A requisition of this organization opens it; one of another does not link.
        let req: Value = request.post("/api/requisitions").json(&json!({ "organization_ref": org, "department": "engineering", "job_title": "Engineer", "headcount": 1, "salary_min_minor": 5_000_000, "salary_max_minor": 6_000_000, "salary_currency": "GBP" })).await.json();
        let foreign: Value = request.post("/api/requisitions").json(&json!({ "organization_ref": other, "department": "engineering", "job_title": "Engineer", "headcount": 1, "salary_min_minor": 5_000_000, "salary_max_minor": 6_000_000, "salary_currency": "GBP" })).await.json();
        let link = |pid: &str, r: Option<&str>| {
            let request = &request;
            let url = format!("/api/positions/{pid}/requisition");
            let body = json!({ "requisition_pid": r });
            async move { request.put(&url).json(&body).await.status_code().as_u16() }
        };
        assert_eq!(link(&a, foreign["pid"].as_str()).await, 422, "another organization's requisition");
        assert_eq!(link(&a, Some("11111111-2222-4333-8444-555555555555")).await, 422);
        assert_eq!(link(&a, req["pid"].as_str()).await, 200);
        assert_eq!(go(&request, &a, "open").await.0, 200);
        assert_eq!(link(&a, None).await, 422, "an open position keeps its requisition");

        // A holder of this organization fills it; one of another does not link.
        let w = worker(&request, &org, "P-1", 60).await;
        let stranger = worker(&request, &other, "P-X", 100).await;
        let hold = |pid: &str, w: Option<&str>| {
            let request = &request;
            let url = format!("/api/positions/{pid}/holder");
            let body = json!({ "worker_pid": w });
            async move { request.put(&url).json(&body).await.status_code().as_u16() }
        };
        assert_eq!(hold(&a, Some(&stranger)).await, 422);
        assert_eq!(go(&request, &a, "filled").await.0, 422, "no holder yet");
        assert_eq!(hold(&a, Some(&w)).await, 200);
        assert_eq!(go(&request, &a, "filled").await.0, 200);

        // A held seat is not abolished or made vacant until the holder is unlinked.
        assert_eq!(go(&request, &a, "abolished").await.0, 422);
        assert_eq!(go(&request, &a, "open").await.0, 422);

        // Establishment: 100 budget held at 60 leaves 40 vacant. A second seat held beyond budget is over, and is not netted.
        let b = seat(&request, &org, "CC-1", 50).await;
        go(&request, &b, "approved").await;
        let w2 = worker(&request, &org, "P-2", 100).await;
        assert_eq!(hold(&b, Some(&w2)).await, 200);
        let planned = seat(&request, &org, "CC-1", 100).await; // still planned: not in the establishment
        let _ = planned;
        let body: Value = request.get(&format!("/api/positions/establishment?organization={org}")).await.json();
        let c = centre(&body, "CC-1");
        assert_eq!(c["positions"], 2);
        assert_eq!(c["budgeted"], json!({ "status": "known", "percent": 150 }));
        assert_eq!(c["held"], json!({ "status": "known", "percent": 160 }));
        assert_eq!(c["vacant"], json!({ "status": "known", "percent": 40 }));
        assert_eq!(c["over"], json!({ "status": "known", "percent": 50 }), "not netted against the 40");
        assert_eq!(c["by_state"]["planned"], 1);
        assert!(!body.to_string().contains(&w) && !body.to_string().contains("Seat P-"), "no holder in the establishment");

        // The list says the seat is held, and (signed in off) who holds it.
        let list: Value = request.get(&format!("/api/positions?organization={org}&state=filled")).await.json();
        assert_eq!(list["positions"].as_array().unwrap().len(), 1);
        assert_eq!(list["positions"][0]["holder_worker_pid"], w);
        assert_eq!(request.get(&format!("/api/positions?organization={org}&state=nonsense")).await.status_code(), 422);

        // Unlink the holder, then the seat can be opened again and abolished; an ended seat is not changed.
        assert_eq!(hold(&a, None).await, 200);
        assert_eq!(go(&request, &a, "open").await.0, 200);
        assert_eq!(go(&request, &a, "abolished").await.0, 200);
        assert_eq!(go(&request, &a, "approved").await.0, 422, "abolished is final");
        let upd = request.put(&format!("/api/positions/{a}")).json(&bad(json!({}))).await;
        assert_eq!(upd.status_code(), 422);
        assert_eq!(hold(&a, Some(&w)).await, 422, "an ended seat has no holder");

        // An unknown holder (the worker's record is gone) makes the figures unknown, not zero.
        ctx.db.execute_unprepared(&format!("UPDATE workers SET deleted_at = now() WHERE pid = '{w2}'")).await.unwrap();
        let body: Value = request.get(&format!("/api/positions/establishment?organization={org}")).await.json();
        assert_eq!(centre(&body, "CC-1")["held"], json!({ "status": "unknown" }));
        assert_eq!(centre(&body, "CC-1")["budgeted"], json!({ "status": "known", "percent": 50 }));

        // Every change is audited with a state and no worker.
        let rows = ctx.db.query_all_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT snapshot::text AS s FROM audit_logs WHERE action LIKE 'position_%'".to_string())).await.unwrap();
        assert!(rows.len() >= 10);
        for row in rows {
            let s: Option<String> = row.try_get("", "s").unwrap();
            assert!(!s.unwrap_or_default().contains(&w), "no worker in an audit snapshot");
        }

        // No audit, no change (WPM-D107).
        crate::requests::ddl(&ctx, "CREATE FUNCTION refuse_audit() RETURNS trigger AS $$ BEGIN RAISE EXCEPTION 'audit refused'; END $$ LANGUAGE plpgsql;
             CREATE TRIGGER refuse_audit BEFORE INSERT ON audit_logs FOR EACH ROW EXECUTE FUNCTION refuse_audit()").await;
        let refused = request.post("/api/positions").json(&bad(json!({ "cost_centre": "CC-NEVER" }))).await;
        assert!(refused.status_code().is_server_error());
        crate::requests::ddl(&ctx, "DROP TRIGGER refuse_audit ON audit_logs; DROP FUNCTION refuse_audit()").await;
        let after: Value = request.get(&format!("/api/positions?organization={org}")).await.json();
        assert!(!after.to_string().contains("CC-NEVER"));
    })
    .await;
}
