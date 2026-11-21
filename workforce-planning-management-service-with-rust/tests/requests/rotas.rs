//! On-call rotas: a rotation through the members, swaps, skipping members on
//! approved leave, and a worker's own on-call stretches.
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::tasks::rota_reminders::send_reminders;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn an_on_call_rota_rotates_swaps_and_skips_leave() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut pids = Vec::new();
        for n in ["A", "B", "C"] {
            let pid = seed_worker!(&request, &org, format!("ROTA-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        let (a, b, c) = (&pids[0], &pids[1], &pids[2]);
        let today = workforce_planning_management::dates::today();
        let day = |n: i64| (today + workforce_planning_management::dates::days(n)).to_string();

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
        let year = today.strftime("%Y").to_string().parse::<i32>().unwrap();
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
        // The rotation keeps the order given, numbered from 1.
        let shown: Value = request.get(&format!("/api/rotas/{rota_pid}")).await.json();
        let order: Vec<(i64, String)> = shown["members"].as_array().unwrap().iter()
            .map(|m| (m["position"].as_i64().unwrap(), m["worker_pid"].as_str().unwrap().to_string()))
            .collect();
        assert_eq!(order, [(1, c.clone()), (2, b.clone()), (3, a.clone())]);

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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn on_call_reminders_go_once_to_whoever_a_turn_starts_for() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut pids = Vec::new();
        for n in ["A", "B"] {
            let pid = seed_worker!(&request, &org, format!("REM-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        let today = workforce_planning_management::dates::today();
        // A's week starts a week ago (so today is mid-turn); B's starts in 6 days.
        request
            .post("/api/rotas")
            .json(
                &json!({ "organization_ref": org, "name": format!("Rem {tag}"),
                           "period_days": 7, "starts_on": (today - workforce_planning_management::dates::days(1)).to_string(),
                           "members": [pids[0], pids[1]] }),
            )
            .await
            .assert_status_ok();
        let reminders = |who: String| {
            let request = &request;
            async move {
                let told: Value = request
                    .get(&format!("/api/workers/{who}/notifications"))
                    .await
                    .json();
                told.as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| n["kind"] == "on_call_reminder")
                    .count()
            }
        };
        // B's turn starts in 6 days (the day after A's 7-day turn ends).
        let starts = today + workforce_planning_management::dates::days(6);
        assert_eq!(send_reminders(&ctx, starts).await.unwrap(), 1);
        assert_eq!(reminders(pids[1].clone()).await, 1);
        assert_eq!(reminders(pids[0].clone()).await, 0);
        // Idempotent: a second run tells nobody again.
        assert_eq!(send_reminders(&ctx, starts).await.unwrap(), 0);
        assert_eq!(reminders(pids[1].clone()).await, 1);
        // A day mid-turn starts nobody's turn.
        assert_eq!(
            send_reminders(&ctx, today + workforce_planning_management::dates::days(3))
                .await
                .unwrap(),
            0
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn swap_requests_move_only_the_requesters_days_when_accepted() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut pids = Vec::new();
        for n in ["A", "B", "C"] {
            let pid = seed_worker!(&request, &org, format!("SW-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        let (a, b, c) = (&pids[0], &pids[1], &pids[2]);
        let today = workforce_planning_management::dates::today();
        let day = |n: i64| (today + workforce_planning_management::dates::days(n)).to_string();
        let rota: Value = request
            .post("/api/rotas")
            .json(
                &json!({ "organization_ref": org, "name": format!("Swap {tag}"),
                           "period_days": 7, "starts_on": today.to_string(),
                           "members": [a, b, c] }),
            )
            .await
            .json();
        let rota = rota["pid"].as_str().unwrap().to_string();
        let url = format!("/api/rotas/{rota}/swap-requests");

        // A is on call this week; B is not, so B cannot hand over this week.
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "requester_pid": b, "taker_pid": c,
                               "starts_on": day(0), "ends_on": day(6) }))
                .await
                .status_code(),
            422,
            "the requester is not on call in that window"
        );
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "requester_pid": a, "taker_pid": a,
                               "starts_on": day(0), "ends_on": day(6) }))
                .await
                .status_code(),
            422
        );

        // A asks B for the whole fortnight window, which only covers A's week.
        let asked: Value = request
            .post(&url)
            .json(&json!({ "requester_pid": a, "taker_pid": b,
                           "starts_on": day(0), "ends_on": day(13), "note": "away" }))
            .await
            .json();
        let first = asked["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "requester_pid": a, "taker_pid": b,
                               "starts_on": day(0), "ends_on": day(13) }))
                .await
                .status_code(),
            422,
            "already asked"
        );
        let incoming: Value = request
            .get(&format!("/api/workers/{b}/swap-requests"))
            .await
            .json();
        assert_eq!(incoming["incoming"].as_array().unwrap().len(), 1);
        assert_eq!(incoming["outgoing"].as_array().unwrap().len(), 0);
        let told: Value = request
            .get(&format!("/api/workers/{b}/notifications"))
            .await
            .json();
        assert!(
            told.as_array()
                .unwrap()
                .iter()
                .any(|n| n["kind"] == "swap_requested")
        );

        // B accepts: only A's own days (week 1) move — B's own turn (week 2) is untouched.
        let ok: Value = request
            .post(&format!("/api/rota-swap-requests/{first}/accept"))
            .await
            .json();
        assert_eq!(ok["status"], "accepted");
        assert_eq!(ok["days_moved"], 1, "one stretch");
        let again = request.post(&format!("/api/rota-swap-requests/{first}/accept")).await;
        assert_eq!(again.status_code(), 422, "{}", again.text());
        assert!(again.text().contains("already accepted"), "{}", again.text());
        let now: Value = request
            .get(&format!("/api/rotas/{rota}/on-call"))
            .await
            .json();
        assert_eq!(now["worker_pid"], b.as_str());
        assert_eq!(now["source"], "override");
        let week2: Value = request
            .get(&format!("/api/rotas/{rota}/on-call?on={}", day(8)))
            .await
            .json();
        assert_eq!(week2["worker_pid"], b.as_str(), "B's own turn");
        let told: Value = request
            .get(&format!("/api/workers/{a}/notifications"))
            .await
            .json();
        assert!(
            told.as_array()
                .unwrap()
                .iter()
                .any(|n| n["kind"] == "swap_decided")
        );
        // Decided once.
        assert_eq!(
            request
                .post(&format!("/api/rota-swap-requests/{first}/decline"))
                .await
                .status_code(),
            422
        );

        // Decline and cancel paths on the next stretch of A's (week 4 = day 21..27).
        let second: Value = request
            .post(&url)
            .json(&json!({ "requester_pid": a, "taker_pid": c,
                           "starts_on": day(21), "ends_on": day(27) }))
            .await
            .json();
        let second = second["pid"].as_str().unwrap();
        request
            .post(&format!("/api/rota-swap-requests/{second}/decline"))
            .await
            .assert_status_ok();
        let third: Value = request
            .post(&url)
            .json(&json!({ "requester_pid": a, "taker_pid": c,
                           "starts_on": day(21), "ends_on": day(27) }))
            .await
            .json();
        request
            .post(&format!(
                "/api/rota-swap-requests/{}/cancel",
                third["pid"].as_str().unwrap()
            ))
            .await
            .assert_status_ok();
        let all: Vec<Value> = request.get(&url).await.json();
        let statuses: Vec<&str> = all.iter().map(|r| r["status"].as_str().unwrap()).collect();
        assert_eq!(statuses, ["cancelled", "declined", "accepted"]);
        let after: Value = request
            .get(&format!("/api/rotas/{rota}/on-call?on={}", day(21)))
            .await
            .json();
        assert_eq!(
            after["worker_pid"],
            a.as_str(),
            "declined and cancelled swaps move nothing"
        );

        // If A's days are given to someone else before the swap is taken, there is nothing left to hand over.
        let fourth: Value = request
            .post(&url)
            .json(&json!({ "requester_pid": a, "taker_pid": c, "starts_on": day(21), "ends_on": day(27) }))
            .await
            .json();
        request
            .post(&format!("/api/rotas/{rota}/overrides"))
            .json(&json!({ "worker_pid": b, "starts_on": day(21), "ends_on": day(27) }))
            .await
            .assert_status_ok();
        let stale = request.post(&format!("/api/rota-swap-requests/{}/accept", fourth["pid"].as_str().unwrap())).await;
        assert_eq!(stale.status_code(), 422, "{}", stale.text());
        assert!(stale.text().contains("no longer on call"), "{}", stale.text());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A rota refuses a blank name and another organization's member; a description is kept and cleared; its swaps are listed;
// a window that ends before it starts is refused.
async fn a_rota_refuses_what_cannot_be_and_lists_its_swaps() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut people = Vec::new();
        for n in ["A", "B"] {
            let pid = seed_worker!(&request, &org, format!("RT-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            people.push(pid);
        }
        let stranger = seed_worker!(&request, &an_org(), format!("RT-{tag}-X"), None).await;
        activate!(&request, &stranger).await;
        let today = workforce_planning_management::dates::today();
        let rota = |name: &str, members: Vec<&String>| {
            json!({ "organization_ref": org, "name": name, "period_days": 7, "starts_on": today.to_string(),
                    "members": members, "description": "  Out of hours  " })
        };
        assert_eq!(request.post("/api/rotas").json(&rota(" ", vec![&people[0]])).await.status_code(), 422, "blank name");
        let foreign = request.post("/api/rotas").json(&rota("Support", vec![&people[0], &stranger])).await;
        assert_eq!(foreign.status_code(), 422, "{}", foreign.text());
        let made: Value = request.post("/api/rotas").json(&rota("Support", vec![&people[0], &people[1]])).await.json();
        let pid = made["pid"].as_str().unwrap().to_string();
        let url = format!("/api/rotas/{pid}");
        let read = || async { request.get(&url).await.json::<Value>() };
        assert_eq!(read().await["description"], "Out of hours", "trimmed and kept");

        assert_eq!(request.put(&url).json(&json!({ "name": " " })).await.status_code(), 422);
        request.put(&url).json(&json!({ "description": " " })).await.assert_status_ok();
        let cleared = read().await;
        assert!(cleared["description"].is_null(), "a blank description clears it: {cleared}");

        let tomorrow = today + workforce_planning_management::dates::days(1);
        request
            .post(&format!("{url}/overrides"))
            .json(&json!({ "worker_pid": people[1], "starts_on": tomorrow.to_string(), "ends_on": tomorrow.to_string(), "note": "swap" }))
            .await
            .assert_status_ok();
        let view = read().await;
        assert_eq!(view["overrides"][0]["worker_pid"], people[1].as_str(), "{view}");
        assert_eq!(view["overrides"][0]["note"], "swap");

        let backwards = request.get(&format!("{url}?from={tomorrow}&to={today}")).await;
        assert_eq!(backwards.status_code(), 422, "{}", backwards.text());
    })
    .await;
}
