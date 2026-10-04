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
async fn workforce_insights_flag_a_shrinking_headcount() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let mut pids = Vec::new();
        for n in ["E-1", "E-2", "E-3"] {
            let pid = seed_worker!(&request, &org, n, None).await;
            activate!(&request, &pid).await;
            pids.push(pid);
        }

        // Shape: a 200 with an `insights` array and a `thresholds` object.
        let before: Value = request
            .get("/api/workforce-intelligence/insights?from=2026-01-01&to=2026-12-31")
            .await
            .json();
        assert!(before["insights"].is_array());
        assert!(before["thresholds"].is_object());
        assert!(before["thresholds"]["headcount_change"]["value"].is_number());

        // Terminate two of three today (termination stamps today's date).
        for pid in &pids[..2] {
            for to in ["offboarding", "terminated"] {
                request
                    .post(&format!("/api/workers/{pid}/status"))
                    .json(&json!({ "to": to }))
                    .await
                    .assert_status_ok();
            }
        }

        // Period of a single day: opening is yesterday (3), closing today (1).
        let today = chrono::Utc::now().date_naive();
        let response = request
            .get(&format!(
                "/api/workforce-intelligence/insights?from={today}&to={today}"
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        let view: Value = response.json();
        assert!(view["thresholds"].is_object());
        let codes: Vec<&str> = view["insights"]
            .as_array()
            .expect("insights array")
            .iter()
            .filter_map(|i| i["code"].as_str())
            .collect();
        assert!(
            codes.contains(&"headcount_shrinking"),
            "3 -> 1 headcount must be flagged, got {codes:?}"
        );

        assert_eq!(
            request
                .get("/api/workforce-intelligence/insights?from=2026-12-31&to=2026-01-01")
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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn role_gap_grades_declarations_against_a_role() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &worker).await;

        let mut skill_pids = Vec::new();
        for name in ["Gap-Met", "Gap-Below", "Gap-Undeclared"] {
            let skill: Value = request
                .post("/api/skills")
                .json(&json!({ "name": name, "category": "technical" }))
                .await
                .json();
            skill_pids.push(skill["pid"].as_str().unwrap().to_string());
        }
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": "Gap Test Role" }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();
        for pid in &skill_pids {
            request
                .put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": pid, "min_proficiency": 3, "importance": "critical" }))
                .await
                .assert_status_ok();
        }
        for (pid, level) in [(&skill_pids[0], 4), (&skill_pids[1], 2)] {
            request
                .put(&format!("/api/workers/{worker}/skills"))
                .json(&json!({ "skill_pid": pid, "proficiency": level }))
                .await
                .assert_status_ok();
        }

        let gap: Value = request
            .get(&format!("/api/workers/{worker}/role-gap?role_profile_pid={role}"))
            .await
            .json();
        let grade_of = |name: &str| {
            gap["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["skill"] == name)
                .map(|r| (r["grade"].clone(), r["shortfall"].clone()))
        };
        assert_eq!(grade_of("Gap-Met"), Some((json!("met"), Value::Null)));
        assert_eq!(grade_of("Gap-Below"), Some((json!("below"), json!(1))));
        assert_eq!(
            grade_of("Gap-Undeclared"),
            Some((json!("undeclared"), Value::Null)),
            "unknown is not a numeric shortfall"
        );
        assert_eq!(gap["critical_met"]["numerator"], 1);
        assert_eq!(gap["critical_met"]["denominator"], 3);

        let workforce: Value = request.get(&format!("/api/role-profiles/{role}/gap")).await.json();
        let met_row = workforce["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["skill"] == "Gap-Met")
            .unwrap();
        assert!(met_row["meeting"].as_u64().unwrap() >= 1);
        assert!(workforce["headcount"].as_u64().unwrap() >= 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn cpd_ledger_tracks_progress_and_registrations() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &worker).await;

        let requirement: Value = request
            .post("/api/cpd-requirements")
            .json(&json!({
                "name": "Annual CPD", "unit": "hours", "required": 10,
                "period_start": "2026-01-01", "period_end": "2026-12-31",
            }))
            .await
            .json();
        let requirement_pid = requirement["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post("/api/cpd-requirements")
                .json(&json!({
                    "name": "Bad", "unit": "hours", "required": 10,
                    "period_start": "2026-12-31", "period_end": "2026-01-01",
                }))
                .await
                .status_code(),
            422,
            "period must be ordered"
        );

        let entry = |amount: f64| {
            json!({
                "entry_date": "2026-03-01", "activity": "Safeguarding course",
                "category": "course", "unit": "hours", "amount": amount,
            })
        };
        let first: Value = request
            .post(&format!("/api/workers/{worker}/cpd-entries"))
            .json(&entry(6.5))
            .await
            .json();
        request
            .post(&format!("/api/workers/{worker}/cpd-entries"))
            .json(&entry(2.0))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/cpd-entries"))
                .json(&entry(0.0))
                .await
                .status_code(),
            422,
            "a zero amount is refused"
        );
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/cpd-entries"))
                .json(&json!({
                    "entry_date": "2999-01-01", "activity": "x", "category": "course",
                    "unit": "hours", "amount": 1,
                }))
                .await
                .status_code(),
            422,
            "future-dated CPD is refused"
        );

        let progress = |view: &Value| {
            view["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["requirement_pid"] == requirement_pid)
                .cloned()
                .unwrap()
        };
        let view: Value = request.get(&format!("/api/workers/{worker}/cpd-progress")).await.json();
        let row = progress(&view);
        assert_eq!(row["recorded"], 8.5);
        assert_eq!(row["verified"], 0.0);
        assert_eq!(row["remaining"], 1.5);
        assert_eq!(row["met"], false);

        // Verifying an entry moves the verified total; a second verify is refused.
        let entry_pid = first["pid"].as_str().unwrap();
        request
            .post(&format!("/api/cpd-entries/{entry_pid}/verify"))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .post(&format!("/api/cpd-entries/{entry_pid}/verify"))
                .await
                .status_code(),
            422
        );
        let after: Value = request.get(&format!("/api/workers/{worker}/cpd-progress")).await.json();
        assert_eq!(progress(&after)["verified"], 6.5);

        // Registrations carry an expiry status.
        request
            .post(&format!("/api/workers/{worker}/registrations"))
            .json(&json!({ "body": "Test Regulator", "expires_on": "2000-01-01" }))
            .await
            .assert_status_ok();
        let regs: Value = request.get(&format!("/api/workers/{worker}/registrations")).await.json();
        assert_eq!(regs[0]["status"], "expired");

        let overview: Value = request.get("/api/cpd/overview").await.json();
        assert!(overview["registrations"]["expired"].as_u64().unwrap() >= 1);
        assert!(overview["requirements"].as_array().unwrap().iter().any(|r| r["requirement_pid"] == requirement_pid));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn internal_mobility_matches_own_skills_and_keeps_interest_private() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1", None).await; // job title "Engineer"
        activate!(&request, &worker).await;

        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": "Mobility-Skill", "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/workers/{worker}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 4 }))
            .await
            .assert_status_ok();

        let mut roles = Vec::new();
        for (title, min) in [("Mobility Fit Role", 3), ("Mobility Stretch Role", 5)] {
            let profile: Value = request
                .post("/api/role-profiles")
                .json(&json!({ "job_title": title }))
                .await
                .json();
            let role = profile["pid"].as_str().unwrap().to_string();
            request
                .put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": skill_pid, "min_proficiency": min, "importance": "critical" }))
                .await
                .assert_status_ok();
            roles.push(role);
        }

        let matches: Value = request.get(&format!("/api/workers/{worker}/role-matches")).await.json();
        let titles: Vec<&str> = matches["roles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["job_title"].as_str().unwrap())
            .collect();
        let fit = titles.iter().position(|t| *t == "Mobility Fit Role").unwrap();
        let stretch = titles.iter().position(|t| *t == "Mobility Stretch Role").unwrap();
        assert!(fit < stretch, "the better-fitting role is listed first");

        // Express interest; a duplicate is refused; both-or-neither target is refused.
        let interest = |body: Value| request.post(&format!("/api/workers/{worker}/mobility-interests")).json(&body);
        let created: Value = interest(json!({ "role_profile_pid": roles[0], "note": "keen" })).await.json();
        assert_eq!(interest(json!({ "role_profile_pid": roles[0] })).await.status_code(), 422);
        assert_eq!(
            interest(json!({ "role_profile_pid": roles[0], "requisition_pid": roles[1] })).await.status_code(),
            422
        );
        assert_eq!(interest(json!({})).await.status_code(), 422);

        // Others see counts, never who.
        let summary: Value = request.get("/api/mobility/interest-summary").await.json();
        let row = summary["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["target_pid"] == roles[0])
            .unwrap();
        assert!(row["interested"].as_u64().unwrap() >= 1);
        assert!(row.get("worker_pid").is_none(), "no worker is named");

        let mine: Value = request.get(&format!("/api/workers/{worker}/mobility-interests")).await.json();
        assert_eq!(mine[0]["title"], "Mobility Fit Role");

        let pid = created["pid"].as_str().unwrap();
        request.delete(&format!("/api/mobility-interests/{pid}")).await.assert_status_ok();
        let after: Value = request.get(&format!("/api/workers/{worker}/mobility-interests")).await.json();
        assert!(after.as_array().unwrap().is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn lms_completions_update_enrollments_and_cpd_idempotently() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &worker).await;
        let detail: Value = request.get(&format!("/api/workers/{worker}")).await.json();
        let person_ref = detail["person_ref"].as_str().unwrap().to_string();

        let course = "course:33333333-3333-4333-8333-333333333333";
        let event = |reference: &str| {
            json!({
                "person_ref": person_ref, "course_ref": course,
                "completed_on": "2026-09-01", "certificate_expires_on": "2027-09-01",
                "hours": 2.5, "external_ref": reference,
            })
        };
        let post = |events: Vec<Value>| {
            request
                .post("/api/lms/completions")
                .json(&json!({ "completions": events }))
        };

        let first: Value = post(vec![event("evt-1")]).await.json();
        assert_eq!(first["summary"]["applied"], 1);
        assert_eq!(first["results"][0]["enrollment"], "created");
        assert_eq!(first["results"][0]["cpd"], "recorded");

        // Redelivery of the same event changes nothing.
        let again: Value = post(vec![event("evt-1")]).await.json();
        assert_eq!(again["summary"]["unchanged"], 1);
        assert_eq!(again["results"][0]["cpd"], "already_recorded");

        // The enrollment is completed with its certificate expiry; the CPD
        // ledger holds one LMS entry.
        let training: Value = request.get(&format!("/api/workers/{worker}/training-enrollments")).await.json();
        let row = training.as_array().unwrap().iter().find(|t| t["course_ref"] == course).unwrap();
        assert_eq!(row["status"], "completed");
        assert_eq!(row["certificate_expires_on"], "2027-09-01");
        let ledger: Value = request.get(&format!("/api/workers/{worker}/cpd-entries")).await.json();
        let lms: Vec<&Value> = ledger.as_array().unwrap().iter().filter(|e| e["source"] == "lms").collect();
        assert_eq!(lms.len(), 1, "an event lands exactly one CPD entry");
        assert_eq!(lms[0]["amount"], 2.5);

        // Bad events are reported per item and do not poison the batch.
        let mixed: Value = post(vec![
            json!({ "person_ref": "person:99999999-9999-4999-8999-999999999999", "course_ref": course,
                    "completed_on": "2026-09-01", "external_ref": "evt-x" }),
            json!({ "person_ref": person_ref, "course_ref": "not-a-urn",
                    "completed_on": "2026-09-01", "external_ref": "evt-y" }),
            event("evt-2"),
        ])
        .await
        .json();
        assert_eq!(mixed["results"][0]["result"], "unmatched");
        assert_eq!(mixed["results"][1]["result"], "invalid");
        assert_eq!(mixed["results"][2]["result"], "applied");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn change_tracker_reports_aggregate_readiness() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        // Two employed "Engineer"s (the seed helper's job title); one declares the rising skill.
        let a = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &a).await;
        let b = seed_worker!(&request, &org, "E-2", None).await;
        activate!(&request, &b).await;

        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": "Prompting", "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/workers/{a}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 4 }))
            .await
            .assert_status_ok();

        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": "Engineer" }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();

        let initiative: Value = request
            .post("/api/change-initiatives")
            .json(&json!({ "name": "Code assistant rollout", "kind": "ai_assistance" }))
            .await
            .json();
        let id = initiative["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post("/api/change-initiatives")
                .json(&json!({ "name": "x", "kind": "magic" }))
                .await
                .status_code(),
            422
        );
        request
            .put(&format!("/api/change-initiatives/{id}/role-impacts"))
            .json(&json!({ "role_profile_pid": role, "impact": "reshaped", "timeframe": "within_1y" }))
            .await
            .assert_status_ok();
        request
            .put(&format!("/api/change-initiatives/{id}/skill-shifts"))
            .json(&json!({ "skill_pid": skill_pid, "direction": "rising" }))
            .await
            .assert_status_ok();

        let view: Value = request
            .get(&format!("/api/change-initiatives/{id}/readiness"))
            .await
            .json();
        assert!(view["affected_workers"].as_u64().unwrap() >= 2);
        let rising = &view["rising_skills"][0];
        assert_eq!(rising["skill"], "Prompting");
        assert!(rising["meeting"].as_u64().unwrap() >= 1);
        assert!(rising["undeclared"].as_u64().unwrap() >= 1, "unknown is not below");
        let text = view.to_string();
        assert!(!text.contains(&a) && !text.contains(&b), "no worker is named");

        // Lifecycle: draft -> completed is refused; closed initiatives are read-only.
        let status = |to: &str| {
            request
                .post(&format!("/api/change-initiatives/{id}/status"))
                .json(&json!({ "to": to }))
        };
        assert_eq!(status("completed").await.status_code(), 422);
        status("active").await.assert_status_ok();
        status("completed").await.assert_status_ok();
        assert_eq!(
            request
                .put(&format!("/api/change-initiatives/{id}/skill-shifts"))
                .json(&json!({ "skill_pid": skill_pid, "direction": "declining" }))
                .await
                .status_code(),
            422,
            "a closed initiative cannot be edited"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn workforce_plan_forecasts_gaps_and_alignment() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let a = seed_worker!(&request, &org, "E-1", None).await; // department "engineering"
        activate!(&request, &a).await;
        let b = seed_worker!(&request, &org, "E-2", None).await;
        activate!(&request, &b).await;

        // A role profile needing one skill; one worker is proficient.
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": "Planning-Skill", "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/workers/{a}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 4 }))
            .await
            .assert_status_ok();
        request
            .put(&format!("/api/workers/{b}/skills"))
            .json(&json!({ "skill_pid": skill_pid, "proficiency": 2 }))
            .await
            .assert_status_ok();
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": "Planning Engineer" }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/role-profiles/{role}/requirements"))
            .json(&json!({ "skill_pid": skill_pid, "min_proficiency": 3, "importance": "critical" }))
            .await
            .assert_status_ok();

        // Validation: horizon order; a line outside the horizon is refused.
        let plan_body = |name: &str, attrition: Option<i32>| {
            json!({
                "name": name, "organization_ref": org,
                "horizon_start": "2026-01-01", "horizon_end": "2027-12-31",
                "attrition_bp": attrition,
            })
        };
        assert_eq!(
            request
                .post("/api/workforce-plans")
                .json(&json!({ "name": "x", "organization_ref": org,
                               "horizon_start": "2027-01-01", "horizon_end": "2026-01-01" }))
                .await
                .status_code(),
            422
        );
        let plan: Value = request.post("/api/workforce-plans").json(&plan_body("FY27 growth", Some(1000))).await.json();
        let plan_pid = plan["pid"].as_str().unwrap().to_string();
        let set_line = |body: Value| request.put(&format!("/api/workforce-plans/{plan_pid}/demand-lines")).json(&body);
        assert_eq!(
            set_line(json!({ "department": "engineering", "target_on": "2030-01-01", "target_headcount": 5 }))
                .await
                .status_code(),
            422,
            "outside the horizon"
        );
        // Dept-level demand at one date, and a role-level line at another.
        set_line(json!({ "department": "engineering", "target_on": "2027-06-30", "target_headcount": 5 }))
            .await
            .assert_status_ok();
        let role_line: Value = set_line(json!({
            "department": "engineering", "role_profile_pid": role,
            "target_on": "2027-09-30", "target_headcount": 4,
        }))
        .await
        .json();

        let forecast: Value = request.get(&format!("/api/workforce-plans/{plan_pid}/forecast")).await.json();
        assert_eq!(forecast["assumptions"]["attrition_source"], "plan_assumption");
        assert_eq!(forecast["assumptions"]["hires_assumed"], 0);
        let first = &forecast["departments"][0];
        assert_eq!(first["department"], "engineering");
        assert_eq!(first["planned_demand"], 5);
        assert!(first["opening_headcount"].as_u64().unwrap() >= 2);
        let gap = first["headcount_gap"].as_i64().unwrap();
        assert_eq!(
            gap,
            5 - first["projected_supply"].as_i64().unwrap(),
            "gap is demand minus projected supply"
        );
        if gap > 0 {
            assert_eq!(first["levers"][0], "hire");
        }
        let competency = &forecast["departments"][1]["competency_gaps"][0];
        assert_eq!(competency["skill"], "Planning-Skill");
        assert_eq!(competency["needed"], 4);
        assert!(competency["proficient_now"].as_u64().unwrap() >= 1);
        assert!(competency["reskill_pool"].as_u64().unwrap() >= 1, "the worker declared below the bar");

        // With no attrition assumption and no snapshots: insufficient history, not a guess.
        let bare: Value = request.post("/api/workforce-plans").json(&plan_body("No assumption", None)).await.json();
        let bare_pid = bare["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/workforce-plans/{bare_pid}/demand-lines"))
            .json(&json!({ "department": "engineering", "target_on": "2027-06-30", "target_headcount": 3 }))
            .await
            .assert_status_ok();
        let none: Value = request.get(&format!("/api/workforce-plans/{bare_pid}/forecast")).await.json();
        assert_eq!(none["assumptions"]["attrition_source"], "insufficient_history");
        assert!(none["departments"][0]["projected_supply"].is_null());
        assert!(none["departments"][0]["headcount_gap"].is_null());

        // Alignment: one objective serves the dept line; another has no demand.
        let served: Value = request
            .post(&format!("/api/workforce-plans/{plan_pid}/objectives"))
            .json(&json!({ "title": "Launch the new service" }))
            .await
            .json();
        request
            .post(&format!("/api/workforce-plans/{plan_pid}/objectives"))
            .json(&json!({ "title": "Enter a new market" }))
            .await
            .assert_status_ok();
        let detail: Value = request.get(&format!("/api/workforce-plans/{plan_pid}")).await.json();
        let dept_line = detail["demand_lines"][0]["pid"].as_str().unwrap();
        request
            .put(&format!("/api/workforce-plans/{plan_pid}/demand-lines/{dept_line}/objectives"))
            .json(&json!({ "objective_pids": [served["pid"]] }))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .put(&format!("/api/workforce-plans/{plan_pid}/demand-lines/{dept_line}/objectives"))
                .json(&json!({ "objective_pids": [uuid::Uuid::new_v4()] }))
                .await
                .status_code(),
            422,
            "objectives must belong to the plan"
        );
        let view: Value = request.get(&format!("/api/workforce-plans/{plan_pid}/alignment")).await.json();
        assert_eq!(view["planned_headcount"], 9);
        assert_eq!(view["aligned_share"]["numerator"], 5);
        assert_eq!(view["aligned_share"]["denominator"], 9);
        assert_eq!(view["unresourced_objectives"][0], "Enter a new market");
        assert_eq!(view["unaligned_demand_lines"].as_array().unwrap().len(), 1);
        let _ = role_line;

        // Lifecycle: one active plan per organization; archived plans are read-only.
        let status = |pid: &str, to: &str| {
            request.post(&format!("/api/workforce-plans/{pid}/status")).json(&json!({ "to": to }))
        };
        status(&plan_pid, "active").await.assert_status_ok();
        assert_eq!(status(&bare_pid, "active").await.status_code(), 422, "one active plan per organization");
        status(&plan_pid, "archived").await.assert_status_ok();
        assert_eq!(
            set_line(json!({ "department": "engineering", "target_on": "2027-06-30", "target_headcount": 9 }))
                .await
                .status_code(),
            422,
            "an archived plan is read-only"
        );
        status(&bare_pid, "active").await.assert_status_ok();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn workforce_plan_costs_hiring_against_a_budget() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        for n in 1..=2 {
            let w = seed_worker!(&request, &org, &format!("E-{n}"), None).await; // "engineering"
            activate!(&request, &w).await;
        }
        // A role with a salary benchmark.
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": "Costed Engineer" }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();
        request
            .post("/api/benchmarks")
            .json(&json!({
                "job_title": "Costed Engineer", "currency": "GBP",
                "min_minor": 3_000_000, "median_minor": 4_000_000, "max_minor": 5_000_000,
                "source": "test", "as_of": "2026-01-01",
            }))
            .await
            .assert_status_ok();

        // Budget and currency must be given together; on-cost is bounded.
        let plan_body = |extra: Value| {
            let mut body = json!({
                "name": "Costed plan", "organization_ref": org,
                "horizon_start": "2026-01-01", "horizon_end": "2027-12-31",
                "attrition_bp": 0, "on_cost_bp": 2500,
            });
            body.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            body
        };
        assert_eq!(
            request.post("/api/workforce-plans").json(&plan_body(json!({ "budget_minor": 100 }))).await.status_code(),
            422
        );
        assert_eq!(
            request
                .post("/api/workforce-plans")
                .json(&plan_body(json!({ "budget_minor": 100, "budget_currency": "gbp" })))
                .await
                .status_code(),
            422
        );
        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&plan_body(json!({ "budget_minor": 20_000_000, "budget_currency": "GBP" })))
            .await
            .json();
        let plan_pid = plan["pid"].as_str().unwrap().to_string();
        // Demand 5 against 2 employed and no attrition ⇒ 3 hires at the benchmark.
        request
            .put(&format!("/api/workforce-plans/{plan_pid}/demand-lines"))
            .json(&json!({ "department": "engineering", "role_profile_pid": role, "target_on": "2027-06-30", "target_headcount": 5 }))
            .await
            .assert_status_ok();
        // A department with no benchmark and no cohort cannot be costed.
        request
            .put(&format!("/api/workforce-plans/{plan_pid}/demand-lines"))
            .json(&json!({ "department": "nowhere", "target_on": "2027-06-30", "target_headcount": 2 }))
            .await
            .assert_status_ok();

        let view: Value = request.get(&format!("/api/workforce-plans/{plan_pid}/cost")).await.json();
        assert_eq!(view["currency"], "GBP");
        assert_eq!(view["salary_visible"], true);
        assert_eq!(view["assumptions"]["on_cost_bp"], 2500);
        let eng = view["groups"].as_array().unwrap().iter().find(|g| g["department"] == "engineering").unwrap();
        assert!(eng["hires_needed"].as_u64().unwrap() >= 3);
        assert_eq!(eng["unit_cost_source"], "benchmark");
        let hires = eng["hires_needed"].as_i64().unwrap();
        assert_eq!(eng["annual_cost_minor"], hires * 5_000_000, "benchmark 4.0m + 25% on-cost");
        let nowhere = view["groups"].as_array().unwrap().iter().find(|g| g["department"] == "nowhere").unwrap();
        assert_eq!(nowhere["reason"], "no_unit_cost", "no benchmark and too small a cohort ⇒ not guessed");
        assert!(view["uncosted_groups"].as_u64().unwrap() >= 1);
        assert_eq!(view["total_annual_cost_minor"], eng["annual_cost_minor"]);
        let fit = &view["affordability"];
        assert_eq!(fit["budget_minor"], 20_000_000);
        assert_eq!(fit["remaining_minor"].as_i64().unwrap(), 20_000_000 - eng["annual_cost_minor"].as_i64().unwrap());

        assert_eq!(
            request.get(&format!("/api/workforce-plans/{plan_pid}/cost?currency=pounds")).await.status_code(),
            422
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn capability_framework_import_and_progression() {
    use workforce_planning_management_service::rules::framework::LevelMapping;
    use workforce_planning_management_service::tasks::import_framework::import_pcf;

    request::<App, _, _>(|request, ctx| async move {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pcf-mini");
        let report = import_pcf(&ctx.db, &fixture, LevelMapping::Identity, false)
            .await
            .expect("import");
        assert_eq!(report.levels_read, 4);
        assert_eq!(report.retired_skipped, 1);
        assert_eq!(report.profiles_created, 3);
        assert_eq!(report.requirements_created, 6, "2 + 3 + 1 baselined skill lines");
        assert_eq!(report.requirements_skipped_no_baseline, 1, "no invented level");

        // Idempotent: a second run creates nothing and refreshes everything.
        let again = import_pcf(&ctx.db, &fixture, LevelMapping::Identity, false)
            .await
            .expect("re-import");
        assert_eq!(again.profiles_created, 0);
        assert_eq!(again.profiles_updated, 3);
        assert_eq!(again.requirements_created, 0);
        assert_eq!(again.requirements_refreshed, 6);

        // The framework carries its attribution and scale.
        let frameworks: Value = request.get("/api/capability-frameworks").await.json();
        let pcf = frameworks
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["slug"] == "uk-gdad-pcf")
            .unwrap();
        assert_eq!(pcf["scale_max"], 4);
        assert!(pcf["attribution"].as_str().unwrap().contains("Open Government Licence"));
        assert!(pcf["profiles"].as_u64().unwrap() >= 3);

        // Profiles: titles as the framework writes them; the management track is flagged.
        let listed: Value = request.get("/api/role-profiles?framework=uk-gdad-pcf").await.json();
        let by_title = |title: &str| {
            listed
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["job_title"] == title)
                .cloned()
                .unwrap_or_else(|| panic!("no profile titled {title}"))
        };
        let junior = by_title("Junior tester");
        let tester = by_title("Tester");
        let lead = by_title("Tester lead - management");
        assert_eq!(junior["profession"], "Test profession");
        assert_eq!(junior["level_order"], 1);
        assert_eq!(lead["management_track"], true);
        assert_eq!(junior["requirement_count"], 2);

        // Requirements keep the source level beside WPM's; the identity mapping keeps the number.
        let detail: Value = request
            .get(&format!("/api/role-profiles/{}", tester["pid"].as_str().unwrap()))
            .await
            .json();
        assert!(detail["framework"]["attribution"].is_string());
        let skill_a = detail["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["skill"] == "Fixture skill A")
            .unwrap();
        assert_eq!(skill_a["source_level"], 3);
        assert_eq!(skill_a["source_scale_max"], 4);
        assert_eq!(skill_a["min_proficiency"], 3);
        assert_eq!(skill_a["importance"], "important", "importance is a draft default for a human to edit");

        // A planner's edit survives a re-import (unless overwrite_levels is asked for).
        let skill_pid = skill_a["skill_pid"].as_str().unwrap();
        request
            .put(&format!("/api/role-profiles/{}/requirements", tester["pid"].as_str().unwrap()))
            .json(&json!({ "skill_pid": skill_pid, "min_proficiency": 5, "importance": "critical" }))
            .await
            .assert_status_ok();
        import_pcf(&ctx.db, &fixture, LevelMapping::Identity, false).await.expect("re-import");
        let kept: Value = request
            .get(&format!("/api/role-profiles/{}", tester["pid"].as_str().unwrap()))
            .await
            .json();
        let kept_a = kept["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["skill"] == "Fixture skill A")
            .unwrap();
        assert_eq!(kept_a["min_proficiency"], 5, "the planner's level is not overwritten");
        import_pcf(&ctx.db, &fixture, LevelMapping::Identity, true).await.expect("overwrite");
        let reset: Value = request
            .get(&format!("/api/role-profiles/{}", tester["pid"].as_str().unwrap()))
            .await
            .json();
        let reset_a = reset["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["skill"] == "Fixture skill A")
            .unwrap();
        assert_eq!(reset_a["min_proficiency"], 3, "overwrite_levels re-derives from the source level");

        // A planner's rename survives a re-import: matched by reference, no duplicate.
        request
            .put(&format!("/api/skills/{skill_pid}"))
            .json(&json!({ "name": "Renamed skill A" }))
            .await
            .assert_status_ok();
        import_pcf(&ctx.db, &fixture, LevelMapping::Identity, false).await.expect("re-import after rename");
        let catalogue: Value = request.get("/api/skills").await.json();
        let names: Vec<&str> = catalogue.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"Renamed skill A"));
        assert!(!names.contains(&"Fixture skill A"), "no duplicate under the framework's name");

        // Progression: junior → tester raises A (1→3), keeps B, adds C.
        let step: Value = request
            .get(&format!("/api/role-profiles/{}/progression", junior["pid"].as_str().unwrap()))
            .await
            .json();
        let next = &step["next"][0];
        assert_eq!(next["job_title"], "Tester");
        assert_eq!(next["raised"][0]["skill"], "Renamed skill A");
        assert_eq!(next["raised"][0]["from"], 1);
        assert_eq!(next["raised"][0]["to"], 3);
        assert_eq!(next["added"][0]["skill"], "Fixture skill C");
        assert_eq!(next["unchanged"], 1);
        let top: Value = request
            .get(&format!("/api/role-profiles/{}/progression", lead["pid"].as_str().unwrap()))
            .await
            .json();
        assert!(top["next"].as_array().unwrap().is_empty(), "nothing above the top level");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn skills_can_be_edited_categorised_and_referenced() {
    request::<App, _, _>(|request, _ctx| async move {
        let make = |name: &str| {
            request
                .post("/api/skills")
                .json(&json!({ "name": name, "category": "other" }))
        };
        let a: Value = make("Stakeholder wrangling").await.json();
        let b: Value = make("Origami").await.json();
        let a_pid = a["pid"].as_str().unwrap().to_string();
        let b_pid = b["pid"].as_str().unwrap().to_string();

        // Suggestions come with their keyword; a skill no rule matches gets none.
        let suggestions: Value = request.get("/api/skills/category-suggestions").await.json();
        let list = suggestions["suggestions"].as_array().unwrap();
        let mine = list.iter().find(|s| s["pid"] == a_pid).unwrap();
        assert_eq!(mine["suggested"], "leadership");
        assert_eq!(mine["keyword"], "stakeholder");
        assert!(list.iter().all(|s| s["pid"] != b_pid));
        // Applying changes only what is asked, only skills still `other`.
        let applied: Value = request
            .post("/api/skills/category-suggestions/apply")
            .json(&json!({ "skill_pids": [a_pid, b_pid] }))
            .await
            .json();
        assert_eq!(applied["applied"], 1);
        assert_eq!(applied["skipped"], 1, "no suggestion for Origami");
        let again: Value = request
            .post("/api/skills/category-suggestions/apply")
            .json(&json!({ "skill_pids": [a_pid] }))
            .await
            .json();
        assert_eq!(again["applied"], 0, "no longer `other`");

        // Rename and recategorise; names are unique; categories are a closed set.
        request
            .put(&format!("/api/skills/{b_pid}"))
            .json(&json!({ "name": "  Paper folding ", "category": "domain" }))
            .await
            .assert_status_ok();
        assert_eq!(
            request.put(&format!("/api/skills/{b_pid}")).json(&json!({ "name": "Stakeholder wrangling" })).await.status_code(),
            422,
            "a name that is taken"
        );
        assert_eq!(
            request.put(&format!("/api/skills/{b_pid}")).json(&json!({ "category": "wizardry" })).await.status_code(),
            422
        );
        let catalogue: Value = request.get("/api/skills").await.json();
        let renamed = catalogue.as_array().unwrap().iter().find(|s| s["pid"] == b_pid).unwrap();
        assert_eq!(renamed["name"], "Paper folding");
        assert_eq!(renamed["category"], "domain");

        // An external reference: one per skill per framework, and a reference names one skill.
        let reference = json!({ "framework_slug": "esco", "ref": "http://data.europa.eu/esco/skill/test-1", "label": "paper folding", "version": "v1.2.1" });
        request.post(&format!("/api/skills/{b_pid}/refs")).json(&reference).await.assert_status_ok();
        assert_eq!(request.post(&format!("/api/skills/{b_pid}/refs")).json(&reference).await.status_code(), 422);
        assert_eq!(request.post(&format!("/api/skills/{a_pid}/refs")).json(&reference).await.status_code(), 422, "the reference is taken by another skill");
        let with_ref: Value = request.get("/api/skills").await.json();
        let row = with_ref.as_array().unwrap().iter().find(|s| s["pid"] == b_pid).unwrap();
        assert_eq!(row["external_refs"][0]["framework"], "esco");
        request.delete(&format!("/api/skills/{b_pid}/refs/esco")).await.assert_status_ok();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn esco_import_search_and_seeding() {
    use workforce_planning_management_service::tasks::import_esco::import_esco;

    request::<App, _, _>(|request, ctx| async move {
        // Catalogue skills that may or may not link to ESCO.
        for name in ["Test software", "Work in teams", "Duplicate label"] {
            request
                .post("/api/skills")
                .json(&json!({ "name": name, "category": "other" }))
                .await
                .assert_status_ok();
        }
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/esco-mini");
        let report = import_esco(&ctx.db, &fixture, "en", "v-test").await.expect("import");
        assert_eq!((report.skills, report.occupations, report.relations), (5, 2, 4));
        assert_eq!(report.relations_dangling, 1, "the relation to a skill not in the file");
        assert_eq!(report.catalogue_linked, 2, "exact normalised labels, one match each");
        assert_eq!(report.catalogue_ambiguous, 1, "two ESCO skills share the label: not guessed");
        let again = import_esco(&ctx.db, &fixture, "en", "v-test").await.expect("re-import");
        assert_eq!((again.skills, again.relations), (5, 4), "replaced, not duplicated");
        assert_eq!(again.catalogue_linked, 0, "already linked");

        // Attribution and the pinned version live on the framework row.
        let frameworks: Value = request.get("/api/capability-frameworks").await.json();
        let esco = frameworks.as_array().unwrap().iter().find(|f| f["slug"] == "esco").unwrap();
        assert!(esco["attribution"].as_str().unwrap().contains("2011/833/EU"));
        assert!(esco["note"].as_str().unwrap().contains("v-test"));
        assert!(esco["note"].as_str().unwrap().contains("no proficiency scale"));

        // Search: literal, case-insensitive, with a minimum length.
        let found: Value = request.get("/api/esco/occupations?q=TEST%20DEV").await.json();
        assert_eq!(found.as_array().unwrap().len(), 1);
        assert_eq!(found[0]["essential_skills"], 2);
        assert_eq!(found[0]["optional_skills"], 1);
        assert_eq!(found[0]["isco_code"], "2512");
        assert_eq!(request.get("/api/esco/occupations?q=a").await.status_code(), 422);
        let none: Value = request.get("/api/esco/occupations?q=%25%25").await.json();
        assert!(none.as_array().unwrap().is_empty(), "a % is matched literally, not as a wildcard");
        let skills: Value = request.get("/api/esco/skills?q=work%20in").await.json();
        assert!(skills[0]["catalogue_skill_pid"].is_string(), "linked to the catalogue skill");

        // An occupation lists essential skills first.
        let occupation: Value = request
            .get("/api/esco/occupation?uri=http://data.europa.eu/esco/occupation/q1")
            .await
            .json();
        let relations: Vec<&str> = occupation["skills"].as_array().unwrap().iter().map(|s| s["relation"].as_str().unwrap()).collect();
        assert_eq!(relations, ["essential", "essential", "optional"]);
        assert!(occupation["description"].as_str().unwrap().contains('\n'), "multi-line description preserved");

        // Seeding needs the planner's level — ESCO states none.
        let seed = |body: Value| request.post("/api/role-profiles/from-esco").json(&body);
        let uri = "http://data.europa.eu/esco/occupation/q1";
        assert_eq!(seed(json!({ "occupation_uri": uri, "default_min_proficiency": 0 })).await.status_code(), 422);
        assert_eq!(seed(json!({ "occupation_uri": "http://nope", "default_min_proficiency": 3 })).await.status_code(), 404);
        let first: Value = seed(json!({ "occupation_uri": uri, "default_min_proficiency": 3, "include_optional": true }))
            .await
            .json();
        assert_eq!(first["requirements_created"], 3, "two essential and one optional skill");
        assert_eq!(first["skills_created"], 1, "Python is new; the other two were already in the catalogue");
        assert_eq!(
            seed(json!({ "occupation_uri": uri, "default_min_proficiency": 3, "job_title": "Another title" })).await.status_code(),
            422,
            "one profile per ESCO occupation"
        );

        let profile: Value = request.get(&format!("/api/role-profiles/{}", first["pid"].as_str().unwrap())).await.json();
        assert_eq!(profile["framework"]["slug"], "esco");
        assert_eq!(profile["profession"], "ISCO-08 2512");
        let reqs = profile["requirements"].as_array().unwrap();
        assert!(reqs.iter().all(|r| r["min_proficiency"] == 3), "every level is the planner's choice");
        assert!(reqs.iter().all(|r| r["source_level"].is_null()), "no framework level is invented");
        let importance = |skill: &str| reqs.iter().find(|r| r["skill"] == skill).unwrap()["importance"].clone();
        assert_eq!(importance("Test software"), json!("important"), "essential drafts `important`");
        assert_eq!(importance("Work in teams"), json!("useful"), "optional drafts `useful`");
        assert!(reqs.iter().any(|r| r["category"] == "domain"), "knowledge drafts the `domain` category");

        // Without optional skills, a different occupation drafts only what is essential.
        let second: Value = seed(json!({
            "occupation_uri": "http://data.europa.eu/esco/occupation/q2", "default_min_proficiency": 2,
        }))
        .await
        .json();
        assert_eq!(second["requirements_created"], 1);
        assert_eq!(second["skills_created"], 0, "the skill is already linked");

        // A manual reference must name a real ESCO skill.
        let catalogue: Value = request.get("/api/skills").await.json();
        let dup = catalogue.as_array().unwrap().iter().find(|s| s["name"] == "Duplicate label").unwrap();
        let dup_pid = dup["pid"].as_str().unwrap();
        assert_eq!(
            request.post(&format!("/api/skills/{dup_pid}/refs")).json(&json!({ "framework_slug": "esco", "ref": "http://not-esco/x" })).await.status_code(),
            422
        );
        request
            .post(&format!("/api/skills/{dup_pid}/refs"))
            .json(&json!({ "framework_slug": "esco", "ref": "http://data.europa.eu/esco/skill/t4", "label": "duplicate label" }))
            .await
            .assert_status_ok();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn skills_merge_keeps_the_stronger_statement_and_delete_refuses_use() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let w1 = seed_worker!(&request, &org, "E-1", None).await;
        let w2 = seed_worker!(&request, &org, "E-2", None).await;
        let make = |name: &str| {
            request.post("/api/skills").json(&json!({ "name": name, "category": "technical" }))
        };
        let a: Value = make("Merge source").await.json();
        let b: Value = make("Merge target").await.json();
        let c: Value = make("Merge unused").await.json();
        let (a, b, c) = (
            a["pid"].as_str().unwrap().to_string(),
            b["pid"].as_str().unwrap().to_string(),
            c["pid"].as_str().unwrap().to_string(),
        );
        let declare = |worker: &str, skill: &str, level: i32, target: Option<i32>| {
            request
                .put(&format!("/api/workers/{worker}/skills"))
                .json(&json!({ "skill_pid": skill, "proficiency": level, "target": target }))
        };
        declare(&w1, &a, 4, Some(5)).await.assert_status_ok(); // both declared by w1
        declare(&w1, &b, 3, None).await.assert_status_ok();
        declare(&w2, &a, 2, None).await.assert_status_ok(); // only the source by w2

        let profile = |title: &str| {
            request.post("/api/role-profiles").json(&json!({ "job_title": title }))
        };
        let both: Value = profile("Merge both").await.json();
        let only: Value = profile("Merge only source").await.json();
        let (both, only) = (both["pid"].as_str().unwrap().to_string(), only["pid"].as_str().unwrap().to_string());
        let require = |role: &str, skill: &str, min: i32, importance: &str| {
            request
                .put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": skill, "min_proficiency": min, "importance": importance }))
        };
        require(&both, &a, 2, "critical").await.assert_status_ok();
        require(&both, &b, 4, "useful").await.assert_status_ok();
        require(&only, &a, 3, "important").await.assert_status_ok();

        request
            .post(&format!("/api/skills/{a}/refs"))
            .json(&json!({ "framework_slug": "other-framework", "ref": "A-in-other" }))
            .await
            .assert_status_ok();

        // In use ⇒ the delete is refused, and usage says why; unused ⇒ deletable.
        assert_eq!(request.delete(&format!("/api/skills/{a}")).await.status_code(), 422);
        let usage: Value = request.get(&format!("/api/skills/{a}/usage")).await.json();
        assert_eq!(usage["declared_by"], 2);
        assert_eq!(usage["required_by_profiles"], 2);
        assert_eq!(usage["deletable"], false);
        assert_eq!(request.get(&format!("/api/skills/{c}/usage")).await.json::<Value>()["deletable"], true);
        request.delete(&format!("/api/skills/{c}")).await.assert_status_ok();

        // Merge: refuse self; then fold A into B.
        assert_eq!(request.post(&format!("/api/skills/{a}/merge")).json(&json!({ "into_pid": a })).await.status_code(), 422);
        let result: Value = request.post(&format!("/api/skills/{a}/merge")).json(&json!({ "into_pid": b })).await.json();
        assert_eq!(result["records_merged"], 2, "w1's two declarations and the profile requiring both");
        assert!(result["records_moved"].as_u64().unwrap() >= 2, "w2's declaration and the other profile move over");

        let catalogue: Value = request.get("/api/skills").await.json();
        let names: Vec<&str> = catalogue.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
        assert!(!names.contains(&"Merge source"), "the source is retired");
        assert!(!names.contains(&"Merge unused"));
        let target = catalogue.as_array().unwrap().iter().find(|s| s["name"] == "Merge target").unwrap();
        assert_eq!(target["external_refs"][0]["ref"], "A-in-other", "the reference moved to the target");

        // The stronger statement survives: w1 keeps proficiency 4 and target 5; w2 now declares the target.
        let w1_skills: Value = request.get(&format!("/api/workers/{w1}/skills")).await.json();
        let kept = w1_skills.as_array().unwrap().iter().find(|s| s["skill_pid"] == b).unwrap();
        assert_eq!(kept["proficiency"], 4);
        assert_eq!(kept["target"], 5);
        assert_eq!(w1_skills.as_array().unwrap().len(), 1, "one declaration remains");
        let w2_skills: Value = request.get(&format!("/api/workers/{w2}/skills")).await.json();
        assert_eq!(w2_skills[0]["skill_pid"], b);

        // The profile that required both keeps the higher minimum and the stronger importance.
        let detail: Value = request.get(&format!("/api/role-profiles/{both}")).await.json();
        let reqs = detail["requirements"].as_array().unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0]["min_proficiency"], 4);
        assert_eq!(reqs[0]["importance"], "critical");
        let moved: Value = request.get(&format!("/api/role-profiles/{only}")).await.json();
        assert_eq!(moved["requirements"][0]["skill"], "Merge target");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_person_selects_their_pcf_and_esco_roles_and_skills() {
    use workforce_planning_management_service::rules::framework::LevelMapping;
    use workforce_planning_management_service::tasks::import_esco::import_esco;
    use workforce_planning_management_service::tasks::import_framework::import_pcf;

    request::<App, _, _>(|request, ctx| async move {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        import_pcf(&ctx.db, &root.join("pcf-mini"), LevelMapping::Identity, false).await.expect("pcf");
        import_esco(&ctx.db, &root.join("esco-mini"), "en", "v-test").await.expect("esco");

        let org = an_org();
        let me = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &me).await;
        let base = format!("/api/workers/{me}/framework-roles");

        // Both frameworks are selectable; nothing is chosen yet.
        let selectable: Value = request.get("/api/frameworks/selectable").await.json();
        assert!(selectable.as_array().unwrap().iter().all(|f| f["available"] == true));
        assert!(request.get(&base).await.json::<Value>().as_array().unwrap().is_empty());
        assert_eq!(request.get(&format!("{base}/uk-gdad-pcf/skills")).await.status_code(), 422, "no role yet");
        assert_eq!(
            request.put(&format!("{base}/uk-gdad-pcf/skills")).json(&json!({ "selections": [] })).await.status_code(),
            422
        );
        assert_eq!(request.put(&format!("{base}/sfia")).json(&json!({})).await.status_code(), 422, "unknown framework");

        // ── UK GDAD PCF: choose a role level; only a PCF role is accepted.
        let profiles: Value = request.get("/api/role-profiles?framework=uk-gdad-pcf").await.json();
        let tester = profiles.as_array().unwrap().iter().find(|p| p["job_title"] == "Tester").unwrap();
        let tester_pid = tester["pid"].as_str().unwrap().to_string();
        let own: Value = request.post("/api/role-profiles").json(&json!({ "job_title": "Not a framework role" })).await.json();
        assert_eq!(
            request.put(&format!("{base}/uk-gdad-pcf")).json(&json!({ "role_profile_pid": own["pid"] })).await.status_code(),
            422,
            "a profile that is not from the PCF"
        );
        assert_eq!(request.put(&format!("{base}/uk-gdad-pcf")).json(&json!({})).await.status_code(), 422);
        let chosen: Value = request.put(&format!("{base}/uk-gdad-pcf")).json(&json!({ "role_profile_pid": tester_pid })).await.json();
        assert_eq!(chosen["role_label"], "Tester");

        let listed: Value = request.get(&format!("{base}/uk-gdad-pcf/skills")).await.json();
        let skills = listed["skills"].as_array().unwrap();
        assert_eq!(skills.len(), 3, "the Tester level names skills A, B and C");
        let skill_a = skills.iter().find(|s| s["label"] == "Fixture skill A").unwrap();
        assert_eq!(skill_a["framework_level"], 3, "the framework's own level is shown as a prompt");
        assert!(skill_a["declared"].is_null());
        let a_ref = skill_a["ref"].as_str().unwrap().to_string();
        let b_ref = skills.iter().find(|s| s["label"] == "Fixture skill B").unwrap()["ref"].as_str().unwrap().to_string();

        // Select skills at the person's own level; an out-of-scale level or a non-PCF skill is refused.
        let put_skills = |framework: &str, body: Value| request.put(&format!("{base}/{framework}/skills")).json(&body);
        assert_eq!(put_skills("uk-gdad-pcf", json!({ "selections": [{ "ref": a_ref, "proficiency": 9 }] })).await.status_code(), 422);
        let not_pcf: Value = request.post("/api/skills").json(&json!({ "name": "Not in the PCF", "category": "other" })).await.json();
        assert_eq!(
            put_skills("uk-gdad-pcf", json!({ "selections": [{ "ref": not_pcf["pid"], "proficiency": 3 }] })).await.status_code(),
            422
        );
        let done: Value = put_skills("uk-gdad-pcf", json!({ "selections": [
            { "ref": a_ref, "proficiency": 3 }, { "ref": b_ref, "proficiency": 2 },
        ] })).await.json();
        assert_eq!(done["declared"], 2);
        let mine: Value = request.get(&format!("/api/workers/{me}/skills")).await.json();
        assert_eq!(mine.as_array().unwrap().len(), 2, "ordinary skill declarations");
        // Deselect B: it is removed, A stays.
        let cleared: Value = put_skills("uk-gdad-pcf", json!({ "selections": [{ "ref": b_ref, "proficiency": null }] })).await.json();
        assert_eq!(cleared["removed"], 1);
        let after: Value = request.get(&format!("{base}/uk-gdad-pcf/skills")).await.json();
        let level_of = |label: &str| after["skills"].as_array().unwrap().iter().find(|s| s["label"] == label).unwrap()["declared"].clone();
        assert_eq!(level_of("Fixture skill A"), json!(3));
        assert!(level_of("Fixture skill B").is_null());

        // ── ESCO: a different framework, a separate selection.
        let occupation = "http://data.europa.eu/esco/occupation/q1";
        assert_eq!(
            request.put(&format!("{base}/esco")).json(&json!({ "occupation_uri": "http://nope" })).await.status_code(),
            422
        );
        request.put(&format!("{base}/esco")).json(&json!({ "occupation_uri": occupation })).await.assert_status_ok();
        let both: Value = request.get(&base).await.json();
        assert_eq!(both.as_array().unwrap().len(), 2, "a role in each framework at once");

        let esco_skills: Value = request.get(&format!("{base}/esco/skills")).await.json();
        let relations: Vec<&str> = esco_skills["skills"].as_array().unwrap().iter().map(|s| s["relation"].as_str().unwrap()).collect();
        assert_eq!(relations, ["essential", "essential", "optional"]);
        let python = "http://data.europa.eu/esco/skill/t1";
        assert_eq!(put_skills("esco", json!({ "selections": [{ "ref": "http://not-esco", "proficiency": 3 }] })).await.status_code(), 422);
        let first: Value = put_skills("esco", json!({ "selections": [{ "ref": python, "proficiency": 4 }] })).await.json();
        assert_eq!(first["declared"], 1);
        assert_eq!(first["skills_created"], 1, "the ESCO skill becomes a linked catalogue skill");
        let second: Value = put_skills("esco", json!({ "selections": [{ "ref": python, "proficiency": 5 }] })).await.json();
        assert_eq!(second["skills_created"], 0, "found by its ESCO reference this time");
        let esco_after: Value = request.get(&format!("{base}/esco/skills")).await.json();
        let py = esco_after["skills"].as_array().unwrap().iter().find(|s| s["ref"] == python).unwrap();
        assert_eq!(py["declared"], 5, "the person's own level, updated");
        // Deselecting an ESCO skill that was never linked is a no-op, not an error.
        let noop: Value = put_skills("esco", json!({ "selections": [{ "ref": "http://data.europa.eu/esco/skill/t2", "proficiency": null }] })).await.json();
        assert_eq!((noop["declared"].as_u64(), noop["removed"].as_u64()), (Some(0), Some(0)));

        // Clearing a role leaves the declared skills alone.
        request.delete(&format!("{base}/esco")).await.assert_status_ok();
        assert_eq!(request.get(&format!("{base}/esco/skills")).await.status_code(), 422);
        let kept: Value = request.get(&format!("/api/workers/{me}/skills")).await.json();
        assert_eq!(kept.as_array().unwrap().len(), 2, "A (PCF) and Python (ESCO) remain declared");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn career_history_and_aspirations() {
    use workforce_planning_management_service::rules::framework::LevelMapping;
    use workforce_planning_management_service::tasks::import_esco::import_esco;
    use workforce_planning_management_service::tasks::import_framework::import_pcf;

    request::<App, _, _>(|request, ctx| async move {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        import_pcf(&ctx.db, &root.join("pcf-mini"), LevelMapping::Identity, false).await.expect("pcf");
        import_esco(&ctx.db, &root.join("esco-mini"), "en", "v-test").await.expect("esco");
        let org = an_org();
        let me = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &me).await;
        let profiles: Value = request.get("/api/role-profiles?framework=uk-gdad-pcf").await.json();
        let profile = |title: &str| profiles.as_array().unwrap().iter().find(|p| p["job_title"] == title).unwrap()["pid"].as_str().unwrap().to_string();
        let (junior, tester) = (profile("Junior tester"), profile("Tester"));
        let role_url = format!("/api/workers/{me}/framework-roles/uk-gdad-pcf");

        // ── Roles over time: changing a role closes the old one; choosing it again changes nothing.
        request.put(&role_url).json(&json!({ "role_profile_pid": junior })).await.assert_status_ok();
        request.put(&role_url).json(&json!({ "role_profile_pid": junior })).await.assert_status_ok();
        request.put(&role_url).json(&json!({ "role_profile_pid": tester })).await.assert_status_ok();
        let history: Value = request.get(&format!("/api/workers/{me}/role-history?framework=uk-gdad-pcf")).await.json();
        let rows = history.as_array().unwrap();
        assert_eq!(rows.len(), 2, "re-selecting the same role added no row");
        assert_eq!(rows[0]["role_label"], "Tester");
        assert_eq!(rows[0]["current"], true);
        assert_eq!(rows[1]["role_label"], "Junior tester");
        assert_eq!(rows[1]["current"], false);
        assert!(rows[1]["ended_at"].is_string(), "the previous role has a stop time");

        // ── A retrospective past role: dated, non-overlapping, and not in the future.
        let past = |from: &str, to: &str| {
            request.post(&format!("{role_url}/past")).json(&json!({ "role_profile_pid": junior, "started_on": from, "ended_on": to }))
        };
        past("2019-01-01", "2020-06-30").await.assert_status_ok();
        assert_eq!(past("2020-01-01", "2020-12-31").await.status_code(), 422, "overlaps the role just added");
        assert_eq!(past("2022-01-01", "2021-01-01").await.status_code(), 422, "ends before it starts");
        assert_eq!(past("2025-01-01", "2999-01-01").await.status_code(), 422, "cannot end in the future");
        let all: Value = request.get(&format!("/api/workers/{me}/role-history")).await.json();
        assert_eq!(all.as_array().unwrap().len(), 3);

        // ── Skills over time.
        let skill: Value = request.post("/api/skills").json(&json!({ "name": "History skill", "category": "technical" })).await.json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        for level in [2, 4, 4, 3] {
            request.put(&format!("/api/workers/{me}/skills")).json(&json!({ "skill_pid": skill_pid, "proficiency": level })).await.assert_status_ok();
        }
        let timeline: Value = request.get(&format!("/api/workers/{me}/skill-history?skill_pid={skill_pid}")).await.json();
        let t = timeline.as_array().unwrap();
        assert_eq!(t.len(), 3, "2 → 4 → 3; declaring 4 twice is not a change");
        assert_eq!(t[0]["proficiency"], 3);
        assert_eq!(t[0]["current"], true);
        assert!(t[1]["ended_at"].is_string() && t[2]["ended_at"].is_string());
        // Deselecting through the framework route closes the interval too.
        // A retrospective level, then "as of" reads.
        let retro = |from: &str, to: &str, level: i32| {
            request.post(&format!("/api/workers/{me}/skill-history/past")).json(&json!({ "skill_pid": skill_pid, "proficiency": level, "started_on": from, "ended_on": to }))
        };
        retro("2020-01-01", "2021-12-31", 1).await.assert_status_ok();
        assert_eq!(retro("2021-06-01", "2022-06-01", 2).await.status_code(), 422, "overlaps the interval just added");
        assert_eq!(retro("2018-01-01", "2018-12-31", 9).await.status_code(), 422, "level is 1-5");
        let then: Value = request.get(&format!("/api/workers/{me}/skills-as-of?at=2020-06-01")).await.json();
        let held = then["skills"].as_array().unwrap();
        assert_eq!(held.len(), 1);
        assert_eq!(held[0]["proficiency"], 1);
        let before: Value = request.get(&format!("/api/workers/{me}/skills-as-of?at=2010-01-01")).await.json();
        assert!(before["skills"].as_array().unwrap().is_empty());
        let today = chrono::Utc::now().date_naive().to_string();
        let now: Value = request.get(&format!("/api/workers/{me}/skills-as-of?at={today}")).await.json();
        assert_eq!(now["skills"][0]["proficiency"], 3, "the current level");
        assert_eq!(now["roles"][0]["role_label"], "Tester");

        // ── Aspirations: a skill target and a role, with progress; validated; private by default.
        let aspire = |body: Value| request.post(&format!("/api/workers/{me}/aspirations")).json(&body);
        assert_eq!(aspire(json!({ "kind": "skill", "skill_pid": skill_pid, "horizon": "within_1y" })).await.status_code(), 422, "needs a target");
        assert_eq!(aspire(json!({ "kind": "skill", "skill_pid": skill_pid, "target_level": 6, "horizon": "within_1y" })).await.status_code(), 422);
        assert_eq!(aspire(json!({ "kind": "role", "framework_slug": "uk-gdad-pcf", "role_profile_pid": tester, "horizon": "soon" })).await.status_code(), 422);
        let goal: Value = aspire(json!({ "kind": "skill", "skill_pid": skill_pid, "target_level": 5, "horizon": "within_1y", "note": "pair with a senior" })).await.json();
        let role_goal: Value = aspire(json!({
            "kind": "role", "framework_slug": "esco", "occupation_uri": "http://data.europa.eu/esco/occupation/q1",
            "horizon": "one_to_three_years", "status": "planned", "visibility": "everyone",
        })).await.json();
        let listed: Value = request.get(&format!("/api/workers/{me}/aspirations")).await.json();
        assert_eq!(listed["viewer_is_the_person"], true);
        let items = listed["aspirations"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        let skill_goal = items.iter().find(|a| a["kind"] == "skill").unwrap();
        assert_eq!(skill_goal["visibility"], "private", "private unless the person shares it");
        assert_eq!(skill_goal["progress"]["current_level"], 3);
        assert_eq!(skill_goal["progress"]["gap"], 2);
        assert_eq!(skill_goal["note"], "pair with a senior");
        let role_item = items.iter().find(|a| a["kind"] == "role").unwrap();
        assert_eq!(role_item["role_label"], "esco test developer");
        assert_eq!(role_item["progress"]["essential_skills"], 2);
        assert_eq!(role_item["visibility"], "everyone");

        // Progress and status move; achieved is recorded, not assumed.
        let goal_pid = goal["pid"].as_str().unwrap();
        request.put(&format!("/api/aspirations/{goal_pid}")).json(&json!({ "status": "in_progress", "visibility": "manager" }))
            .await.assert_status_ok();
        assert_eq!(request.put(&format!("/api/aspirations/{goal_pid}")).json(&json!({ "visibility": "friends" })).await.status_code(), 422);
        request.put(&format!("/api/aspirations/{goal_pid}")).json(&json!({ "visibility": "manager" })).await.assert_status_ok();
        assert_eq!(request.put(&format!("/api/aspirations/{goal_pid}")).json(&json!({ "status": "wishing" })).await.status_code(), 422);
        request.put(&format!("/api/workers/{me}/skills")).json(&json!({ "skill_pid": skill_pid, "proficiency": 5 })).await.assert_status_ok();
        let after: Value = request.get(&format!("/api/workers/{me}/aspirations")).await.json();
        let done = after["aspirations"].as_array().unwrap().iter().find(|a| a["pid"] == goal_pid).unwrap();
        assert_eq!(done["progress"]["achieved"], true);
        assert_eq!(done["visibility"], "manager");
        request.delete(&format!("/api/aspirations/{}", role_goal["pid"].as_str().unwrap())).await.assert_status_ok();
        let last: Value = request.get(&format!("/api/workers/{me}/aspirations")).await.json();
        assert_eq!(last["aspirations"].as_array().unwrap().len(), 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn reporting_lines_downline_aspirations_and_groups() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        // ceo ← vp ← lead ← dev;  ceo ← peer
        let mut pids = Vec::new();
        for n in ["R-1", "R-2", "R-3", "R-4", "R-5"] {
            pids.push(seed_worker!(&request, &org, n, None).await);
        }
        let (ceo, vp, lead, dev, peer) = (&pids[0], &pids[1], &pids[2], &pids[3], &pids[4]);
        for (w, m) in [(vp, ceo), (lead, vp), (dev, lead), (peer, ceo)] {
            request.put(&format!("/api/workers/{w}")).json(&json!({ "manager_pid": m })).await.assert_status_ok();
        }

        // ── Upline: nearest first, level 1 is the direct manager.
        let up: Value = request.get(&format!("/api/workers/{dev}/upline")).await.json();
        let chain = up["upline"].as_array().unwrap();
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0]["pid"], lead.as_str());
        assert_eq!(chain[0]["direct_manager"], true);
        assert_eq!(chain[2]["pid"], ceo.as_str());
        assert_eq!(chain[2]["level"], 3);
        let top: Value = request.get(&format!("/api/workers/{ceo}/upline")).await.json();
        assert!(top["upline"].as_array().unwrap().is_empty());

        // ── Downline: direct and indirect reports.
        let down: Value = request.get(&format!("/api/workers/{vp}/downline")).await.json();
        assert_eq!(down["summary"], json!({ "direct": 1, "indirect": 1, "total": 2 }));
        let team = down["downline"].as_array().unwrap();
        let kind_of = |pid: &str| team.iter().find(|w| w["pid"] == pid).map(|w| w["report_kind"].clone());
        assert_eq!(kind_of(lead), Some(json!("direct")));
        assert_eq!(kind_of(dev), Some(json!("indirect")));
        assert_eq!(kind_of(peer), None, "a peer branch is not downline");
        let all: Value = request.get(&format!("/api/workers/{ceo}/downline")).await.json();
        assert_eq!(all["summary"]["total"], 4);
        let indirect: Value = request.get(&format!("/api/workers/{ceo}/reports?kind=indirect")).await.json();
        assert_eq!(indirect["reports"].as_array().unwrap().len(), 2);
        let direct: Value = request.get(&format!("/api/workers/{ceo}/reports?kind=direct")).await.json();
        assert_eq!(direct["reports"].as_array().unwrap().len(), 2);
        assert_eq!(request.get(&format!("/api/workers/{ceo}/reports?kind=sideways")).await.status_code(), 422);

        // ── Dotted-line reports: anyone, several, separate from the solid line.
        let dot = |who: &str, mgr: &str| request.post(&format!("/api/workers/{who}/dotted-line-managers")).json(&json!({ "manager_pid": mgr, "note": "project X" }));
        dot(dev, peer).await.assert_status_ok();
        dot(dev, ceo).await.assert_status_ok();
        dot(dev, peer).await.assert_status_ok();
        assert_eq!(dot(dev, dev).await.status_code(), 422, "not their own manager");
        dot(peer, dev).await.assert_status_ok();
        let dl: Value = request.get(&format!("/api/workers/{dev}/dotted-line")).await.json();
        assert_eq!(dl["dotted_line_managers"].as_array().unwrap().len(), 2, "two dotted-line managers; re-adding added none");
        assert_eq!(dl["dotted_line_reports"].as_array().unwrap().len(), 1, "a loop is allowed");
        let chart_up: Value = request.get(&format!("/api/workers/{dev}/upline")).await.json();
        assert_eq!(chart_up["upline"].as_array().unwrap().len(), 3, "the solid line is unchanged");
        let peers_down: Value = request.get(&format!("/api/workers/{peer}/downline")).await.json();
        assert_eq!(peers_down["summary"]["total"], 0, "a dotted-line report is not a direct or indirect report");
        request.delete(&format!("/api/workers/{dev}/dotted-line-managers/{peer}")).await.assert_status_ok();
        assert_eq!(request.delete(&format!("/api/workers/{dev}/dotted-line-managers/{peer}")).await.status_code(), 404);
        let now: Value = request.get(&format!("/api/workers/{dev}/dotted-line")).await.json();
        assert_eq!(now["dotted_line_managers"].as_array().unwrap().len(), 1);
        let past: Value = request.get(&format!("/api/workers/{dev}/dotted-line?include_past=true")).await.json();
        assert_eq!(past["dotted_line_managers"].as_array().unwrap().len(), 2, "the ended line is kept");

        // ── A manager sees downline aspirations shared with managers or everyone — never private ones.
        let skill: Value = request.post("/api/skills").json(&json!({ "name": "Downline skill", "category": "technical" })).await.json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        for (who, visibility) in [(dev, "private"), (dev, "manager"), (lead, "everyone"), (peer, "manager")] {
            request.post(&format!("/api/workers/{who}/aspirations"))
                .json(&json!({ "kind": "skill", "skill_pid": skill_pid, "target_level": 4, "horizon": "within_1y", "visibility": visibility }))
                .await.assert_status_ok();
        }
        let view: Value = request.get(&format!("/api/workers/{vp}/downline-aspirations")).await.json();
        assert_eq!(view["viewer"], "manager");
        let people = view["team"].as_array().unwrap();
        assert_eq!(people.len(), 2, "lead and dev only; the peer branch is not the vp's team");
        let dev_view = people.iter().find(|p| p["worker_pid"] == dev.as_str()).unwrap();
        assert_eq!(dev_view["direct_report"], false);
        assert_eq!(dev_view["aspirations"].as_array().unwrap().len(), 1, "the private one is not shown");
        assert_eq!(dev_view["aspirations"][0]["visibility"], "manager");
        let lead_view = people.iter().find(|p| p["worker_pid"] == lead.as_str()).unwrap();
        assert_eq!(lead_view["direct_report"], true);
        assert_eq!(lead_view["aspirations"].as_array().unwrap().len(), 1);
        assert!(!view.to_string().contains("private_hidden"), "the number of hidden items is not revealed");

        // ── Groups: a worker is in several at once; leaving closes the membership.
        let make = |name: &str, kind: &str| request.post("/api/groups").json(&json!({ "organization_ref": org, "name": name, "kind": kind, "description": "d" }));
        let rust: Value = make("Rust guild", "practice").await.json();
        let chess: Value = make("Chess club", "interest").await.json();
        assert_eq!(make("rust GUILD", "practice").await.status_code(), 422, "names are unique, ignoring case");
        assert_eq!(make("Odd one", "club").await.status_code(), 422);
        let (rust, chess) = (rust["pid"].as_str().unwrap(), chess["pid"].as_str().unwrap());
        let join = |group: &str, who: &str, role: &str| request.post(&format!("/api/groups/{group}/members")).json(&json!({ "worker_pid": who, "role": role }));
        join(rust, dev, "member").await.assert_status_ok();
        join(chess, dev, "lead").await.assert_status_ok();
        join(rust, lead, "lead").await.assert_status_ok();
        join(rust, dev, "lead").await.assert_status_ok();
        assert_eq!(join(rust, dev, "owner").await.status_code(), 422);
        let mine: Value = request.get(&format!("/api/workers/{dev}/groups")).await.json();
        assert_eq!(mine["groups"].as_array().unwrap().len(), 2, "two groups at once; rejoining added no row");
        assert!(mine["groups"].as_array().unwrap().iter().any(|g| g["name"] == "Rust guild" && g["role"] == "lead"));
        let members: Value = request.get(&format!("/api/groups/{rust}/members")).await.json();
        assert_eq!(members["group"]["members"], 2);
        request.delete(&format!("/api/groups/{rust}/members/{dev}")).await.assert_status_ok();
        assert_eq!(request.delete(&format!("/api/groups/{rust}/members/{dev}")).await.status_code(), 404, "already left");
        let now: Value = request.get(&format!("/api/workers/{dev}/groups")).await.json();
        assert_eq!(now["groups"].as_array().unwrap().len(), 1);
        let past: Value = request.get(&format!("/api/workers/{dev}/groups?include_past=true")).await.json();
        assert_eq!(past["groups"].as_array().unwrap().len(), 2, "past membership is kept");
        join(rust, dev, "member").await.assert_status_ok();
        let list: Value = request.get("/api/groups").await.json();
        assert_eq!(list.as_array().unwrap().iter().find(|g| g["name"] == "Rust guild").unwrap()["members"], 2);
        // ── Per organization: a group is in one organization and only its workers can join.
        assert_eq!(request.post("/api/groups").json(&json!({ "name": "No org", "kind": "other" })).await.status_code(), 422, "organization_ref is required");
        let other_org = an_org();
        let elsewhere = seed_worker!(&request, &other_org, "R-9", None).await;
        assert_eq!(join(rust, &elsewhere, "member").await.status_code(), 422, "a worker joins only their own organization's groups");
        let twin: Value = request.post("/api/groups").json(&json!({ "organization_ref": other_org, "name": "Rust guild", "kind": "practice" })).await.json();
        assert!(twin["pid"].is_string(), "the same name is fine in another organization");
        let mine_only: Value = request.get(&format!("/api/groups?organization_ref={org}")).await.json();
        assert!(mine_only.as_array().unwrap().iter().all(|g| g["organization_ref"] == org));
        assert_eq!(mine_only.as_array().unwrap().iter().filter(|g| g["name"] == "Rust guild").count(), 1);

        // ── Skill roll-up: aggregate, floored at three.
        let roll = request.get(&format!("/api/groups/{rust}/skills")).await.json::<Value>();
        assert_eq!(roll["skills"].as_array().unwrap().len(), 0, "two members: everything withheld");
        for (who, level) in [(dev, 2), (lead, 4), (peer, 4), (vp, 5)] {
            join(rust, who, "member").await.assert_status_ok();
            request.put(&format!("/api/workers/{who}/skills")).json(&json!({ "skill_pid": skill_pid, "proficiency": level })).await.assert_status_ok();
        }
        join(rust, ceo, "member").await.assert_status_ok(); // declares nothing
        let roll = request.get(&format!("/api/groups/{rust}/skills")).await.json::<Value>();
        let rows = roll["skills"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["declared"], 4);
        assert_eq!(rows[0]["levels"], json!({ "1": 0, "2": 1, "3": 0, "4": 2, "5": 1 }));
        assert_eq!(rows[0]["coverage"], 0.8, "four of five members");
        assert!(!roll.to_string().contains(dev.as_str()), "no member is named");
        request.delete(&format!("/api/groups/{chess}")).await.assert_status_ok();
        let after: Value = request.get(&format!("/api/workers/{dev}/groups")).await.json();
        assert_eq!(after["groups"].as_array().unwrap().len(), 1, "a retired group no longer lists");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn confederation_groups_and_transfers() {
    request::<App, _, _>(|request, _ctx| async move {
        let (parent, child_a, child_b, outsider) = (an_org(), an_org(), an_org(), an_org());
        for child in [&child_a, &child_b] {
            request.post("/api/organization-confederations")
                .json(&json!({ "parent_organization_ref": parent, "child_organization_ref": child, "starts_on": "2026-01-01" }))
                .await.assert_status_ok();
        }
        let in_a = seed_worker!(&request, &child_a, "T-1", None).await;
        let in_b = seed_worker!(&request, &child_b, "T-2", None).await;
        let in_out = seed_worker!(&request, &outsider, "T-3", None).await;
        let group = |org: &str, name: &str, scope: &str| request.post("/api/groups").json(&json!({ "organization_ref": org, "name": name, "kind": "practice", "scope": scope }));

        // A confederation group needs member organizations beneath it.
        assert_eq!(group(&child_a, "Leaf community", "confederation").await.status_code(), 422);
        assert_eq!(group(&parent, "Odd", "galaxy").await.status_code(), 422);
        let community: Value = group(&parent, "Federation guild", "confederation").await.json();
        let own: Value = group(&child_a, "A-only club", "organization").await.json();
        let (community, own) = (community["pid"].as_str().unwrap(), own["pid"].as_str().unwrap());

        // Workers from different child organizations can join it; an outsider cannot.
        let join = |g: &str, w: &str| request.post(&format!("/api/groups/{g}/members")).json(&json!({ "worker_pid": w }));
        join(community, &in_a).await.assert_status_ok();
        join(community, &in_b).await.assert_status_ok();
        assert_eq!(join(community, &in_out).await.status_code(), 422);
        join(own, &in_a).await.assert_status_ok();
        assert_eq!(join(own, &in_b).await.status_code(), 422, "an organization group is just its own");

        // "Open to my organization" lists the community and the org's own groups.
        let open_a: Value = request.get(&format!("/api/groups?organization_ref={child_a}")).await.json();
        let names: Vec<&str> = open_a.as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"Federation guild") && names.contains(&"A-only club"));
        let open_b: Value = request.get(&format!("/api/groups?organization_ref={child_b}")).await.json();
        let names_b: Vec<&str> = open_b.as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap()).collect();
        assert!(names_b.contains(&"Federation guild") && !names_b.contains(&"A-only club"));

        // Transfer: A → B ends the A-only membership, keeps the community one.
        let t: Value = request.post(&format!("/api/workers/{in_a}/transfer")).json(&json!({ "organization_ref": child_b })).await.json();
        assert_eq!(t["to"], child_b.as_str());
        assert_eq!(t["ended_group_memberships"], json!(["A-only club"]));
        let now: Value = request.get(&format!("/api/workers/{in_a}/groups")).await.json();
        let current: Vec<&str> = now["groups"].as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap()).collect();
        assert_eq!(current, ["Federation guild"]);
        let past: Value = request.get(&format!("/api/workers/{in_a}/groups?include_past=true")).await.json();
        assert_eq!(past["groups"].as_array().unwrap().len(), 2, "the ended membership is kept as history");
        let w: Value = request.get(&format!("/api/workers/{in_a}")).await.json();
        assert_eq!(w["organization_ref"], child_b.as_str());

        // Moving somewhere the community does not cover ends it too; a no-op move is refused.
        let t2: Value = request.post(&format!("/api/workers/{in_a}/transfer")).json(&json!({ "organization_ref": outsider })).await.json();
        assert_eq!(t2["ended_group_memberships"], json!(["Federation guild"]));
        assert_eq!(request.post(&format!("/api/workers/{in_a}/transfer")).json(&json!({ "organization_ref": outsider })).await.status_code(), 422);
        assert_eq!(request.post(&format!("/api/workers/{in_a}/transfer")).json(&json!({ "organization_ref": "nonsense" })).await.status_code(), 422);
    })
    .await;
}
