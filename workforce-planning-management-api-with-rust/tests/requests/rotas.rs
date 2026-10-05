//! On-call rotas: a rotation through the members, swaps, skipping members on
//! approved leave, and a worker's own on-call stretches.

use chrono::{Duration, Utc};
use loco_rs::testing::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;
use workforce_planning_management_service::tasks::rota_reminders::send_reminders;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn an_on_call_rota_rotates_swaps_and_skips_leave() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut pids = Vec::new();
        for n in ["A", "B", "C"] {
            let pid = seed_worker!(&request, &org, format!("ROTA-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        let (a, b, c) = (&pids[0], &pids[1], &pids[2]);
        let today = Utc::now().date_naive();
        let day = |n: i64| (today + Duration::days(n)).to_string();

        // Shape is validated: a stranger, nobody, a repeat, a bad period.
        let rota = |members: Value, period: i32| {
            json!({ "organization_ref": org, "name": format!("Platform {tag}"),
                    "period_days": period, "starts_on": today.to_string(), "members": members })
        };
        for bad in [
            rota(json!([]), 7),
            rota(json!([a, a]), 7),
            rota(json!([a]), 0),
            rota(json!([uuid::Uuid::new_v4()]), 7),
        ] {
            assert_eq!(request.post("/api/rotas").json(&bad).await.status_code(), 422, "{bad}");
        }
        let created: Value = request
            .post("/api/rotas")
            .json(&rota(json!([a, b, c]), 7))
            .await
            .json();
        let rota_pid = created["pid"].as_str().expect("pid").to_string();
        assert_eq!(
            request.post("/api/rotas").json(&rota(json!([a]), 7)).await.status_code(),
            422,
            "the name is taken in this organization"
        );

        // Everyone named is told, with the rota's name and nothing else.
        for who in [a, b, c] {
            let told: Value = request.get(&format!("/api/workers/{who}/notifications")).await.json();
            let kinds: Vec<&str> = told
                .as_array()
                .expect("notifications")
                .iter()
                .filter(|n| n["kind"] == "rota_added")
                .filter_map(|n| n["body"].as_str())
                .collect();
            assert_eq!(kinds, [format!("You were added to the on-call rota Platform {tag}.")]);
        }

        // Week 1: A, week 2: B, week 3: C, week 4: A.
        let view: Value = request.get(&format!("/api/rotas/{rota_pid}")).await.json();
        assert_eq!(view["on_call_today"]["worker_pid"], a.as_str());
        let runs = view["runs"].as_array().expect("runs");
        assert_eq!(runs.len(), 4);
        let who: Vec<&str> = runs.iter().map(|r| r["worker_pid"].as_str().unwrap()).collect();
        assert_eq!(who, [a.as_str(), b.as_str(), c.as_str(), a.as_str()]);
        assert!(runs.iter().all(|r| r["source"] == "rotation"));
        let load = view["load"].as_array().expect("load");
        assert_eq!(load.iter().map(|l| l["days"].as_i64().unwrap()).sum::<i64>(), 28);

        let next: Value = request
            .get(&format!("/api/rotas/{rota_pid}/on-call?on={}", day(7)))
            .await
            .json();
        assert_eq!(next["worker_pid"], b.as_str());

        // A swap puts C on call for two days; undoing it restores A.
        let swap: Value = request
            .post(&format!("/api/rotas/{rota_pid}/overrides"))
            .json(&json!({ "worker_pid": c, "starts_on": day(0), "ends_on": day(1), "note": "A at a conference" }))
            .await
            .json();
        let now: Value = request.get(&format!("/api/rotas/{rota_pid}/on-call")).await.json();
        assert_eq!(now["worker_pid"], c.as_str());
        assert_eq!(now["source"], "override");
        let told: Value = request.get(&format!("/api/workers/{c}/notifications")).await.json();
        assert!(
            told.as_array().unwrap().iter().any(|n| n["kind"] == "on_call_swap"
                && n["body"].as_str().unwrap().starts_with(&format!("You are on call for Platform {tag} from"))),
            "the swapped-in person is told"
        );
        assert_eq!(
            request
                .post(&format!("/api/rotas/{rota_pid}/overrides"))
                .json(&json!({ "worker_pid": c, "starts_on": day(3), "ends_on": day(2) }))
                .await
                .status_code(),
            422
        );
        request
            .delete(&format!("/api/rota-overrides/{}", swap["pid"].as_str().unwrap()))
            .await
            .assert_status_ok();
        let now: Value = request.get(&format!("/api/rotas/{rota_pid}/on-call")).await.json();
        assert_eq!(now["worker_pid"], a.as_str());

        // B takes approved leave across week 2: C is skipped in for B.
        let year = today.format("%Y").to_string().parse::<i32>().unwrap();
        for y in [year, year + 1] {
            request
                .post(&format!("/api/workers/{b}/leave-entitlements"))
                .json(&json!({ "kind": "annual", "year": y, "entitled_days": 30 }))
                .await
                .assert_status_ok();
        }
        let leave: Value = request
            .post(&format!("/api/workers/{b}/leave-requests"))
            .json(&json!({ "kind": "annual", "start_on": day(7), "end_on": day(13) }))
            .await
            .json();
        request
            .post(&format!("/api/leave-requests/{}/approve", leave["pid"].as_str().unwrap()))
            .await
            .assert_status_ok();
        let covered: Value = request
            .get(&format!("/api/rotas/{rota_pid}/on-call?on={}", day(9)))
            .await
            .json();
        assert_eq!(covered["worker_pid"], c.as_str(), "B is on leave, so C takes it");
        assert_eq!(covered["source"], "skipped");

        // A's own stretches, across rotas.
        let mine: Vec<Value> = request.get(&format!("/api/workers/{a}/on-call")).await.json();
        assert_eq!(mine.len(), 2, "weeks 1 and 4");
        assert_eq!(mine[0]["rota_name"], format!("Platform {tag}"));

        // Re-order the rotation: C first.
        request
            .put(&format!("/api/rotas/{rota_pid}"))
            .json(&json!({ "members": [c, b, a] }))
            .await
            .assert_status_ok();
        let now: Value = request.get(&format!("/api/rotas/{rota_pid}/on-call")).await.json();
        assert_eq!(now["worker_pid"], c.as_str());

        // It appears in the list with who is on call today; then retire it.
        let list: Vec<Value> = request.get("/api/rotas").await.json();
        let mine = list.iter().find(|r| r["pid"] == rota_pid.as_str()).expect("listed");
        assert_eq!(mine["members"], 3);
        assert_eq!(mine["on_call_today"]["worker_pid"], c.as_str());
        request.delete(&format!("/api/rotas/{rota_pid}")).await.assert_status_ok();
        assert_eq!(
            request.get(&format!("/api/rotas/{rota_pid}")).await.status_code(),
            404
        );

        // A window past three months is refused.
        let other: Value = request
            .post("/api/rotas")
            .json(&json!({ "organization_ref": org, "name": format!("Second {tag}"),
                           "period_days": 1, "starts_on": today.to_string(), "members": [a] }))
            .await
            .json();
        assert_eq!(
            request
                .get(&format!(
                    "/api/rotas/{}?from={}&to={}",
                    other["pid"].as_str().unwrap(),
                    day(0),
                    day(100)
                ))
                .await
                .status_code(),
            422
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn on_call_reminders_go_once_to_whoever_a_turn_starts_for() {
    request::<App, _, _>(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut pids = Vec::new();
        for n in ["A", "B"] {
            let pid = seed_worker!(&request, &org, format!("REM-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        let today = Utc::now().date_naive();
        // A's week starts a week ago (so today is mid-turn); B's starts in 6 days.
        request
            .post("/api/rotas")
            .json(&json!({ "organization_ref": org, "name": format!("Rem {tag}"),
                           "period_days": 7, "starts_on": (today - Duration::days(1)).to_string(),
                           "members": [pids[0], pids[1]] }))
            .await
            .assert_status_ok();
        let reminders = |who: String| {
            let request = &request;
            async move {
                let told: Value = request.get(&format!("/api/workers/{who}/notifications")).await.json();
                told.as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| n["kind"] == "on_call_reminder")
                    .count()
            }
        };
        // B's turn starts in 6 days (the day after A's 7-day turn ends).
        let starts = today + Duration::days(6);
        assert_eq!(send_reminders(&ctx, starts).await.unwrap(), 1);
        assert_eq!(reminders(pids[1].clone()).await, 1);
        assert_eq!(reminders(pids[0].clone()).await, 0);
        // Idempotent: a second run tells nobody again.
        assert_eq!(send_reminders(&ctx, starts).await.unwrap(), 0);
        assert_eq!(reminders(pids[1].clone()).await, 1);
        // A day mid-turn starts nobody's turn.
        assert_eq!(send_reminders(&ctx, today + Duration::days(3)).await.unwrap(), 0);
    })
    .await;
}
