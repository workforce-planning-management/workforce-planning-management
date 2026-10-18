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
        use workforce_planning_management::models::_entities::{
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
    use workforce_planning_management::tasks::engagement_end_reminders::send_reminders;
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
    use workforce_planning_management::tasks::engagement_end_reminders::send_reminders;
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

        // The database itself refuses a second open plan, whatever the handler does.
        let duplicate = ctx.db.execute_unprepared(&format!(
            "INSERT INTO conversion_plans (pid, worker_pid, intent, target_on, status) \
             VALUES ('{}', '{w}', 'end', '{}', 'proposed')", uuid::Uuid::new_v4(), day(5))).await;
        assert!(duplicate.is_err(), "one open plan per worker is a database rule");

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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Post funding (WPM-R62, WPM-R98): kind and date only, compared with the contract end, an unknown
// never a match, in the export, erased with the worker.
async fn post_funding_is_compared_with_the_contract_end_exported_and_erased() {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let r = request.post("/api/workers").json(&json!({
            "person_ref": a_person(), "organization_ref": an_org(), "worker_number": format!("PF-{tag}"),
            "display_name": "Test Worker PF", "employment_type": "fixed_term", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2026-01-05", "engagement_ends_on": day(100),
        })).await;
        let w = r.json::<Value>()["pid"].as_str().unwrap().to_string();
        let url = format!("/api/workers/{w}/funding");
        let none: Value = request.get(&url).await.json();
        assert_eq!((none["funding"].clone(), none["alignment"].as_str()), (Value::Null, Some("unknown")));
        for (why, body) in [
            ("unknown kind", json!({ "kind": "lottery" })),
            ("time-limited needs an end", json!({ "kind": "time_limited" })),
            ("ends before it started", json!({ "kind": "core", "ends_on": "2025-12-31" })),
            ("programme is a thing URN", json!({ "kind": "programme", "programme_ref": "not-a-urn" })),
        ] {
            assert_eq!(request.put(&url).json(&body).await.status_code(), 422, "{why}");
        }
        for (body, expect, attention) in [
            (json!({ "kind": "time_limited", "ends_on": day(100) }), "matched", false),
            (json!({ "kind": "time_limited", "ends_on": day(40) }), "contract_outlasts_funding", true),
            (json!({ "kind": "external", "ends_on": day(200) }), "funding_outlasts_contract", true),
            (json!({ "kind": "core" }), "funding_open_ended", false),
        ] {
            let got: Value = request.put(&url).json(&body).await.json();
            assert_eq!((got["alignment"].as_str(), got["needs_attention"].as_bool()), (Some(expect), Some(attention)), "{body}");
        }
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).await.json();
        assert_eq!(export["post_funding"].as_array().unwrap().len(), 1);
        let audits = serde_json::to_string(&request.get(&format!("/api/audits/{w}")).await.json::<Value>()).unwrap();
        assert!(audits.contains("post_funding_set"));
        request.delete(&url).await.assert_status_ok();
        assert_eq!(request.delete(&url).await.status_code(), 404);
        request.put(&url).json(&json!({ "kind": "core" })).await.assert_status_ok();
        for to in ["active", "offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        request.post(&format!("/api/workers/{w}/erase")).await.assert_status_ok();
        use sea_orm::{ConnectionTrait, Statement};
        let left = ctx.db.query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT count(*) AS n FROM post_fundings WHERE worker_pid = '{w}'"),
        )).await.unwrap().unwrap().try_get::<i64>("", "n").unwrap();
        assert_eq!(left, 0, "erased with the worker");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Contingent share and funding horizon (WPM-R62): shares apart, months, small months withheld.
async fn the_contingent_view_shows_shares_and_funding_months_and_withholds_small_groups() {
    crate::requests::request_open(|request, _ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = chrono::Utc::now().date_naive();
        let end = today + chrono::Duration::days(75);
        let month = end.format("%Y-%m").to_string();
        // Ten people whose funding ends in one month, and one whose ends in another.
        for i in 0..11 {
            let kind = if i < 2 { "contractor" } else if i < 4 { "fixed_term" } else { "permanent" };
            let r = request.post("/api/workers").json(&json!({
                "person_ref": a_person(), "organization_ref": an_org(), "worker_number": format!("CT-{tag}-{i}"),
                "display_name": "Test Worker CT", "employment_type": kind, "department": format!("ct-{tag}"),
                "job_title": "Engineer", "hired_on": "2026-01-05", "engagement_ends_on": (kind != "permanent").then(|| (today + chrono::Duration::days(300)).to_string()),
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
            let w = r.json::<Value>()["pid"].as_str().unwrap().to_string();
            let ends = if i < 10 { end } else { today + chrono::Duration::days(200) };
            request.put(&format!("/api/workers/{w}/funding")).json(&json!({ "kind": "external", "ends_on": ends.to_string() })).await.assert_status_ok();
        }
        let v: Value = request.get("/api/workforce-intelligence/contingent").await.json();
        assert_eq!(v["small_group_floor"], 10);
        let shares = (v["contractor_percent_of_fte"].as_f64().unwrap(), v["fixed_term_percent_of_fte"].as_f64().unwrap());
        assert!(shares.0 > 0.0 && shares.1 > 0.0, "each is shown, apart");
        let months = v["funding_ends"].as_array().unwrap();
        assert_eq!(months.len(), 13);
        assert_eq!(months[0]["month"], "already_ended");
        let hit = months.iter().find(|m| m["month"] == month.as_str()).unwrap();
        assert!(hit["people"].as_u64().unwrap() >= 10, "ten at the floor are shown");
        // The one whose funding ends alone in another month is not shown by itself.
        let lone = (today + chrono::Duration::days(200)).format("%Y-%m").to_string();
        let lone = months.iter().find(|m| m["month"] == lone.as_str()).unwrap();
        if lone["people"].is_null() {
            assert!(lone["fte"].is_null(), "withheld means no figure at all");
        } else {
            assert!(lone["people"].as_u64().unwrap() >= 10);
        }
        // The named list behind the figures: HR sees worker numbers, a stranger token is refused, the read is audited.
        let listed = request.get("/api/engagements/contingent-list?list=past_end_undecided").await;
        assert_eq!(listed.status_code(), 200);
        let listed: Value = listed.json();
        assert_eq!(listed["list"], "past_end_undecided");
        assert!(listed["people"].is_array());
        assert!(!listed.to_string().contains("Test Worker CT"), "worker numbers, not names");
        assert_eq!(request.get("/api/engagements/contingent-list?list=everyone").await.status_code(), 422);
        assert_eq!(request.get("/api/engagements/contingent-list").await.status_code(), 422);
        let e = &v["engagements"];
        for key in ["ending_within_30_calendar_days", "ending_31_to_60_calendar_days", "ending_61_to_90_calendar_days", "past_end_with_no_decision"] {
            assert!(e[key].is_null() || e[key].is_u64(), "{key}: a count, or withheld, never a guess");
        }
        assert_eq!(e["extended_more_than"]["times"], 2);
        assert_eq!(e["contractors_engaged_longer_than"]["calendar_months"], 24);
        assert!(e.get("total").is_none(), "no total, so nothing withheld can be worked out");
        let text = v.to_string();
        assert!(!text.contains("Test Worker CT"), "no names");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// The contingent trend (WPM-R62, WPM-T139): snapshots carry the split by basis; too little history says so.
async fn the_contingent_trend_comes_from_snapshots_split_by_basis() {
    use workforce_planning_management::tasks::snapshot::run_snapshot;

    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        for i in 0..12 {
            let kind = if i < 3 { "contractor" } else if i < 5 { "fixed_term" } else { "permanent" };
            let r = request.post("/api/workers").json(&json!({
                "person_ref": a_person(), "organization_ref": an_org(), "worker_number": format!("TR-{tag}-{i}"),
                "display_name": "Test Worker TR", "employment_type": kind, "department": format!("tr-{tag}"),
                "job_title": "Engineer", "hired_on": "2019-01-05",
                "engagement_ends_on": (kind != "permanent").then(|| "2030-01-01".to_string()),
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
        }
        let date = |m, d| chrono::NaiveDate::from_ymd_opt(2025, m, d).unwrap();
        run_snapshot(&ctx.db, date(1, 1)).await.unwrap();
        let v: Value = request.get("/api/workforce-intelligence/contingent").await.json();
        assert_eq!(v["trend"]["status"], "insufficient_history", "one date is not a trend");
        assert!(v["trend"]["points"].as_array().unwrap().is_empty());
        run_snapshot(&ctx.db, date(2, 15)).await.unwrap();
        run_snapshot(&ctx.db, date(4, 1)).await.unwrap();
        let v: Value = request.get("/api/workforce-intelligence/contingent").await.json();
        assert_eq!(v["trend"]["status"], "ok");
        let points = v["trend"]["points"].as_array().unwrap();
        assert_eq!(points.len(), 3);
        assert!(points[2]["contractor_percent_of_fte"].as_f64().unwrap() > 0.0);
        assert!(points[2]["fixed_term_percent_of_fte"].as_f64().unwrap() > 0.0);
        // A row recorded before the split is unknown, not zero: its whole date is left out.
        use sea_orm::{ConnectionTrait, Statement};
        ctx.db.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "UPDATE headcount_snapshots SET contractor_headcount = NULL, contractor_fte_percent_total = NULL, \
             fixed_term_headcount = NULL, fixed_term_fte_percent_total = NULL WHERE as_of = '2025-02-15'".to_string(),
        )).await.unwrap();
        let v: Value = request.get("/api/workforce-intelligence/contingent").await.json();
        assert_eq!(v["trend"]["status"], "insufficient_history", "two usable dates remain");
        assert_eq!(v["trend"]["dates_left_out_without_split"], 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// The programme roll-up (WPM-R83): a programme at the floor shows by month; a smaller one is left out and counted.
async fn the_programme_rollup_shows_a_programme_at_the_floor_and_leaves_out_a_small_one() {
    crate::requests::request_open(|request, _ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = chrono::Utc::now().date_naive();
        let big = format!("thing:{}", uuid::Uuid::new_v4());
        let small = format!("thing:{}", uuid::Uuid::new_v4());
        for i in 0..12 {
            let programme = if i < 10 { &big } else { &small };
            let kind = if i < 2 { "contractor" } else if i < 4 { "fixed_term" } else { "permanent" };
            let r = request.post("/api/workers").json(&json!({
                "person_ref": a_person(), "organization_ref": an_org(), "worker_number": format!("PG-{tag}-{i}"),
                "display_name": "Test Worker PG", "employment_type": kind, "department": format!("pg-{tag}"),
                "job_title": "Engineer", "hired_on": "2026-01-05",
                "engagement_ends_on": (kind != "permanent").then(|| (today + chrono::Duration::days(300)).to_string()),
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
            let w = r.json::<Value>()["pid"].as_str().unwrap().to_string();
            request.put(&format!("/api/workers/{w}/funding")).json(&json!({
                "kind": "programme", "programme_ref": programme, "ends_on": (today + chrono::Duration::days(400)).to_string(),
            })).await.assert_status_ok();
        }
        let v: Value = request.get("/api/workforce-intelligence/programmes").await.json();
        assert_eq!(v["small_group_floor"], 10);
        let programmes = v["programmes"].as_array().unwrap();
        let shown = programmes.iter().find(|p| p["programme"] == big.as_str()).expect("ten people reach the floor");
        assert!(programmes.iter().all(|p| p["programme"] != small.as_str()), "two people are left out");
        assert!(v["withheld_programmes"].as_u64().unwrap() >= 1);
        let months = shown["months"].as_array().unwrap();
        assert_eq!(months.len(), 12);
        assert_eq!(months[0]["funded_fte"], 10.0);
        assert_eq!(months[0]["contractor_percent_of_fte"], 20.0);
        assert_eq!(months[0]["fixed_term_percent_of_fte"], 20.0);
        assert!(v.get("total").is_none(), "no total across programmes");
        assert!(!v.to_string().contains("Test Worker PG"), "no names");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// A requisition carries programme, basis and duration (WPM-R82); the hire carries the basis and the end date.
async fn a_requisition_carries_its_basis_and_programme_onto_the_hire() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let programme = format!("thing:{}", uuid::Uuid::new_v4());
        let body = |extra: Value| {
            let mut b = json!({ "organization_ref": org, "department": "engineering", "job_title": "Fixed Engineer", "headcount": 1 });
            b.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            b
        };
        for (why, extra) in [
            ("fixed-term needs a duration", json!({ "basis": "fixed_term" })),
            ("permanent has no duration", json!({ "basis": "permanent", "expected_duration_months": 6 })),
            ("a duration beyond ten years", json!({ "basis": "contractor", "expected_duration_months": 121 })),
            ("a programme must be a thing URN", json!({ "programme_ref": "not-a-urn" })),
            ("an unknown basis", json!({ "basis": "volunteer" })),
        ] {
            assert_eq!(request.post("/api/requisitions").json(&body(extra)).await.status_code(), 422, "{why}");
        }
        let made: Value = request
            .post("/api/requisitions")
            .json(&body(json!({ "basis": "fixed_term", "expected_duration_months": 6, "programme_ref": programme })))
            .await
            .json();
        let req = made["pid"].as_str().unwrap().to_string();
        let shown: Value = request.get(&format!("/api/requisitions/{req}")).await.json();
        assert_eq!((shown["basis"].as_str(), shown["expected_duration_months"].as_i64()), (Some("fixed_term"), Some(6)));
        assert_eq!(shown["programme_ref"], programme.as_str());
        for to in ["open", "interviewing"] {
            request.post(&format!("/api/requisitions/{req}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        let candidate: Value = request.post("/api/candidates")
            .json(&json!({ "display_name": "Test Applicant 140", "email": "a140@example.com", "source": "referral", "person_ref": a_person() }))
            .await.json();
        let application: Value = request.post(&format!("/api/requisitions/{req}/applications"))
            .json(&json!({ "candidate_pid": candidate["pid"] })).await.json();
        let app = application["pid"].as_str().unwrap().to_string();
        let stage = format!("/api/applications/{app}/stage");
        for to in ["screened", "interviewing", "offer"] {
            request.post(&stage).json(&json!({ "to": to })).await.assert_status_ok();
        }
        // No basis and no end date on the hire: both come from the requisition.
        let hired: Value = request.post(&stage)
            .json(&json!({ "to": "hired", "worker_number": format!("RQ-{tag}"), "hired_on": "2026-01-05" }))
            .await.json();
        let worker = hired["worker_pid"].as_str().expect("hired").to_string();
        let w: Value = request.get(&format!("/api/workers/{worker}")).await.json();
        assert_eq!(w["employment_type"], "fixed_term");
        let e: Value = request.get(&format!("/api/workers/{worker}/engagement")).await.json();
        assert_eq!(e["engagement_ends_on"], "2026-07-04", "six calendar months from 5 January, to the day before");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Demand lines carry programme, FTE and basis (WPM-R82); the forecast reports them and projects each basis on its own terms.
async fn demand_lines_by_basis_and_programme_reach_the_forecast() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let programme = format!("thing:{}", uuid::Uuid::new_v4());
        let dept = format!("fc-{tag}");
        // One permanent, one fixed-term ending before the target, one contractor ending after it.
        let mut made: Vec<String> = Vec::new();
        for (i, (kind, ends)) in [("permanent", None), ("fixed_term", Some("2027-03-31")), ("contractor", Some("2028-01-31"))].into_iter().enumerate() {
            let r = request.post("/api/workers").json(&json!({
                "person_ref": a_person(), "organization_ref": org, "worker_number": format!("FC-{tag}-{i}"),
                "display_name": "Test Worker FC", "employment_type": kind, "department": dept,
                "job_title": "Engineer", "hired_on": "2026-01-05", "engagement_ends_on": ends,
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
            made.push(r.json::<Value>()["pid"].as_str().unwrap().to_string());
        }
        let plan: Value = request.post("/api/workforce-plans").json(&json!({
            "name": "Basis plan", "organization_ref": org,
            "horizon_start": "2026-01-01", "horizon_end": "2028-12-31", "attrition_bp": 0,
        })).await.json();
        let plan = plan["pid"].as_str().unwrap().to_string();
        let line = |body: Value| request.put(&format!("/api/workforce-plans/{plan}/demand-lines")).json(&body);
        for (why, body) in [
            ("fixed-term needs an end", json!({ "basis": "fixed_term" })),
            ("permanent has no end", json!({ "basis": "permanent", "expected_ends_on": "2027-12-31" })),
            ("FTE beyond the heads", json!({ "target_fte_percent": 301 })),
            ("a programme must be a thing URN", json!({ "programme_ref": "nope" })),
        ] {
            let mut b = json!({ "department": dept, "target_on": "2027-06-30", "target_headcount": 3 });
            b.as_object_mut().unwrap().extend(body.as_object().unwrap().clone());
            assert_eq!(line(b).await.status_code(), 422, "{why}");
        }
        line(json!({ "department": dept, "target_on": "2027-06-30", "target_headcount": 2, "target_fte_percent": 150, "programme_ref": programme })).await.assert_status_ok();
        line(json!({ "department": dept, "target_on": "2027-06-30", "target_headcount": 1, "basis": "fixed_term", "expected_ends_on": "2027-12-31", "programme_ref": programme })).await.assert_status_ok();
        line(json!({ "department": dept, "target_on": "2027-06-30", "target_headcount": 1, "basis": "contractor", "expected_ends_on": "2027-12-31" })).await.assert_status_ok();
        let shown: Value = request.get(&format!("/api/workforce-plans/{plan}")).await.json();
        assert_eq!(shown["demand_lines"].as_array().unwrap().len(), 3, "the same department and date can hold a line for each basis");
        let f: Value = request.get(&format!("/api/workforce-plans/{plan}/forecast")).await.json();
        let d = &f["departments"][0];
        assert_eq!(d["planned_demand"], 4);
        assert!(d["planned_demand_fte"].is_null(), "two lines state no FTE, so the total is unknown, not guessed");
        let basis = |name: &str| d["demand_by_basis"].as_array().unwrap().iter().find(|b| b["basis"] == name).cloned().unwrap();
        assert_eq!(basis("permanent")["fte"], 1.5);
        assert!(basis("fixed_term")["fte"].is_null());
        let by_prog = d["demand_by_programme"].as_array().unwrap();
        assert_eq!(by_prog.iter().find(|p| p["programme_ref"] == programme.as_str()).unwrap()["headcount"], 3);
        assert_eq!(by_prog.iter().find(|p| p["programme_ref"].is_null()).unwrap()["headcount"], 1);
        // Supply on 30 June 2027: the permanent stays (no attrition), the fixed-term left on 31 March, the contractor remains.
        let sb = &d["supply_by_basis"];
        assert_eq!(sb["permanent"]["headcount"], 1);
        assert_eq!(sb["fixed_term"]["headcount"], 0);
        assert_eq!(sb["fixed_term"]["leaving_before_the_date"], 1);
        assert_eq!(sb["contractor"]["headcount"], 1);
        assert_eq!(d["projected_supply"], 2);
        assert_eq!(d["headcount_gap"], 2);
        let levers: Vec<_> = d["levers"].as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect();
        assert!(levers.contains(&"fixed_term_hire") && levers.contains(&"engage_contractor"));
        // The department holds a fixed-term and a contractor engagement today, so a conversion is on offer, with its evidence.
        assert!(levers.contains(&"convert_contingent"), "contract FTE exists, so the conversion lever is offered");
        assert_eq!(d["contract_fte_now"]["fixed_term"]["headcount"], 1);
        assert_eq!(d["contract_fte_now"]["contractor"]["headcount"], 1);
        assert!(f["programmes"].as_array().unwrap().iter().any(|p| p["programme_ref"] == programme.as_str() && p["headcount"] == 3));
        // Conversion plans (WPM-T164): none yet; a proposed one is a hope; an approved one is agreed. Neither
        // moves the projected supply, and a plan that is abandoned counts for nothing.
        let counts = |f: &Value| {
            let c = &f["departments"][0]["conversions"];
            (c["agreed"]["headcount"].as_i64(), c["hoped"]["headcount"].as_i64(), c["agreed"]["fte"].as_f64(), c["hoped"]["fte"].as_f64())
        };
        assert_eq!(counts(&f), (Some(0), Some(0), Some(0.0), Some(0.0)));
        let contractor = &made[2];
        let proposed = request
            .post(&format!("/api/workers/{contractor}/conversion-plans"))
            .json(&json!({ "intent": "convert", "target_on": "2027-06-01", "post_funding_kind": "core" }))
            .await;
        assert_eq!(proposed.status_code(), 200, "{}", proposed.text());
        let conversion = proposed.json::<Value>()["pid"].as_str().unwrap().to_string();
        let forecast = || async { request.get(&format!("/api/workforce-plans/{plan}/forecast")).await.json::<Value>() };
        let hoped = forecast().await;
        assert_eq!(counts(&hoped), (Some(0), Some(1), Some(0.0), Some(1.0)), "proposed is a hope");
        request.post(&format!("/api/conversion-plans/{conversion}/approve")).await.assert_status_ok();
        let agreed = forecast().await;
        assert_eq!(counts(&agreed), (Some(1), Some(0), Some(1.0), Some(0.0)), "approved is agreed, and no longer only a hope");
        assert_eq!(agreed["departments"][0]["projected_supply"], 2, "an agreed conversion does not change the projection");
        assert!(!agreed.to_string().contains(contractor), "the forecast names no one");
        request.post(&format!("/api/conversion-plans/{conversion}/abandon")).await.assert_status_ok();
        assert_eq!(counts(&forecast().await), (Some(0), Some(0), Some(0.0), Some(0.0)), "an abandoned plan counts for nothing");
        // The same key again updates the line instead of adding one.
        line(json!({ "department": dept, "target_on": "2027-06-30", "target_headcount": 2, "target_fte_percent": 200, "programme_ref": programme })).await.assert_status_ok();
        let again: Value = request.get(&format!("/api/workforce-plans/{plan}")).await.json();
        assert_eq!(again["demand_lines"].as_array().unwrap().len(), 3);
    })
    .await;
}
