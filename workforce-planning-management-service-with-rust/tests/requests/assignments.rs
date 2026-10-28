//! Assignments (WPM-R161): a primary one from the worker row, further ones capped at 100 FTE, a change that ends one and starts
//! another and writes through, the worker-row triggers, and the subject export and erasure.

use chrono::{Duration, Utc};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

async fn scalar(ctx: &loco_rs::app::AppContext, sql: String) -> i64 {
    ctx.db
        .query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_worker_has_a_primary_assignment_and_a_change_ends_one_and_starts_another() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("AS-{tag}"), None).await;
        activate!(&request, &w).await;
        let url = format!("/api/workers/{w}/assignments");

        // A new worker already has one primary assignment, from the worker row, with no category recorded.
        let first: Value = request.get(&url).await.json();
        assert_eq!(first["assignments"].as_array().map(Vec::len), Some(1), "{first}");
        let primary = first["assignments"][0].clone();
        assert_eq!(primary["is_primary"], true);
        assert_eq!(primary["job_title"], "Engineer");
        assert_eq!(primary["staff_group"], "unknown", "a category nobody recorded reads unknown");
        assert_eq!(first["fte_total"], 100);
        assert!(primary.get("recorded_by").is_none());
        assert!(primary.get("id").is_none());
        let primary_pid = primary["pid"].as_str().unwrap().to_string();

        // Over 100 is a 422 that names the total; so are bad words and a start before the first day.
        let over = request
            .post(&url)
            .json(&json!({ "job_title": "Locum", "department": "ward", "fte_percent": 50 }))
            .await;
        assert_eq!(over.status_code(), 422);
        assert!(over.text().contains("150"), "{}", over.text());
        for bad in [
            json!({ "job_title": " ", "department": "ward", "fte_percent": 10 }),
            json!({ "job_title": "Locum", "department": "ward", "fte_percent": 0 }),
            json!({ "job_title": "Locum", "department": "ward", "fte_percent": 10, "starts_on": "2000-01-01" }),
            json!({ "job_title": "Locum", "department": "ward", "fte_percent": 10, "staff_group": " " }),
        ] {
            assert_eq!(request.post(&url).json(&bad).await.status_code(), 422, "{bad}");
        }

        // A change: effective in the future, before the start, or over 100 is refused.
        let change = format!("/api/assignments/{primary_pid}/change");
        let future = (Utc::now().date_naive() + Duration::days(3)).to_string();
        assert_eq!(request.post(&change).json(&json!({ "effective_on": future, "fte_percent": 60 })).await.status_code(), 422);
        assert_eq!(request.post(&change).json(&json!({ "effective_on": "2026-01-05", "fte_percent": 60 })).await.status_code(), 422);
        assert_eq!(request.post(&change).json(&json!({ "fte_percent": 101 })).await.status_code(), 422);

        // A real change ends the old one the day before and starts a new one that keeps what was not changed.
        let done = request
            .post(&change)
            .json(&json!({ "fte_percent": 60, "job_title": "Senior Engineer", "staff_group": "clinical" }))
            .await;
        assert_eq!(done.status_code(), 200, "{}", done.text());
        let done: Value = done.json();
        let yesterday = (Utc::now().date_naive() - Duration::days(1)).to_string();
        assert_eq!(done["previous_ends_on"], yesterday);
        let after: Value = request.get(&url).await.json();
        let rows = after["assignments"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        let (new, old) = (&rows[0], &rows[1]);
        assert_eq!((new["job_title"].as_str(), new["fte_percent"].as_i64(), new["staff_group"].as_str()), (Some("Senior Engineer"), Some(60), Some("clinical")));
        assert_eq!(new["department"], "engineering", "kept");
        assert_eq!((new["is_primary"].as_bool(), new["current"].as_bool()), (Some(true), Some(true)));
        assert_eq!((old["ends_on"].as_str(), old["current"].as_bool(), old["fte_percent"].as_i64()), (Some(yesterday.as_str()), Some(false), Some(100)));
        assert_eq!(after["fte_total"], 60, "only what is in force counts");

        // It wrote through to the worker row.
        let worker: Value = request.get(&format!("/api/workers/{w}")).await.json();
        assert_eq!((worker["job_title"].as_str(), worker["fte_percent"].as_i64()), (Some("Senior Engineer"), Some(60)), "{worker}");

        // Now there is room for 40, and not for 41.
        let add = |fte: i64| {
            let request = &request;
            let url = url.clone();
            async move { request.post(&url).json(&json!({ "job_title": "Bank shifts", "department": "ward", "fte_percent": fte })).await }
        };
        assert_eq!(add(41).await.status_code(), 422);
        let extra = add(40).await;
        assert_eq!(extra.status_code(), 200, "{}", extra.text());
        let total: Value = request.get(&url).await.json();
        assert_eq!(total["fte_total"], 100);
        let opens = scalar(&ctx, format!("SELECT count(*) AS n FROM assignments WHERE worker_pid = '{w}' AND is_primary AND ends_on IS NULL")).await;
        assert_eq!(opens, 1, "exactly one open primary");

        // The worker-row trigger: an older writer correcting the department corrects the open primary in place.
        ctx.db.execute_unprepared(&format!("UPDATE workers SET department = 'cardiology' WHERE pid = '{w}'")).await.unwrap();
        let dept = ctx
            .db
            .query_one_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT department AS d FROM assignments WHERE worker_pid = '{w}' AND is_primary AND ends_on IS NULL"),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<String>("", "d")
            .unwrap();
        assert_eq!(dept, "cardiology");
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM assignments WHERE worker_pid = '{w}'")).await, 3, "no history is made by a correction");

        // A stranger worker pid is a 404; the audit trail names the change.
        assert_eq!(request.get(&format!("/api/workers/{}/assignments", uuid::Uuid::new_v4())).await.status_code(), 404);
        assert!(scalar(&ctx, format!("SELECT count(*) AS n FROM audit_logs WHERE action = 'assignment_changed' AND entity_pid = '{}'", done["pid"].as_str().unwrap())).await >= 1);

        // The subject export lists the assignments; erasure removes them.
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).await.json();
        assert_eq!(export["assignments"].as_array().map(Vec::len), Some(3), "{export}");
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        request.post(&format!("/api/workers/{w}/erase")).await.assert_status_ok();
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM assignments WHERE worker_pid = '{w}'")).await, 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_job_role_and_a_staff_group_must_agree_where_the_vocabulary_says_so() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("VO-{tag}"), None).await;
        activate!(&request, &w).await;
        let entry = |body: Value| {
            let request = &request;
            async move { request.post("/api/category-vocabulary").json(&body).await }
        };

        // Entries: a bad kind, a standard without its code, a staff group on a site, and a repeated code are refused.
        for bad in [
            json!({ "organization_ref": org, "kind": "ward", "code": "A", "label": "A" }),
            json!({ "organization_ref": org, "kind": "site", "code": "A", "label": "A", "standard": "ESR" }),
            json!({ "organization_ref": org, "kind": "site", "code": "A", "label": "A", "staff_group": "nursing" }),
            json!({ "organization_ref": org, "kind": "site", "code": " ", "label": "A" }),
        ] {
            assert_eq!(entry(bad.clone()).await.status_code(), 422, "{bad}");
        }
        for ok in [
            json!({ "organization_ref": org, "kind": "staff_group", "code": "nursing", "label": "Nursing", "standard": "synthetic", "standard_code": "S-1" }),
            json!({ "organization_ref": org, "kind": "staff_group", "code": "admin", "label": "Administrative" }),
            json!({ "organization_ref": org, "kind": "job_role", "code": "RN", "label": "Registered nurse", "staff_group": "nursing" }),
            json!({ "organization_ref": org, "kind": "job_role", "code": "FREE", "label": "No group stated" }),
        ] {
            let made = entry(ok.clone()).await;
            assert_eq!(made.status_code(), 200, "{ok}: {}", made.text());
        }
        let again = entry(json!({ "organization_ref": org, "kind": "job_role", "code": "RN", "label": "Again" })).await;
        assert_eq!(again.status_code(), 422, "a code is unique in its kind");
        let roles: Value = request.get(&format!("/api/category-vocabulary?organization={org}&kind=job_role")).await.json();
        assert_eq!(roles["entries"].as_array().map(Vec::len), Some(2), "{roles}");
        assert!(roles["entries"][0].get("recorded_by").is_none());
        assert_eq!(request.get(&format!("/api/category-vocabulary?organization={org}&kind=ward")).await.status_code(), 422);

        // A disagreement is a 422; agreement, an unlisted role and a role with no stated group are not.
        let url = format!("/api/workers/{w}/assignments");
        let primary_of = || {
            let request = &request;
            let url = url.clone();
            async move {
                let all: Value = request.get(&url).await.json();
                all["assignments"].as_array().unwrap().iter().find(|a| a["is_primary"] == true && a["current"] == true).unwrap()["pid"].as_str().unwrap().to_string()
            }
        };
        // Make room first: the primary is cut to 50, so the one-percent shifts below fit.
        let cut = request.post(&format!("/api/assignments/{}/change", primary_of().await)).json(&json!({ "fte_percent": 50, "effective_on": "2026-06-01" })).await;
        assert_eq!(cut.status_code(), 200, "{}", cut.text());
        let add = |role: &str, group: Option<&str>| {
            let request = &request;
            let url = url.clone();
            let body = json!({ "job_title": "Shift", "department": "ward", "fte_percent": 1, "job_role": role, "staff_group": group });
            async move { request.post(&url).json(&body).await.status_code() }
        };
        assert_eq!(add("RN", Some("admin")).await, 422, "the role belongs to nursing");
        assert_eq!(add("RN", Some("nursing")).await, 200);
        assert_eq!(add("RN", None).await, 200, "no staff group recorded is unknown, not a mismatch");
        assert_eq!(add("UNLISTED", Some("admin")).await, 200, "the vocabulary does not say");
        assert_eq!(add("FREE", Some("admin")).await, 200, "a role with no stated group");

        // A change is checked the same way.
        let primary = primary_of().await;
        let change = format!("/api/assignments/{primary}/change");
        assert_eq!(request.post(&change).json(&json!({ "job_role": "RN", "staff_group": "admin" })).await.status_code(), 422);
        assert_eq!(request.post(&change).json(&json!({ "job_role": "RN", "staff_group": "nursing", "fte_percent": 90 })).await.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_breakdown_by_category_shows_nothing_under_the_floor_and_gives_no_total() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut workers = Vec::new();
        for n in 0..10 {
            let w = seed_worker!(&request, &org, format!("BD-{tag}-{n}"), None).await;
            activate!(&request, &w).await;
            workers.push(w);
        }
        // All ten are clinical; three also sit in one area of work.
        ctx.db.execute_unprepared(&format!(
            "UPDATE assignments SET staff_group = 'clinical' WHERE worker_pid IN (SELECT pid FROM workers WHERE organization_ref = '{org}')"
        )).await.unwrap();
        for w in &workers[..3] {
            ctx.db.execute_unprepared(&format!("UPDATE assignments SET area_of_work = 'ward' WHERE worker_pid = '{w}'")).await.unwrap();
        }
        let get = |by: &'static str| {
            let request = &request;
            let org = org.clone();
            async move { request.get(&format!("/api/assignments/breakdown?organization={org}&by={by}")).await.json::<Value>() }
        };
        let group = get("staff_group").await;
        assert_eq!(group["floor"], 10);
        assert_eq!(group["cells"][0], json!({ "category": "clinical", "headcount": 10, "fte_percent": 1000 }), "{group}");
        let area = get("area_of_work").await;
        let cells = area["cells"].as_array().unwrap();
        assert_eq!(cells.len(), 2, "{area}");
        for c in cells {
            assert!(c["headcount"].is_null() && c["fte_percent"].is_null(), "3 and 7 are under the floor of 10: {c}");
        }
        assert!(cells.iter().any(|c| c["category"] == "unknown"), "not recorded is its own cell");
        assert!(area.get("total").is_none() && !area.to_string().contains(&workers[0]), "no total and no worker");
        let role = get("job_role").await;
        assert_eq!(role["cells"][0]["category"], "unknown");
        assert_eq!(role["cells"][0]["headcount"], 10);
        assert_eq!(request.get(&format!("/api/assignments/breakdown?organization={org}&by=salary")).await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn an_assignment_names_a_cost_centre_that_is_open_and_unmatched_position_values_are_listed() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("CC-{tag}"), None).await;
        activate!(&request, &w).await;
        let url = format!("/api/workers/{w}/assignments");
        let cc = |body: Value| {
            let request = &request;
            async move { request.post("/api/cost-centres").json(&body).await }
        };

        // A register: a root, a child under it; refusals for a repeat, a missing parent, a blank name.
        let top: Value = cc(json!({ "organization_ref": org, "code": "TOP", "name": "Trust", "valid_from": "2020-01-01" })).await.json();
        let top_pid = top["pid"].as_str().unwrap().to_string();
        assert_eq!(cc(json!({ "organization_ref": org, "code": "TOP", "name": "Again" })).await.status_code(), 422);
        assert_eq!(cc(json!({ "organization_ref": org, "code": "X1", "name": "X", "parent_code": "NOPE" })).await.status_code(), 422);
        assert_eq!(cc(json!({ "organization_ref": org, "code": "X1", "name": " " })).await.status_code(), 422);
        let ward = cc(json!({ "organization_ref": org, "code": "WARD4", "name": "Ward 4", "parent_code": "TOP", "valid_from": "2024-01-01" })).await;
        assert_eq!(ward.status_code(), 200, "{}", ward.text());
        let ward_pid = ward.json::<Value>()["pid"].as_str().unwrap().to_string();
        let listed: Value = request.get(&format!("/api/cost-centres?organization={org}")).await.json();
        let w4 = listed["cost_centres"].as_array().unwrap().iter().find(|c| c["code"] == "WARD4").unwrap();
        assert_eq!((w4["open"].as_bool(), w4["chain"].clone()), (Some(true), json!(["WARD4", "TOP"])));
        assert!(w4.get("recorded_by").is_none());

        // An assignment must name a code that is open on its start day.
        let add = |code: &str, from: &str| {
            let request = &request;
            let url = url.clone();
            let body = json!({ "job_title": "Shift", "department": "ward", "fte_percent": 1, "cost_centre": code, "starts_on": from });
            async move { request.post(&url).json(&body).await.status_code() }
        };
        let cut = request.get(&url).await.json::<Value>()["assignments"][0]["pid"].as_str().unwrap().to_string();
        let fte = request.post(&format!("/api/assignments/{cut}/change")).json(&json!({ "fte_percent": 50, "effective_on": "2026-06-01" })).await;
        assert_eq!(fte.status_code(), 200, "{}", fte.text());
        assert_eq!(add("WARD4", "2026-07-01").await, 200);
        assert_eq!(add("NOPE", "2026-07-01").await, 422, "not in the register");
        assert_eq!(add("WARD4", "2023-12-31").await, 422, "not open yet, and before the first day");

        // Closed: a later write is refused, an earlier day is still fine, and it is never deleted.
        let closed = request.post(&format!("/api/cost-centres/{ward_pid}/close")).json(&json!({ "valid_to": "2026-08-31" })).await;
        assert_eq!(closed.status_code(), 200, "{}", closed.text());
        assert_eq!(request.post(&format!("/api/cost-centres/{ward_pid}/close")).json(&json!({ "valid_to": "2026-09-30" })).await.status_code(), 422, "already closed");
        assert_eq!(add("WARD4", "2026-09-01").await, 422);
        assert_eq!(add("WARD4", "2026-08-31").await, 200);
        assert_eq!(request.post(&format!("/api/cost-centres/{top_pid}/close")).json(&json!({ "valid_to": "2019-01-01" })).await.status_code(), 422, "before it opened");

        // A category nobody recorded reads unknown, including the cost centre.
        let rows: Value = request.get(&url).await.json();
        assert!(rows["assignments"].as_array().unwrap().iter().any(|a| a["cost_centre"] == "unknown"));

        // Position cost centres that match nothing are listed, not matched. A position needs its own fields; seed through SQL.
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO positions (pid, organization_ref, cost_centre, title, budget_fte_percent, funding_kind, state)
             VALUES (gen_random_uuid(), '{org}', 'ward-4 (old)', 'Nurse', 100, 'core', 'planned'),
                    (gen_random_uuid(), '{org}', 'ward-4 (old)', 'Nurse', 100, 'core', 'planned'),
                    (gen_random_uuid(), '{org}', 'TOP', 'Chief', 100, 'core', 'planned')"
        )).await.unwrap();
        let un: Value = request.get(&format!("/api/cost-centres/unmatched?organization={org}")).await.json();
        assert_eq!(un["unmatched"], json!([{ "value": "ward-4 (old)", "positions": 2 }]), "{un}");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_absence_categories_are_listed_by_pay_and_a_government_holiday_has_no_balance() {
    crate::requests::request_open(|request, _ctx| async move {
        let all: Value = request.get("/api/absence-categories").await.json();
        let kinds: Vec<&str> = all["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["kind"].as_str().unwrap())
            .collect();
        assert_eq!(
            kinds,
            [
                "annual",
                "public_holiday",
                "sick",
                "parental",
                "unpaid",
                "other"
            ]
        );
        let pick = |pay: &'static str| {
            let request = &request;
            async move {
                let v: Value = request
                    .get(&format!("/api/absence-categories?pay={pay}"))
                    .await
                    .json();
                v["categories"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["kind"].as_str().unwrap().to_string())
                    .collect::<Vec<_>>()
            }
        };
        assert_eq!(pick("paid").await, ["annual", "public_holiday"]);
        assert_eq!(pick("unpaid").await, ["unpaid"]);
        assert_eq!(pick("policy").await, ["sick", "parental", "other"]);
        assert_eq!(
            request
                .get("/api/absence-categories?pay=free")
                .await
                .status_code(),
            422
        );
        let sick = all["categories"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == "sick")
            .unwrap();
        assert_eq!(
            (sick["reason_allowed"].as_bool(), sick["balance"].as_str()),
            (Some(false), Some("may_go_negative"))
        );

        // A government holiday is accepted as a kind of leave request, and takes no balance.
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("AC-{tag}"), None).await;
        activate!(&request, &w).await;
        let day = (Utc::now().date_naive() + Duration::days(30)).to_string();
        let made = request
            .post(&format!("/api/workers/{w}/leave-requests"))
            .json(&json!({ "kind": "public_holiday", "start_on": day, "end_on": day }))
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        // Sick leave still refuses a reason, and nothing here accepts a stress kind.
        let sick_reason = request
            .post(&format!("/api/workers/{w}/leave-requests"))
            .json(&json!({ "kind": "sick", "start_on": day, "end_on": day, "reason": "stress" }))
            .await;
        assert_eq!(sick_reason.status_code(), 422);
        let stress = request
            .post(&format!("/api/workers/{w}/leave-requests"))
            .json(&json!({ "kind": "workplace_stress", "start_on": day, "end_on": day }))
            .await;
        assert_eq!(stress.status_code(), 422, "there is no such category");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn holidays_are_generated_per_region_safely_repeated_and_kept_apart_from_hand_added_days() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let other = an_org();

        // The regions with a pack are listed and none claims to be verified.
        let regions: Value = request.get("/api/holiday-regions").await.json();
        let list = regions["regions"].as_array().unwrap();
        assert_eq!(list.iter().map(|r| r["region"].as_str().unwrap()).collect::<Vec<_>>(), ["GB-ENG", "US", "DE"]);
        assert!(list.iter().all(|r| r["verified"] == false), "{regions}");

        // Generating: a region with no pack, a bad code and a year out of range are refused.
        let generate = |org: &str, region: &str, year: i64| {
            let request = &request;
            let body = json!({ "organization_ref": org, "region": region, "year": year });
            async move { request.post("/api/holidays/generate").json(&body).await }
        };
        assert_eq!(generate(&org, "FR", 2026).await.status_code(), 422, "no pack");
        assert_eq!(generate(&org, "gb eng", 2026).await.status_code(), 422);
        assert_eq!(generate(&org, "GB-ENG", 1900).await.status_code(), 422);

        let first = generate(&org, "GB-ENG", 2026).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let first: Value = first.json();
        assert_eq!((first["created"].as_i64(), first["updated"].as_i64(), first["unchanged"].as_i64()), (Some(8), Some(0), Some(0)), "{first}");
        assert_eq!(first["verified"], false);
        let listed: Value = request.get(&format!("/api/holidays?organization={org}&region=GB-ENG&year=2026")).await.json();
        let days: Vec<&str> = listed["holidays"].as_array().unwrap().iter().map(|h| h["observed_on"].as_str().unwrap()).collect();
        assert_eq!(days, ["2026-01-01", "2026-04-03", "2026-04-06", "2026-05-04", "2026-05-25", "2026-08-31", "2026-12-25", "2026-12-28"]);
        assert!(listed["holidays"][0].get("recorded_by").is_none());

        // Repeating changes nothing; a second region and another organization are separate.
        let again: Value = generate(&org, "GB-ENG", 2026).await.json();
        assert_eq!((again["created"].as_i64(), again["unchanged"].as_i64()), (Some(0), Some(8)), "{again}");
        assert_eq!(generate(&org, "US", 2026).await.json::<Value>()["created"], 11);
        assert_eq!(generate(&other, "GB-ENG", 2026).await.json::<Value>()["created"], 8);
        let only_gb: Value = request.get(&format!("/api/holidays?organization={org}&region=GB-ENG")).await.json();
        assert_eq!(only_gb["holidays"].as_array().map(Vec::len), Some(8), "region filter");

        // A day added by hand is kept, and a generated row that moves is brought up to date.
        let hand = request
            .post("/api/holidays")
            .json(&json!({ "organization_ref": org, "region": "GB-ENG", "name": "Founders' day", "on": "2026-06-12" }))
            .await;
        assert_eq!(hand.status_code(), 200, "{}", hand.text());
        let dup = request
            .post("/api/holidays")
            .json(&json!({ "organization_ref": org, "region": "GB-ENG", "name": "Founders' day", "on": "2026-06-12" }))
            .await;
        assert_eq!(dup.status_code(), 422);
        assert_eq!(request.post("/api/holidays").json(&json!({ "organization_ref": org, "region": "GB-ENG", "name": " ", "on": "2026-06-12" })).await.status_code(), 422);

        // A hand-edited generated row is rewritten, a manual row of the same name and date is not.
        ctx.db.execute_unprepared(&format!("UPDATE holidays SET observed_on = '2026-12-29' WHERE organization_ref = '{org}' AND region = 'GB-ENG' AND name = 'Boxing Day'")).await.unwrap();
        ctx.db.execute_unprepared(&format!("UPDATE holidays SET source = 'manual', observed_on = '2026-01-02' WHERE organization_ref = '{org}' AND region = 'GB-ENG' AND name = 'New Year''s Day'")).await.unwrap();
        let fixed: Value = generate(&org, "GB-ENG", 2026).await.json();
        assert_eq!((fixed["updated"].as_i64(), fixed["kept_manual"].as_i64(), fixed["unchanged"].as_i64()), (Some(1), Some(1), Some(6)), "{fixed}");
        let after: Value = request.get(&format!("/api/holidays?organization={org}&region=GB-ENG&year=2026")).await.json();
        let on = |name: &str| after["holidays"].as_array().unwrap().iter().find(|h| h["name"] == name).unwrap()["observed_on"].as_str().unwrap().to_string();
        assert_eq!(on("Boxing Day"), "2026-12-28", "brought up to date");
        assert_eq!(on("New Year's Day"), "2026-01-02", "a hand-kept row is left alone");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_link_token_works_once_for_its_purpose_and_only_its_hash_is_stored() {
    use chrono::Duration as D;
    use workforce_planning_management::models::link_tokens::{consume, issue, purge_expired};
    crate::requests::request_open(|_request, ctx| async move {
        let now = Utc::now();
        let subject = uuid::Uuid::new_v4();

        // A bad purpose or life is refused; nothing is stored.
        assert!(issue(&ctx.db, "Bad Purpose", None, 30, now).await.is_err());
        assert!(issue(&ctx.db, "confirm", None, 0, now).await.is_err());
        assert!(issue(&ctx.db, "confirm", None, 7 * 24 * 60 + 1, now).await.is_err());

        let token = issue(&ctx.db, "confirm", Some(subject), 30, now).await.unwrap();
        assert_eq!(token.len(), 43);
        let other = issue(&ctx.db, "confirm", None, 30, now).await.unwrap();
        assert_ne!(token, other, "tokens are random");

        // Only the hash is stored: no column can hold a token, and the token text is nowhere in the table.
        let cols = scalar(&ctx, "SELECT count(*) AS n FROM information_schema.columns WHERE table_name = 'link_tokens' AND column_name IN ('token', 'secret', 'value', 'plaintext')".to_string()).await;
        assert_eq!(cols, 0, "there is no column for the token itself");
        let leaked = scalar(&ctx, format!("SELECT count(*) AS n FROM link_tokens WHERE token_hash = '{token}' OR purpose = '{token}'")).await;
        assert_eq!(leaked, 0);
        let hash = workforce_planning_management::rules::link_token::hash(&token);
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM link_tokens WHERE token_hash = '{hash}'")).await, 1);

        // The wrong purpose does not use it up, and does not work.
        assert_eq!(consume(&ctx.db, "reset", &token, now + D::minutes(1)).await.unwrap(), None);
        // Malformed input never reaches the table.
        assert_eq!(consume(&ctx.db, "confirm", "short", now).await.unwrap(), None);
        assert_eq!(consume(&ctx.db, "confirm", &format!("{}!", &token[..42]), now).await.unwrap(), None);

        // The right purpose works exactly once.
        let first = consume(&ctx.db, "confirm", &token, now + D::minutes(2)).await.unwrap();
        assert_eq!(first.map(|c| c.subject_pid), Some(Some(subject)));
        assert_eq!(consume(&ctx.db, "confirm", &token, now + D::minutes(3)).await.unwrap(), None, "used");

        // A used token cannot be made usable again, even by an UPDATE.
        let reuse = ctx.db.execute_unprepared(&format!("UPDATE link_tokens SET used_at = NULL WHERE token_hash = '{hash}'")).await;
        assert!(reuse.is_err(), "used_at is written once");

        // Expiry is exclusive and an expired token is refused.
        let brief = issue(&ctx.db, "confirm", None, 1, now).await.unwrap();
        assert_eq!(consume(&ctx.db, "confirm", &brief, now + D::minutes(1)).await.unwrap(), None, "expired at the minute");
        let live = issue(&ctx.db, "confirm", None, 5, now).await.unwrap();
        assert!(consume(&ctx.db, "confirm", &live, now + D::minutes(4)).await.unwrap().is_some());

        // The audit trail has the reasons and never the token.
        let reasons = |r: &str| scalar(&ctx, format!("SELECT count(*) AS n FROM audit_logs WHERE entity = 'link_token' AND action = 'link_token_refused' AND snapshot->>'reason' = '{r}'"));
        assert!(reasons("wrong_purpose").await >= 1 && reasons("used").await >= 1 && reasons("expired").await >= 1 && reasons("malformed").await >= 2);
        let in_audit = scalar(&ctx, format!("SELECT count(*) AS n FROM audit_logs WHERE snapshot::text LIKE '%{token}%' OR snapshot::text LIKE '%{brief}%'")).await;
        assert_eq!(in_audit, 0, "a token is never written to the audit trail");

        // Old tokens are purged.
        let removed = purge_expired(&ctx.db, now + D::hours(1)).await.unwrap();
        assert!(removed >= 4, "{removed}");
        assert_eq!(scalar(&ctx, "SELECT count(*) AS n FROM link_tokens".to_string()).await, 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_tables_found_by_the_inventory_are_exported_and_erased() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("INV-{tag}"), None).await;
        activate!(&request, &w).await;
        let person: String = ctx
            .db
            .query_one_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT person_ref AS p FROM workers WHERE pid = '{w}'"),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "p")
            .unwrap();
        for sql in [
            format!("INSERT INTO onboarding_items (pid, worker_pid, name, mandatory, status, waived_reason) VALUES (gen_random_uuid(), '{w}', 'Laptop', true, 'waived', 'has their own')"),
            format!("INSERT INTO time_entry_weeks (worker_pid, week_start, minutes) VALUES ('{w}', '2026-10-05', 2250)"),
            format!("INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) VALUES (gen_random_uuid(), '{person}', '{org}', '{w}', 'member', '2026-01-05')"),
            format!("INSERT INTO succession_candidates (pid, plan_pid, worker_pid, readiness, rank) VALUES (gen_random_uuid(), gen_random_uuid(), '{w}', 'ready_now', 1)"),
        ] {
            ctx.db.execute_unprepared(&sql).await.unwrap();
        }
        let count = |table: &'static str| {
            let ctx = &ctx;
            let w = w.clone();
            async move { scalar(ctx, format!("SELECT count(*) AS n FROM {table} WHERE worker_pid = '{w}'")).await }
        };

        // The export carries what the worker is owed and not the succession rating.
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).await.json();
        assert_eq!(export["onboarding_items"][0]["waived_reason"], "has their own", "{export}");
        assert_eq!(export["time_entry_weeks"][0]["minutes"], 2250);
        assert!(export["organization_memberships"].as_array().is_some_and(|m| !m.is_empty()));
        assert!(export.get("succession_candidates").is_none(), "readiness and rank are withheld (WPM-D128)");

        // Erasure deletes the weekly totals, the memberships and the succession entry, and scrubs the waiver reason.
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        let erased = request.post(&format!("/api/workers/{w}/erase")).await;
        erased.assert_status_ok();
        assert_eq!((count("time_entry_weeks").await, count("organization_memberships").await, count("succession_candidates").await), (0, 0, 0));
        let reasons = scalar(&ctx, format!("SELECT count(*) AS n FROM onboarding_items WHERE worker_pid = '{w}' AND waived_reason IS NOT NULL")).await;
        assert_eq!(reasons, 0, "the free-text reason is scrubbed");
        assert_eq!(count("onboarding_items").await, 1, "the checklist item stays; it is the employer's");
    })
    .await;
}
