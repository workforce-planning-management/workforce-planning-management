//! Skills gap analysis: a person's gaps merged from their role, their own
//! targets and (for them alone) aspirations; and the workforce roll-up, which
//! counts people and never names them.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn skill_gaps_merge_sources_rank_by_priority_and_roll_up_without_names() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let a = seed_worker!(&request, &org, format!("SG-{tag}-A"), None).await;
        let b = seed_worker!(&request, &org, format!("SG-{tag}-B"), None).await;
        activate!(&request, &a).await;
        activate!(&request, &b).await;

        let mut skills = Vec::new();
        for n in 1..=4 {
            let s: Value = request
                .post("/api/skills")
                .json(&json!({ "name": format!("SG{tag}-{n}"), "category": "technical" }))
                .await
                .json();
            skills.push(s["pid"].as_str().unwrap().to_string());
        }
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("SG role {tag}") }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();
        for (skill, level, importance) in [
            (&skills[0], 4, "critical"),
            (&skills[1], 3, "important"),
            (&skills[2], 2, "useful"),
        ] {
            request
                .put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": skill, "min_proficiency": level, "importance": importance }))
                .await
                .assert_status_ok();
        }
        // Make it a PCF role and give both workers that current role.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "UPDATE role_profiles SET framework_slug = 'uk-gdad-pcf' WHERE pid = '{role}'"
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        for w in [&a, &b] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                    "INSERT INTO worker_framework_roles \
                     (pid, worker_pid, framework_slug, role_profile_pid, role_label, selected_on, started_at, on_behalf) \
                     VALUES (gen_random_uuid(), '{w}', 'uk-gdad-pcf', '{role}', 'SG role', CURRENT_DATE, now(), false)"
                ))).execute(workforce_planning_management::db::pool(&ctx))
                .await
                .unwrap();
        }
        let declare = |worker: &str, skill: &str, level: i32, target: Option<i32>| {
            let mut body = json!({ "skill_pid": skill, "proficiency": level });
            if let Some(t) = target {
                body["target"] = json!(t);
            }
            (format!("/api/workers/{worker}/skills"), body)
        };
        // A: S1 at 2 with an own target of 3 (the role's critical 4 still rules: the strongest importance and the
        // highest level win), S2 at 3 with a target of 4. B: S1 at 4 (met), S2 at 1.
        for (url, body) in [
            declare(&a, &skills[0], 2, Some(3)),
            declare(&a, &skills[1], 3, Some(4)),
            declare(&b, &skills[0], 4, None),
            declare(&b, &skills[1], 1, None),
        ] {
            request.put(&url).json(&body).await.assert_status_ok();
        }
        // A privately aspires to S3 at level 3; B to the unrelated S4.
        for (worker, skill) in [(&a, &skills[2]), (&b, &skills[3])] {
            request
                .post(&format!("/api/workers/{worker}/aspirations"))
                .json(&json!({ "kind": "skill", "skill_pid": skill, "target_level": 3, "horizon": "within_1y" }))
                .await
                .assert_status_ok();
        }

        // One person: ranked, merged, with sources; undeclared is unknown.
        let mine: Value = request.get(&format!("/api/workers/{a}/skill-gaps")).await.json();
        assert_eq!(mine["includes_aspirations"], true, "their own view");
        assert_eq!(mine["counts"], json!({ "met": 0, "below": 2, "undeclared": 1 }));
        let gaps = mine["gaps"].as_array().unwrap();
        let order: Vec<&str> = gaps.iter().map(|g| g["skill_pid"].as_str().unwrap()).collect();
        assert_eq!(order[..2], [skills[0].as_str(), skills[1].as_str()], "real gaps first, by priority");
        assert_eq!(gaps[0]["shortfall"], 2);
        assert_eq!(gaps[0]["priority"], 6, "critical 3 × 2 levels");
        assert_eq!(gaps[1]["required"], 4, "the person's own target raised the level");
        assert_eq!(gaps[1]["sources"], json!(["role", "target"]));
        let s3 = gaps.iter().find(|g| g["skill_pid"] == skills[2].as_str()).unwrap();
        assert_eq!(s3["status"], "undeclared");
        assert!(s3["shortfall"].is_null() && s3["priority"] == 0, "unknown has no size");
        assert_eq!(s3["sources"], json!(["role", "aspiration"]));

        // The workforce: counts, ranked, nobody named, aspirations excluded.
        let all: Value = request
            .get("/api/workforce-intelligence/skill-gaps?department=ENGINEERING&limit=100")
            .await
            .json();
        let rows: Vec<&Value> = all["skills"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| skills.iter().any(|s| r["skill_pid"] == s.as_str()))
            .collect();
        let by = |i: usize| rows.iter().find(|r| r["skill_pid"] == skills[i].as_str()).copied();
        let s1 = by(0).unwrap();
        assert_eq!((s1["needed_by"].as_u64(), s1["met"].as_u64(), s1["below"].as_u64()), (Some(2), Some(1), Some(1)));
        assert_eq!(s1["score"], 6);
        assert_eq!(s1["critical_below"], 1);
        let s2 = by(1).unwrap();
        assert_eq!(s2["below"], 2, "A (target 4, has 3) and B (needs 3, has 1)");
        assert_eq!(s2["total_shortfall"], 3);
        assert_eq!(s2["score"], 6, "important 2 × 3 levels");
        let s3 = by(2).unwrap();
        assert_eq!((s3["undeclared"].as_u64(), s3["score"].as_u64()), (Some(2), Some(0)));
        assert!(by(3).is_none(), "an aspiration is never part of the workforce view");
        let order: Vec<&str> = rows.iter().map(|r| r["skill_pid"].as_str().unwrap()).collect();
        assert_eq!(order, [skills[0].as_str(), skills[1].as_str(), skills[2].as_str()], "score, then critical-below");
        // No one is named.
        let text = all.to_string();
        assert!(!text.contains(a.as_str()) && !text.contains(b.as_str()), "workers are not named");
        assert!(all["workers_considered"].as_u64().unwrap() >= 2);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn training_plans_recommend_courses_estimate_the_rest_and_schedule_by_priority() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("TP-{tag}"), None).await;
        activate!(&request, &w).await;

        let mut skills = Vec::new();
        for n in 1..=3 {
            let s: Value = request
                .post("/api/skills")
                .json(&json!({ "name": format!("TP{tag}-{n}"), "category": "technical" }))
                .await
                .json();
            skills.push(s["pid"].as_str().unwrap().to_string());
        }
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("TP role {tag}") }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();
        for (skill, level, importance) in
            [(&skills[0], 4, "critical"), (&skills[1], 3, "important"), (&skills[2], 2, "useful")]
        {
            request
                .put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": skill, "min_proficiency": level, "importance": importance }))
                .await
                .assert_status_ok();
        }
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "UPDATE role_profiles SET framework_slug = 'uk-gdad-pcf' WHERE pid = '{role}'"
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "INSERT INTO worker_framework_roles \
                 (pid, worker_pid, framework_slug, role_profile_pid, role_label, selected_on, started_at, on_behalf) \
                 VALUES (gen_random_uuid(), '{w}', 'uk-gdad-pcf', '{role}', 'TP role', CURRENT_DATE, now(), false)"
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        // S1: 2 → needs 4 (short 2). S2: 1 → needs 3 (short 2). S3: undeclared.
        for (skill, level) in [(&skills[0], 2), (&skills[1], 1)] {
            request
                .put(&format!("/api/workers/{w}/skills"))
                .json(&json!({ "skill_pid": skill, "proficiency": level }))
                .await
                .assert_status_ok();
        }

        // The catalogue: validation, then S1 gets two courses and S2 a per-level figure.
        let fast = "course:11111111-1111-4111-8111-1111111111a1";
        let big = "course:11111111-1111-4111-8111-1111111111a2";
        let courses = format!("/api/skills/{}/courses", skills[0]);
        for bad in [
            json!({ "course_ref": "", "title": "x", "hours": 5 }),
            json!({ "course_ref": fast, "title": "x", "hours": 0 }),
            json!({ "course_ref": fast, "title": "x", "hours": 5, "levels": 5 }),
        ] {
            assert_eq!(request.post(&courses).json(&bad).await.status_code(), 422, "{bad}");
        }
        let fast_row: Value = request
            .post(&courses)
            .json(&json!({ "course_ref": fast, "title": "Fast intro", "hours": 10 }))
            .await
            .json();
        request
            .post(&courses)
            .json(&json!({ "course_ref": big, "title": "Deep dive", "hours": 60, "levels": 2 }))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .post(&courses)
                .json(&json!({ "course_ref": fast, "title": "again", "hours": 10 }))
                .await
                .status_code(),
            422,
            "already listed"
        );
        assert_eq!(
            request
                .put(&format!("/api/skills/{}/training-hours", skills[1]))
                .json(&json!({ "hours_per_level": 0 }))
                .await
                .status_code(),
            422
        );
        request
            .put(&format!("/api/skills/{}/training-hours", skills[1]))
            .json(&json!({ "hours_per_level": 20 }))
            .await
            .assert_status_ok();

        // The plan: S1 (priority 6) before S2 (4); S1 by courses (10h + 60h), S2 by the estimate (2 × 20h).
        let url = format!("/api/workers/{w}/training-plan?weekly_hours=5&start=2026-10-05");
        let plan: Value = request.get(&url).await.json();
        assert_eq!(plan["weekly_hours"], 5);
        let items = plan["plan"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["skill_pid"], skills[0].as_str());
        assert_eq!(items[0]["recommendation"]["basis"], "courses");
        assert_eq!(items[0]["recommendation"]["hours"], 70);
        assert_eq!(items[0]["recommendation"]["courses"][0]["title"], "Fast intro", "cheapest per level first");
        assert_eq!(items[1]["recommendation"]["basis"], "estimate");
        assert_eq!(items[1]["recommendation"]["hours"], 40);
        // 70h at 5/week = 14 weeks (5 Oct → 10 Jan); then 40h = 8 weeks from 11 Jan.
        assert_eq!(items[0]["weeks"], 14);
        assert_eq!(items[0]["starts_on"], "2026-10-05");
        assert_eq!(items[0]["ends_on"], "2027-01-10");
        assert_eq!(items[1]["starts_on"], "2027-01-11");
        assert_eq!(plan["total_hours"], 110);
        assert_eq!(plan["total_weeks"], 22);
        assert_eq!(plan["finish_on"], "2027-03-07");
        // The undeclared skill has no hours: assess first.
        let assess = plan["assess_first"].as_array().unwrap();
        assert_eq!(assess.len(), 1);
        assert_eq!(assess[0]["skill_pid"], skills[2].as_str());
        assert!(items.iter().all(|i| i["skill_pid"] != skills[2].as_str()));

        // Bad pace refused; the default pace follows FTE (full time = 4h).
        assert_eq!(
            request.get(&format!("/api/workers/{w}/training-plan?weekly_hours=0")).await.status_code(),
            422
        );
        let default: Value = request.get(&format!("/api/workers/{w}/training-plan")).await.json();
        assert_eq!(default["weekly_hours"], 4);

        // Completing a course drops it from the recommendation.
        let enrolled: Value = request
            .post(&format!("/api/workers/{w}/training-enrollments"))
            .json(&json!({ "course_ref": fast }))
            .await
            .json();
        request
            .put(&format!("/api/training-enrollments/{}", enrolled["pid"].as_str().unwrap()))
            .json(&json!({ "status": "completed", "completed_on": "2026-09-01" }))
            .await
            .assert_status_ok();
        let after: Value = request.get(&url).await.json();
        let first = &after["plan"][0]["recommendation"];
        assert_eq!(first["courses"].as_array().unwrap().len(), 1);
        assert_eq!(first["courses"][0]["title"], "Deep dive");
        assert_eq!(first["hours"], 60);

        // The workforce view: hours and counts, nobody named.
        let demand: Value = request
            .get("/api/workforce-intelligence/training-demand?department=engineering")
            .await
            .json();
        let s1 = demand["skills"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["skill_pid"] == skills[0].as_str())
            .expect("S1 in demand");
        assert_eq!((s1["people"].as_u64(), s1["hours"].as_u64(), s1["people_on_estimate"].as_u64()), (Some(1), Some(60), Some(0)));
        assert!(demand["total_hours"].as_u64().unwrap() >= 100);
        assert!(!demand.to_string().contains(w.as_str()), "workers are not named");

        // Removing a course returns that skill to the estimate.
        request
            .delete(&format!("/api/skill-courses/{}", fast_row["pid"].as_str().unwrap()))
            .await
            .assert_status_ok();
        let listed: Value = request.get(&courses).await.json();
        assert_eq!(listed["courses"].as_array().unwrap().len(), 1);
        assert_eq!(listed["default_hours_per_level"], 30);

        // Every catalogue write is audited (WPM-T246): the course's creation and removal, and the per-level figure.
        let actions = |entity: &'static str, pid: String| {
            let db = workforce_planning_management::db::pool(&ctx).clone();
            async move {
                sqlx::query_scalar::<_, String>(
                    "SELECT action FROM audit_logs WHERE entity = $1 AND entity_pid = $2::uuid ORDER BY id",
                )
                .bind(entity)
                .bind(pid)
                .fetch_all(&db)
                .await
                .unwrap()
            }
        };
        let fast_pid = fast_row["pid"].as_str().unwrap().to_string();
        assert_eq!(actions("skill_course", fast_pid).await, ["created", "deleted"]);
        assert_eq!(actions("skill", skills[1].clone()).await.last().map(String::as_str), Some("updated"));
    })
    .await;
}

/// A person's role in a framework, and the skills they select from it (`framework_roles`): choosing the same role again changes
/// nothing, a new choice ends the old one, the role's skills carry the catalogue's name and the person's own level, a selection
/// declares, a `null` level withdraws and a later level brings the same declaration back, and a skill outside the framework is
/// refused.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_framework_role_lists_its_skills_and_the_selection_declares_them() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("FR-{tag}"), None).await;
        activate!(&request, &w).await;
        let mut skills = Vec::new();
        for n in 1..=2 {
            let s: Value = request
                .post("/api/skills")
                .json(&json!({ "name": format!("FR{tag}-{n}"), "category": "technical" }))
                .await
                .json();
            skills.push(s["pid"].as_str().unwrap().to_string());
        }
        let mut roles = Vec::new();
        for n in 1..=2 {
            let profile: Value = request
                .post("/api/role-profiles")
                .json(&json!({ "job_title": format!("FR role {tag} {n}") }))
                .await
                .json();
            roles.push(profile["pid"].as_str().unwrap().to_string());
        }
        request
            .put(&format!("/api/role-profiles/{}/requirements", roles[0]))
            .json(&json!({ "skill_pid": skills[0], "min_proficiency": 3, "importance": "critical" }))
            .await
            .assert_status_ok();
        // Both roles are PCF levels; only the first skill carries a PCF reference.
        let pool = workforce_planning_management::db::pool(&ctx);
        for role in &roles {
            sqlx::query("UPDATE role_profiles SET framework_slug = 'uk-gdad-pcf' WHERE pid = $1")
                .bind(uuid::Uuid::parse_str(role).unwrap())
                .execute(pool)
                .await
                .unwrap();
        }
        sqlx::query(
            r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label) VALUES ($1, $2, 'uk-gdad-pcf', $3, 'FR')"#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(uuid::Uuid::parse_str(&skills[0]).unwrap())
        .bind(format!("pcf:{tag}"))
        .execute(pool)
        .await
        .unwrap();

        let selectable: Value = request.get("/api/frameworks/selectable").await.json();
        let pcf = selectable.as_array().unwrap().iter().find(|f| f["slug"] == "uk-gdad-pcf").unwrap().clone();
        assert!(pcf["roles"].as_i64().unwrap() >= 2 && pcf["available"] == true, "{pcf}");

        let base = format!("/api/workers/{w}/framework-roles");
        let pcf_role = format!("{base}/uk-gdad-pcf");
        // The second role first, then the first: the second ends, so one current role remains.
        let first: Value =
            request.put(&pcf_role).json(&json!({ "role_profile_pid": roles[1] })).await.json();
        assert_eq!(first["role_label"], format!("FR role {tag} 2"));
        let again: Value =
            request.put(&pcf_role).json(&json!({ "role_profile_pid": roles[1] })).await.json();
        assert_eq!(again["started_at"], first["started_at"], "the same role again changes nothing");
        let chosen: Value =
            request.put(&pcf_role).json(&json!({ "role_profile_pid": roles[0] })).await.json();
        assert_eq!(chosen["role_label"], format!("FR role {tag} 1"));
        let current: Value = request.get(&base).await.json();
        assert_eq!(current.as_array().unwrap().len(), 1, "a new role ends the old one: {current}");
        assert_eq!(current[0]["role_profile_pid"], roles[0].as_str());
        let ended: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM worker_framework_roles WHERE worker_pid = $1 AND ended_at IS NOT NULL",
        )
        .bind(uuid::Uuid::parse_str(&w).unwrap())
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(ended, 1);

        let listed = role_skills(&request, &pcf_role).await;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["ref"], skills[0].as_str());
        assert_eq!(listed[0]["label"], format!("FR{tag}-1"), "named from the catalogue");
        assert_eq!(listed[0]["role_expects"], 3);
        assert!(listed[0]["declared"].is_null(), "nothing declared yet");

        let select = |level: Value| json!({ "selections": [{ "ref": skills[0], "proficiency": level }] });
        let set: Value = request.put(&format!("{pcf_role}/skills")).json(&select(json!(2))).await.json();
        assert_eq!((set["declared"].as_u64(), set["removed"].as_u64()), (Some(1), Some(0)));
        assert_eq!(role_skills(&request, &pcf_role).await[0]["declared"], 2);
        let cleared: Value = request.put(&format!("{pcf_role}/skills")).json(&select(Value::Null)).await.json();
        assert_eq!(cleared["removed"], 1);
        assert!(
            role_skills(&request, &pcf_role).await[0]["declared"].is_null(),
            "a null level withdraws the declaration"
        );
        request.put(&format!("{pcf_role}/skills")).json(&select(json!(4))).await.assert_status_ok();
        assert_eq!(
            role_skills(&request, &pcf_role).await[0]["declared"],
            4,
            "a later level brings it back"
        );
        let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM worker_skills WHERE worker_pid = $1 AND skill_pid = $2")
            .bind(uuid::Uuid::parse_str(&w).unwrap())
            .bind(uuid::Uuid::parse_str(&skills[0]).unwrap())
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(rows, 1, "one declaration, revived rather than duplicated");

        let outside = json!({ "selections": [{ "ref": skills[1], "proficiency": 2 }] });
        assert_eq!(request.put(&format!("{pcf_role}/skills")).json(&outside).await.status_code(), 422);

        request.delete(&pcf_role).await.assert_status_ok();
        let none: Value = request.get(&base).await.json();
        assert!(none.as_array().unwrap().is_empty(), "cleared: {none}");
        assert_eq!(
            request.get(&format!("{pcf_role}/skills")).await.status_code(),
            422,
            "no role, no skills to list"
        );
    })
    .await;
}

/// The skills of a worker's current role in a framework, as `GET .../skills` lists them.
async fn role_skills(request: &axa_test::TestServer, pcf_role: &str) -> Vec<Value> {
    let body: Value = request.get(&format!("{pcf_role}/skills")).await.json();
    body["skills"].as_array().unwrap().clone()
}
