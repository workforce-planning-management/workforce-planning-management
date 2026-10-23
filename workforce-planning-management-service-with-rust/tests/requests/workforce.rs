//! Workforce flows (WPM-R4–R6): time caps + overtime, the leave
//! balance journey (+ the two-approver race), and shift conflicts.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Time: >24h/day refused; overtime derives beyond the FTE-scaled
// contracted day; approval flips status.
async fn time_caps_and_overtime() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1001", None).await;
        activate!(&request, &worker).await;
        // 20h recorded fine; +5h more the same day breaks the cap.
        let first: Value = request
            .post(&format!("/api/workers/{worker}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-06", "minutes": 1200 }))
            .await
            .json();
        assert!(first["pid"].is_string());
        let over = request
            .post(&format!("/api/workers/{worker}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-06", "minutes": 300 }))
            .await;
        assert_eq!(over.status_code(), 422, "day total over 24h refused");
        // Overtime: 1200 min regular vs 450 contracted ⇒ 750 overtime.
        let listed: Value = request
            .get(&format!("/api/workers/{worker}/time-entries"))
            .await
            .json();
        assert_eq!(listed["overtime"][0]["overtime_minutes"], 750);
        // Approve.
        let entry_pid = first["pid"].as_str().unwrap();
        request
            .post(&format!("/api/time-entries/{entry_pid}/approve"))
            .await
            .assert_status_ok();
        let again = request
            .post(&format!("/api/time-entries/{entry_pid}/approve"))
            .await;
        assert_eq!(again.status_code(), 422, "double approval refused");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Leave: annual over-balance 422; sick goes negative flagged;
// approval decrements the balance in-tx; the second approver loses;
// cancelling an approved request restores the balance.
async fn leave_balance_journey() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1002", None).await;
        activate!(&request, &worker).await;
        request
            .post(&format!("/api/workers/{worker}/leave-entitlements"))
            .json(&json!({ "kind": "annual", "year": 2026, "entitled_days": 5 }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/workers/{worker}/leave-entitlements"))
            .json(&json!({ "kind": "sick", "year": 2026, "entitled_days": 2 }))
            .await
            .assert_status_ok();
        // Annual over balance: 6 days vs 5 ⇒ 422.
        let over = request
            .post(&format!("/api/workers/{worker}/leave-requests"))
            .json(&json!({ "kind": "annual", "start_on": "2026-08-03", "end_on": "2026-08-08" }))
            .await;
        assert_eq!(over.status_code(), 422);
        // Annual within balance.
        let annual: Value = request
            .post(&format!("/api/workers/{worker}/leave-requests"))
            .json(&json!({ "kind": "annual", "start_on": "2026-08-03", "end_on": "2026-08-07" }))
            .await
            .json();
        assert_eq!(annual["days"], 5);
        assert_eq!(annual["negative_balance"], false);
        // Sick beyond balance: allowed but flagged.
        let sick: Value = request
            .post(&format!("/api/workers/{worker}/leave-requests"))
            .json(&json!({ "kind": "sick", "start_on": "2026-09-01", "end_on": "2026-09-04" }))
            .await
            .json();
        assert_eq!(sick["negative_balance"], true);
        // Approve the annual request; balance drops to 0.
        let annual_pid = annual["pid"].as_str().unwrap();
        request
            .post(&format!("/api/leave-requests/{annual_pid}/approve"))
            .await
            .assert_status_ok();
        let balances: Value = request
            .get(&format!("/api/workers/{worker}/leave-entitlements"))
            .await
            .json();
        let annual_balance = balances
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["kind"] == "annual")
            .unwrap();
        assert_eq!(annual_balance["used_days"], 5);
        // The second decision on the same request is refused (the
        // race's loser sees the decided status).
        let second = request
            .post(&format!("/api/leave-requests/{annual_pid}/reject"))
            .await;
        assert_eq!(second.status_code(), 422, "already decided");
        // Cancelling the approved request restores the balance.
        request
            .post(&format!("/api/leave-requests/{annual_pid}/cancel"))
            .await
            .assert_status_ok();
        let balances: Value = request
            .get(&format!("/api/workers/{worker}/leave-entitlements"))
            .await
            .json();
        let annual_balance = balances
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["kind"] == "annual")
            .unwrap();
        assert_eq!(
            annual_balance["used_days"], 0,
            "cancel restores the balance"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Shifts: double-booking refused; assignment over approved leave
// refused; back-to-back shifts are fine.
async fn shift_conflicts() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1003", None).await;
        activate!(&request, &worker).await;
        let early: Value = request
            .post("/api/shifts")
            .json(&json!({
                "department": "engineering",
                "starts_at": "2026-07-06T08:00:00Z", "ends_at": "2026-07-06T16:00:00Z",
            }))
            .await
            .json();
        let overlapping: Value = request
            .post("/api/shifts")
            .json(&json!({
                "department": "engineering",
                "starts_at": "2026-07-06T15:00:00Z", "ends_at": "2026-07-06T23:00:00Z",
            }))
            .await
            .json();
        let late: Value = request
            .post("/api/shifts")
            .json(&json!({
                "department": "engineering",
                "starts_at": "2026-07-06T16:00:00Z", "ends_at": "2026-07-06T23:00:00Z",
            }))
            .await
            .json();
        // Assign the early shift; the overlapping one is refused; the
        // back-to-back one is fine.
        request
            .post(&format!(
                "/api/shifts/{}/assignments",
                early["pid"].as_str().unwrap()
            ))
            .json(&json!({ "worker_pid": worker }))
            .await
            .assert_status_ok();
        let double = request
            .post(&format!(
                "/api/shifts/{}/assignments",
                overlapping["pid"].as_str().unwrap()
            ))
            .json(&json!({ "worker_pid": worker }))
            .await;
        assert_eq!(double.status_code(), 422, "double booking refused");
        request
            .post(&format!(
                "/api/shifts/{}/assignments",
                late["pid"].as_str().unwrap()
            ))
            .json(&json!({ "worker_pid": worker }))
            .await
            .assert_status_ok();
        // Approved leave blocks a same-day assignment.
        request
            .post(&format!("/api/workers/{worker}/leave-entitlements"))
            .json(&json!({ "kind": "annual", "year": 2026, "entitled_days": 10 }))
            .await
            .assert_status_ok();
        let leave: Value = request
            .post(&format!("/api/workers/{worker}/leave-requests"))
            .json(&json!({ "kind": "annual", "start_on": "2026-07-10", "end_on": "2026-07-10" }))
            .await
            .json();
        request
            .post(&format!(
                "/api/leave-requests/{}/approve",
                leave["pid"].as_str().unwrap()
            ))
            .await
            .assert_status_ok();
        let on_leave_shift: Value = request
            .post("/api/shifts")
            .json(&json!({
                "department": "engineering",
                "starts_at": "2026-07-10T08:00:00Z", "ends_at": "2026-07-10T16:00:00Z",
            }))
            .await
            .json();
        let conflicted = request
            .post(&format!(
                "/api/shifts/{}/assignments",
                on_leave_shift["pid"].as_str().unwrap()
            ))
            .json(&json!({ "worker_pid": worker }))
            .await;
        assert_eq!(
            conflicted.status_code(),
            422,
            "assignment over approved leave refused"
        );
    })
    .await;
}

/// Working-time guardrails (WPM-R27): the 17-week 48-hour average over
/// recorded minutes and the 11-hour rest gap flag — advisory only, and
/// scoped by the department filter.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn working_time_guardrails() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let heavy = seed_worker!(&request, &org, "WT-1", None).await;
        activate!(&request, &heavy).await;
        let light = seed_worker!(&request, &org, "WT-2", None).await;
        activate!(&request, &light).await;

        // 35 recorded 24-hour days inside the 17-week window ending
        // 2026-07-10 — 50 400 min, over the 48 960-min ceiling. Unapproved
        // on purpose: a safety signal must not wait for approval.
        let mut day = chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        for _ in 0..35 {
            request
                .post(&format!("/api/workers/{heavy}/time-entries"))
                .json(&json!({ "worked_on": day, "minutes": 1440 }))
                .await
                .assert_status_ok();
            day += chrono::Duration::days(1);
        }
        // The light worker records one modest day.
        request
            .post(&format!("/api/workers/{light}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-01", "minutes": 480 }))
            .await
            .assert_status_ok();

        // A 10-hour turnaround: 14:00-22:00 then 08:00-16:00 next day.
        for (starts, ends) in [
            ("2026-07-05T14:00:00Z", "2026-07-05T22:00:00Z"),
            ("2026-07-06T08:00:00Z", "2026-07-06T16:00:00Z"),
        ] {
            let shift: Value = request
                .post("/api/shifts")
                .json(&json!({ "department": "engineering", "starts_at": starts,
                               "ends_at": ends, "required_headcount": 1 }))
                .await
                .json();
            let shift_pid = shift["pid"].as_str().unwrap();
            request
                .post(&format!("/api/shifts/{shift_pid}/assignments"))
                .json(&json!({ "worker_pid": heavy }))
                .await
                .assert_status_ok();
        }

        // A worker flagged for rest alone still shows their (modest) total; a turnaround more than
        // 28 days from `as_of` is outside the window and flags no one.
        let rest_only = seed_worker!(&request, &org, "WT-3", None).await;
        activate!(&request, &rest_only).await;
        request
            .post(&format!("/api/workers/{rest_only}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-02", "minutes": 600 }))
            .await
            .assert_status_ok();
        let far = seed_worker!(&request, &org, "WT-4", None).await;
        activate!(&request, &far).await;
        for (who, starts, ends) in [
            (&rest_only, "2026-07-12T14:00:00Z", "2026-07-12T22:00:00Z"),
            (&rest_only, "2026-07-13T08:00:00Z", "2026-07-13T16:00:00Z"),
            (&far, "2026-09-05T14:00:00Z", "2026-09-05T22:00:00Z"),
            (&far, "2026-09-06T08:00:00Z", "2026-09-06T16:00:00Z"),
        ] {
            let shift: Value = request
                .post("/api/shifts")
                .json(&json!({ "department": "engineering", "starts_at": starts,
                               "ends_at": ends, "required_headcount": 1 }))
                .await
                .json();
            request
                .post(&format!(
                    "/api/shifts/{}/assignments",
                    shift["pid"].as_str().unwrap()
                ))
                .json(&json!({ "worker_pid": who }))
                .await
                .assert_status_ok();
        }

        let signals: Value = request
            .get("/api/workforce/working-time?as_of=2026-07-10")
            .await
            .json();
        assert!(signals["workers_checked"].as_u64().unwrap() >= 2);
        let flagged = signals["flagged"].as_array().unwrap();
        let row = flagged
            .iter()
            .find(|f| f["worker_pid"] == heavy.as_str())
            .expect("heavy worker flagged");
        assert_eq!(row["over_48h"], true);
        assert_eq!(row["average_weekly"]["numerator_minutes"], 50_400);
        assert_eq!(row["average_weekly"]["denominator_weeks"], 17);
        assert_eq!(row["rest_breaches"].as_array().unwrap().len(), 1);
        assert_eq!(row["rest_breaches"][0]["gap_minutes"], 600);
        assert!(
            !flagged.iter().any(|f| f["worker_pid"] == light.as_str()),
            "a modest week is not flagged"
        );
        let rest = flagged
            .iter()
            .find(|f| f["worker_pid"] == rest_only.as_str())
            .expect("a rest breach alone flags");
        assert_eq!(rest["over_48h"], false);
        assert_eq!(rest["average_weekly"]["numerator_minutes"], 600);
        assert_eq!(rest["rest_breaches"][0]["gap_minutes"], 600);
        assert!(
            !flagged.iter().any(|f| f["worker_pid"] == far.as_str()),
            "a turnaround outside the 28-day window is not flagged"
        );
        // The department filter scopes the check.
        let scoped: Value = request
            .get("/api/workforce/working-time?department=finance&as_of=2026-07-10")
            .await
            .json();
        assert_eq!(scoped["workers_checked"], 0);
        assert!(scoped["flagged"].as_array().unwrap().is_empty());
    })
    .await;
}

/// The flagged row for `who` in the working-time view as of 2026-07-10, if any.
async fn flagged_as_of(request: &loco_rs::TestServer, who: &str) -> Option<Value> {
    let signals: Value = request
        .get("/api/workforce/working-time?as_of=2026-07-10")
        .await
        .json();
    signals["flagged"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["worker_pid"] == who)
        .cloned()
}

/// The weekly rollup behind the working-time view (WPM-D96) and the window it answers over: a total
/// exactly at the ceiling is not flagged; a minute on the day the window starts (excluded) or the day
/// after `as_of` changes nothing; a minute on `as_of` (included) flags; deleting an entry unflags.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn working_time_window_edges_and_rollup_follow_the_entries() {
    use sea_orm::{ConnectionTrait, Statement};
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let who = seed_worker!(&request, &org, "WT-EDGE", None).await;
        activate!(&request, &who).await;
        let record = |day: chrono::NaiveDate, minutes: i32| {
            let request = &request;
            let who = &who;
            async move {
                request
                    .post(&format!("/api/workers/{who}/time-entries"))
                    .json(&json!({ "worked_on": day, "minutes": minutes }))
                    .await
                    .assert_status_ok();
            }
        };
        // as_of 2026-07-10 → the window is the 119 days 2026-03-14 ..= 2026-07-10.
        let first = chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap();
        for n in 0..34 {
            record(first + chrono::Duration::days(n), 1440).await; // 48 960 = the ceiling
        }
        record(first - chrono::Duration::days(1), 5).await; // the day the window starts: excluded
        record(chrono::NaiveDate::from_ymd_opt(2026, 7, 11).unwrap(), 5).await; // after as_of: excluded
        assert!(flagged_as_of(&request, &who).await.is_none(), "exactly the ceiling is not over it");
        record(chrono::NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(), 1).await;
        let row = flagged_as_of(&request, &who).await.expect("one more minute on as_of flags");
        assert_eq!(row["average_weekly"]["numerator_minutes"], 48_961);

        // The rollup equals the entries after an update, a soft delete, a restore and a hard delete.
        let rollup_matches = || async {
            let rows = ctx
                .db
                .query_all_raw(Statement::from_string(
                    sea_orm::DbBackend::Postgres,
                    "SELECT (SELECT COALESCE(SUM(minutes), 0) FROM time_entry_weeks)::bigint AS rolled,
                            (SELECT COALESCE(SUM(minutes), 0) FROM time_entries WHERE deleted_at IS NULL)::bigint AS entered,
                            (SELECT COUNT(*) FROM (
                                 SELECT worker_pid, date_trunc('week', worked_on)::date AS w, SUM(minutes) AS m
                                   FROM time_entries WHERE deleted_at IS NULL GROUP BY 1, 2
                                 EXCEPT SELECT worker_pid, week_start, minutes FROM time_entry_weeks) x)::bigint AS missing,
                            (SELECT COUNT(*) FROM (
                                 SELECT worker_pid, week_start, minutes FROM time_entry_weeks
                                 EXCEPT SELECT worker_pid, date_trunc('week', worked_on)::date, SUM(minutes)
                                   FROM time_entries WHERE deleted_at IS NULL GROUP BY 1, 2) y)::bigint AS extra",
                ))
                .await
                .unwrap();
            let get = |c: &str| rows[0].try_get::<i64>("", c).unwrap();
            assert_eq!((get("missing"), get("extra")), (0, 0), "rollup rows equal the entries");
            assert_eq!(get("rolled"), get("entered"));
        };
        rollup_matches().await;
        let sql = |s: String| {
            let db = ctx.db.clone();
            async move { db.execute_unprepared(&s).await.unwrap() }
        };
        let pid = who.clone();
        sql(format!("UPDATE time_entries SET minutes = minutes + 60, worked_on = worked_on + 9 WHERE worker_pid = '{pid}' AND minutes = 1")).await;
        rollup_matches().await;
        sql(format!("UPDATE time_entries SET deleted_at = now() WHERE worker_pid = '{pid}' AND minutes = 61")).await;
        rollup_matches().await;
        assert!(flagged_as_of(&request, &who).await.is_none(), "with the extra entry deleted the total is the ceiling again");
        sql(format!("UPDATE time_entries SET deleted_at = NULL WHERE worker_pid = '{pid}' AND minutes = 61")).await;
        rollup_matches().await;
        sql(format!("DELETE FROM time_entries WHERE worker_pid = '{pid}' AND minutes = 5")).await;
        rollup_matches().await;
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn an_unassigned_shift_seat_and_an_ended_benefit_enrolment_leave_their_lists() {
    use serde_json::{Value, json};

    crate::requests::request_open(|request, _ctx| async move {
        let org = super::an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = super::seed_worker!(&request, &org, format!("UN-{tag}"), None).await;
        super::activate!(&request, &worker).await;

        // A shift seat, then unassigned: a second delete is a 404.
        let day = chrono::Utc::now().date_naive() + chrono::Duration::days(30);
        let starts = day.and_hms_opt(9, 0, 0).unwrap().and_utc().to_rfc3339();
        let ends = day.and_hms_opt(17, 0, 0).unwrap().and_utc().to_rfc3339();
        let shift: Value = request
            .post("/api/shifts")
            .json(&json!({ "department": "engineering", "starts_at": starts, "ends_at": ends, "required_headcount": 1 }))
            .await
            .json();
        let shift = shift["pid"].as_str().unwrap().to_string();
        let seat: Value = request
            .post(&format!("/api/shifts/{shift}/assignments"))
            .json(&json!({ "worker_pid": worker }))
            .await
            .json();
        let seat = seat["pid"].as_str().unwrap().to_string();
        request.delete(&format!("/api/shift-assignments/{seat}")).await.assert_status_ok();
        assert_eq!(request.delete(&format!("/api/shift-assignments/{seat}")).await.status_code(), 404);

        // A benefit enrolment, then ended: it leaves the worker's list.
        let plan: Value = request
            .post("/api/benefit-plans")
            .json(&json!({ "name": format!("Plan {tag}"), "kind": "pension", "provider": "Demo Provider",
                           "worker_cost_minor": 100, "employer_cost_minor": 200, "currency": "GBP" }))
            .await
            .json();
        let enrolled: Value = request
            .post(&format!("/api/workers/{worker}/benefit-enrollments"))
            .json(&json!({ "plan_pid": plan["pid"], "starts_on": "2026-01-01" }))
            .await
            .json();
        let enrolment = enrolled["pid"].as_str().unwrap().to_string();
        let listed: Vec<Value> = request.get(&format!("/api/workers/{worker}/benefit-enrollments")).await.json();
        assert!(listed.iter().any(|e| e["pid"] == enrolment.as_str()));
        request.delete(&format!("/api/benefit-enrollments/{enrolment}")).await.assert_status_ok();
        let listed: Vec<Value> = request.get(&format!("/api/workers/{worker}/benefit-enrollments")).await.json();
        assert!(!listed.iter().any(|e| e["pid"] == enrolment.as_str()));
        assert_eq!(request.delete(&format!("/api/benefit-enrollments/{enrolment}")).await.status_code(), 404);
    })
    .await;
}
