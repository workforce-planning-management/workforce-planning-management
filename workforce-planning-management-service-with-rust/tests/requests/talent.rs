//! The talent-strategy round-trip: upskilling and reskilling plans
//! (declared vs verified progress), talent pipelines, apprenticeships
//! and internships (including the off-the-job hours gate), succession
//! risk, and the workforce-intelligence views.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one seeded team, the whole talent surface
async fn development_plans_track_claimed_and_verified_progress() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "T-1", None).await;
        activate!(&request, &worker).await;

        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": "Kubernetes", "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().expect("skill pid").to_string();

        // ── Upskill vs reskill: the target role decides the kind.
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/development-plans"))
                .json(&json!({ "kind": "reskill" }))
                .await
                .status_code(),
            422,
            "a reskill must name its target role"
        );
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/development-plans"))
                .json(&json!({ "kind": "upskill", "target_job_title": "Data Engineer" }))
                .await
                .status_code(),
            422,
            "an upskill deepens the current role"
        );
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/development-plans"))
                .json(&json!({ "kind": "reskill", "target_job_title": "Engineer" }))
                .await
                .status_code(),
            422,
            "reskilling into the current role is an upskill"
        );
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/development-plans"))
                .json(&json!({
                    "kind": "upskill",
                    "items": [{
                        "skill_pid": skill_pid, "current_level": 4, "target_level": 3,
                        "method": "course",
                    }],
                }))
                .await
                .status_code(),
            422,
            "a step must raise the level"
        );

        let plan: Value = request
            .post(&format!("/api/workers/{worker}/development-plans"))
            .json(&json!({
                "kind": "upskill",
                "rationale": "platform ownership",
                "items": [
                    { "skill_pid": skill_pid, "current_level": 2, "target_level": 4, "method": "course" },
                ],
            }))
            .await
            .json();
        let plan_pid = plan["pid"].as_str().expect("plan pid").to_string();

        // ── The item is claimed achieved, but proficiency has not moved.
        let listed: Value = request
            .get(&format!("/api/workers/{worker}/development-plans"))
            .await
            .json();
        let item_pid = listed["plans"][0]["items"][0]["pid"]
            .as_str()
            .expect("item pid")
            .to_string();
        request
            .put(&format!("/api/development-plan-items/{item_pid}"))
            .json(&json!({ "status": "achieved" }))
            .await
            .assert_status_ok();

        let claimed: Value = request
            .get(&format!("/api/workers/{worker}/development-plans"))
            .await
            .json();
        assert_eq!(claimed["plans"][0]["declared_progress"]["numerator"], 1);
        assert_eq!(
            claimed["plans"][0]["verified_progress"]["numerator"], 0,
            "a claim without declared proficiency is not verified"
        );

        // Declaring the proficiency is what makes it verified.
        request
            .put(&format!("/api/workers/{worker}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 4 }))
            .await
            .assert_status_ok();
        let verified: Value = request
            .get(&format!("/api/workers/{worker}/development-plans"))
            .await
            .json();
        assert_eq!(verified["plans"][0]["verified_progress"]["numerator"], 1);

        // ── Lifecycle: activate, then complete once every item is resolved.
        assert_eq!(
            request
                .post(&format!("/api/development-plans/{plan_pid}/status"))
                .json(&json!({ "to": "completed" }))
                .await
                .status_code(),
            422,
            "must activate first"
        );
        request
            .post(&format!("/api/development-plans/{plan_pid}/status"))
            .json(&json!({ "to": "active" }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/development-plans/{plan_pid}/status"))
            .json(&json!({ "to": "completed" }))
            .await
            .assert_status_ok();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // pipelines + early careers + intelligence in one flow
async fn pipelines_apprenticeships_and_intelligence() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let apprentice = seed_worker!(&request, &org, "P-1", None).await;
        activate!(&request, &apprentice).await;
        let supervisor = seed_worker!(&request, &org, "P-2", None).await;
        activate!(&request, &supervisor).await;

        // ── Talent pipeline: stages move forward, and readiness may regress.
        let pipeline: Value = request
            .post("/api/talent-pipelines")
            .json(&json!({
                "name": "Future team leads",
                "purpose": "succession",
                "target_job_title": "Team Lead",
            }))
            .await
            .json();
        let pipeline_pid = pipeline["pid"].as_str().expect("pipeline pid").to_string();
        assert_eq!(
            request
                .post("/api/talent-pipelines")
                .json(&json!({ "name": "Bad", "purpose": "vibes" }))
                .await
                .status_code(),
            422,
            "unknown purpose refused"
        );

        let member: Value = request
            .post(&format!("/api/talent-pipelines/{pipeline_pid}/members"))
            .json(&json!({ "subject_kind": "worker", "subject_pid": apprentice }))
            .await
            .json();
        let member_pid = member["pid"].as_str().expect("member pid").to_string();
        assert_eq!(
            request
                .post(&format!("/api/talent-pipelines/{pipeline_pid}/members"))
                .json(&json!({ "subject_kind": "worker", "subject_pid": apprentice }))
                .await
                .status_code(),
            422,
            "one row per subject"
        );
        assert_eq!(
            request
                .post(&format!("/api/pipeline-members/{member_pid}/stage"))
                .json(&json!({ "to": "placed" }))
                .await
                .status_code(),
            422,
            "no skipping straight to placed"
        );
        for stage in ["assessing", "developing", "ready"] {
            request
                .post(&format!("/api/pipeline-members/{member_pid}/stage"))
                .json(&json!({ "to": stage }))
                .await
                .assert_status_ok();
        }
        let ready: Value = request
            .get(&format!("/api/talent-pipelines/{pipeline_pid}"))
            .await
            .json();
        assert_eq!(ready["health"]["ready"], 1);
        assert_eq!(ready["health"]["live"], 1);
        // Readiness can regress — the pipeline must be able to say so.
        request
            .post(&format!("/api/pipeline-members/{member_pid}/stage"))
            .json(&json!({ "to": "developing", "readiness": "ready_1y" }))
            .await
            .assert_status_ok();
        let regressed: Value = request
            .get(&format!("/api/talent-pipelines/{pipeline_pid}"))
            .await
            .json();
        assert_eq!(regressed["health"]["ready"], 0, "the bench shrank honestly");

        // ── Apprenticeship: the off-the-job hours are the gate.
        assert_eq!(
            request
                .post("/api/early-career-programs")
                .json(&json!({
                    "name": "No hours", "kind": "apprenticeship", "duration_months": 18,
                }))
                .await
                .status_code(),
            422,
            "an apprenticeship must declare its off-the-job hours"
        );
        let program: Value = request
            .post("/api/early-career-programs")
            .json(&json!({
                "name": "Software Developer L4", "kind": "apprenticeship", "level": 4,
                "duration_months": 18, "min_off_the_job_hours": 100,
            }))
            .await
            .json();
        let program_pid = program["pid"].as_str().expect("program pid").to_string();

        let placement: Value = request
            .post(&format!(
                "/api/early-career-programs/{program_pid}/placements"
            ))
            .json(&json!({
                "worker_pid": apprentice,
                "supervisor_pid": supervisor,
                "started_on": "2026-02-02",
            }))
            .await
            .json();
        let placement_pid = placement["pid"]
            .as_str()
            .expect("placement pid")
            .to_string();

        // Hours only accrue on an active placement.
        assert_eq!(
            request
                .post(&format!("/api/program-placements/{placement_pid}/hours"))
                .json(&json!({ "hours": 10 }))
                .await
                .status_code(),
            422,
            "an offered placement does not accrue hours"
        );
        request
            .post(&format!("/api/program-placements/{placement_pid}/status"))
            .json(&json!({ "to": "active" }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/program-placements/{placement_pid}/hours"))
            .json(&json!({ "hours": 60 }))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .post(&format!("/api/program-placements/{placement_pid}/status"))
                .json(&json!({ "to": "completed", "outcome": "converted" }))
                .await
                .status_code(),
            422,
            "60 of 100 off-the-job hours ⇒ cannot complete the apprenticeship"
        );
        request
            .post(&format!("/api/program-placements/{placement_pid}/hours"))
            .json(&json!({ "hours": 40 }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/program-placements/{placement_pid}/status"))
            .json(&json!({ "to": "completed", "outcome": "converted" }))
            .await
            .assert_status_ok();

        let placements: Value = request
            .get(&format!("/api/workers/{apprentice}/placements"))
            .await
            .json();
        assert_eq!(placements["placements"][0]["off_the_job"]["hours"], 100);
        assert_eq!(placements["placements"][0]["off_the_job"]["met"], true);

        // ── Workforce intelligence.
        let programs: Value = request
            .get("/api/early-career-programs?kind=apprenticeship")
            .await
            .json();
        let conversion = &programs["programs"][0]["placements"]["conversion_rate"];
        assert_eq!(conversion["numerator"], 1);
        assert_eq!(conversion["denominator"], 1);

        let overview: Value = request
            .get("/api/workforce-intelligence/overview")
            .await
            .json();
        assert!(
            overview["headcount"].as_u64().expect("headcount") >= 2,
            "the seeded team is counted"
        );
        assert!(
            overview["by_department"]["engineering"]
                .as_u64()
                .expect("dept")
                >= 2
        );

        let funnel: Value = request
            .get("/api/workforce-intelligence/pipelines")
            .await
            .json();
        assert_eq!(funnel["by_purpose"]["succession"], 1);
        let apprenticeships = funnel["early_careers"]
            .as_array()
            .expect("kinds")
            .iter()
            .find(|k| k["kind"] == "apprenticeship")
            .expect("apprenticeship rollup")
            .clone();
        assert_eq!(apprenticeships["completed_placements"], 1);
        assert_eq!(apprenticeships["conversion_rate"]["value"], 1.0);

        // ── Succession: risk of loss makes an uncovered role a single
        // point of failure, and a ready successor clears it.
        let plan: Value = request
            .post("/api/succession-plans")
            .json(&json!({
                "role_title": "Head of Platform", "department": "engineering",
                "criticality": 5, "incumbent_pid": supervisor, "risk_of_loss": "high",
            }))
            .await
            .json();
        let plan_pid = plan["pid"].as_str().expect("plan pid").to_string();
        let exposed: Value = request
            .get("/api/workforce-intelligence/succession")
            .await
            .json();
        assert!(
            exposed["single_points_of_failure"]
                .as_array()
                .expect("spofs")
                .iter()
                .any(|s| s["plan_pid"].as_str() == Some(plan_pid.as_str())),
            "an uncovered critical role is a single point of failure"
        );

        let candidate: Value = request
            .post(&format!("/api/succession-plans/{plan_pid}/candidates"))
            .json(&json!({ "worker_pid": apprentice, "readiness": "ready_2y" }))
            .await
            .json();
        let candidate_pid = candidate["pid"]
            .as_str()
            .expect("candidate pid")
            .to_string();
        let developing: Value = request
            .get("/api/workforce-intelligence/succession")
            .await
            .json();
        assert_eq!(
            developing["by_coverage"]["developing"], 1,
            "a ready_2y bench is not cover"
        );

        request
            .put(&format!("/api/succession-candidates/{candidate_pid}"))
            .json(&json!({ "readiness": "ready_now" }))
            .await
            .assert_status_ok();
        let covered: Value = request
            .get("/api/workforce-intelligence/succession")
            .await
            .json();
        assert_eq!(covered["by_coverage"]["covered_now"], 1);
        assert!(
            !covered["single_points_of_failure"]
                .as_array()
                .expect("spofs")
                .iter()
                .any(|s| s["plan_pid"].as_str() == Some(plan_pid.as_str())),
            "a ready successor clears the exposure"
        );
    })
    .await;
}

/// The pipeline list gives each pipeline its own members' health, and one pipeline names each member and keeps a readiness a
/// stage move left out (the reads and the update moved to sqlx, WPM-T466).
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn pipeline_list_health_is_per_pipeline_and_members_are_named() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "PL-1", None).await;
        activate!(&request, &worker).await;

        let mut pids = Vec::new();
        for name in ["Pipeline with one", "Pipeline with none"] {
            let created: Value = request
                .post("/api/talent-pipelines")
                .json(&json!({ "name": name, "purpose": "hiring" }))
                .await
                .json();
            pids.push(created["pid"].as_str().expect("pipeline pid").to_string());
        }
        let member: Value = request
            .post(&format!("/api/talent-pipelines/{}/members", pids[0]))
            .json(&json!({ "subject_kind": "worker", "subject_pid": worker, "readiness": "ready_now" }))
            .await
            .json();
        let member_pid = member["pid"].as_str().expect("member pid").to_string();
        let moved: Value = request
            .post(&format!("/api/pipeline-members/{member_pid}/stage"))
            .json(&json!({ "to": "assessing" }))
            .await
            .json();
        assert_eq!(
            moved["readiness"], "ready_now",
            "a move without readiness keeps it"
        );

        let list: Value = request.get("/api/talent-pipelines").await.json();
        let health = |pid: &str| {
            list["pipelines"]
                .as_array()
                .expect("pipelines")
                .iter()
                .find(|p| p["pid"] == pid)
                .expect("listed")["health"]
                .clone()
        };
        assert_eq!(
            health(&pids[0])["live"],
            1,
            "the member counts in its own pipeline"
        );
        assert_eq!(health(&pids[1])["live"], 0, "and in no other");

        let one: Value = request
            .get(&format!("/api/talent-pipelines/{}", pids[0]))
            .await
            .json();
        assert_eq!(one["members"][0]["display_name"], "Test Worker PL-1");
        assert_eq!(one["members"][0]["stage"], "assessing");
    })
    .await;
}

/// Each refusal of a programme, a placement and its hours, an unknown kind to list by, and a placement withdrawn.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one programme, read top to bottom
async fn early_career_programmes_refuse_what_cannot_be_and_a_placement_can_be_withdrawn() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let learner = seed_worker!(&request, &org, format!("EC-{tag}-L"), None).await;
        let mentor = seed_worker!(&request, &org, format!("EC-{tag}-M"), None).await;
        for w in [&learner, &mentor] {
            activate!(&request, w).await;
        }
        for (body, why) in [
            (json!({ "name": "Zero", "kind": "graduate", "duration_months": 0 }), "duration"),
            (json!({ "name": "No hours", "kind": "apprenticeship", "duration_months": 12, "min_off_the_job_hours": 0 }), "hours"),
            (json!({ "name": "Level", "kind": "apprenticeship", "duration_months": 12, "min_off_the_job_hours": 10, "level": 0 }), "level"),
            (json!({ "name": "Provider", "kind": "graduate", "duration_months": 12, "provider_ref": "nope" }), "provider"),
        ] {
            assert_eq!(request.post("/api/early-career-programs").json(&body).await.status_code(), 422, "{why}");
        }
        assert_eq!(request.get("/api/early-career-programs?kind=sabbatical").await.status_code(), 422);
        let program: Value = request
            .post("/api/early-career-programs")
            .json(&json!({ "name": format!("Graduate {tag}"), "kind": "graduate", "duration_months": 24 }))
            .await
            .json();
        let placements = format!("/api/early-career-programs/{}/placements", program["pid"].as_str().unwrap());
        for (body, why) in [
            (json!({ "worker_pid": learner, "started_on": "2026-03-01", "ends_on": "2026-02-01" }), "ends before it starts"),
            (json!({ "worker_pid": learner, "supervisor_pid": learner, "started_on": "2026-03-01" }), "supervises themself"),
        ] {
            assert_eq!(request.post(&placements).json(&body).await.status_code(), 422, "{why}");
        }
        let placement: Value = request
            .post(&placements)
            .json(&json!({ "worker_pid": learner, "supervisor_pid": mentor, "started_on": "2026-03-01" }))
            .await
            .json();
        let pid = placement["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/program-placements/{pid}/status")).json(&json!({ "to": "active" })).await.assert_status_ok();
        let hours = format!("/api/program-placements/{pid}/hours");
        assert_eq!(request.post(&hours).json(&json!({ "hours": 0 })).await.status_code(), 422, "hours must be positive");
        request.post(&hours).json(&json!({ "hours": i32::MAX })).await.assert_status_ok();
        let over = request.post(&hours).json(&json!({ "hours": 1 })).await;
        assert_eq!(over.status_code(), 422, "the total would overflow: {}", over.text());
        let withdrawn = request.post(&format!("/api/program-placements/{pid}/status")).json(&json!({ "to": "withdrawn" })).await;
        assert_eq!(withdrawn.status_code(), 200, "{}", withdrawn.text());
        let mine: Value = request.get(&format!("/api/workers/{learner}/placements")).await.json();
        assert_eq!(mine["placements"][0]["outcome"], "withdrawn", "{mine}");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A plan that names one skill twice is refused, and a plan with an open item cannot be completed.
async fn a_plan_names_a_skill_once_and_completes_only_when_every_item_is_resolved() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("TP-{tag}"), None).await;
        activate!(&request, &worker).await;
        let skill: Value = request.post("/api/skills").json(&json!({ "name": format!("Terraform {tag}"), "category": "technical" })).await.json();
        let item = json!({ "skill_pid": skill["pid"], "current_level": 1, "target_level": 3, "method": "course" });
        let plans = format!("/api/workers/{worker}/development-plans");
        let twice = request.post(&plans).json(&json!({ "kind": "upskill", "items": [item, item] })).await;
        assert_eq!(twice.status_code(), 422, "{}", twice.text());
        assert!(twice.text().contains("more than once"), "{}", twice.text());

        let plan: Value = request.post(&plans).json(&json!({ "kind": "upskill", "items": [item] })).await.json();
        let status = format!("/api/development-plans/{}/status", plan["pid"].as_str().unwrap());
        request.post(&status).json(&json!({ "to": "active" })).await.assert_status_ok();
        let early = request.post(&status).json(&json!({ "to": "completed" })).await;
        assert_eq!(early.status_code(), 422, "{}", early.text());
        assert!(early.text().contains("still open"), "{}", early.text());
    })
    .await;
}
