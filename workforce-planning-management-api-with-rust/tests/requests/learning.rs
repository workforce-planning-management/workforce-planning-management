//! The learning & development round-trip: skills catalog + declared
//! proficiency + the matrix/gaps, learning paths + honest progress,
//! training analytics, and the mentorship lifecycle + overview.

use loco_rs::testing::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one seeded team, the whole L&D surface
async fn learning_round_trip() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let mentor = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &mentor).await;
        let mentee = seed_worker!(&request, &org, "E-2", None).await;
        activate!(&request, &mentee).await;
        let bystander = seed_worker!(&request, &org, "E-3", None).await;
        activate!(&request, &bystander).await;

        // ── Skills catalog + declared proficiency (validated 1-5).
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": "Rust", "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post("/api/skills")
                .json(&json!({ "name": "X", "category": "wizardry" }))
                .await
                .status_code(),
            422,
            "unknown category refused"
        );
        assert_eq!(
            request
                .put(&format!("/api/workers/{mentor}/skills"))
                .json(&json!({ "skill_pid": skill_pid, "proficiency": 9 }))
                .await
                .status_code(),
            422,
            "proficiency is 1-5"
        );
        request
            .put(&format!("/api/workers/{mentor}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 5 }))
            .await
            .assert_status_ok();
        // A gap: mentee at 2, target 4.
        request
            .put(&format!("/api/workers/{mentee}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 2, "target": 4 }))
            .await
            .assert_status_ok();
        // Upsert: re-declare mentor to 4 (one row).
        request
            .put(&format!("/api/workers/{mentor}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 4 }))
            .await
            .assert_status_ok();
        let listed: Value = request.get(&format!("/api/workers/{mentor}/skills")).await.json();
        assert_eq!(listed.as_array().unwrap().len(), 1, "upsert keeps one row");

        // ── Skills matrix: engineering has both, one below target.
        let matrix: Value = request.get("/api/learning/skills-matrix").await.json();
        let cell = matrix["matrix"].as_array().unwrap().iter()
            .find(|c| c["department"] == "engineering" && c["skill"] == "Rust")
            .expect("matrix cell").clone();
        assert_eq!(cell["workers"], 2);
        assert_eq!(cell["below_target"], 1);
        assert_eq!(matrix["gaps"].as_array().unwrap().len(), 1);
        assert_eq!(matrix["gaps"][0]["skill"], "Rust");

        // ── Learning path + honest progress.
        let path: Value = request
            .post("/api/learning-paths")
            .json(&json!({
                "name": "Backend basics",
                "steps": [
                    { "course_ref": "course:11111111-1111-4111-8111-111111111111", "title": "Intro" },
                    { "course_ref": "course:22222222-2222-4222-8222-222222222222", "title": "Advanced" },
                ],
            }))
            .await
            .json();
        let path_pid = path["pid"].as_str().unwrap().to_string();
        request
            .post(&format!("/api/learning-paths/{path_pid}/enrollments"))
            .json(&json!({ "worker_pid": mentee }))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .post(&format!("/api/learning-paths/{path_pid}/enrollments"))
                .json(&json!({ "worker_pid": mentee }))
                .await
                .status_code(),
            422,
            "double enrollment refused"
        );
        // Mentee completes one of the two courses.
        let enrollment: Value = request
            .post(&format!("/api/workers/{mentee}/training-enrollments"))
            .json(&json!({ "course_ref": "course:11111111-1111-4111-8111-111111111111" }))
            .await
            .json();
        let e_pid = enrollment["pid"].as_str().unwrap();
        request
            .put(&format!("/api/training-enrollments/{e_pid}"))
            .json(&json!({ "status": "completed", "completed_on": "2026-07-01" }))
            .await
            .assert_status_ok();
        let progress: Value = request
            .get(&format!("/api/learning-paths/{path_pid}/progress"))
            .await
            .json();
        let member = &progress["members"][0];
        assert_eq!(member["completed_steps"], 1);
        assert_eq!(member["total_steps"], 2);

        // ── Training analytics: engineering completion 1/1.
        let analytics: Value = request.get("/api/learning/training-analytics").await.json();
        let dept = analytics["departments"].as_array().unwrap().iter()
            .find(|d| d["department"] == "engineering")
            .expect("dept row").clone();
        assert_eq!(dept["completion_rate"]["numerator"], 1);
        assert_eq!(dept["completion_rate"]["denominator"], 1);

        // ── Mentorship lifecycle + sessions + overview.
        assert_eq!(
            request
                .post("/api/mentorships")
                .json(&json!({ "mentor_pid": mentor, "mentee_pid": mentor, "focus": "self" }))
                .await
                .status_code(),
            422,
            "mentor != mentee"
        );
        let mentorship: Value = request
            .post("/api/mentorships")
            .json(&json!({ "mentor_pid": mentor, "mentee_pid": mentee, "focus": "Rust growth" }))
            .await
            .json();
        let m_pid = mentorship["pid"].as_str().unwrap().to_string();
        // A session before activation is refused.
        assert_eq!(
            request
                .post(&format!("/api/mentorships/{m_pid}/sessions"))
                .json(&json!({ "held_on": "2026-07-10", "notes": "kickoff" }))
                .await
                .status_code(),
            422,
            "no sessions on a proposed mentorship"
        );
        // Skipping proposed→completed is refused.
        assert_eq!(
            request
                .post(&format!("/api/mentorships/{m_pid}/status"))
                .json(&json!({ "to": "completed" }))
                .await
                .status_code(),
            422
        );
        let activated: Value = request
            .post(&format!("/api/mentorships/{m_pid}/status"))
            .json(&json!({ "to": "active" }))
            .await
            .json();
        assert!(activated["started_on"].is_string());
        request
            .post(&format!("/api/mentorships/{m_pid}/sessions"))
            .json(&json!({ "held_on": "2026-07-10", "notes": "kickoff" }))
            .await
            .assert_status_ok();
        let detail: Value = request.get(&format!("/api/mentorships/{m_pid}")).await.json();
        assert_eq!(detail["sessions"].as_array().unwrap().len(), 1);

        let overview: Value = request
            .get("/api/learning/mentorship-overview?days=3650")
            .await
            .json();
        assert_eq!(overview["active_pairings"], 1);
        assert_eq!(overview["mentor_load"][0]["active_mentees"], 1);
        // The bystander is the only unmatched active worker.
        let unmatched: Vec<&str> = overview["unmatched_workers"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|u| u["display_name"].as_str())
            .collect();
        assert!(unmatched.contains(&"Test Worker E-3"));
        assert!(!unmatched.contains(&"Test Worker E-1"));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn capability_analysis_reports_skill_depth() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let a = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &a).await;
        let b = seed_worker!(&request, &org, "E-2", None).await;
        activate!(&request, &b).await;

        let mut pids = Vec::new();
        for name in ["Rust", "Go", "Cobol"] {
            let skill: Value = request
                .post("/api/skills")
                .json(&json!({ "name": name, "category": "technical" }))
                .await
                .json();
            pids.push(skill["pid"].as_str().unwrap().to_string());
        }
        // Rust: two proficient (adequate). Go: one proficient (thin).
        // Cobol: declared at 1 only (no_proficient). Nothing undeclared.
        for (worker, skill, level) in [
            (&a, &pids[0], 4),
            (&b, &pids[0], 3),
            (&a, &pids[1], 5),
            (&b, &pids[2], 1),
        ] {
            request
                .put(&format!("/api/workers/{worker}/skills"))
                .json(&json!({ "skill_pid": skill, "proficiency": level }))
                .await
                .assert_status_ok();
        }

        let view: Value = request
            .get("/api/workforce-intelligence/capability-analysis")
            .await
            .json();
        assert_eq!(view["thresholds"]["min_proficiency"], 3);
        assert_eq!(view["thresholds"]["min_depth"], 2);
        let status_of = |name: &str| {
            view["skills"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["skill"] == name)
                .map(|s| s["status"].clone())
        };
        assert_eq!(status_of("Rust"), Some(json!("adequate")));
        assert_eq!(status_of("Go"), Some(json!("thin")));
        assert_eq!(status_of("Cobol"), Some(json!("no_proficient")));

        // A stricter bar re-grades the same declarations.
        let strict: Value = request
            .get("/api/workforce-intelligence/capability-analysis?min_proficiency=4&min_depth=1")
            .await
            .json();
        let rust = strict["skills"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["skill"] == "Rust")
            .unwrap();
        assert_eq!(rust["proficient"], 1, "only the level-4 declaration clears the bar");
        assert_eq!(rust["status"], "adequate", "depth bar of one is met");

        assert_eq!(
            request
                .get("/api/workforce-intelligence/capability-analysis?min_proficiency=9")
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .get("/api/workforce-intelligence/capability-analysis?min_depth=0")
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
async fn workforce_metrics_report_defined_numbers() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        // Two workers hired 2026-01-05 (the helper's fixed date).
        let a = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &a).await;
        let _b = seed_worker!(&request, &org, "E-2", None).await;

        let view: Value = request
            .get("/api/workforce-intelligence/metrics?from=2026-01-01&to=2026-12-31")
            .await
            .json();
        assert_eq!(view["period"]["from"], "2026-01-01");
        assert_eq!(view["headcount"]["opening"], 0, "nobody employed on 2025-12-31");
        assert!(view["headcount"]["closing"].as_u64().unwrap() >= 2);
        assert!(view["starters"].as_u64().unwrap() >= 2, "both hired in the period");
        assert_eq!(view["leavers"], 0);
        assert!(
            view["time_to_fill"].is_null() || view["time_to_fill"]["requisitions"].is_u64(),
            "time-to-fill is a summary or absent, never a guess"
        );
        assert!(view["definitions"]["headcount"].is_string());

        // One definition of headcount: `/overview` and `/metrics` agree.
        let today_metrics: Value = request
            .get("/api/workforce-intelligence/metrics")
            .await
            .json();
        let overview: Value = request
            .get("/api/workforce-intelligence/overview")
            .await
            .json();
        assert_eq!(
            overview["headcount"], today_metrics["headcount"]["closing"],
            "overview and metrics count the same employed population"
        );

        assert_eq!(
            request
                .get("/api/workforce-intelligence/metrics?from=2026-12-31&to=2026-01-01")
                .await
                .status_code(),
            422,
            "inverted period refused"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn headcount_snapshots_are_recorded_idempotently() {
    use workforce_planning_management_service::tasks::snapshot::run_snapshot;

    request::<App, _, _>(|request, ctx| async move {
        let org = an_org();
        let a = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &a).await;

        let first_day = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let written = run_snapshot(&ctx.db, first_day).await.expect("first snapshot");
        assert!(written >= 1, "the seeded worker's department is recorded");
        assert_eq!(
            run_snapshot(&ctx.db, first_day).await.expect("re-run"),
            0,
            "a re-run for the same date writes nothing"
        );

        let history: Value = request
            .get(&format!(
                "/api/workforce-intelligence/headcount-history?organization={org}"
            ))
            .await
            .json();
        let rows = history["snapshots"].as_array().unwrap();
        let row = rows.iter().find(|r| r["department"] == "engineering").unwrap();
        assert_eq!(row["as_of"], "2026-07-01");
        assert!(row["headcount"].as_u64().unwrap() >= 1);
        assert!(row["starters"].is_null(), "the first snapshot has no window");

        // A later snapshot measures starters/leavers from the earlier one.
        run_snapshot(&ctx.db, chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap())
            .await
            .expect("second snapshot");
        let later: Value = request
            .get(&format!(
                "/api/workforce-intelligence/headcount-history?organization={org}&from=2026-08-01"
            ))
            .await
            .json();
        let later_row = &later["snapshots"].as_array().unwrap()[0];
        assert_eq!(later_row["as_of"], "2026-08-01");
        assert!(later_row["starters"].is_u64(), "a known window reports a count");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn role_profiles_hold_required_skills() {
    request::<App, _, _>(|request, _ctx| async move {
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": "Triage", "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();

        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": "  Triage Nurse ", "source_ref": "test:fixture" }))
            .await
            .json();
        let pid = profile["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post("/api/role-profiles")
                .json(&json!({ "job_title": "Triage Nurse" }))
                .await
                .status_code(),
            422,
            "one live profile per job title (after trimming)"
        );

        let put = |level: i32, importance: &str| {
            request
                .put(&format!("/api/role-profiles/{pid}/requirements"))
                .json(&json!({ "skill_pid": skill_pid, "min_proficiency": level, "importance": importance }))
        };
        put(3, "critical").await.assert_status_ok();
        put(4, "important").await.assert_status_ok(); // upsert: still one row
        assert_eq!(put(9, "critical").await.status_code(), 422, "proficiency is 1-5");
        assert_eq!(put(3, "essential").await.status_code(), 422, "importance is a closed set");

        let detail: Value = request.get(&format!("/api/role-profiles/{pid}")).await.json();
        assert_eq!(detail["job_title"], "Triage Nurse");
        let reqs = detail["requirements"].as_array().unwrap();
        assert_eq!(reqs.len(), 1, "an upsert keeps one row per skill");
        assert_eq!(reqs[0]["min_proficiency"], 4);
        assert_eq!(reqs[0]["skill"], "Triage");

        let listed: Value = request.get("/api/role-profiles").await.json();
        let row = listed
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["pid"] == pid)
            .unwrap();
        assert_eq!(row["requirement_count"], 1);

        request
            .delete(&format!("/api/role-profiles/{pid}/requirements/{skill_pid}"))
            .await
            .assert_status_ok();
        let after: Value = request.get(&format!("/api/role-profiles/{pid}")).await.json();
        assert!(after["requirements"].as_array().unwrap().is_empty());
    })
    .await;
}
