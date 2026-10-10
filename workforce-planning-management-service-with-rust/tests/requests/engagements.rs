//! Engagements (WPM-R79, WPM-R80): the end date by basis, extensions with their history,
//! a contractor's details, and the employment-status assessment.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, activate, an_org};

async fn create(
    request: &loco_rs::TestServer,
    org: &str,
    number: &str,
    basis: &str,
    ends_on: Option<&str>,
) -> (u16, Value) {
    let mut body = json!({
        "person_ref": a_person(), "organization_ref": org, "worker_number": number,
        "display_name": format!("Test Worker {number}"), "employment_type": basis,
        "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
    });
    if let Some(end) = ends_on {
        body["engagement_ends_on"] = json!(end);
    }
    let response = request.post("/api/workers").json(&body).await;
    let status = response.status_code().as_u16();
    let value = if status == 200 {
        response.json()
    } else {
        Value::Null
    };
    (status, value)
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Permanent refuses an end date, fixed-term and contractor need one, an intern may have one;
// an end must be after the start; a recorded end moves only by extension; the same rules
// apply on hire.
async fn the_end_date_follows_the_basis() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let n = |s: &str| format!("EN-{tag}-{s}");
        for (basis, end, expect) in [
            ("permanent", None, 200),
            ("permanent", Some("2026-12-31"), 422),
            ("fixed_term", None, 422),
            ("fixed_term", Some("2026-12-31"), 200),
            ("contractor", None, 422),
            ("contractor", Some("2026-12-31"), 200),
            ("intern", None, 200),
            ("intern", Some("2026-08-31"), 200),
            ("fixed_term", Some("2026-01-05"), 422),
            ("fixed_term", Some("2026-01-04"), 422),
            ("gig", Some("2026-12-31"), 422),
        ] {
            let response = create(
                &request,
                &org,
                &n(&format!(
                    "{basis}-{}-{expect}-{}",
                    end.unwrap_or("none"),
                    uuid::Uuid::new_v4().simple()
                )),
                basis,
                end,
            )
            .await;
            assert_eq!(response.0, expect, "{basis} {end:?}");
        }
        let permanent: Value = create(&request, &org, &n("perm"), "permanent", None)
            .await
            .1;
        let fixed: Value = create(
            &request,
            &org,
            &n("fixed"),
            "fixed_term",
            Some("2026-12-31"),
        )
        .await
        .1;
        let permanent = permanent["pid"].as_str().unwrap().to_string();
        let fixed = fixed["pid"].as_str().unwrap().to_string();

        // The end date is shown, with where it stands.
        let shown: Value = request
            .get(&format!("/api/workers/{fixed}/engagement"))
            .await
            .json();
        assert_eq!(shown["engagement_ends_on"], "2026-12-31");
        assert_eq!(shown["extension_count"], 0);
        assert_eq!(shown["window_calendar_days"], 60);

        // A permanent worker cannot be given one; a set one cannot be overwritten by an update.
        assert_eq!(
            request
                .put(&format!("/api/workers/{permanent}"))
                .json(&json!({ "engagement_ends_on": "2027-01-01" }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .put(&format!("/api/workers/{fixed}"))
                .json(&json!({ "engagement_ends_on": "2027-06-30" }))
                .await
                .status_code(),
            422,
            "moves only by extension, so the history is kept"
        );

        // Extensions: later only, with history; none for permanent; none without a date.
        let extend = format!("/api/workers/{fixed}/engagement/extensions");
        for bad in ["2026-12-31", "2026-12-30"] {
            assert_eq!(
                request
                    .post(&extend)
                    .json(&json!({ "new_end": bad }))
                    .await
                    .status_code(),
                422,
                "{bad}"
            );
        }
        request
            .post(&extend)
            .json(&json!({ "new_end": "2027-03-31", "reason": "Project extended" }))
            .await
            .assert_status_ok();
        request
            .post(&extend)
            .json(&json!({ "new_end": "2027-06-30" }))
            .await
            .assert_status_ok();
        let shown: Value = request
            .get(&format!("/api/workers/{fixed}/engagement"))
            .await
            .json();
        assert_eq!(shown["engagement_ends_on"], "2027-06-30");
        assert_eq!(shown["extension_count"], 2);
        assert_eq!(shown["extensions"][0]["previous_end"], "2026-12-31");
        assert_eq!(shown["extensions"][0]["new_end"], "2027-03-31");
        assert_eq!(shown["extensions"][1]["previous_end"], "2027-03-31");
        assert_eq!(
            request
                .post(&format!("/api/workers/{permanent}/engagement/extensions"))
                .json(&json!({ "new_end": "2027-06-30" }))
                .await
                .status_code(),
            422,
            "a permanent engagement has no end to extend"
        );
        // The audit trail names the event and not the reason.
        let audits: Value = request.get(&format!("/api/audits/{fixed}")).await.json();
        let text = serde_json::to_string(&audits).unwrap();
        assert!(text.contains("engagement_extended"));
        assert!(!text.contains("Project extended"));

        // Hire: the same rule.
        let requisition: Value = request
            .post("/api/requisitions")
            .json(
                &json!({ "organization_ref": org, "department": "engineering",
                           "job_title": "Contract Engineer", "headcount": 1 }),
            )
            .await
            .json();
        let req = requisition["pid"].as_str().unwrap().to_string();
        for to in ["open", "interviewing"] {
            request
                .post(&format!("/api/requisitions/{req}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        let candidate: Value = request
            .post("/api/candidates")
            .json(
                &json!({ "display_name": "Test Applicant 002", "email": "a2@example.com",
                           "source": "referral", "person_ref": a_person() }),
            )
            .await
            .json();
        let application: Value = request
            .post(&format!("/api/requisitions/{req}/applications"))
            .json(&json!({ "candidate_pid": candidate["pid"] }))
            .await
            .json();
        let app = application["pid"].as_str().unwrap().to_string();
        for to in ["screened", "interviewing", "offer"] {
            request
                .post(&format!("/api/applications/{app}/stage"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        let stage = format!("/api/applications/{app}/stage");
        let without = request
            .post(&stage)
            .json(
                &json!({ "to": "hired", "worker_number": format!("EN-{tag}-H"),
                           "employment_type": "contractor" }),
            )
            .await;
        assert_eq!(
            without.status_code(),
            422,
            "a contractor hire needs an end date"
        );
        let hired: Value = request
            .post(&stage)
            .json(
                &json!({ "to": "hired", "worker_number": format!("EN-{tag}-H"),
                           "employment_type": "contractor", "hired_on": "2026-02-02",
                           "engagement_ends_on": "2026-08-31" }),
            )
            .await
            .json();
        let worker = hired["worker_pid"].as_str().expect("hired").to_string();
        let shown: Value = request
            .get(&format!("/api/workers/{worker}/engagement"))
            .await
            .json();
        assert_eq!(shown["engagement_ends_on"], "2026-08-31");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// A worker already in the table without an end date (a row from before the column) is listed
// as missing and never given an invented date; recording one removes it from the list.
async fn existing_engagements_without_an_end_date_are_listed_not_invented() {
    crate::requests::request_open(|request, ctx| async move {
        use sea_orm::ConnectionTrait;
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let made: Value = create(
            &request,
            &org,
            &format!("EM-{tag}"),
            "fixed_term",
            Some("2026-12-31"),
        )
        .await
        .1;
        let pid = made["pid"].as_str().unwrap().to_string();
        // Simulate a row from before the column existed.
        ctx.db
            .execute_unprepared(&format!(
                "UPDATE workers SET engagement_ends_on = NULL WHERE pid = '{pid}'"
            ))
            .await
            .unwrap();
        let permanent: Value = create(&request, &org, &format!("EP-{tag}"), "permanent", None)
            .await
            .1;
        let listed: Vec<Value> = request
            .get(&format!(
                "/api/engagements/missing-end-date?organization={org}"
            ))
            .await
            .json();
        assert_eq!(listed.len(), 1, "only the contingent worker with no date");
        assert_eq!(listed[0]["worker_pid"], pid.as_str());
        assert_ne!(listed[0]["worker_pid"], permanent["pid"]);
        let shown: Value = request
            .get(&format!("/api/workers/{pid}/engagement"))
            .await
            .json();
        assert_eq!(shown["engagement_ends_on"], Value::Null, "never invented");
        assert_eq!(shown["standing"], "missing_end_date");
        // HR records one: it leaves the list.
        request
            .put(&format!("/api/workers/{pid}"))
            .json(&json!({ "engagement_ends_on": "2027-01-31" }))
            .await
            .assert_status_ok();
        let listed: Vec<Value> = request
            .get(&format!(
                "/api/engagements/missing-end-date?organization={org}"
            ))
            .await
            .json();
        assert!(listed.is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// A contractor's supplier, route and rate; an assessment; the audit carries no rate; the
// subject-access export includes them and erasure removes them.
async fn contractor_details_and_assessments() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let contractor: Value = create(
            &request,
            &org,
            &format!("EC-{tag}"),
            "contractor",
            Some("2026-12-31"),
        )
        .await
        .1;
        let worker = contractor["pid"].as_str().unwrap().to_string();
        let permanent: Value = create(&request, &org, &format!("EQ-{tag}"), "permanent", None)
            .await
            .1;
        let perm = permanent["pid"].as_str().unwrap().to_string();
        let url = format!("/api/workers/{worker}/contractor-details");

        // None yet.
        let none: Value = request.get(&url).await.json();
        assert_eq!(none["contractor_details"], Value::Null);
        // Only a contractor has them.
        assert_eq!(
            request
                .put(&format!("/api/workers/{perm}/contractor-details"))
                .json(&json!({ "route": "agency" }))
                .await
                .status_code(),
            422
        );
        let supplier = an_org();
        let good = json!({ "supplier_ref": supplier, "route": "agency",
                           "rate_minor": 45_000, "rate_currency": "GBP", "rate_basis": "day" });
        for (field, value) in [
            ("supplier_ref", json!(a_person())),
            ("route", json!("freelance")),
            ("rate_minor", json!(-1)),
            ("rate_currency", json!("gbp")),
            ("rate_basis", json!("week")),
        ] {
            let mut bad = good.clone();
            bad[field] = value.clone();
            assert_eq!(
                request.put(&url).json(&bad).await.status_code(),
                422,
                "{field} = {value}"
            );
        }
        // A partial rate is refused.
        assert_eq!(
            request
                .put(&url)
                .json(&json!({ "rate_minor": 100 }))
                .await
                .status_code(),
            422
        );
        let set: Value = request.put(&url).json(&good).await.json();
        assert_eq!(
            set["contractor_details"]["rate_minor"],
            Value::Null,
            "a write never echoes the rate"
        );
        assert_eq!(set["contractor_details"]["rate_masked"], true);
        let shown: Value = request.get(&url).await.json();
        assert_eq!(shown["contractor_details"]["rate_minor"], 45_000);
        assert_eq!(shown["contractor_details"]["rate_basis"], "day");
        assert_eq!(
            shown["contractor_details"]["supplier_ref"],
            supplier.as_str()
        );

        // Assessments: a known outcome, not in the future, contractor only; newest first.
        let assess = format!("/api/workers/{worker}/engagement/status-assessments");
        assert_eq!(
            request
                .post(&assess)
                .json(&json!({ "outcome": "maybe" }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&assess)
                .json(&json!({ "outcome": "contractor", "assessed_on": "2999-01-01" }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&format!(
                    "/api/workers/{perm}/engagement/status-assessments"
                ))
                .json(&json!({ "outcome": "contractor" }))
                .await
                .status_code(),
            422
        );
        request
            .post(&assess)
            .json(&json!({ "outcome": "undetermined", "assessed_on": "2026-02-01" }))
            .await
            .assert_status_ok();
        request
            .post(&assess)
            .json(&json!({ "outcome": "contractor", "assessed_on": "2026-03-01" }))
            .await
            .assert_status_ok();
        let listed: Vec<Value> = request.get(&assess).await.json();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0]["outcome"], "contractor");

        // No audit entry carries the rate, the supplier or the outcome.
        let audits: Value = request.get(&format!("/api/audits/{worker}")).await.json();
        let text = serde_json::to_string(&audits).unwrap();
        assert!(text.contains("contractor_details_set") && text.contains("status_assessed"));
        assert!(!text.contains("45000") && !text.contains(&supplier));

        // Subject access includes all of it.
        let export: Value = request
            .get(&format!("/api/workers/{worker}/subject-access"))
            .await
            .json();
        assert_eq!(export["contractor_details"][0]["rate_minor"], 45_000);
        assert_eq!(
            export["employment_status_assessments"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(export["worker"]["engagement_ends_on"], "2026-12-31");

        // Clearing works once, then it is a 404.
        request.delete(&url).await.assert_status_ok();
        assert_eq!(request.delete(&url).await.status_code(), 404);
        request.put(&url).json(&good).await.assert_status_ok();

        // Erasure removes the details, the assessments and the extensions. It needs ended
        // employment, so end it first.
        request
            .post(&format!("/api/workers/{worker}/engagement/extensions"))
            .json(&json!({ "new_end": "2027-03-31" }))
            .await
            .assert_status_ok();
        activate!(&request, &worker).await;
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        request
            .post(&format!("/api/workers/{worker}/erase"))
            .await
            .assert_status_ok();
        // The worker is gone from reads; the rows about them are gone from the tables.
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
        use workforce_planning_management_service::models::_entities::{
            engagement_extensions, engagement_status_assessments, worker_contractor_details,
        };
        let pid: uuid::Uuid = worker.parse().unwrap();
        assert_eq!(
            worker_contractor_details::Entity::find()
                .filter(worker_contractor_details::Column::WorkerPid.eq(pid))
                .count(&ctx.db)
                .await
                .unwrap(),
            0,
            "details erased with the person"
        );
        assert_eq!(
            engagement_status_assessments::Entity::find()
                .filter(engagement_status_assessments::Column::WorkerPid.eq(pid))
                .count(&ctx.db)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            engagement_extensions::Entity::find()
                .filter(engagement_extensions::Column::WorkerPid.eq(pid))
                .count(&ctx.db)
                .await
                .unwrap(),
            0
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
// The daily task tells the manager and HR once about each end date, including an engagement
// already past its end with no decision; it names no rate; a decision settles the end date it was
// made for, and an extension gives a new date and so a new question.
async fn end_of_engagement_reminders_and_decisions() {
    use sea_orm::ConnectionTrait;
    use workforce_planning_management_service::tasks::engagement_end_reminders::send_reminders;
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let make = |number: String, basis: &'static str, ends: Option<String>, manager: Option<String>| {
            let (request, org) = (&request, org.clone());
            async move {
                let mut body = json!({
                    "person_ref": a_person(), "organization_ref": org, "worker_number": number,
                    "display_name": format!("Test Worker {number}"), "employment_type": basis,
                    "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
                });
                if let Some(e) = ends { body["engagement_ends_on"] = json!(e); }
                if let Some(m) = manager { body["manager_pid"] = json!(m); }
                let r = request.post("/api/workers").json(&body).await;
                assert_eq!(r.status_code(), 200);
                r.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let manager = make(format!("RM-{tag}"), "permanent", None, None).await;
        let hr = make(format!("RH-{tag}"), "permanent", None, None).await;
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
             VALUES ('{}', 'person:{}', '{org}', '{hr}', 'hr_admin', '2023-01-05')", uuid::Uuid::new_v4(), uuid::Uuid::new_v4())).await.unwrap();
        let ending = make(format!("RA-{tag}"), "contractor", Some(day(30)), Some(manager.clone())).await;
        let _later = make(format!("RB-{tag}"), "fixed_term", Some(day(90)), Some(manager.clone())).await;
        let past = make(format!("RC-{tag}"), "contractor", Some(day(-1)), Some(manager.clone())).await;
        let settled = make(format!("RD-{tag}"), "contractor", Some(day(-1)), Some(manager.clone())).await;
        let _permanent = make(format!("RE-{tag}"), "permanent", None, Some(manager.clone())).await;
        // A rate and a supplier exist for the one ending soon; they must not reach a notification.
        request.put(&format!("/api/workers/{ending}/contractor-details"))
            .json(&json!({ "supplier_ref": an_org(), "route": "agency", "rate_minor": 45_000, "rate_currency": "GBP", "rate_basis": "day" }))
            .await.assert_status_ok();
        // A decision recorded first settles the engagement that already ended.
        let decided = request.post(&format!("/api/workers/{settled}/engagement/decision")).json(&json!({ "decision": "end" })).await;
        assert_eq!(decided.status_code(), 200);

        // First run: one ending, one ended with no decision; the manager and HR are told of each.
        let first = send_reminders(&ctx, today, 60).await.unwrap();
        assert_eq!((first.ending, first.ended_undecided, first.told), (1, 1, 4));
        let notes = |pid: &str| {
            let request = &request;
            let pid = pid.to_string();
            async move { request.get(&format!("/api/workers/{pid}/notifications")).await.json::<Value>() }
        };
        for (who, pid) in [("manager", &manager), ("HR", &hr)] {
            let got = notes(pid).await;
            let text = serde_json::to_string(&got).unwrap();
            assert!(text.contains("engagement_ending") && text.contains("engagement_ended_undecided"), "{who}: {text}");
            assert!(text.contains(&format!("RA-{tag}")) || text.contains("Test Worker"), "{who} names the worker");
            for secret in ["45000", "45_000", "agency", "rate", "supplier"] {
                assert!(!text.contains(secret), "{who}'s notification must not mention `{secret}`: {text}");
            }
            assert!(text.contains(&day(30)) && text.contains(&day(-1)));
        }
        // Not the one ending outside the window, the permanent one, or the decided one.
        let mgr_text = serde_json::to_string(&notes(&manager).await).unwrap();
        assert!(!mgr_text.contains(&day(90)), "outside the window");
        assert_eq!(mgr_text.matches("engagement_ending").count(), 1);
        assert_eq!(mgr_text.matches("engagement_ended_undecided").count(), 1, "the decided one is not asked about");

        // Idempotent: a re-run tells no one twice.
        assert_eq!(send_reminders(&ctx, today, 60).await.unwrap(), Default::default());
        // The window moves: the one 90 calendar days out is now inside it, once.
        let later = send_reminders(&ctx, today + chrono::Duration::days(40), 60).await.unwrap();
        // The 90-day one is now ending; and the first contractor's end date (30 days out) has itself
        // passed with no decision, which is a different reminder for the same worker.
        assert_eq!((later.ending, later.ended_undecided), (1, 1), "the 90-day one is ending; the 30-day one has ended");

        // An extension gives a new end date and so a new reminder, once it is inside the window.
        request.post(&format!("/api/workers/{ending}/engagement/extensions")).json(&json!({ "new_end": day(130) })).await.assert_status_ok();
        assert_eq!(send_reminders(&ctx, today + chrono::Duration::days(41), 60).await.unwrap().ending, 0, "130 days out is still beyond 60");
        let again = send_reminders(&ctx, today + chrono::Duration::days(75), 60).await.unwrap();
        assert_eq!(again.ending, 1, "the new end date is a new question");

        // Decisions: validated, against the end date they settle, and the standing follows.
        let url = |pid: &str| format!("/api/workers/{pid}/engagement/decision");
        assert_eq!(request.post(&url(&past)).json(&json!({ "decision": "renew" })).await.status_code(), 422);
        assert_eq!(request.post(&url(&_permanent)).json(&json!({ "decision": "end" })).await.status_code(), 422, "a permanent engagement has no end to decide about");
        let before: Value = request.get(&format!("/api/workers/{past}/engagement")).await.json();
        assert_eq!(before["standing"], "past_end_undecided", "never treated as continuing");
        assert_eq!(before["decision"], Value::Null);
        request.post(&url(&past)).json(&json!({ "decision": "convert" })).await.assert_status_ok();
        let after: Value = request.get(&format!("/api/workers/{past}/engagement")).await.json();
        assert_eq!((after["standing"].as_str(), after["decision"].as_str()), (Some("past_end_decided"), Some("convert")));
        // A decision belongs to its end date: after an extension it no longer settles anything.
        request.post(&url(&ending)).json(&json!({ "decision": "extend" })).await.assert_status_ok();
        let settled_view: Value = request.get(&format!("/api/workers/{ending}/engagement")).await.json();
        assert_eq!(settled_view["decision"], "extend");
        request.post(&format!("/api/workers/{ending}/engagement/extensions")).json(&json!({ "new_end": day(200) })).await.assert_status_ok();
        let renewed: Value = request.get(&format!("/api/workers/{ending}/engagement")).await.json();
        assert_eq!(renewed["decision"], Value::Null, "a new end date is a new question");
        assert_eq!(renewed["decisions"].as_array().unwrap().len(), 1, "the history is kept");
        // The audit names the event, not the decision.
        let audits = serde_json::to_string(&request.get(&format!("/api/audits/{past}")).await.json::<Value>()).unwrap();
        assert!(audits.contains("engagement_decision_recorded") && !audits.contains("convert"));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
// A conversion plan (WPM-R98): its refusals, its flags, the reminder that names its status and
// review date but never its reason, the one-open-plan rule, the export and the erasure.
async fn a_conversion_plan_is_kept_flagged_reminded_exported_and_erased() {
    use workforce_planning_management_service::tasks::engagement_end_reminders::send_reminders;
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let make = |number: String, basis: &'static str, ends: Option<String>, manager: Option<String>| {
            let (request, org) = (&request, org.clone());
            async move {
                let mut body = json!({
                    "person_ref": a_person(), "organization_ref": org, "worker_number": number,
                    "display_name": format!("Test Worker {number}"), "employment_type": basis,
                    "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
                });
                if let Some(e) = ends { body["engagement_ends_on"] = json!(e); }
                if let Some(m) = manager { body["manager_pid"] = json!(m); }
                let r = request.post("/api/workers").json(&body).await;
                assert_eq!(r.status_code(), 200);
                r.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let manager = make(format!("CM-{tag}"), "permanent", None, None).await;
        let w = make(format!("CW-{tag}"), "contractor", Some(day(30)), Some(manager.clone())).await;
        let permanent = make(format!("CP-{tag}"), "permanent", None, None).await;
        let plans = format!("/api/workers/{w}/conversion-plans");
        let propose = |body: Value| {
            let (request, plans) = (&request, plans.clone());
            async move { request.post(&plans).json(&body).await }
        };

        // Refusals, each with the reason it is refused.
        assert_eq!(propose(json!({ "intent": "convert", "target_on": day(10) })).await.status_code(), 422, "no post funding and no reason");
        assert_eq!(propose(json!({ "intent": "convert", "target_on": day(30), "post_funding_kind": "core" })).await.status_code(), 422, "the decision date falls before the end");
        assert_eq!(propose(json!({ "intent": "convert", "target_on": day(-1), "post_funding_kind": "core" })).await.status_code(), 422, "in the past");
        assert_eq!(propose(json!({ "intent": "convert", "target_on": day(10), "post_funding_kind": "lottery" })).await.status_code(), 422);
        assert_eq!(propose(json!({ "intent": "convert", "target_on": day(10), "post_funding_kind": "core", "reason": "x".repeat(501) })).await.status_code(), 422);
        let on_permanent = request.post(&format!("/api/workers/{permanent}/conversion-plans")).json(&json!({ "intent": "end", "target_on": day(10) })).await;
        assert_eq!(on_permanent.status_code(), 422, "a permanent worker has nothing to convert");

        // Before any plan the contract is flagged only once it has ended.
        let none: Vec<Value> = request.get(&plans).await.json();
        assert!(none.is_empty());

        // A plan with no funding, but a stated reason, is accepted; one open plan at a time.
        let ok = propose(json!({ "intent": "convert", "target_on": day(10), "reason": "Funding bid pending", "review_on": day(5) })).await;
        assert_eq!(ok.status_code(), 200);
        let pid = ok.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(propose(json!({ "intent": "extend", "target_on": day(10) })).await.status_code(), 422);

        // The reminder names the plan's status and review date and never its reason.
        let report = send_reminders(&ctx, today, 60).await.unwrap();
        assert_eq!(report.ending, 1);
        let notes = request.get(&format!("/api/workers/{manager}/notifications")).await.text();
        assert!(notes.contains("A conversion plan is proposed, to be reviewed on"), "{notes}");
        assert!(notes.contains(&day(5)));
        assert!(!notes.contains("Funding bid pending"), "the reason stays in the plan");

        // In the list of open plans.
        let listed: Vec<Value> = request.get("/api/conversion-plans").await.json();
        let mine = listed.iter().find(|e| e["worker_pid"] == w.as_str()).expect("listed");
        assert_eq!(mine["plan"]["status"], "proposed");

        // Abandoning keeps the plan as history and frees the slot.
        assert_eq!(request.post(&format!("/api/conversion-plans/{pid}/done")).await.status_code(), 422, "not approved yet");
        request.post(&format!("/api/conversion-plans/{pid}/abandon")).await.assert_status_ok();
        assert_eq!(request.post(&format!("/api/conversion-plans/{pid}/approve")).await.status_code(), 422, "abandoned");
        let second = propose(json!({ "intent": "extend", "target_on": day(10) })).await;
        assert_eq!(second.status_code(), 200);
        let history: Vec<Value> = request.get(&plans).await.json();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0]["status"], "abandoned");

        // An undecided plan cannot be approved; an approved extension settles the end date.
        let second_pid = second.json::<Value>()["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/conversion-plans/{second_pid}/approve")).await.assert_status_ok();
        let engagement: Value = request.get(&format!("/api/workers/{w}/engagement")).await.json();
        assert_eq!(engagement["decision"], "extend");
        // Reminders no longer ask about an end date a person has decided.
        assert_eq!(send_reminders(&ctx, today + chrono::Duration::days(1), 60).await.unwrap().ending, 0);
        // Only a plan to convert is carried out.
        assert_eq!(request.post(&format!("/api/conversion-plans/{second_pid}/done")).await.status_code(), 422);

        // The export names the plans; erasing the worker removes them.
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).await.json();
        assert_eq!(export["conversion_plans"].as_array().unwrap().len(), 2);
        for to in ["active", "offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        request.post(&format!("/api/workers/{w}/erase")).await.assert_status_ok();
        use sea_orm::{ConnectionTrait, Statement};
        let left = ctx.db.query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT count(*) AS n FROM conversion_plans WHERE worker_pid = '{w}'"),
        )).await.unwrap().unwrap().try_get::<i64>("", "n").unwrap();
        assert_eq!(left, 0, "erased with the worker");
    })
    .await;
}
