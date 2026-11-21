//! The learning & development round-trip: skills catalog + declared
//! proficiency + the matrix/gaps, learning paths + honest progress,
//! training analytics, and the mentorship lifecycle + overview.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one seeded team, the whole L&D surface
async fn learning_round_trip() {
    crate::requests::request_open(|request, _ctx| async move {
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
        // The steps keep the order given, from position 0, and the list counts them.
        let steps: Vec<(String, i64)> = progress["steps"].as_array().unwrap().iter()
            .map(|s| (s["title"].as_str().unwrap().to_string(), s["position"].as_i64().unwrap()))
            .collect();
        assert_eq!(steps, [("Intro".to_string(), 0), ("Advanced".to_string(), 1)]);
        let listed: Vec<Value> = request.get("/api/learning-paths").await.json();
        let this = listed.iter().find(|p| p["pid"] == path["pid"]).unwrap();
        assert_eq!(this["steps"], 2);
        assert_eq!(member["display_name"], "Test Worker E-2");

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
        assert!(activated["ended_on"].is_null());
        // Completing stamps the end and keeps the start (a second pairing, so the overview below is unchanged).
        let second: Value = request
            .post("/api/mentorships")
            .json(&json!({ "mentor_pid": mentee, "mentee_pid": mentor, "focus": "reverse" }))
            .await
            .json();
        let second = format!("/api/mentorships/{}/status", second["pid"].as_str().unwrap());
        let started: Value = request.post(&second).json(&json!({ "to": "active" })).await.json();
        let completed: Value = request.post(&second).json(&json!({ "to": "completed" })).await.json();
        assert_eq!(completed["status"], "completed");
        assert_eq!(completed["started_on"], started["started_on"]);
        assert!(completed["ended_on"].is_string());

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
        // The names are capped and the total is always given: with a limit of one, one name (the newest)
        // is listed and the total still counts everyone unmatched.
        let capped: Value = request
            .get("/api/learning/mentorship-overview?days=3650&unmatched_limit=1")
            .await
            .json();
        assert_eq!(capped["unmatched_workers"].as_array().unwrap().len(), 1);
        assert_eq!(
            capped["unmatched_total"].as_u64().unwrap(),
            overview["unmatched_total"].as_u64().unwrap()
        );
        assert!(capped["unmatched_total"].as_u64().unwrap() >= 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn capability_analysis_reports_skill_depth() {
    crate::requests::request_open(|request, _ctx| async move {
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
        assert_eq!(
            rust["proficient"], 1,
            "only the level-4 declaration clears the bar"
        );
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn workforce_metrics_report_defined_numbers() {
    crate::requests::request_open(|request, _ctx| async move {
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
        assert_eq!(
            view["headcount"]["opening"], 0,
            "nobody employed on 2025-12-31"
        );
        assert!(view["headcount"]["closing"].as_u64().unwrap() >= 2);
        assert!(
            view["starters"].as_u64().unwrap() >= 2,
            "both hired in the period"
        );
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn directory_lists_employed_workers_without_sensitive_fields() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let stay = seed_worker!(&request, &org, format!("DIR-{tag}-A"), Some(450_000)).await;
        let leave = seed_worker!(&request, &org, format!("DIR-{tag}-B"), Some(500_000)).await;
        activate!(&request, &stay).await;
        activate!(&request, &leave).await;
        // Terminated today ⇒ not employed ⇒ not listed.
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{leave}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }

        let response = request
            .get(&format!("/api/directory?q=dir-{tag}"))
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(response.headers().get("link").is_none());
        let rows: Vec<Value> = response.json();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["pid"], stay.as_str());
        assert_eq!(rows[0]["job_title"], "Engineer");
        assert_eq!(rows[0]["department"], "engineering");
        for sensitive in ["salary_minor", "salary_currency", "hired_on", "person_ref"] {
            assert!(
                rows[0].get(sensitive).is_none(),
                "directory must not expose {sensitive}"
            );
        }

        // Away today: approved leave shows as away (never the kind), with
        // the best-ranked available backup covering.
        let backup = seed_worker!(&request, &org, format!("DIR-{tag}-C"), None).await;
        activate!(&request, &backup).await;
        request
            .post(&format!("/api/workers/{stay}/backups"))
            .json(&json!({ "backup_pid": backup }))
            .await
            .assert_status_ok();
        let today = workforce_planning_management::dates::today();
        request
            .post(&format!("/api/workers/{stay}/leave-entitlements"))
            .json(&json!({ "kind": "sick", "year": today.strftime("%Y").to_string().parse::<i32>().unwrap(),
                           "entitled_days": 30 }))
            .await
            .assert_status_ok();
        let leave: Value = request
            .post(&format!("/api/workers/{stay}/leave-requests"))
            .json(&json!({ "kind": "sick", "start_on": today.to_string(),
                           "end_on": (today + workforce_planning_management::dates::days(6)).to_string() }))
            .await
            .json();
        request
            .post(&format!("/api/leave-requests/{}/approve", leave["pid"].as_str().unwrap()))
            .await
            .assert_status_ok();
        let away: Vec<Value> = request
            .get(&format!("/api/directory?q=dir-{tag}-a"))
            .await
            .json();
        assert_eq!(away[0]["away_today"], true);
        assert_eq!(away[0]["covered_by"], "Test Worker DIR-".to_string() + &tag + "-C");
        assert!(away[0].get("kind").is_none(), "never the kind of leave");
        let present: Vec<Value> = request
            .get(&format!("/api/directory?q=dir-{tag}-c"))
            .await
            .json();
        assert_eq!(present[0]["away_today"], false);
        assert!(present[0]["covered_by"].is_null());

        // On call today: a rota with this worker first shows by name.
        request
            .post("/api/rotas")
            .json(&json!({ "organization_ref": org, "name": format!("Dir rota {tag}"),
                           "period_days": 7, "starts_on": today.to_string(),
                           "members": [backup, stay] }))
            .await
            .assert_status_ok();
        let on_call: Vec<Value> = request
            .get(&format!("/api/directory?q=dir-{tag}-c"))
            .await
            .json();
        assert_eq!(on_call[0]["on_call"], json!([format!("Dir rota {tag}")]));
        let not_on_call: Vec<Value> = request
            .get(&format!("/api/directory?q=dir-{tag}-a"))
            .await
            .json();
        assert_eq!(not_on_call[0]["on_call"], json!([]));

        // Department filter and a miss.
        let by_dept: Vec<Value> = request
            .get(&format!("/api/directory?q=dir-{tag}&department=ENGINEERING"))
            .await
            .json();
        // The backup added above shares the tag and department.
        assert_eq!(by_dept.len(), 2);
        let none: Vec<Value> = request
            .get(&format!("/api/directory?q=dir-{tag}&department=finance"))
            .await
            .json();
        assert_eq!(none.len(), 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn workforce_insights_flag_a_shrinking_headcount() {
    crate::requests::request_open(|request, _ctx| async move {
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
        let today = workforce_planning_management::dates::today();
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn headcount_snapshots_are_recorded_idempotently() {
    use workforce_planning_management::tasks::snapshot::run_snapshot;

    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let a = seed_worker!(&request, &org, "E-1", None).await;
        activate!(&request, &a).await;

        let first_day = workforce_planning_management::dates::ymd(2026, 7, 1).unwrap();
        let written = run_snapshot(workforce_planning_management::db::pool(&ctx), first_day)
            .await
            .expect("first snapshot");
        assert!(written >= 1, "the seeded worker's department is recorded");
        assert_eq!(
            run_snapshot(workforce_planning_management::db::pool(&ctx), first_day)
                .await
                .expect("re-run"),
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
        let row = rows
            .iter()
            .find(|r| r["department"] == "engineering")
            .unwrap();
        assert_eq!(row["as_of"], "2026-07-01");
        assert!(row["headcount"].as_u64().unwrap() >= 1);
        assert!(
            row["starters"].is_null(),
            "the first snapshot has no window"
        );

        // A later snapshot measures starters/leavers from the earlier one.
        run_snapshot(
            workforce_planning_management::db::pool(&ctx),
            workforce_planning_management::dates::ymd(2026, 8, 1).unwrap(),
        )
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
        assert!(
            later_row["starters"].is_u64(),
            "a known window reports a count"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn role_profiles_hold_required_skills() {
    crate::requests::request_open(|request, _ctx| async move {
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn role_gap_grades_declarations_against_a_role() {
    crate::requests::request_open(|request, _ctx| async move {
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
            .get(&format!(
                "/api/workers/{worker}/role-gap?role_profile_pid={role}"
            ))
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

        let workforce: Value = request
            .get(&format!("/api/role-profiles/{role}/gap"))
            .await
            .json();
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn cpd_ledger_tracks_progress_and_registrations() {
    crate::requests::request_open(|request, _ctx| async move {
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
        let view: Value = request
            .get(&format!("/api/workers/{worker}/cpd-progress"))
            .await
            .json();
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
        let after: Value = request
            .get(&format!("/api/workers/{worker}/cpd-progress"))
            .await
            .json();
        assert_eq!(progress(&after)["verified"], 6.5);

        // Registrations carry an expiry status.
        request
            .post(&format!("/api/workers/{worker}/registrations"))
            .json(&json!({ "body": "Test Regulator", "expires_on": "2000-01-01" }))
            .await
            .assert_status_ok();
        let regs: Value = request
            .get(&format!("/api/workers/{worker}/registrations"))
            .await
            .json();
        assert_eq!(regs[0]["status"], "expired");

        let overview: Value = request.get("/api/cpd/overview").await.json();
        assert!(overview["registrations"]["expired"].as_u64().unwrap() >= 1);
        assert!(
            overview["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["requirement_pid"] == requirement_pid)
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn internal_mobility_matches_own_skills_and_keeps_interest_private() {
    crate::requests::request_open(|request, _ctx| async move {
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
        // A different role is a different interest, not a duplicate.
        let other: Value = interest(json!({ "role_profile_pid": roles[1] })).await.json();
        request.delete(&format!("/api/mobility-interests/{}", other["pid"].as_str().unwrap())).await.assert_status_ok();

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
        assert_eq!(request.delete(&format!("/api/mobility-interests/{pid}")).await.status_code(), 404, "withdrawn once");
        let after: Value = request.get(&format!("/api/workers/{worker}/mobility-interests")).await.json();
        assert!(after.as_array().unwrap().is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn lms_completions_update_enrollments_and_cpd_idempotently() {
    crate::requests::request_open(|request, _ctx| async move {
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn change_tracker_reports_aggregate_readiness() {
    crate::requests::request_open(|request, _ctx| async move {
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn workforce_plan_forecasts_gaps_and_alignment() {
    crate::requests::request_open(|request, _ctx| async move {
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
        // The plan shows which objectives each line serves, and the list counts its lines.
        let detail: Value = request.get(&format!("/api/workforce-plans/{plan_pid}")).await.json();
        let lines = detail["demand_lines"].as_array().unwrap();
        assert_eq!(lines[0]["objective_pids"], json!([served["pid"]]));
        assert_eq!(lines[1]["objective_pids"], json!([]));
        assert_eq!(detail["objectives"].as_array().unwrap().len(), 2);
        let listed: Vec<Value> = request.get("/api/workforce-plans").await.json();
        let this = listed.iter().find(|p| p["pid"] == detail["pid"]).unwrap();
        assert_eq!(this["demand_lines"], 2);
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn workforce_plan_costs_hiring_against_a_budget() {
    crate::requests::request_open(|request, _ctx| async move {
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn capability_framework_import_and_progression() {
    use workforce_planning_management::rules::framework::LevelMapping;
    use workforce_planning_management::tasks::import_framework::import_pcf;

    crate::requests::request_open(|request, ctx| async move {
        let fixture =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pcf-mini");
        let report = import_pcf(
            workforce_planning_management::db::pool(&ctx),
            &fixture,
            LevelMapping::Identity,
            false,
        )
        .await
        .expect("import");
        assert_eq!(report.levels_read, 4);
        assert_eq!(report.retired_skipped, 1);
        assert_eq!(report.profiles_created, 3);
        assert_eq!(
            report.requirements_created, 6,
            "2 + 3 + 1 baselined skill lines"
        );
        assert_eq!(
            report.requirements_skipped_no_baseline, 1,
            "no invented level"
        );

        // Idempotent: a second run creates nothing and refreshes everything.
        let again = import_pcf(
            workforce_planning_management::db::pool(&ctx),
            &fixture,
            LevelMapping::Identity,
            false,
        )
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
        assert!(
            pcf["attribution"]
                .as_str()
                .unwrap()
                .contains("Open Government Licence")
        );
        assert!(pcf["profiles"].as_u64().unwrap() >= 3);

        // Profiles: titles as the framework writes them; the management track is flagged.
        let listed: Value = request
            .get("/api/role-profiles?framework=uk-gdad-pcf")
            .await
            .json();
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
            .get(&format!(
                "/api/role-profiles/{}",
                tester["pid"].as_str().unwrap()
            ))
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
        assert_eq!(
            skill_a["importance"], "important",
            "importance is a draft default for a human to edit"
        );

        // A planner's edit survives a re-import (unless overwrite_levels is asked for).
        let skill_pid = skill_a["skill_pid"].as_str().unwrap();
        request
            .put(&format!(
                "/api/role-profiles/{}/requirements",
                tester["pid"].as_str().unwrap()
            ))
            .json(
                &json!({ "skill_pid": skill_pid, "min_proficiency": 5, "importance": "critical" }),
            )
            .await
            .assert_status_ok();
        import_pcf(
            workforce_planning_management::db::pool(&ctx),
            &fixture,
            LevelMapping::Identity,
            false,
        )
        .await
        .expect("re-import");
        let kept: Value = request
            .get(&format!(
                "/api/role-profiles/{}",
                tester["pid"].as_str().unwrap()
            ))
            .await
            .json();
        let kept_a = kept["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["skill"] == "Fixture skill A")
            .unwrap();
        assert_eq!(
            kept_a["min_proficiency"], 5,
            "the planner's level is not overwritten"
        );
        import_pcf(
            workforce_planning_management::db::pool(&ctx),
            &fixture,
            LevelMapping::Identity,
            true,
        )
        .await
        .expect("overwrite");
        let reset: Value = request
            .get(&format!(
                "/api/role-profiles/{}",
                tester["pid"].as_str().unwrap()
            ))
            .await
            .json();
        let reset_a = reset["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["skill"] == "Fixture skill A")
            .unwrap();
        assert_eq!(
            reset_a["min_proficiency"], 3,
            "overwrite_levels re-derives from the source level"
        );

        // A planner's rename survives a re-import: matched by reference, no duplicate.
        request
            .put(&format!("/api/skills/{skill_pid}"))
            .json(&json!({ "name": "Renamed skill A" }))
            .await
            .assert_status_ok();
        import_pcf(
            workforce_planning_management::db::pool(&ctx),
            &fixture,
            LevelMapping::Identity,
            false,
        )
        .await
        .expect("re-import after rename");
        let catalogue: Value = request.get("/api/skills").await.json();
        let names: Vec<&str> = catalogue
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"Renamed skill A"));
        assert!(
            !names.contains(&"Fixture skill A"),
            "no duplicate under the framework's name"
        );

        // Progression: junior → tester raises A (1→3), keeps B, adds C.
        let step: Value = request
            .get(&format!(
                "/api/role-profiles/{}/progression",
                junior["pid"].as_str().unwrap()
            ))
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
            .get(&format!(
                "/api/role-profiles/{}/progression",
                lead["pid"].as_str().unwrap()
            ))
            .await
            .json();
        assert!(
            top["next"].as_array().unwrap().is_empty(),
            "nothing above the top level"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn skills_can_be_edited_categorised_and_referenced() {
    crate::requests::request_open(|request, _ctx| async move {
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn esco_import_search_and_seeding() {
    use workforce_planning_management::tasks::import_esco::import_esco;

    crate::requests::request_open(|request, ctx| async move {
        // Catalogue skills that may or may not link to ESCO.
        for name in ["Test software", "Work in teams", "Duplicate label"] {
            request
                .post("/api/skills")
                .json(&json!({ "name": name, "category": "other" }))
                .await
                .assert_status_ok();
        }
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/esco-mini");
        let report = import_esco(workforce_planning_management::db::pool(&ctx), &fixture, "en", "v-test").await.expect("import");
        assert_eq!((report.skills, report.occupations, report.relations), (5, 2, 4));
        assert_eq!(report.relations_dangling, 1, "the relation to a skill not in the file");
        assert_eq!(report.catalogue_linked, 2, "exact normalised labels, one match each");
        assert_eq!(report.catalogue_ambiguous, 1, "two ESCO skills share the label: not guessed");
        let again = import_esco(workforce_planning_management::db::pool(&ctx), &fixture, "en", "v-test").await.expect("re-import");
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
        assert_eq!(request.get("/api/esco/skills?q=a").await.status_code(), 422, "a skill search needs two characters too");
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn skills_merge_keeps_the_stronger_statement_and_delete_refuses_use() {
    crate::requests::request_open(|request, _ctx| async move {
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_person_selects_their_pcf_and_esco_roles_and_skills() {
    use workforce_planning_management::rules::framework::LevelMapping;
    use workforce_planning_management::tasks::import_esco::import_esco;
    use workforce_planning_management::tasks::import_framework::import_pcf;

    crate::requests::request_open(|request, ctx| async move {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        import_pcf(workforce_planning_management::db::pool(&ctx), &root.join("pcf-mini"), LevelMapping::Identity, false).await.expect("pcf");
        import_esco(workforce_planning_management::db::pool(&ctx), &root.join("esco-mini"), "en", "v-test").await.expect("esco");

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
        assert_eq!(
            put_skills("uk-gdad-pcf", json!({ "selections": [{ "ref": "not-a-skill-id", "proficiency": 3 }] })).await.status_code(),
            422,
            "a PCF ref is a skill id"
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
        assert_eq!(request.put(&format!("{base}/esco")).json(&json!({})).await.status_code(), 422, "ESCO needs an occupation");
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn career_history_and_aspirations() {
    use workforce_planning_management::rules::framework::LevelMapping;
    use workforce_planning_management::tasks::import_esco::import_esco;
    use workforce_planning_management::tasks::import_framework::import_pcf;

    crate::requests::request_open(|request, ctx| async move {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        import_pcf(workforce_planning_management::db::pool(&ctx), &root.join("pcf-mini"), LevelMapping::Identity, false).await.expect("pcf");
        import_esco(workforce_planning_management::db::pool(&ctx), &root.join("esco-mini"), "en", "v-test").await.expect("esco");
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

        // ── Clearing a framework with nothing selected is harmless; an unknown framework is refused.
        request.delete(&format!("/api/workers/{me}/framework-roles/esco")).await.assert_status_ok();
        assert_eq!(request.delete(&format!("/api/workers/{me}/framework-roles/sfia")).await.status_code(), 422);

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
        let today = workforce_planning_management::dates::today().to_string();
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

/// A manager who has left is not staff: the chain through them stops, above and below (WPM-R142: the
/// views read the chain, not every worker, and answer as before).
#[tokio::test]
#[ignore = "needs Postgres"]
async fn reporting_lines_stop_at_a_manager_who_has_left() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let mut pids = Vec::new();
        for n in ["L-1", "L-2", "L-3", "L-4"] {
            pids.push(seed_worker!(&request, &org, n, None).await);
        }
        let (top, middle, low, other) = (&pids[0], &pids[1], &pids[2], &pids[3]);
        for (w, m) in [(middle, top), (low, middle), (other, top)] {
            request
                .put(&format!("/api/workers/{w}"))
                .json(&json!({ "manager_pid": m }))
                .await
                .assert_status_ok();
        }
        let len = |v: &Value, key: &str| v[key].as_array().unwrap().len();
        let up: Value = request
            .get(&format!("/api/workers/{low}/upline"))
            .await
            .json();
        assert_eq!(len(&up, "upline"), 2);
        let down: Value = request
            .get(&format!("/api/workers/{top}/downline"))
            .await
            .json();
        assert_eq!(down["summary"]["total"], 3);

        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "UPDATE workers SET terminated_on = '2026-02-01' WHERE pid = '{middle}'"
        )))
        .execute(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        let up: Value = request
            .get(&format!("/api/workers/{low}/upline"))
            .await
            .json();
        assert_eq!(
            len(&up, "upline"),
            0,
            "the chain stops at a manager who has left"
        );
        let down: Value = request
            .get(&format!("/api/workers/{top}/downline"))
            .await
            .json();
        assert_eq!(
            down["summary"]["total"], 1,
            "and so does the downline: only the other direct report remains"
        );
        assert_eq!(down["downline"][0]["pid"], other.as_str());
        let gone: Value = request
            .get(&format!("/api/workers/{middle}/upline"))
            .await
            .json();
        assert_eq!(
            len(&gone, "upline"),
            0,
            "someone who has left has no upline"
        );
        let dot = request
            .post(&format!("/api/workers/{low}/dotted-line-managers"))
            .json(&json!({ "manager_pid": middle }))
            .await;
        assert_eq!(
            dot.status_code(),
            422,
            "a dotted-line manager must be employed"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn reporting_lines_downline_aspirations_and_groups() {
    crate::requests::request_open(|request, _ctx| async move {
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
        let capped: Value = request.get(&format!("/api/workers/{ceo}/downline?limit=1")).await.json();
        assert_eq!(capped["downline"].as_array().unwrap().len(), 1, "the list is capped");
        assert_eq!(capped["summary"]["total"], 4, "the summary still counts everyone");
        assert_eq!(capped["truncated"], true);
        assert_eq!(all["truncated"], false);
        let capped: Value = request.get(&format!("/api/workers/{ceo}/reports?kind=indirect&limit=1")).await.json();
        assert_eq!((capped["reports"].as_array().unwrap().len(), &capped["total"], &capped["truncated"]), (1, &json!(2), &json!(true)));
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn confederation_groups_and_transfers() {
    crate::requests::request_open(|request, ctx| async move {
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
        let ended: i64 = sqlx::query_scalar(
            "SELECT (snapshot->>'group_memberships_ended')::bigint FROM audit_logs
             WHERE entity_pid = $1::text::uuid AND action = 'transferred'",
        )
        .bind(&in_a)
        .fetch_one(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        assert_eq!(ended, 1, "the audit entry keeps what the move ended");
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

async fn capability_view(request: &axa_test::TestServer) -> Value {
    request
        .get("/api/workforce-intelligence/capability")
        .await
        .json()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// The capability view counts, in the database, what it used to count in memory: holders at and below their
// own target, the people with a gap and how many of them have an active plan, completed sittings per category.
// Read as the difference from before, since the test database holds other people.
async fn capability_counts_targets_plans_and_completed_sittings() {
    crate::requests::request_open(|request, ctx| async move {
        let org = super::an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let before = capability_view(&request).await;
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("Cap {tag}"), "category": "technical" }))
            .await
            .json();
        let skill_pid = skill["pid"].as_str().unwrap().to_string();
        let mut people = Vec::new();
        // (proficiency, target): at target, below with a plan, below without a plan, no target
        for (n, (level, target)) in [(3, Some(3)), (2, Some(4)), (1, Some(2)), (5, None)].into_iter().enumerate() {
            let w = super::seed_worker!(&request, &org, format!("CAP-{tag}-{n}"), None).await;
            super::activate!(&request, &w).await;
            let mut body = json!({ "skill_pid": skill_pid, "proficiency": level });
            if let Some(t) = target {
                body["target"] = json!(t);
            }
            request.put(&format!("/api/workers/{w}/skills")).json(&body).await.assert_status_ok();
            people.push(w);
        }
        // The second person has a development plan, made active.
        request
            .post(&format!("/api/workers/{}/development-plans", people[1]))
            .json(&json!({
                "kind": "upskill",
                "items": [{ "skill_pid": skill_pid, "current_level": 2, "target_level": 4, "method": "course" }],
            }))
            .await
            .assert_status_ok();
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "UPDATE development_plans SET status = 'active' WHERE worker_pid = '{}'",
                people[1]
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        // The first person completed an aptitude sitting; the third only scheduled one.
        let instrument: Value = request
            .post("/api/assessment-instruments")
            .json(&json!({ "name": format!("Apt {tag}"), "category": "aptitude", "scales": ["numerical_reasoning"] }))
            .await
            .json();
        for (who, complete) in [(&people[0], true), (&people[2], false), (&people[3], false)] {
            let sitting: Value = request
                .post("/api/assessments")
                .json(&json!({ "instrument_pid": instrument["pid"], "subject_kind": "worker", "subject_pid": who }))
                .await
                .json();
            let pid = sitting["pid"].as_str().unwrap();
            if complete {
                request
                    .post(&format!("/api/assessments/{pid}/results"))
                    .json(&json!({ "scale": "numerical_reasoning", "percentile": 60 }))
                    .await
                    .assert_status_ok();
                request
                    .post(&format!("/api/assessments/{pid}/status"))
                    .json(&json!({ "to": "completed" }))
                    .await
                    .assert_status_ok();
            }
        }

        let after = capability_view(&request).await;
        let row = after["skills"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["skill"] == format!("Cap {tag}").as_str())
            .expect("the skill is in the catalogue");
        assert_eq!(row["declared_by"], 4);
        assert_eq!(row["at_or_above_target"], 1, "3 against a target of 3 is at target");
        assert_eq!(row["below_target"], 2);
        let n = |v: &Value, k: &str| v[k].as_u64().unwrap();
        assert_eq!(n(&after, "people_with_a_declared_gap"), n(&before, "people_with_a_declared_gap") + 2);
        let covered = |v: &Value| v["gaps_with_an_active_plan"]["numerator"].as_u64().unwrap_or(0);
        assert_eq!(covered(&after), covered(&before) + 1, "only the one with an active plan");
        let assessed = |v: &Value| {
            v["assessment_coverage"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["category"] == "aptitude")
                .unwrap()["workers_assessed"]
                .as_u64()
                .unwrap()
        };
        assert_eq!(assessed(&after), assessed(&before) + 1, "a scheduled sitting does not count");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_group_is_renamed_recast_and_redescribed() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let make = |name: &str| request.post("/api/groups").json(&json!({ "organization_ref": org, "name": name, "kind": "interest", "description": "old" }));
        let group: Value = make("Book club").await.json();
        let other: Value = make("Film club").await.json();
        let (group, other) = (group["pid"].as_str().unwrap(), other["pid"].as_str().unwrap());
        let rename = |pid: &str, name: &str| {
            request.put(&format!("/api/groups/{pid}")).json(&json!({ "name": name, "kind": "practice", "description": "new" }))
        };
        let answer: Value = rename(group, "  Reading circle ").await.json();
        assert_eq!(answer["pid"], group);
        let listed: Value = request.get(&format!("/api/groups?organization_ref={org}")).await.json();
        let shown = listed.as_array().unwrap().iter().find(|g| g["pid"] == group).unwrap().clone();
        assert_eq!((&shown["name"], &shown["kind"], &shown["description"]), (&json!("Reading circle"), &json!("practice"), &json!("new")));
        assert_eq!(rename(group, "film CLUB").await.status_code(), 422, "another group's name, ignoring case");
        rename(group, "READING circle").await.assert_status_ok();
        let again: Value = request.get(&format!("/api/groups?organization_ref={org}")).await.json();
        assert_eq!(again.as_array().unwrap().iter().find(|g| g["pid"] == group).unwrap()["name"], "READING circle", "its own name, recased");
        let untouched = again.as_array().unwrap().iter().find(|g| g["pid"] == other).unwrap().clone();
        assert_eq!((&untouched["name"], &untouched["kind"], &untouched["description"]), (&json!("Film club"), &json!("interest"), &json!("old")));
    })
    .await;
}

/// Each refusal of a past role, a past skill and an aspiration, and a role aspiration's progress against a PCF level's skills.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn past_roles_past_skills_and_aspirations_refuse_what_they_cannot_name() {
    use workforce_planning_management::rules::framework::LevelMapping;
    use workforce_planning_management::tasks::import_esco::import_esco;
    use workforce_planning_management::tasks::import_framework::import_pcf;

    crate::requests::request_open(|request, ctx| async move {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        import_pcf(workforce_planning_management::db::pool(&ctx), &root.join("pcf-mini"), LevelMapping::Identity, false).await.expect("pcf");
        import_esco(workforce_planning_management::db::pool(&ctx), &root.join("esco-mini"), "en", "v-test").await.expect("esco");
        let org = an_org();
        let me = seed_worker!(&request, &org, format!("CR-{}", uuid::Uuid::new_v4().simple()), None).await;
        activate!(&request, &me).await;
        let profiles: Value = request.get("/api/role-profiles?framework=uk-gdad-pcf").await.json();
        let tester = profiles.as_array().unwrap().iter().find(|p| p["job_title"] == "Tester").unwrap()["pid"].clone();
        let own: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("Own {}", uuid::Uuid::new_v4()) })).await.json();
        let dates = json!({ "started_on": "2015-01-01", "ended_on": "2015-12-31" });
        let with = |extra: Value| {
            let mut body = dates.clone();
            for (k, v) in extra.as_object().unwrap() {
                body[k] = v.clone();
            }
            body
        };

        // A past role: an unknown framework, a PCF role missing or not from the PCF, an ESCO occupation missing or unknown.
        let past = |framework: &str, body: Value| request.post(&format!("/api/workers/{me}/framework-roles/{framework}/past")).json(&body);
        for (framework, body, why) in [
            ("sfia", with(json!({})), "unknown framework"),
            ("uk-gdad-pcf", with(json!({})), "no profile"),
            ("uk-gdad-pcf", with(json!({ "role_profile_pid": own["pid"] })), "not a PCF level"),
            ("esco", with(json!({})), "no occupation"),
            ("esco", with(json!({ "occupation_uri": "http://data.europa.eu/esco/occupation/nope" })), "not in the pinned copy"),
        ] {
            assert_eq!(past(framework, body).await.status_code(), 422, "{why}");
        }
        let esco = past("esco", with(json!({ "occupation_uri": "http://data.europa.eu/esco/occupation/q1" }))).await;
        assert_eq!(esco.status_code(), 200, "{}", esco.text());

        // A past skill: an unknown skill is a 404, and dates that end before they start a 422.
        let skill: Value = request.post("/api/skills").json(&json!({ "name": format!("Past {}", uuid::Uuid::new_v4()), "category": "technical" })).await.json();
        let past_skill = |body: Value| request.post(&format!("/api/workers/{me}/skill-history/past")).json(&body);
        assert_eq!(past_skill(json!({ "skill_pid": uuid::Uuid::new_v4(), "proficiency": 2, "started_on": "2015-01-01", "ended_on": "2015-12-31" })).await.status_code(), 404);
        assert_eq!(past_skill(json!({ "skill_pid": skill["pid"], "proficiency": 2, "started_on": "2016-01-01", "ended_on": "2015-01-01" })).await.status_code(), 422);

        // Aspirations: each missing or unknown target is refused.
        let aspire = |body: Value| request.post(&format!("/api/workers/{me}/aspirations")).json(&body);
        let h = "within_1y";
        for (body, code, why) in [
            (json!({ "kind": "role", "horizon": h }), 422, "a role needs a framework"),
            (json!({ "kind": "role", "framework_slug": "uk-gdad-pcf", "horizon": h }), 422, "the PCF needs a profile"),
            (json!({ "kind": "role", "framework_slug": "uk-gdad-pcf", "role_profile_pid": own["pid"], "horizon": h }), 422, "not a PCF level"),
            (json!({ "kind": "role", "framework_slug": "esco", "horizon": h }), 422, "ESCO needs an occupation"),
            (json!({ "kind": "role", "framework_slug": "esco", "occupation_uri": "http://data.europa.eu/esco/occupation/nope", "horizon": h }), 422, "not in the pinned copy"),
            (json!({ "kind": "skill", "target_level": 3, "horizon": h }), 422, "a skill needs a skill"),
            (json!({ "kind": "skill", "skill_pid": uuid::Uuid::new_v4(), "target_level": 3, "horizon": h }), 404, "an unknown skill"),
        ] {
            assert_eq!(aspire(body).await.status_code(), code, "{why}");
        }

        // A PCF role aspiration counts the level's required skills against those held.
        let made = aspire(json!({ "kind": "role", "framework_slug": "uk-gdad-pcf", "role_profile_pid": tester, "horizon": h })).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let listed: Value = request.get(&format!("/api/workers/{me}/aspirations")).await.json();
        let role = listed["aspirations"].as_array().unwrap().iter().find(|a| a["kind"] == "role").unwrap().clone();
        assert!(role["progress"]["requirements"].is_number() && role["progress"]["met"].is_number(), "{role}");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A merge folds the source's initiative shifts, plan items and external references into the target: each moves over unless
// the target already has one for the same initiative, plan or framework, when the target's stands and the source's goes.
async fn a_merge_folds_shifts_plan_items_and_references_into_the_target() {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let skill = |name: String| {
            let request = &request;
            async move {
                let made: Value = request.post("/api/skills").json(&json!({ "name": name, "category": "technical" })).await.json();
                made["pid"].as_str().unwrap().to_string()
            }
        };
        let (source, target) = (skill(format!("Fold source {tag}")).await, skill(format!("Fold target {tag}")).await);
        let (shared_initiative, own_initiative) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
        let (shared_plan, own_plan) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
        let mut sql = String::new();
        for (initiative, skill) in [(shared_initiative, &source), (shared_initiative, &target), (own_initiative, &source)] {
            sql.push_str(&format!(
                "INSERT INTO initiative_skill_shifts (pid, initiative_pid, skill_pid, direction) VALUES ('{}', '{initiative}', '{skill}', 'grow');",
                uuid::Uuid::new_v4()
            ));
        }
        for (plan, skill) in [(shared_plan, &source), (shared_plan, &target), (own_plan, &source)] {
            sql.push_str(&format!(
                "INSERT INTO development_plan_items (pid, plan_pid, skill_pid, current_level, target_level, method) \
                 VALUES ('{}', '{plan}', '{skill}', 1, 3, 'course');",
                uuid::Uuid::new_v4()
            ));
        }
        for (skill, framework) in [(&source, "esco"), (&target, "esco"), (&source, "sfia")] {
            sql.push_str(&format!(
                "INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, \"ref\") VALUES ('{}', '{skill}', '{framework}', '{framework}-{}');",
                uuid::Uuid::new_v4(),
                uuid::Uuid::new_v4().simple()
            ));
        }
        let pool = workforce_planning_management::db::pool(&ctx);
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql)).execute(pool).await.unwrap();

        let result = request.post(&format!("/api/skills/{source}/merge")).json(&json!({ "into_pid": target })).await;
        assert_eq!(result.status_code(), 200, "{}", result.text());
        let result: Value = result.json();
        assert_eq!(result["records_merged"], 2, "the shared initiative's shift and the shared plan's item: {result}");
        assert_eq!(result["records_moved"], 2, "the other initiative's shift and the other plan's item: {result}");

        let count = |table: &str, skill: &str| {
            let sql = format!("SELECT count(*) FROM {table} WHERE skill_pid = '{skill}'");
            async move { sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql)).fetch_one(pool).await.unwrap() }
        };
        for table in ["initiative_skill_shifts", "development_plan_items", "skill_external_refs"] {
            assert_eq!(count(table, &source).await, 0, "{table}: nothing is left on the source");
            assert_eq!(count(table, &target).await, 2, "{table}: the target's own and the one moved over");
        }
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// The import refuses a directory that is not the framework's clone, skips a role whose title another profile holds, and
// brings back a framework skill that was retired.
async fn the_framework_import_skips_a_taken_title_and_revives_a_retired_skill() {
    use workforce_planning_management::rules::framework::LevelMapping;
    use workforce_planning_management::tasks::import_framework::import_pcf;

    crate::requests::request_open(|_request, ctx| async move {
        let pool = workforce_planning_management::db::pool(&ctx);
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pcf-mini");
        let not_a_clone = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let refused = import_pcf(pool, &not_a_clone, LevelMapping::Identity, false).await;
        assert!(refused.is_err_and(|e| e.to_string().contains("not found")));

        import_pcf(pool, &fixture, LevelMapping::Identity, false).await.expect("import");
        let (moved_id, original_ref): (i32, String) = sqlx::query_as(
            "SELECT id, external_ref FROM role_profiles WHERE id = (SELECT min(id) FROM role_profiles WHERE framework_slug = 'uk-gdad-pcf')",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        // One profile no longer carries its framework reference, so its title is held by a profile the import does not own;
        // and one framework skill is retired.
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            "UPDATE role_profiles SET external_ref = 'moved-away' \
              WHERE id = (SELECT min(id) FROM role_profiles WHERE framework_slug = 'uk-gdad-pcf'); \
             UPDATE skills SET deleted_at = now() \
              WHERE pid = (SELECT skill_pid FROM skill_external_refs WHERE framework_slug = 'uk-gdad-pcf' ORDER BY id LIMIT 1)",
        ))
        .execute(pool)
        .await
        .unwrap();

        let again = import_pcf(pool, &fixture, LevelMapping::Identity, false).await.expect("re-import");
        assert_eq!(again.title_taken_skipped, 1, "{again:?}");
        assert_eq!(again.profiles_created, 0, "{again:?}");
        let retired: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM skills s JOIN skill_external_refs r ON r.skill_pid = s.pid \
              WHERE r.framework_slug = 'uk-gdad-pcf' AND s.deleted_at IS NOT NULL",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(retired, 0, "the retired framework skill is back");
        // Put the reference back, so a later import in the same database finds its own profile again.
        sqlx::query("UPDATE role_profiles SET external_ref = $2 WHERE id = $1")
            .bind(moved_id)
            .bind(&original_ref)
            .execute(pool)
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// Interest in a requisition: a draft one is refused, an open one is taken and counted by requisition with its title, and a
// note over 500 characters is refused.
async fn interest_in_an_open_requisition_is_counted_and_a_draft_is_refused() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("MB-{tag}"), None).await;
        activate!(&request, &worker).await;
        let title = format!("Data Lead {tag}");
        let requisition = |title: &str| {
            let (request, body) = (
                &request,
                json!({ "organization_ref": org, "department": "engineering", "job_title": title,
                                                      "headcount": 1, "salary_min_minor": 3_000_000, "salary_max_minor": 4_000_000, "salary_currency": "GBP" }),
            );
            async move {
                request
                    .post("/api/requisitions")
                    .json(&body)
                    .await
                    .json::<Value>()["pid"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }
        };
        let (draft, open) = (
            requisition(&format!("Draft {tag}")).await,
            requisition(&title).await,
        );
        request
            .post(&format!("/api/requisitions/{open}/status"))
            .json(&json!({ "to": "open" }))
            .await
            .assert_status_ok();
        let interests = format!("/api/workers/{worker}/mobility-interests");
        let refused = request
            .post(&interests)
            .json(&json!({ "requisition_pid": draft }))
            .await;
        assert_eq!(refused.status_code(), 422, "{}", refused.text());
        let long = request
            .post(&interests)
            .json(&json!({ "requisition_pid": open, "note": "x".repeat(501) }))
            .await;
        assert_eq!(long.status_code(), 422);
        request
            .post(&interests)
            .json(&json!({ "requisition_pid": open, "note": "keen" }))
            .await
            .assert_status_ok();

        let summary: Value = request.get("/api/mobility/interest-summary").await.json();
        let row = summary["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["target_pid"] == open.as_str())
            .expect("counted");
        assert_eq!(row["kind"], "requisition");
        assert_eq!(row["interested"], 1);
        assert_eq!(row["title"], title.as_str());
        assert!(!summary.to_string().contains(&worker), "no one is named");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A group's skills are ordered by how many declare them, then by name; a skill only one member declares is withheld and
// counted as such.
async fn a_groups_skills_are_ordered_and_a_lone_one_is_withheld() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let group: Value = request
            .post("/api/groups")
            .json(&json!({ "organization_ref": org, "name": format!("Guild {tag}"), "kind": "practice" }))
            .await
            .json();
        let group = group["pid"].as_str().unwrap().to_string();
        let mut members = Vec::new();
        for n in 0..4 {
            let w = seed_worker!(&request, &org, format!("GS-{tag}-{n}"), None).await;
            activate!(&request, &w).await;
            request.post(&format!("/api/groups/{group}/members")).json(&json!({ "worker_pid": w, "role": "member" })).await.assert_status_ok();
            members.push(w);
        }
        let skill = |name: String| {
            let request = &request;
            async move {
                let s: Value = request.post("/api/skills").json(&json!({ "name": name, "category": "technical" })).await.json();
                s["pid"].as_str().unwrap().to_string()
            }
        };
        let (gamma, alpha, beta, delta) = (
            skill(format!("Gamma {tag}")).await,
            skill(format!("alpha {tag}")).await,
            skill(format!("Beta {tag}")).await,
            skill(format!("Delta {tag}")).await,
        );
        for (skill, declared_by) in [(&gamma, 4), (&alpha, 3), (&beta, 3), (&delta, 1)] {
            for who in &members[..declared_by] {
                request
                    .put(&format!("/api/workers/{who}/skills"))
                    .json(&json!({ "skill_pid": skill, "proficiency": 3 }))
                    .await
                    .assert_status_ok();
            }
        }
        let roll: Value = request.get(&format!("/api/groups/{group}/skills")).await.json();
        let order: Vec<&str> = roll["skills"].as_array().unwrap().iter().map(|s| s["skill_pid"].as_str().unwrap()).collect();
        assert_eq!(order, [gamma.as_str(), alpha.as_str(), beta.as_str()], "most declared first, then by name ignoring case");
        assert_eq!(roll["withheld_below_floor"], 1, "{roll}");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A critical role at high risk of loss with nobody on its bench, in a department the plan has demand in, is counted as a
// critical role without a bench in the plan's departments.
async fn the_alignment_counts_a_critical_role_without_a_bench_in_the_plans_departments() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let department = format!("al-{}", uuid::Uuid::new_v4().simple());
        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({ "name": format!("Plan {department}"), "organization_ref": org,
                           "horizon_start": "2026-01-01", "horizon_end": "2027-12-31", "attrition_bp": 1000 }))
            .await
            .json();
        let plan = plan["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/workforce-plans/{plan}/demand-lines"))
            .json(&json!({ "department": department, "target_on": "2027-06-30", "target_headcount": 4 }))
            .await
            .assert_status_ok();
        request
            .post("/api/succession-plans")
            .json(&json!({ "role_title": format!("Head of {department}"), "department": department, "criticality": 5,
                           "risk_of_loss": "high" }))
            .await
            .assert_status_ok();
        let view: Value = request.get(&format!("/api/workforce-plans/{plan}/alignment")).await.json();
        let bench = &view["critical_roles_without_bench"];
        assert_eq!(bench["succession_plans_in_plan_departments"], 1, "{view}");
        assert_eq!(bench["in_plan_departments"], 1, "{view}");
        assert!(bench["all_departments"].as_u64().unwrap() >= 1, "{view}");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// With no attrition assumption, the forecast uses the organization's own snapshots: departments on the same day are added
// together, and the rate is observed, not assumed.
async fn a_forecast_with_no_assumption_uses_the_observed_snapshots() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let mut sql = String::new();
        // Three days, 181 days apart end to end; 60 + 40 people each day; 3 + 2 leavers after the first.
        for (as_of, leavers) in [("2026-01-01", None), ("2026-04-01", Some((3, 2))), ("2026-07-01", Some((3, 2)))] {
            for (department, headcount, left) in [("engineering", 60, leavers.map(|l: (i32, i32)| l.0)), ("sales", 40, leavers.map(|l| l.1))] {
                let left = left.map_or_else(|| "NULL".to_string(), |n| n.to_string());
                sql.push_str(&format!(
                    "INSERT INTO headcount_snapshots (pid, organization_ref, department, as_of, headcount, fte_percent_total, leavers) \
                     VALUES ('{}', '{org}', '{department}', '{as_of}', {headcount}, {}, {left});",
                    uuid::Uuid::new_v4(),
                    headcount * 100
                ));
            }
        }
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql)).execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();

        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({ "name": "Observed", "organization_ref": org, "horizon_start": "2026-01-01", "horizon_end": "2027-12-31" }))
            .await
            .json();
        let plan = plan["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/workforce-plans/{plan}/demand-lines"))
            .json(&json!({ "department": "engineering", "target_on": "2027-06-30", "target_headcount": 3 }))
            .await
            .assert_status_ok();
        let forecast: Value = request.get(&format!("/api/workforce-plans/{plan}/forecast")).await.json();
        assert_eq!(forecast["assumptions"]["attrition_source"], "observed_snapshots", "{forecast}");
        // 10 leavers over 181 days, a mean of 100 people: about 20% a year.
        let bp = forecast["assumptions"]["attrition_bp"].as_i64().expect("an observed rate");
        assert!((2_000..=2_030).contains(&bp), "{bp}");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A worker moved to another organization while their manager stays behind is flagged, so the reporting line is looked at.
async fn a_transfer_flags_a_manager_left_in_the_old_organization() {
    crate::requests::request_open(|request, _ctx| async move {
        let (from, to) = (an_org(), an_org());
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let boss = seed_worker!(&request, &from, format!("TR-{tag}-B"), None).await;
        let worker = seed_worker!(&request, &from, format!("TR-{tag}-W"), None).await;
        request
            .put(&format!("/api/workers/{worker}"))
            .json(&json!({ "manager_pid": boss }))
            .await
            .assert_status_ok();
        let moved = request
            .post(&format!("/api/workers/{worker}/transfer"))
            .json(&json!({ "organization_ref": to }))
            .await;
        assert_eq!(moved.status_code(), 200, "{}", moved.text());
        let moved: Value = moved.json();
        assert_eq!(moved["manager_in_other_organization"], true, "{moved}");
        assert_eq!(
            (moved["from"].as_str(), moved["to"].as_str()),
            (Some(from.as_str()), Some(to.as_str()))
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A role profile's description and a requirement's note are length-checked, a requirement needs a level, and a profile not
// from a framework has no next level to progress to.
async fn a_role_profile_checks_its_text_and_a_plain_one_has_no_progression() {
    crate::requests::request_open(|request, _ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let long = "x".repeat(1025);
        let too_long = json!({ "job_title": format!("Long {tag}"), "description": long });
        assert_eq!(request.post("/api/role-profiles").json(&too_long).await.status_code(), 422);
        let profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("Plain {tag}") })).await.json();
        let pid = profile["pid"].as_str().unwrap().to_string();
        let skill: Value = request.post("/api/skills").json(&json!({ "name": format!("Plain skill {tag}"), "category": "technical" })).await.json();
        let requirements = format!("/api/role-profiles/{pid}/requirements");
        for (body, why) in [
            (json!({ "skill_pid": skill["pid"], "importance": "critical", "min_proficiency": 3, "note": long }), "a note too long"),
            (json!({ "skill_pid": skill["pid"], "importance": "critical" }), "no level at all"),
        ] {
            assert_eq!(request.put(&requirements).json(&body).await.status_code(), 422, "{why}");
        }
        let progression: Value = request.get(&format!("/api/role-profiles/{pid}/progression")).await.json();
        assert_eq!(progression["next"], json!([]), "{progression}");
        assert_eq!(progression["profile"], format!("Plain {tag}").as_str());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A pairing made active that has never met is listed as stale, with both names and no last session.
async fn a_pairing_that_has_never_met_is_stale() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mentor = seed_worker!(&request, &org, format!("MS-{tag}-M"), None).await;
        let mentee = seed_worker!(&request, &org, format!("MS-{tag}-E"), None).await;
        activate!(&request, &mentor).await;
        activate!(&request, &mentee).await;
        let made: Value = request
            .post("/api/mentorships")
            .json(&json!({ "mentor_pid": mentor, "mentee_pid": mentee, "focus": "growth" }))
            .await
            .json();
        let pid = made["pid"].as_str().expect("mentorship pid").to_string();
        request
            .post(&format!("/api/mentorships/{pid}/status"))
            .json(&json!({ "to": "active" }))
            .await
            .assert_status_ok();
        let overview: Value = request
            .get("/api/learning/mentorship-overview?days=30")
            .await
            .json();
        let stale = overview["stale_mentorships"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["pid"] == pid.as_str());
        let stale = stale.expect("never met, so stale");
        assert!(stale["last_session"].is_null(), "{stale}");
        assert_eq!(stale["mentor"], format!("Test Worker MS-{tag}-M").as_str());
        assert_eq!(stale["mentee"], format!("Test Worker MS-{tag}-E").as_str());
    })
    .await;
}
