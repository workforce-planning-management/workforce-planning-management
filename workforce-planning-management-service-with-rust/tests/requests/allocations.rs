//! Per-person allocation (WPM-R275): allocate, validate, confirm and tell the person, read the month standings with their signals,
//! unknown until the organization tracks allocation, the summary against an active claim, removal, and the subject export and
//! erasure. Sign-in is off here, so the caller is HR; who else may read and write is `tests/enforcement_allocations.rs`.
use serde_json::{Value, json};
use serial_test::serial;
use sqlx::Row;
use workforce_planning_management::absent::AbsentIsEmpty as _;

use super::{activate, an_org, seed_worker};

async fn scalar(ctx: &axa::AppContext, sql: String) -> i64 {
    sqlx::query(sqlx::AssertSqlSafe(&*sql))
        .fetch_optional(workforce_planning_management::db::pool(ctx))
        .await
        .unwrap()
        .unwrap()
        .try_get("n")
        .unwrap()
}

async fn exec(ctx: &axa::AppContext, sql: String) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(workforce_planning_management::db::pool(ctx))
        .await
        .unwrap();
}

fn month<'a>(standing: &'a Value, m: &str) -> &'a Value {
    standing["months"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["month"] == m)
        .unwrap_or_else(|| panic!("no month {m} in {standing}"))
}

fn kinds(month: &Value) -> Vec<String> {
    month["signals"]
        .as_array()
        .map(|s| {
            s.iter()
                .map(|x| x["kind"].as_str().unwrap().to_string())
                .collect()
        })
        .or_empty()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn an_allocation_is_planned_confirmed_read_summarized_and_erased() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("AL-{tag}"), None).await;
        activate!(&request, &w).await;
        let assignments: Value = request.get(&format!("/api/workers/{w}/assignments")).await.json();
        let assignment = assignments["assignments"][0]["pid"].as_str().unwrap().to_string();
        let pool: Value = request
            .post("/api/skill-pools")
            .json(&json!({ "organization_ref": org, "name": format!("Pool {tag}") }))
            .await
            .json();
        let pool = pool["pid"].as_str().unwrap().to_string();
        let url = format!("/api/workers/{w}/allocations");
        let good = json!({
            "assignment_pid": assignment, "source_ref": "thing:programme-a", "pool_pid": pool,
            "from_month": "2027-01-01", "to_month": "2027-03-01", "fte_centi": 60,
        });

        // Refusals: not this worker's assignment, a month that is not a first day, over the assignment's FTE, before the
        // assignment, a bad source, a pool of another organization, a bad status, a long note.
        let mut bad = Vec::new();
        for (key, value) in [
            ("assignment_pid", json!(uuid::Uuid::new_v4())),
            ("from_month", json!("2027-01-02")),
            ("fte_centi", json!(101)),
            ("fte_centi", json!(0)),
            ("from_month", json!("2025-12-01")),
            ("source_ref", json!("programme")),
            ("pool_pid", json!(uuid::Uuid::new_v4())),
            ("status", json!("ended")),
            ("note", json!("x".repeat(501))),
            ("to_month", json!("2026-12-01")),
        ] {
            let mut body = good.clone();
            body[key] = value;
            bad.push(body);
        }
        for body in bad {
            let r = request.post(&url).json(&body).await;
            assert_eq!(r.status_code(), 422, "{body}: {}", r.text());
        }
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM allocations WHERE worker_pid = '{w}'")).await, 0);

        // A tentative allocation and a confirmed one; only the confirmed one tells the person.
        let tentative: Value = request.post(&url).json(&good).await.json();
        let tentative = tentative["pid"].as_str().unwrap().to_string();
        let mut confirmed = good.clone();
        confirmed["source_ref"] = json!("thing:programme-b");
        confirmed["fte_centi"] = json!(30);
        confirmed["status"] = json!("confirmed");
        confirmed["from_month"] = json!("2027-02-01");
        confirmed["to_month"] = json!("2027-02-01");
        let made = request.post(&url).json(&confirmed).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let confirmed: Value = made.json();
        let confirmed = confirmed["pid"].as_str().unwrap().to_string();
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM notifications WHERE worker_pid = '{w}' AND kind = 'allocation_confirmed'")).await, 1);

        // Untracked: every month is unknown, never zero.
        let read: Value = request.get(&format!("{url}?from=2027-01-01&months=4")).await.json();
        assert_eq!(read["tracked"], false);
        assert_eq!(read["allocations"].as_array().map(Vec::len), Some(2), "{read}");
        assert_eq!(month(&read, "2027-02-01")["standing"], "unknown");
        assert!(read["derivation"].as_str().is_some());
        assert!(read["allocations"][0].get("recorded_by").is_none(), "who wrote it is in the audit trail, not the read");

        // Tracking on: the standings are worked out.
        let set = request
            .put("/api/capacity/settings")
            .json(&json!({ "organization_ref": org, "allocation_tracked": true }))
            .await;
        assert_eq!(set.status_code(), 200, "{}", set.text());
        assert_eq!(set.json::<Value>()["allocation_tracked"], true);
        let kept = request.put("/api/capacity/settings").json(&json!({ "organization_ref": org, "wip_limit": 2 })).await.json::<Value>();
        assert_eq!(kept["allocation_tracked"], true, "absent keeps what is set");
        let read: Value = request.get(&format!("{url}?from=2027-01-01&months=4")).await.json();
        assert_eq!(read["tracked"], true);
        let feb = month(&read, "2027-02-01");
        assert_eq!((feb["contracted_centi"].as_i64(), feb["allocated_centi"].as_i64(), feb["tentative_centi"].as_i64()), (Some(100), Some(30), Some(60)));
        assert_eq!(kinds(feb), vec!["available"], "30 of 100 confirmed: {feb}");
        assert_eq!(kinds(month(&read, "2027-04-01")), vec!["available"], "nothing allocated, tracked: a known zero");

        // Confirm the tentative one: 90 of 100. Then approved leave in February makes it a clash, never naming the kind.
        let moved = request.post(&format!("/api/allocations/{tentative}/status")).json(&json!({ "to": "confirmed" })).await;
        assert_eq!(moved.status_code(), 200, "{}", moved.text());
        assert_eq!(request.post(&format!("/api/allocations/{tentative}/status")).json(&json!({ "to": "confirmed" })).await.status_code(), 422, "not a move");
        exec(&ctx, format!(
            "INSERT INTO leave_requests (pid, worker_pid, kind, start_on, end_on, days, status) VALUES
             ('{}', '{w}', 'sick', '2027-02-01', '2027-02-14', 10, 'approved')", uuid::Uuid::new_v4())).await;
        let read: Value = request.get(&format!("{url}?from=2027-01-01&months=4")).await.json();
        let feb = month(&read, "2027-02-01");
        assert_eq!(kinds(feb), vec!["leave_clash"], "{feb}");
        assert!(!read.to_string().contains("sick"), "the kind of leave is never shown");
        assert_eq!(kinds(month(&read, "2027-01-01")), vec!["available"], "60 of 100 in January is below 80%");

        // Over the contract: another 50 in January.
        let mut more = good.clone();
        more["source_ref"] = json!("thing:programme-c");
        more["status"] = json!("confirmed");
        more["fte_centi"] = json!(50);
        more["to_month"] = json!("2027-01-01");
        request.post(&url).json(&more).await.assert_status_ok();
        let read: Value = request.get(&format!("{url}?from=2027-01-01&months=1")).await.json();
        let jan = month(&read, "2027-01-01");
        assert_eq!(kinds(jan), vec!["over_allocated", "assignment_over_allocated"], "{jan}");
        assert_eq!(jan["signals"][0]["by_centi"], 10);

        // A change to a confirmed allocation tells the person; an ended one is not changed.
        let changed = request.put(&format!("/api/allocations/{confirmed}")).json(&json!({ "fte_centi": 20, "note": "  less  " })).await;
        assert_eq!(changed.status_code(), 200, "{}", changed.text());
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM notifications WHERE worker_pid = '{w}' AND kind = 'allocation_changed'")).await, 1);
        assert_eq!(request.put(&format!("/api/allocations/{confirmed}")).json(&json!({ "fte_centi": 999 })).await.status_code(), 422);
        request.post(&format!("/api/allocations/{confirmed}/status")).json(&json!({ "to": "ended" })).await.assert_status_ok();
        assert_eq!(request.put(&format!("/api/allocations/{confirmed}")).json(&json!({ "fte_centi": 10 })).await.status_code(), 422, "ended is final");
        assert_eq!(request.post(&format!("/api/allocations/{confirmed}/status")).json(&json!({ "to": "confirmed" })).await.status_code(), 422);

        // The summary against an active claim: 60 confirmed of a 100 claim for programme-a in February.
        exec(&ctx, format!(
            "INSERT INTO programme_demands (pid, programme_ref, pool_pid, month, fte_centi, status) VALUES
             ('{}', 'thing:programme-a', '{pool}', '2027-02-01', 100, 'active')", uuid::Uuid::new_v4())).await;
        let summary: Value = request.get(&format!("/api/allocations/summary?organization={org}&from=2027-02-01&months=1")).await.json();
        let cell = summary["cells"].as_array().unwrap().iter().find(|c| c["source_ref"] == "thing:programme-a").unwrap().clone();
        assert_eq!((cell["claim_centi"].as_i64(), cell["allocated_centi"].as_i64(), cell["people"].as_i64()), (Some(100), Some(60), Some(1)), "{summary}");
        assert_eq!(cell["staffing"]["understaffed_by_centi"], 40);
        assert!(!summary.to_string().contains(&w), "the summary names no one");

        // The named list for HR, by source, and the audit of each named read.
        let listed: Value = request.get(&format!("/api/allocations?organization={org}&source=thing:programme-c&from=2027-01-01&months=2")).await.json();
        assert_eq!(listed.as_array().map(Vec::len), Some(1), "{listed}");
        let none: Value = request.get(&format!("/api/allocations?organization={org}&source=thing:nothing&from=2027-01-01&months=2")).await.json();
        assert_eq!(none.as_array().map(Vec::len), Some(0));
        assert!(scalar(&ctx, "SELECT count(*) AS n FROM audit_logs WHERE action = 'allocations_read'".to_string()).await >= 2);
        assert!(scalar(&ctx, format!("SELECT count(*) AS n FROM audit_logs WHERE action = 'allocation_status_changed' AND entity_pid = '{tentative}'")).await >= 1);
        let snapshot = sqlx::query(sqlx::AssertSqlSafe(format!("SELECT snapshot::text AS s FROM audit_logs WHERE action = 'allocation_created' AND entity_pid = '{tentative}'")))
            .fetch_one(workforce_planning_management::db::pool(&ctx)).await.unwrap().try_get::<String, _>("s").unwrap();
        assert!(!snapshot.contains("fte") && !snapshot.contains("programme"), "the audit names no share and no source: {snapshot}");

        // Removal, then a removed one is a 404.
        request.delete(&format!("/api/allocations/{tentative}")).await.assert_status_ok();
        assert_eq!(request.delete(&format!("/api/allocations/{tentative}")).await.status_code(), 404);
        assert_eq!(request.get(&format!("/api/workers/{}/allocations", uuid::Uuid::new_v4())).await.status_code(), 404);

        // The subject export holds them; erasure deletes every row, soft-deleted or not.
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).await.json();
        assert_eq!(export["allocations"].as_array().map(Vec::len), Some(3), "{export}");
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        request.post(&format!("/api/workers/{w}/erase")).await.assert_status_ok();
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM allocations WHERE worker_pid = '{w}'")).await, 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_window_the_moves_and_every_staffing_answer_are_checked() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("AW-{tag}"), None).await;
        activate!(&request, &w).await;
        let assignments: Value = request.get(&format!("/api/workers/{w}/assignments")).await.json();
        let assignment = assignments["assignments"][0]["pid"].as_str().unwrap().to_string();
        let pool: Value = request
            .post("/api/skill-pools")
            .json(&json!({ "organization_ref": org, "name": format!("Pool {tag}") }))
            .await
            .json();
        let pool = pool["pid"].as_str().unwrap().to_string();
        let url = format!("/api/workers/{w}/allocations");

        // The window: none and more than the horizon are refused, on both reads.
        for months in [0, 37] {
            assert_eq!(request.get(&format!("{url}?months={months}")).await.status_code(), 422);
            assert_eq!(
                request.get(&format!("/api/allocations/summary?organization={org}&months={months}")).await.status_code(),
                422
            );
        }

        // An unknown status is refused, and a tentative allocation can end without telling anyone.
        let made: Value = request
            .post(&url)
            .json(&json!({ "assignment_pid": assignment, "source_ref": "thing:programme-x", "pool_pid": pool,
                            "from_month": "2027-05-01", "to_month": "2027-05-01", "fte_centi": 50 }))
            .await
            .json();
        let pid = made["pid"].as_str().unwrap().to_string();
        assert_eq!(request.post(&format!("/api/allocations/{pid}/status")).json(&json!({ "to": "maybe" })).await.status_code(), 422);
        request.post(&format!("/api/allocations/{pid}/status")).json(&json!({ "to": "ended" })).await.assert_status_ok();
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM notifications WHERE worker_pid = '{w}'")).await, 0);

        // Untracked: the comparison is unknown and the figures are not given.
        let confirmed = |source: &str, fte: i64| {
            let (request, url, assignment, pool) = (&request, url.clone(), assignment.clone(), pool.clone());
            let source = source.to_string();
            async move {
                request
                    .post(&url)
                    .json(&json!({ "assignment_pid": assignment, "source_ref": source, "pool_pid": pool,
                                    "from_month": "2027-06-01", "to_month": "2027-06-01", "fte_centi": fte, "status": "confirmed" }))
                    .await
                    .assert_status_ok();
            }
        };
        confirmed("thing:programme-staffed", 40).await;
        confirmed("thing:programme-over", 30).await;
        for (source, claim) in [("thing:programme-staffed", 40), ("thing:programme-over", 20)] {
            exec(&ctx, format!(
                "INSERT INTO programme_demands (pid, programme_ref, pool_pid, month, fte_centi, status) VALUES
                 ('{}', '{source}', '{pool}', '2027-06-01', {claim}, 'active')", uuid::Uuid::new_v4())).await;
        }
        let summary_url = format!("/api/allocations/summary?organization={org}&from=2027-06-01&months=1");
        let untracked: Value = request.get(&summary_url).await.json();
        assert_eq!(untracked["tracked"], false);
        for cell in untracked["cells"].as_array().unwrap() {
            assert_eq!(cell["staffing"], "unknown", "{cell}");
            assert!(cell["allocated_centi"].is_null() && cell["people"].is_null(), "{cell}");
        }

        // Tracked: staffed where the allocations equal the claim, overstaffed where they pass it.
        request.put("/api/capacity/settings").json(&json!({ "organization_ref": org, "allocation_tracked": true })).await.assert_status_ok();
        let tracked: Value = request.get(&summary_url).await.json();
        let of = |source: &str| tracked["cells"].as_array().unwrap().iter().find(|c| c["source_ref"] == source).unwrap().clone();
        assert_eq!(of("thing:programme-staffed")["staffing"], "staffed");
        assert_eq!(of("thing:programme-over")["staffing"]["overstaffed_by_centi"], 10);
        assert_eq!(request.get("/api/capacity/settings").await.status_code(), 422, "the organization is required");
    })
    .await;
}
