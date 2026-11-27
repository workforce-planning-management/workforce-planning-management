//! Positions (WPM-R152, WPM-D104): seats with a budget, moves that need their facts, links that copy nothing, and an establishment
//! that never nets an over-establishment against a vacancy.

use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::absent::AbsentIsEmpty as _;

use super::{a_person, an_org};
use sqlx::Row;

async fn worker(request: &axa_test::TestServer, org: &str, number: &str, fte: i32) -> String {
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

async fn seat(request: &axa_test::TestServer, org: &str, cost_centre: &str, budget: i32) -> String {
    let made = request
        .post("/api/positions")
        .json(&json!({ "organization_ref": org, "cost_centre": cost_centre, "title": "Engineer", "budget_fte_percent": budget, "funding_kind": "core" }))
        .await;
    assert_eq!(made.status_code(), 200, "{}", made.text());
    made.json::<Value>()["pid"].as_str().unwrap().to_string()
}

async fn go(request: &axa_test::TestServer, pid: &str, to: &str) -> (u16, String) {
    let r = request
        .post(&format!("/api/positions/{pid}/status"))
        .json(&json!({ "to": to }))
        .await;
    (r.status_code().as_u16(), r.text())
}

/// One group of a vacancy-age report (`cost_centres` or `bands`).
fn group<'a>(body: &'a Value, list: &str, name: &str) -> &'a Value {
    body[list]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["group"] == name)
        .unwrap()
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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
            json!({ "number": "x".repeat(41) }),
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
        let req = request.put(&format!("/api/positions/{a}/requisition")).json(&json!({ "requisition_pid": null })).await;
        assert_eq!(req.status_code(), 422, "an ended seat takes no requisition: {}", req.text());
        // A seat stays in its organization.
        let live = seat(&request, &org, "CC-9", 100).await;
        let mut moved = bad(json!({}));
        moved["organization_ref"] = json!(other);
        let refused = request.put(&format!("/api/positions/{live}")).json(&moved).await;
        assert_eq!(refused.status_code(), 422, "{}", refused.text());
        assert!(refused.text().contains("another organization"), "{}", refused.text());

        // An unknown holder (the worker's record is gone) makes the figures unknown, not zero.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("UPDATE workers SET deleted_at = now() WHERE pid = '{w2}'"))).execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();
        let body: Value = request.get(&format!("/api/positions/establishment?organization={org}")).await.json();
        assert_eq!(centre(&body, "CC-1")["held"], json!({ "status": "unknown" }));
        assert_eq!(centre(&body, "CC-1")["budgeted"], json!({ "status": "known", "percent": 50 }));

        // Every change is audited with a state and no worker.
        let rows = sqlx::query("SELECT snapshot::text AS s FROM audit_logs WHERE action LIKE 'position_%'").fetch_all(workforce_planning_management::db::pool(&ctx)).await.unwrap();
        assert!(rows.len() >= 10);
        for row in rows {
            let s: Option<String> = row.try_get("s").unwrap();
            assert!(!s.or_empty().contains(&w), "no worker in an audit snapshot");
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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn overtime_status_starts_unknown_is_recorded_by_a_person_and_a_stray_value_is_refused() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let pid = seat(&request, &org, "CC-OT", 100).await;
        let read = || async {
            let list: Value = request
                .get(&format!("/api/positions?organization={org}"))
                .await
                .json();
            list["positions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["pid"] == pid.as_str())
                .cloned()
                .unwrap()
        };
        assert_eq!(read().await["overtime_status"], "unknown", "never inferred");

        let body = |extra: Value| {
            let mut b = json!({ "cost_centre": "CC-OT", "title": "Engineer", "budget_fte_percent": 100, "funding_kind": "core" });
            b.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            b
        };
        let stray = request
            .put(&format!("/api/positions/{pid}"))
            .json(&body(json!({ "overtime_status": "salaried" })))
            .await;
        assert_eq!(stray.status_code(), 422, "{}", stray.text());
        assert_eq!(read().await["overtime_status"], "unknown");

        let set = request
            .put(&format!("/api/positions/{pid}"))
            .json(&body(json!({ "overtime_status": "exempt" })))
            .await;
        assert_eq!(set.status_code(), 200, "{}", set.text());
        assert_eq!(read().await["overtime_status"], "exempt");

        // An update that says nothing about it keeps what a person recorded.
        let keep = request.put(&format!("/api/positions/{pid}")).json(&body(json!({ "title": "Senior engineer" }))).await;
        assert_eq!(keep.status_code(), 200, "{}", keep.text());
        assert_eq!(read().await["overtime_status"], "exempt");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_number_names_one_position_and_an_update_without_it_keeps_it() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let body = |cost_centre: &str, number: Option<&str>| {
            let mut b = json!({ "organization_ref": org, "cost_centre": cost_centre, "title": "Engineer",
                                "budget_fte_percent": 100, "funding_kind": "core" });
            if let Some(n) = number {
                b["number"] = json!(n);
            }
            b
        };
        let created = request.post("/api/positions").json(&body("CC-A", Some(" P-1 "))).await;
        assert_eq!(created.status_code(), 200, "{}", created.text());
        let pid = created.json::<Value>()["pid"].as_str().unwrap().to_string();
        let again = request.post("/api/positions").json(&body("CC-B", Some("P-1"))).await;
        assert_eq!(again.status_code(), 422, "a second use in the organization is refused");
        let elsewhere = request
            .post("/api/positions")
            .json(&json!({ "organization_ref": an_org(), "cost_centre": "CC-A", "title": "Engineer",
                           "budget_fte_percent": 100, "funding_kind": "core", "number": "P-1" }))
            .await;
        assert_eq!(elsewhere.status_code(), 200, "another organization may use it");
        let other = request.post("/api/positions").json(&body("CC-B", None)).await;
        assert_eq!(other.status_code(), 200);

        // Its own number is no clash, and an update that leaves it out keeps it.
        let url = format!("/api/positions/{pid}");
        request.put(&url).json(&body("CC-A", Some("P-1"))).await.assert_status_ok();
        request.put(&url).json(&body("CC-A", None)).await.assert_status_ok();
        let listed: Value = request
            .get(&format!("/api/positions?organization={org}&cost_centre=CC-A"))
            .await
            .json();
        let positions = listed["positions"].as_array().unwrap();
        assert_eq!(positions.len(), 1, "the cost centre filters: {listed}");
        assert_eq!(positions[0]["pid"], pid.as_str());
        assert_eq!(positions[0]["number"], "P-1", "trimmed, and kept");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn every_move_is_a_dated_row_a_freeze_has_its_three_facts_and_vacancy_age_counts_open_posts()
{
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let cc = "CC-HIST";
        let a = seat(&request, &org, cc, 100).await;
        let b = seat(&request, &org, cc, 100).await;
        let gone = seat(&request, &org, cc, 100).await;
        let future = (workforce_planning_management::dates::today() + jiff::Span::new().days(30)).to_string();
        let past = (workforce_planning_management::dates::today() - jiff::Span::new().days(1)).to_string();

        // A new seat is planned and has one move, from nothing, dated today and known.
        let row = sqlx::query("SELECT from_state, to_state, start_known FROM position_state_moves WHERE position_pid = $1::uuid")
            .bind(&a)
            .fetch_one(&ctx.db)
            .await
            .unwrap();
        assert_eq!(row.get::<Option<String>, _>("from_state"), None);
        assert_eq!(row.get::<String, _>("to_state"), "planned");
        assert!(row.get::<bool, _>("start_known"));

        for p in [&a, &b] {
            assert_eq!(go(&request, p, "approved").await.0, 200);
        }
        assert_eq!(go(&request, &gone, "abolished").await.0, 200);

        // A freeze needs a reason, a deciding role and a future review date; the other moves take none of them.
        let freeze = |body: Value| {
            let request = &request;
            let b = b.clone();
            async move {
                let r = request.post(&format!("/api/positions/{b}/status")).json(&body).await;
                (r.status_code().as_u16(), r.text())
            }
        };
        for fault in [
            json!({ "to": "frozen" }),
            json!({ "to": "frozen", "reason": "funding", "decided_by_role": "Finance director" }),
            json!({ "to": "frozen", "reason": "whim", "decided_by_role": "Finance director", "review_on": future }),
            json!({ "to": "frozen", "reason": "funding", "decided_by_role": " ", "review_on": future }),
            json!({ "to": "frozen", "reason": "funding", "decided_by_role": "Finance director", "review_on": past }),
            json!({ "to": "abolished", "reason": "funding" }),
        ] {
            let (code, text) = freeze(fault.clone()).await;
            assert_eq!(code, 422, "{fault}: {text}");
        }
        let (code, text) = freeze(json!({ "to": "frozen", "reason": "funding", "decided_by_role": "Finance director", "review_on": future })).await;
        assert_eq!(code, 200, "{text}");

        // The history is append-only: the database refuses a change.
        let refused = sqlx::query("UPDATE position_state_moves SET to_state = 'filled' WHERE position_pid = $1::uuid")
            .bind(&a)
            .execute(&ctx.db)
            .await;
        assert!(refused.is_err());
        let refused = sqlx::query("DELETE FROM position_state_moves WHERE position_pid = $1::uuid")
            .bind(&a)
            .execute(&ctx.db)
            .await;
        assert!(refused.is_err());

        // A post that was approved before the history began has a start that is unknown, and the report says so.
        sqlx::query("INSERT INTO positions (pid, organization_ref, cost_centre, title, budget_fte_percent, funding_kind, state) VALUES (gen_random_uuid(), $1, $2, 'Old', 100, 'core', 'approved')")
            .bind(&org)
            .bind(cc)
            .execute(&ctx.db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO position_state_moves (position_pid, organization_ref, from_state, to_state, moved_on, start_known, recorded_by) SELECT pid, organization_ref, NULL, state, CURRENT_DATE, FALSE, 'backfill' FROM positions WHERE organization_ref = $1 AND title = 'Old'")
            .bind(&org)
            .execute(&ctx.db)
            .await
            .unwrap();

        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        let unit = group(&report, "cost_centres", cc);
        // a, b (frozen) and the old one are vacancies; the abolished post is beside them, not in them.
        assert_eq!(unit["vacancies"], 3, "{report}");
        assert_eq!(unit["by_age"]["0-30"], 2);
        assert_eq!(unit["start_unknown"], 1);
        assert_eq!(unit["frozen"], 1);
        assert_eq!(unit["frozen_without_review_date"], 0);
        assert_eq!(unit["review_overdue"], 0);
        assert_eq!(unit["abolished"], 1);
        assert_eq!(unit["withheld"], false);
        assert!(!report.to_string().contains("holder_worker_pid"));

        // A post somebody holds is not a vacancy, whatever its state; when the holder leaves, the vacancy starts that day.
        let w = worker(&request, &org, "H-1", 100).await;
        assert_eq!(go(&request, &a, "open").await.0, 422, "no requisition");
        let held = request.put(&format!("/api/positions/{a}/holder")).json(&json!({ "worker_pid": w })).await;
        assert_eq!(held.status_code(), 200);
        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        assert_eq!(group(&report, "cost_centres", cc)["vacancies"], 2);
        let unlinked = request.put(&format!("/api/positions/{a}/holder")).json(&json!({ "worker_pid": null })).await;
        assert_eq!(unlinked.status_code(), 200);
        let row = sqlx::query("SELECT from_state, to_state, holder_left FROM position_state_moves WHERE position_pid = $1::uuid ORDER BY id DESC LIMIT 1")
            .bind(&a)
            .fetch_one(&ctx.db)
            .await
            .unwrap();
        assert_eq!(row.get::<Option<String>, _>("from_state").as_deref(), Some("approved"));
        assert_eq!(row.get::<String, _>("to_state"), "approved");
        assert!(row.get::<bool, _>("holder_left"), "the leaving is a row of its own, so the age starts there");
        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        assert_eq!(group(&report, "cost_centres", cc)["vacancies"], 3);
        // Unlinking where nobody holds the post adds no row.
        let before: i64 = sqlx::query_scalar("SELECT count(*) FROM position_state_moves WHERE position_pid = $1::uuid")
            .bind(&a)
            .fetch_one(&ctx.db)
            .await
            .unwrap();
        assert_eq!(request.put(&format!("/api/positions/{a}/holder")).json(&json!({ "worker_pid": null })).await.status_code(), 200);
        let after: i64 = sqlx::query_scalar("SELECT count(*) FROM position_state_moves WHERE position_pid = $1::uuid")
            .bind(&a)
            .fetch_one(&ctx.db)
            .await
            .unwrap();
        assert_eq!(before, after);

        // Another organization sees none of it.
        let other: Value = request.get(&format!("/api/positions/vacancy-age?organization={}", an_org())).await.json();
        assert_eq!(other["cost_centres"].as_array().unwrap().len(), 0);
        assert_eq!(request.get("/api/positions/vacancy-age?organization=").await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn every_holder_is_a_tenure_that_survives_their_erasure_without_naming_them() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let post = seat(&request, &org, "CC-TEN", 100).await;
        assert_eq!(go(&request, &post, "approved").await.0, 200);
        let first = worker(&request, &org, "T-1", 100).await;
        let second = worker(&request, &org, "T-2", 100).await;
        let link = |who: Value| {
            let request = &request;
            let url = format!("/api/positions/{post}/holder");
            async move { request.put(&url).json(&json!({ "worker_pid": who })).await.status_code().as_u16() }
        };
        let tenures = || async {
            sqlx::query("SELECT worker_pid::text AS w, ended_on IS NULL AS open FROM position_holders WHERE position_pid = $1::uuid ORDER BY id")
                .bind(&post)
                .fetch_all(&ctx.db)
                .await
                .unwrap()
                .iter()
                .map(|r| (r.get::<Option<String>, _>("w"), r.get::<bool, _>("open")))
                .collect::<Vec<_>>()
        };

        // Linking opens a tenure; the same holder again changes nothing; a replacement closes it and opens another; unlinking closes it.
        assert_eq!(link(json!(first)).await, 200);
        assert_eq!(link(json!(first)).await, 200);
        assert_eq!(tenures().await, vec![(Some(first.clone()), true)]);
        assert_eq!(link(json!(second)).await, 200);
        assert_eq!(link(Value::Null).await, 200);
        assert_eq!(tenures().await, vec![(Some(first.clone()), false), (Some(second.clone()), false)]);
        assert_eq!(link(json!(first)).await, 200);
        assert_eq!(tenures().await.len(), 3, "one person holding twice is two tenures");

        // The database refuses any other change: a delete, a reopened tenure, a moved start, a worker swapped for another.
        let pool = &ctx.db;
        for refused in [
            format!("DELETE FROM position_holders WHERE position_pid = '{post}'"),
            format!("UPDATE position_holders SET ended_on = NULL WHERE position_pid = '{post}' AND ended_on IS NOT NULL"),
            format!("UPDATE position_holders SET started_on = started_on - 1 WHERE position_pid = '{post}'"),
            format!("UPDATE position_holders SET worker_pid = '{second}' WHERE position_pid = '{post}' AND worker_pid = '{first}'"),
        ] {
            assert!(sqlx::query(sqlx::AssertSqlSafe(refused.clone())).execute(pool).await.is_err(), "{refused}");
        }

        // One post's history, for HR: three tenures, the open one last, and continuity counted from the post's creation today.
        let body: Value = request.get(&format!("/api/positions/{post}/holders")).await.json();
        let holders = body["holders"].as_array().unwrap();
        assert_eq!(holders.len(), 3, "{body}");
        assert_eq!(holders[2]["worker_pid"], first.as_str());
        assert_eq!(holders[2]["ended_on"], Value::Null);
        assert_eq!(body["continuity"]["tenures"], 3);
        assert_eq!(body["continuity"]["complete"], false, "the post is new, so 36 months are not counted");
        assert_eq!(body["continuity"]["counted_from"], workforce_planning_management::dates::today().to_string());
        assert_eq!(request.get("/api/positions/11111111-2222-4333-8444-555555555555/holders").await.status_code(), 404);

        // The person's own export carries their tenures.
        let export: Value = request.get(&format!("/api/workers/{first}/subject-access")).await.json();
        assert_eq!(export["position_holder_history"].as_array().unwrap().len(), 2, "{export}");

        // Erasure cuts the person and keeps the dates: the count of tenures stays and the open one ends.
        assert_eq!(link(Value::Null).await, 200);
        for to in ["active", "offboarding", "terminated"] {
            request.post(&format!("/api/workers/{first}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        assert_eq!(request.post(&format!("/api/workers/{first}/erase")).await.status_code(), 200);
        let after = tenures().await;
        assert_eq!(after.len(), 3);
        assert_eq!(after.iter().filter(|(w, _)| w.is_none()).count(), 2, "{after:?}");
        assert!(after.iter().all(|(_, open)| !open));
        let body: Value = request.get(&format!("/api/positions/{post}/holders")).await.json();
        assert!(!body.to_string().contains(&first), "an erased holder is named nowhere");
        assert_eq!(body["holders"][0]["erased"], true);
        assert_eq!(body["continuity"]["tenures"], 3);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_post_is_marked_critical_and_graded_and_the_vacancy_age_is_cut_by_band_and_for_critical_posts()
 {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let made = request
            .post("/api/positions")
            .json(&json!({ "organization_ref": org, "cost_centre": "CC-G", "title": "Lead", "budget_fte_percent": 100, "funding_kind": "core", "critical": true }))
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let lead = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let plain = seat(&request, &org, "CC-G", 100).await;
        for p in [&lead, &plain] {
            assert_eq!(go(&request, p, "approved").await.0, 200);
        }

        // A grade names a published band and a pay point of it; anything else is refused, and none recorded is unknown.
        let grade = |pid: &str, body: Value| {
            let request = &request;
            let url = format!("/api/positions/{pid}/grade");
            async move { request.put(&url).json(&body).await.status_code().as_u16() }
        };
        assert_eq!(grade(&lead, json!({ "scale": "no-such-scale", "band": "5" })).await, 422);
        assert_eq!(grade(&lead, json!({ "scale": "national-2026-27", "band": "99" })).await, 422);
        assert_eq!(grade(&lead, json!({ "scale": "national-2026-27", "band": "5", "pay_point": 99 })).await, 422);
        assert_eq!(grade(&lead, json!({ "scale": "national-2026-27", "band": "5", "pay_point": 1 })).await, 200);
        let listed: Value = request.get(&format!("/api/positions?organization={org}")).await.json();
        let row = listed["positions"].as_array().unwrap().iter().find(|p| p["pid"] == lead.as_str()).unwrap();
        assert_eq!(row["critical"], true);
        assert_eq!((row["grade_scale"].as_str(), row["grade_band"].as_str(), row["grade_pay_point"].as_i64()), (Some("national-2026-27"), Some("5"), Some(1)));

        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        assert_eq!(group(&report, "bands", "national-2026-27:5")["vacancies"], 1, "{report}");
        assert_eq!(group(&report, "bands", "unknown")["vacancies"], 1, "an ungraded post is unknown, not a band");
        assert_eq!(report["critical"]["group"], "critical");
        assert_eq!(report["critical"]["vacancies"], 1);

        // An update without critical keeps it; with false clears it, and then no critical group is reported.
        let update = |critical: Value| {
            let request = &request;
            let url = format!("/api/positions/{lead}");
            async move {
                let mut body = json!({ "cost_centre": "CC-G", "title": "Lead", "budget_fte_percent": 100, "funding_kind": "core" });
                if !critical.is_null() {
                    body["critical"] = critical;
                }
                request.put(&url).json(&body).await.status_code().as_u16()
            }
        };
        assert_eq!(update(Value::Null).await, 200);
        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        assert_eq!(report["critical"]["vacancies"], 1, "left out, critical is kept");
        assert_eq!(update(json!(false)).await, 200);
        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        assert_eq!(report["critical"], Value::Null);

        // Clearing the grade makes it unknown again; clearing none is a 404.
        let url = format!("/api/positions/{lead}/grade");
        assert_eq!(request.delete(&url).await.status_code(), 200);
        assert_eq!(request.delete(&url).await.status_code(), 404);
        let report: Value = request.get(&format!("/api/positions/vacancy-age?organization={org}")).await.json();
        assert_eq!(group(&report, "bands", "unknown")["vacancies"], 2);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_establishment_is_given_by_band_and_pay_point_and_critical_posts_have_a_continuity_report()
 {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let made = |critical: bool| {
            let request = &request;
            let org = org.clone();
            async move {
                let r = request
                    .post("/api/positions")
                    .json(&json!({ "organization_ref": org, "cost_centre": "CC-E", "title": "Post", "budget_fte_percent": 100, "funding_kind": "core", "critical": critical }))
                    .await;
                r.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let a = made(true).await;
        let b = made(true).await;
        let c = made(false).await;
        for p in [&a, &b, &c] {
            assert_eq!(go(&request, p, "approved").await.0, 200);
        }
        for (p, grade) in [(&a, json!({ "scale": "national-2026-27", "band": "5", "pay_point": 1 })), (&b, json!({ "scale": "national-2026-27", "band": "5" }))] {
            assert_eq!(request.put(&format!("/api/positions/{p}/grade")).json(&grade).await.status_code(), 200);
        }

        // By band and by pay point: a graded post with no point is band:unknown, an ungraded one unknown.
        let est: Value = request.get(&format!("/api/positions/establishment?organization={org}")).await.json();
        assert_eq!(group(&est, "bands", "national-2026-27:5")["positions"], 2, "{est}");
        assert_eq!(group(&est, "bands", "unknown")["positions"], 1);
        assert_eq!(group(&est, "pay_points", "national-2026-27:5:1")["positions"], 1);
        assert_eq!(group(&est, "pay_points", "national-2026-27:5:unknown")["positions"], 1);
        assert_eq!(group(&est, "pay_points", "unknown")["positions"], 1);
        assert_eq!(group(&est, "bands", "national-2026-27:5")["vacant"]["percent"], 200, "two whole vacant seats");

        // The critical posts: two, under the floor of ten, so withheld for every caller, and the total with them.
        let w = worker(&request, &org, "CR-1", 100).await;
        assert_eq!(request.put(&format!("/api/positions/{a}/holder")).json(&json!({ "worker_pid": w })).await.status_code(), 200);
        let report: Value = request.get(&format!("/api/positions/critical-continuity?organization={org}")).await.json();
        let centres = report["cost_centres"]["groups"].as_array().unwrap();
        assert_eq!(centres.len(), 1, "{report}");
        assert_eq!(centres[0]["withheld"], true);
        assert_eq!(centres[0]["tenures"], Value::Null);
        assert_eq!(report["cost_centres"]["total"]["withheld"], true);
        assert_eq!(report["bands"]["groups"][0]["group"], "national-2026-27:5");
        assert!(!report.to_string().contains(&w), "no holder is named");
        assert_eq!(report["small_group_floor"], 10);
        assert_eq!(request.get("/api/positions/critical-continuity?organization=").await.status_code(), 422);
    })
    .await;
}
