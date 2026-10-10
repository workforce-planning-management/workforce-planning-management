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
