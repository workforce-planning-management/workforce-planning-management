//! The skills crosswalk (WPM-R158, WPM-D109): mappings with a stated basis, and gap analysis reading them.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_mapping_lets_a_declared_skill_stand_in_for_a_need_only_when_it_is_exact_or_close() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let a = seed_worker!(&request, &org, format!("CW-{tag}-A"), None).await;
        activate!(&request, &a).await;
        let mut skills = Vec::new();
        for n in 1..=3 {
            let s: Value = request.post("/api/skills").json(&json!({ "name": format!("CW{tag}-{n}"), "category": "technical" })).await.json();
            skills.push(s["pid"].as_str().unwrap().to_string());
        }
        // The role needs S1 at 3 and S3 at 5; the person declared only S2, at 4.
        let profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("CW role {tag}") })).await.json();
        let role = profile["pid"].as_str().unwrap().to_string();
        for (skill, level) in [(&skills[0], 3), (&skills[2], 5)] {
            request.put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": skill, "min_proficiency": level, "importance": "important" })).await.assert_status_ok();
        }
        ctx.db.execute_unprepared(&format!("UPDATE role_profiles SET framework_slug = 'uk-gdad-pcf' WHERE pid = '{role}'")).await.unwrap();
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO worker_framework_roles (pid, worker_pid, framework_slug, role_profile_pid, role_label, selected_on, started_at, on_behalf) \
             VALUES (gen_random_uuid(), '{a}', 'uk-gdad-pcf', '{role}', 'CW role', CURRENT_DATE, now(), false)")).await.unwrap();
        request.put(&format!("/api/workers/{a}/skills")).json(&json!({ "skill_pid": skills[1], "proficiency": 4 })).await.assert_status_ok();
        let gaps_url = format!("/api/workers/{a}/skill-gaps");

        // No mapping: both needs are unknown, and the view says how many no mapping reaches.
        let none: Value = request.get(&gaps_url).await.json();
        assert_eq!(none["counts"], json!({ "met": 0, "below": 0, "undeclared": 2 }));
        assert_eq!(none["crosswalk"]["unmapped_undeclared"], 2);

        // A mapping needs two real, different skills, a known confidence and basis, and a stated source; the pair is unique.
        let url = "/api/skill-mappings";
        for bad in [
            json!({ "from_skill_pid": skills[1], "to_skill_pid": skills[1], "confidence": "exact", "basis": "authority", "source": "x" }),
            json!({ "from_skill_pid": skills[1], "to_skill_pid": skills[0], "confidence": "similar", "basis": "authority", "source": "x" }),
            json!({ "from_skill_pid": skills[1], "to_skill_pid": skills[0], "confidence": "exact", "basis": "hunch", "source": "x" }),
            json!({ "from_skill_pid": skills[1], "to_skill_pid": skills[0], "confidence": "exact", "basis": "judgement", "source": " " }),
            json!({ "from_skill_pid": uuid::Uuid::new_v4(), "to_skill_pid": skills[0], "confidence": "exact", "basis": "authority", "source": "x" }),
        ] {
            assert_eq!(request.post(url).json(&bad).await.status_code(), 422, "{bad}");
        }

        // Broader: shown, never counted.
        let broader: Value = request.post(url)
            .json(&json!({ "from_skill_pid": skills[1], "to_skill_pid": skills[0], "confidence": "broader", "basis": "judgement", "source": "Head of Resourcing, reviewed 2026" }))
            .await.json();
        let again = request.post(url)
            .json(&json!({ "from_skill_pid": skills[1], "to_skill_pid": skills[0], "confidence": "exact", "basis": "authority", "source": "x" })).await;
        assert_eq!(again.status_code(), 422, "the pair is already mapped");
        let seen: Value = request.get(&gaps_url).await.json();
        assert_eq!(seen["counts"], json!({ "met": 0, "below": 0, "undeclared": 2 }));
        assert_eq!(seen["crosswalk"]["related_not_counted"].as_array().map(Vec::len), Some(1));
        assert_eq!(seen["crosswalk"]["unmapped_undeclared"], 1, "S3 is still reached by nothing");

        // Withdraw it and state an exact one for S1 and a close one for S3: S1 is met (4 against 3), S3 is short by one (4 against 5).
        request.delete(&format!("/api/skill-mappings/{}", broader["pid"].as_str().unwrap())).await.assert_status_ok();
        for (to, confidence) in [(&skills[0], "exact"), (&skills[2], "close")] {
            let r = request.post(url)
                .json(&json!({ "from_skill_pid": skills[1], "to_skill_pid": to, "confidence": confidence, "basis": "authority", "source": "A published crosswalk, 2025" })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
        }
        let through: Value = request.get(&gaps_url).await.json();
        assert_eq!(through["counts"], json!({ "met": 1, "below": 1, "undeclared": 0 }));
        assert_eq!(through["crosswalk"]["graded_through_a_mapping"], 2);
        let listed = through["gaps"].as_array().unwrap();
        assert_eq!(listed.len(), 1, "only S3 is still a gap");
        assert_eq!(listed[0]["skill_pid"], skills[2].as_str());
        assert_eq!(listed[0]["shortfall"], 1);
        assert_eq!(listed[0]["through_mapping"]["confidence"], "close");
        assert_eq!(listed[0]["through_mapping"]["declared_skill_pid"], skills[1].as_str());

        // The list names both sides and says what counts.
        let all: Value = request.get(url).await.json();
        let mine: Vec<&Value> = all["mappings"].as_array().unwrap().iter().filter(|m| m["from_skill_pid"] == skills[1].as_str()).collect();
        assert_eq!(mine.len(), 2);
        assert!(mine.iter().all(|m| m["counts"] == true && m["from_skill"].is_string()));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_workforce_roll_up_grades_through_the_crosswalk_as_one_person_does() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut skills = Vec::new();
        for n in 1..=4 {
            let s: Value = request.post("/api/skills").json(&json!({ "name": format!("RU{tag}-{n}"), "category": "technical" })).await.json();
            skills.push(s["pid"].as_str().unwrap().to_string());
        }
        // The role needs S1 at 3 and S3 at 5. Worker A declared S2 at 4. Worker B declared S2 at 4 and S4 at 2.
        let profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("RU role {tag}") })).await.json();
        let role = profile["pid"].as_str().unwrap().to_string();
        for (skill, level) in [(&skills[0], 3), (&skills[2], 5)] {
            request.put(&format!("/api/role-profiles/{role}/requirements"))
                .json(&json!({ "skill_pid": skill, "min_proficiency": level, "importance": "important" })).await.assert_status_ok();
        }
        ctx.db.execute_unprepared(&format!("UPDATE role_profiles SET framework_slug = 'uk-gdad-pcf' WHERE pid = '{role}'")).await.unwrap();
        let mut people = Vec::new();
        for who in ["A", "B"] {
            let w = seed_worker!(&request, &org, format!("RU-{tag}-{who}"), None).await;
            activate!(&request, &w).await;
            ctx.db.execute_unprepared(&format!(
                "UPDATE workers SET department = 'ru{tag}' WHERE pid = '{w}'; \
                 INSERT INTO worker_framework_roles (pid, worker_pid, framework_slug, role_profile_pid, role_label, selected_on, started_at, on_behalf) \
                 VALUES (gen_random_uuid(), '{w}', 'uk-gdad-pcf', '{role}', 'RU role', CURRENT_DATE, now(), false)")).await.unwrap();
            request.put(&format!("/api/workers/{w}/skills")).json(&json!({ "skill_pid": skills[1], "proficiency": 4 })).await.assert_status_ok();
            people.push(w);
        }
        request.put(&format!("/api/workers/{}/skills", people[1])).json(&json!({ "skill_pid": skills[3], "proficiency": 2 })).await.assert_status_ok();

        let roll = |skill: String| {
            let request = &request;
            let department = format!("ru{tag}");
            async move {
                let body: Value = request.get(&format!("/api/workforce-intelligence/skill-gaps?department={department}")).await.json();
                body["skills"].as_array().unwrap().iter().find(|s| s["skill_pid"] == skill.as_str()).cloned().unwrap_or(Value::Null)
            }
        };
        let counts = |row: &Value| (row["met"].as_i64(), row["below"].as_i64(), row["undeclared"].as_i64());

        // No mapping: both needs are unknown for both people.
        assert_eq!(counts(&roll(skills[0].clone()).await), (Some(0), Some(0), Some(2)));

        // A broader mapping is never counted.
        let post = |from: &str, to: &str, confidence: &str| {
            let request = &request;
            let body = json!({ "from_skill_pid": from, "to_skill_pid": to, "confidence": confidence, "basis": "authority", "source": "A published crosswalk, 2025" });
            async move { request.post("/api/skill-mappings").json(&body).await }
        };
        let broader: Value = post(&skills[1], &skills[0], "broader").await.json();
        assert_eq!(counts(&roll(skills[0].clone()).await), (Some(0), Some(0), Some(2)), "broader is shown by the person's view and counted by nobody's");
        request.delete(&format!("/api/skill-mappings/{}", broader["pid"].as_str().unwrap())).await.assert_status_ok();

        // S2 close to S1 (4 against 3) and exact to S3 (4 against 5): A is met on S1 and short on S3.
        post(&skills[1], &skills[0], "close").await.assert_status_ok();
        post(&skills[1], &skills[2], "exact").await.assert_status_ok();
        // B also declared S4 at 2, exact to S1: the exact mapping outranks the close one, so B is short on S1 (2 against 3).
        post(&skills[3], &skills[0], "exact").await.assert_status_ok();
        let s1 = roll(skills[0].clone()).await;
        assert_eq!(counts(&s1), (Some(1), Some(1), Some(0)), "A met through close; B below through exact");
        let s3 = roll(skills[2].clone()).await;
        assert_eq!(counts(&s3), (Some(0), Some(2), Some(0)), "both are short of 5 with 4");

        // The same people, one at a time, say the same thing.
        let one = |w: &str| {
            let request = &request;
            let url = format!("/api/workers/{w}/skill-gaps");
            async move { request.get(&url).await.json::<Value>() }
        };
        let a = one(&people[0]).await;
        assert_eq!(a["counts"], json!({ "met": 1, "below": 1, "undeclared": 0 }));
        let b = one(&people[1]).await;
        assert_eq!(b["counts"], json!({ "met": 0, "below": 2, "undeclared": 0 }));
        let b_s1 = b["gaps"].as_array().unwrap().iter().find(|g| g["skill_pid"] == skills[0].as_str()).unwrap();
        assert_eq!(b_s1["through_mapping"]["confidence"], "exact");
        assert_eq!(b_s1["shortfall"], 1);

        // A direct declaration is never overridden by a mapping: B declares S1 at 5 and is met on it.
        request.put(&format!("/api/workers/{}/skills", people[1])).json(&json!({ "skill_pid": skills[0], "proficiency": 5 })).await.assert_status_ok();
        assert_eq!(counts(&roll(skills[0].clone()).await), (Some(2), Some(0), Some(0)));
    })
    .await;
}
