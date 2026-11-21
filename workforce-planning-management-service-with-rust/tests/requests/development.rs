//! Reviews and succession: the review lifecycle (draft → submitted → calibrated → shared),
//! goals and feedback, what a worker's review list redacts until it is shared, the
//! expiring-certificate list, and succession plans restated and checked for bench gaps.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_review_moves_through_its_lifecycle_and_shows_content_only_once_shared() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("RV-{tag}-W"), None).await;
        let reviewer = seed_worker!(&request, &org, format!("RV-{tag}-R"), None).await;

        let cycle: Value = request
            .post("/api/review-cycles")
            .json(&json!({ "name": format!("Cycle {tag}"), "period_start": "2026-01-01", "period_end": "2026-12-31" }))
            .await
            .json();
        let review: Value = request
            .post(&format!("/api/review-cycles/{}/reviews", cycle["pid"].as_str().unwrap()))
            .json(&json!({ "worker_pid": worker, "reviewer_ref": format!("worker:{reviewer}") }))
            .await
            .json();
        let review = review["pid"].as_str().unwrap().to_string();

        // A draft takes a rating and content; both are range-checked.
        assert_eq!(
            request.put(&format!("/api/reviews/{review}")).json(&json!({ "rating": 9 })).await.status_code(),
            422
        );
        request
            .put(&format!("/api/reviews/{review}"))
            .json(&json!({ "rating": 4, "content": "Steady and generous with their time." }))
            .await
            .assert_status_ok();

        // Goals: a weight outside 1-100 and an unknown status are refused; a goal can be met.
        let goals_url = format!("/api/reviews/{review}/goals");
        assert_eq!(request.post(&goals_url).json(&json!({ "title": "x", "weight_percent": 0 })).await.status_code(), 422);
        assert_eq!(request.post(&goals_url).json(&json!({ "title": " " })).await.status_code(), 422);
        let goal: Value = request
            .post(&goals_url)
            .json(&json!({ "title": "Ship the migration", "weight_percent": 60 }))
            .await
            .json();
        let goal_url = format!("/api/goals/{}", goal["pid"].as_str().unwrap());
        assert_eq!(request.put(&goal_url).json(&json!({ "status": "won" })).await.status_code(), 422);
        let met: Value = request.put(&goal_url).json(&json!({ "status": "met" })).await.json();
        assert_eq!(met["status"], "met");
        assert_eq!(
            request.put(&format!("/api/goals/{}", uuid::Uuid::new_v4())).json(&json!({ "status": "met" })).await.status_code(),
            404
        );

        // Feedback names its author as a worker URN.
        let feedback_url = format!("/api/reviews/{review}/feedback");
        assert_eq!(
            request.post(&feedback_url).json(&json!({ "author_ref": "nobody", "content": "Great" })).await.status_code(),
            422
        );
        request
            .post(&feedback_url)
            .json(&json!({ "author_ref": format!("worker:{reviewer}"), "content": "Great teammate" }))
            .await
            .assert_status_ok();
        let detail: Value = request.get(&format!("/api/reviews/{review}")).await.json();
        assert_eq!(detail["goals"].as_array().unwrap().len(), 1);
        assert_eq!(detail["feedback"].as_array().unwrap().len(), 1);

        // Until shared, the worker's own list carries no rating and no content.
        let status_url = format!("/api/reviews/{review}/status");
        let hidden: Vec<Value> = request.get(&format!("/api/workers/{worker}/reviews")).await.json();
        assert_eq!(hidden.len(), 1);
        assert_eq!(hidden[0]["content"], Value::Null);
        assert_eq!(hidden[0]["rating"], Value::Null);

        // Transitions: unknown and illegal ones are refused, the legal ones go in order.
        for bad in ["bogus", "shared", "calibrated"] {
            assert_eq!(request.post(&status_url).json(&json!({ "to": bad })).await.status_code(), 422, "{bad}");
        }
        request.post(&status_url).json(&json!({ "to": "submitted" })).await.assert_status_ok();
        // Submitted is no longer editable.
        assert_eq!(request.put(&format!("/api/reviews/{review}")).json(&json!({ "rating": 3 })).await.status_code(), 422);
        request.post(&status_url).json(&json!({ "to": "calibrated" })).await.assert_status_ok();
        let shared: Value = request.post(&status_url).json(&json!({ "to": "shared" })).await.json();
        assert_eq!(shared["status"], "shared");
        assert_eq!(request.post(&status_url).json(&json!({ "to": "draft" })).await.status_code(), 422);

        let visible: Vec<Value> = request.get(&format!("/api/workers/{worker}/reviews")).await.json();
        assert_eq!(visible[0]["rating"], 4);
        assert_eq!(visible[0]["content"], "Steady and generous with their time.");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn succession_plans_are_restated_and_critical_ones_without_a_ready_now_candidate_are_gaps() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let holder = seed_worker!(&request, &org, format!("SP-{tag}-H"), None).await;
        let heir = seed_worker!(&request, &org, format!("SP-{tag}-E"), None).await;
        activate!(&request, &holder).await;
        activate!(&request, &heir).await;

        let plan: Value = request
            .post("/api/succession-plans")
            .json(&json!({ "role_title": format!("Head of {tag}"), "department": "engineering", "criticality": 5, "incumbent_pid": holder }))
            .await
            .json();
        let plan = plan["pid"].as_str().unwrap().to_string();
        let gaps = |gaps: Value| -> bool { gaps["gaps"].as_array().unwrap().iter().any(|g| g["pid"] == plan.as_str()) };

        // A critical plan with nobody ready now is a gap.
        assert!(gaps(request.get("/api/succession-plans/gaps").await.json()));

        // Restating it: out-of-range and unknown values are refused, a stranger is not an incumbent.
        let url = format!("/api/succession-plans/{plan}");
        for bad in [
            json!({ "criticality": 0 }),
            json!({ "risk_of_loss": "certain" }),
            json!({ "incumbent_pid": uuid::Uuid::new_v4() }),
        ] {
            assert!(request.put(&url).json(&bad).await.status_code().is_client_error(), "{bad}");
        }
        assert_eq!(request.put(&format!("/api/succession-plans/{}", uuid::Uuid::new_v4())).json(&json!({})).await.status_code(), 404);
        let restated: Value = request
            .put(&url)
            .json(&json!({ "risk_of_loss": "high", "vacancy_expected_on": "2027-03-01" }))
            .await
            .json();
        assert_eq!(restated["risk_of_loss"], "high");
        assert_eq!(restated["criticality"], 5, "what was not restated is kept");

        // A ready-now candidate closes the gap.
        request
            .post(&format!("/api/succession-plans/{plan}/candidates"))
            .json(&json!({ "worker_pid": heir, "readiness": "ready_now" }))
            .await
            .assert_status_ok();
        assert!(!gaps(request.get("/api/succession-plans/gaps").await.json()));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn expiring_training_lists_certificates_that_lapse_inside_the_horizon() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("TX-{tag}-W"), None).await;
        let course = format!("course:{}", uuid::Uuid::new_v4());
        let day = |n: i64| (workforce_planning_management::dates::today() + workforce_planning_management::dates::days(n)).to_string();
        for (n, tail) in [(10_i64, "near"), (200, "far")] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                    "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status, completed_on, certificate_expires_on) \
                     VALUES ('{}', '{worker}', '{course}-{tail}', 'completed', '{}', '{}')",
                    uuid::Uuid::new_v4(),
                    day(-30),
                    day(n)
                ))).execute(workforce_planning_management::db::pool(&ctx))
                .await
                .unwrap();
        }
        let listed = |body: &Value, tail: &str| {
            body["expiring"].as_array().unwrap().iter().any(|e| e["course_ref"] == format!("{course}-{tail}").as_str())
        };
        let default: Value = request.get("/api/training/expiring").await.json();
        assert!(listed(&default, "near") && !listed(&default, "far"), "the default horizon is 90 days");
        let wide: Value = request.get("/api/training/expiring?within_days=365").await.json();
        assert!(listed(&wide, "near") && listed(&wide, "far"));
        assert_eq!(wide["horizon"], day(365));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn cycles_training_and_the_succession_bench_refuse_what_cannot_be() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("DV-{tag}-W"), None).await;
        activate!(&request, &worker).await;

        // A cycle that ends before it starts is refused; a closed cycle takes no new review.
        let reversed = request
            .post("/api/review-cycles")
            .json(&json!({ "name": format!("Back {tag}"), "period_start": "2026-12-31", "period_end": "2026-01-01" }))
            .await;
        assert_eq!(reversed.status_code(), 422);
        let cycle: Value = request
            .post("/api/review-cycles")
            .json(&json!({ "name": format!("Closed {tag}"), "period_start": "2026-01-01", "period_end": "2026-12-31" }))
            .await
            .json();
        let cycle = cycle["pid"].as_str().unwrap().to_string();
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("UPDATE review_cycles SET status = 'closed' WHERE pid = '{cycle}'")))
            .execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        let late = request.post(&format!("/api/review-cycles/{cycle}/reviews")).json(&json!({ "worker_pid": worker })).await;
        assert_eq!(late.status_code(), 422, "{}", late.text());

        // Training: a course reference that is not a course is refused; an enrolment moves to in progress.
        let training = format!("/api/workers/{worker}/training-enrollments");
        assert_eq!(request.post(&training).json(&json!({ "course_ref": format!("person:{}", uuid::Uuid::new_v4()) })).await.status_code(), 422);
        let enrolled: Value = request.post(&training).json(&json!({ "course_ref": format!("course:{}", uuid::Uuid::new_v4()) })).await.json();
        let moved = request
            .put(&format!("/api/training-enrollments/{}", enrolled["pid"].as_str().unwrap()))
            .json(&json!({ "status": "in_progress" }))
            .await;
        assert_eq!(moved.status_code(), 200, "{}", moved.text());

        // Succession: criticality outside 1 to 5, and a rank below 1 on adding or restating a candidate, are refused.
        let bad = json!({ "role_title": format!("Lead {tag}"), "department": "engineering", "criticality": 6 });
        assert_eq!(request.post("/api/succession-plans").json(&bad).await.status_code(), 422);
        let plan: Value = request
            .post("/api/succession-plans")
            .json(&json!({ "role_title": format!("Lead {tag}"), "department": "engineering", "criticality": 3 }))
            .await
            .json();
        let candidates = format!("/api/succession-plans/{}/candidates", plan["pid"].as_str().unwrap());
        assert_eq!(request.post(&candidates).json(&json!({ "worker_pid": worker, "readiness": "ready_1y", "rank": 0 })).await.status_code(), 422);
        let added: Value = request.post(&candidates).json(&json!({ "worker_pid": worker, "readiness": "ready_1y", "rank": 1 })).await.json();
        let candidate = added["pid"].as_str().unwrap().to_string();
        assert_eq!(request.put(&format!("/api/succession-candidates/{candidate}")).json(&json!({ "rank": 0 })).await.status_code(), 422);
        let listed = request.get("/api/succession-plans").await;
        assert_eq!(listed.status_code(), 200);
        assert!(listed.text().contains("\"candidates\""), "plans come with their bench");
    })
    .await;
}
