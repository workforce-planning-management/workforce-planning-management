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
