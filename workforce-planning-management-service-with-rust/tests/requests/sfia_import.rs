//! `import_sfia` (WPM-R109, WPM-R113): refuses a file with no recorded licence check, records the framework as
//! internal-only, is idempotent, retires what a newer version drops, and links by exact name only. The text is invented.

use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait};
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::models::_entities::{sfia_skill_levels, sfia_skills};
use workforce_planning_management::tasks::import_sfia::{Licence, import_sfia, import_sfia_levels};
use workforce_planning_management::tasks::import_sfiaplus::import_sfiaplus;

use super::{activate, an_org, seed_worker};

const HEAD: &str =
    "skill_code,skill_name,category,subcategory,description,level,level_description,attributes\n";

fn licence(version: &str, checked_on: &str) -> Licence {
    Licence {
        version: version.to_string(),
        checked_on: checked_on.to_string(),
        holder: Some("Test holder".to_string()),
        reference: None,
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_import_needs_a_licence_check_records_internal_only_retires_and_links_exactly() {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string()[..6].to_uppercase();
        let (a, b) = (format!("A{tag}"), format!("B{tag}"));
        let name_a = format!("Invented tuning {tag}");
        let file = format!(
            "{HEAD}{a},{name_a},Cat,Sub,Invented A,2,x,\n{a},{name_a},Cat,Sub,Invented A,3,y,\n\
             {b},Invented review {tag},Cat,Sub,Invented B,4,z,\n"
        );

        // No licence check recorded, or no version: refused, and nothing is written.
        assert!(
            import_sfia(&ctx.db, &file, &licence("9", ""))
                .await
                .is_err()
        );
        assert!(
            import_sfia(&ctx.db, &file, &licence("9", "yesterday"))
                .await
                .is_err()
        );
        assert!(
            import_sfia(&ctx.db, &file, &licence(" ", "2026-10-27"))
                .await
                .is_err()
        );
        let before = sfia_skills::Entity::find().count(&ctx.db).await.unwrap();
        assert!(
            import_sfia(&ctx.db, "code,name\nA,B\n", &licence("9", "2026-10-27"))
                .await
                .is_err(),
            "a wrong header"
        );
        assert_eq!(
            sfia_skills::Entity::find().count(&ctx.db).await.unwrap(),
            before
        );

        // A catalogue skill with the same name (any case) links; one without does not.
        let linked: Value = request
            .post("/api/skills")
            .json(&json!({ "name": name_a.to_lowercase(), "category": "technical" }))
            .await
            .json();
        request
            .post("/api/skills")
            .json(&json!({ "name": format!("Unrelated {tag}"), "category": "technical" }))
            .await
            .assert_status_ok();

        let report = import_sfia(&ctx.db, &file, &licence("9", "2026-10-27"))
            .await
            .expect("import");
        assert_eq!(
            (report.skills, report.levels, report.catalogue_linked),
            (2, 3, 1)
        );
        let again = import_sfia(&ctx.db, &file, &licence("9", "2026-10-27"))
            .await
            .expect("re-import");
        assert_eq!(
            (
                again.skills_retired,
                again.levels_retired,
                again.catalogue_linked
            ),
            (0, 0, 0),
            "idempotent"
        );

        let frameworks: Value = request.get("/api/capability-frameworks").await.json();
        let sfia = frameworks
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["slug"] == "sfia")
            .unwrap();
        assert_eq!(sfia["redistribution"], "internal_only");
        assert_eq!(sfia["scale_max"], 7);
        assert_eq!(sfia["version"], "9");
        assert_eq!(sfia["licence_checked_on"], "2026-10-27");
        let all = frameworks.as_array().unwrap();
        assert!(
            all.iter()
                .filter(|f| f["slug"] == "esco" || f["slug"] == "uk-gdad-pcf")
                .all(|f| f["redistribution"] == "open")
        );

        let listed: Value = request.get("/api/skills").await.json();
        let row = listed
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["pid"] == linked["pid"])
            .unwrap();
        assert_eq!(row["external_refs"][0]["framework"], "sfia");
        assert_eq!(row["external_refs"][0]["ref"], a);

        // A newer version without skill B and without A's level 3: kept, marked retired, never deleted.
        let newer = format!("{HEAD}{a},{name_a},Cat,Sub,Invented A,2,x2,\n");
        let report = import_sfia(&ctx.db, &newer, &licence("10", "2026-10-28"))
            .await
            .expect("newer");
        assert_eq!(
            (report.skills_retired, report.levels_retired),
            (1, 2),
            "B and its level 4, and A level 3"
        );
        let skill_b = sfia_skills::Entity::find_by_id(
            sfia_skills::Entity::find()
                .all(&ctx.db)
                .await
                .unwrap()
                .into_iter()
                .find(|s| s.code == b)
                .unwrap()
                .id,
        )
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
        assert_eq!(skill_b.retired_in.as_deref(), Some("10"));
        let levels = sfia_skill_levels::Entity::find()
            .all(&ctx.db)
            .await
            .unwrap();
        let l3 = levels
            .iter()
            .find(|l| l.code == a && l.level == 3)
            .expect("kept");
        assert_eq!(l3.retired_in.as_deref(), Some("10"));
        let l2 = levels.iter().find(|l| l.code == a && l.level == 2).unwrap();
        assert_eq!(
            (l2.description.as_str(), l2.retired_in.clone()),
            ("x2", None),
            "updated in place"
        );

        // A skill that comes back is restored.
        import_sfia(&ctx.db, &file, &licence("11", "2026-10-29"))
            .await
            .expect("back");
        let levels = sfia_skill_levels::Entity::find()
            .all(&ctx.db)
            .await
            .unwrap();
        assert!(
            levels
                .iter()
                .filter(|l| l.code == a || l.code == b)
                .all(|l| l.retired_in.is_none())
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_levels_of_responsibility_are_a_job_level_framework_like_the_coded_one() {
    crate::requests::request_open(|request, ctx| async move {
        let levels = include_str!("../fixtures/sfia-synthetic/levels.csv");
        assert!(
            import_sfia_levels(&ctx.db, levels, &licence("9", ""))
                .await
                .is_err(),
            "no licence check"
        );
        assert!(
            import_sfia_levels(&ctx.db, "level,title\n1,a\n", &licence("9", "2026-10-27"))
                .await
                .is_err()
        );
        let none: Value = request.get("/api/job-levels").await.json();
        assert!(
            none.as_array()
                .unwrap()
                .iter()
                .all(|f| f["id"] != "sfia-levels"),
            "refusals wrote nothing"
        );

        assert_eq!(
            import_sfia_levels(&ctx.db, levels, &licence("9", "2026-10-27"))
                .await
                .unwrap(),
            7
        );
        assert_eq!(
            import_sfia_levels(&ctx.db, levels, &licence("9", "2026-10-27"))
                .await
                .unwrap(),
            7,
            "idempotent"
        );

        let list: Value = request.get("/api/job-levels").await.json();
        let ids: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["id"].as_str().unwrap())
            .collect();
        assert!(ids.contains(&"google-levels") && ids.contains(&"sfia-levels"));
        let one: Value = request.get("/api/job-levels/sfia-levels").await.json();
        assert_eq!(one["levels"].as_array().unwrap().len(), 7);
        assert_eq!(one["levels"][3]["code"], "L4");
        assert_eq!(one["levels"][3]["attributes"].as_array().unwrap().len(), 5);
        let google: Value = request.get("/api/job-levels/google-levels").await.json();
        assert!(
            google["levels"][0].get("attributes").is_none(),
            "the coded ladder is unchanged"
        );
        let step: Value = request
            .get("/api/job-levels/sfia-levels/levels/l4")
            .await
            .json();
        assert_eq!(step["next_level"]["number"], 5);

        // A worker's and a role's level can be an SFIA level; the same validation applies.
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("SL-{tag}"), None).await;
        activate!(&request, &worker).await;
        let url = format!("/api/workers/{worker}/job-level");
        for bad in [
            json!({ "framework": "sfia-levels", "level": "L8" }),
            json!({ "framework": "sfia-levels", "level": "0" }),
        ] {
            assert_eq!(request.put(&url).json(&bad).await.status_code(), 422);
        }
        let set: Value = request
            .put(&url)
            .json(&json!({ "framework": "sfia-levels", "level": "4" }))
            .await
            .json();
        assert_eq!(set["job_level"]["framework"], "sfia-levels");
        assert_eq!(set["job_level"]["level"]["title"], "Invented level 4");
        let got: Value = request.get(&url).await.json();
        assert_eq!(got["job_level"]["level"]["number"], 4);

        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("SL role {tag}") }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap();
        let grade: Value = request
            .put(&format!("/api/role-profiles/{role}/grade"))
            .json(&json!({ "job_level": { "framework": "sfia-levels", "level": "L6" } }))
            .await
            .json();
        assert_eq!(grade["job_level"]["level"]["code"], "L6");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_native_level_is_kept_mapped_shown_and_never_invented() {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string()[..6].to_uppercase();
        let (a, b) = (format!("N{tag}"), format!("M{tag}"));
        let name_a = format!("Invented drafting {tag}");
        let file = format!(
            "{HEAD}{a},{name_a},Cat,Sub,Invented A,2,words two,\n{a},{name_a},Cat,Sub,Invented A,3,words three,\n\
             {a},{name_a},Cat,Sub,Invented A,5,words five,\n{b},Invented checking {tag},Cat,Sub,Invented B,4,words b4,\n"
        );
        let skill: Value = request.post("/api/skills").json(&json!({ "name": name_a, "category": "technical" })).await.json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        import_sfia(&ctx.db, &file, &licence("9", "2026-10-27")).await.expect("import");

        // Search and show: the wording comes from the deployer's own import.
        let found: Value = request.get(&format!("/api/sfia/skills?q={}", a.to_lowercase())).await.json();
        assert_eq!(found[0]["code"], a);
        assert_eq!(found[0]["levels"], json!([2, 3, 5]));
        assert_eq!(request.get("/api/sfia/skills?q=a").await.status_code(), 422);
        let shown: Value = request.get(&format!("/api/sfia/skills/{a}")).await.json();
        assert_eq!(shown["mapping"], "linear");
        assert_eq!(shown["levels"][1]["wpm_level"], 2, "SFIA 3 is WPM 2");
        assert_eq!(request.get("/api/sfia/skills/NOPE").await.status_code(), 404);

        // A declared skill keeps SFIA's level, the 1–5 figure comes from the mapping, and the wording is shown.
        let org = an_org();
        let worker = seed_worker!(&request, &org, format!("NL-{tag}"), None).await;
        activate!(&request, &worker).await;
        let url = format!("/api/workers/{worker}/skills");
        let declared: Value = request
            .put(&url)
            .json(&json!({ "skill_pid": skill, "framework": "sfia", "framework_level": 5 }))
            .await
            .json();
        assert_eq!((declared["proficiency"].clone(), declared["source_level"].clone(), declared["source_scale_max"].clone()), (json!(4), json!(5), json!(7)));
        let listed: Value = request.get(&url).await.json();
        let row = listed.as_array().unwrap().iter().find(|r| r["skill_pid"] == skill.as_str()).unwrap();
        assert_eq!((row["mapping"].as_str(), row["native_description"].as_str()), (Some("linear"), Some("words five")));
        assert_eq!(row["source_framework_name"], "SFIA — Skills Framework for the Information Age", "named, not the slug");
        for bad in [
            json!({ "skill_pid": skill, "framework": "sfia", "framework_level": 4 }),
            json!({ "skill_pid": skill, "framework": "sfia", "framework_level": 8 }),
            json!({ "skill_pid": skill, "framework": "sfia" }),
            json!({ "skill_pid": skill, "framework_level": 3 }),
            json!({ "skill_pid": skill, "framework": "nope", "framework_level": 3 }),
            json!({ "skill_pid": skill }),
        ] {
            assert_eq!(request.put(&url).json(&bad).await.status_code(), 422, "{bad}");
        }
        // A stated 1–5 stands; a plain declaration clears the native level rather than leave a stale one.
        let both: Value = request
            .put(&url)
            .json(&json!({ "skill_pid": skill, "proficiency": 2, "framework": "sfia", "framework_level": 3 }))
            .await
            .json();
        assert_eq!((both["proficiency"].clone(), both["source_level"].clone()), (json!(2), json!(3)));
        let plain: Value = request.put(&url).json(&json!({ "skill_pid": skill, "proficiency": 3 })).await.json();
        assert_eq!(plain["source_level"], Value::Null);

        // A role requirement the same way.
        let profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("NL role {tag}") })).await.json();
        let role = profile["pid"].as_str().unwrap();
        request
            .put(&format!("/api/role-profiles/{role}/requirements"))
            .json(&json!({ "skill_pid": skill, "importance": "critical", "framework": "sfia", "framework_level": 3 }))
            .await
            .assert_status_ok();
        let got: Value = request.get(&format!("/api/role-profiles/{role}")).await.json();
        let req = &got["requirements"][0];
        assert_eq!((req["min_proficiency"].clone(), req["source_framework"].clone(), req["source_level"].clone()), (json!(2), json!("sfia"), json!(3)));
        assert_eq!((req["mapping"].as_str(), req["native_description"].as_str()), (Some("linear"), Some("words three")));
        assert_eq!(req["source_framework_name"], "SFIA — Skills Framework for the Information Age");

        // Start a role from SFIA skills: only levels SFIA defines, nothing created when one is refused.
        let title = format!("NL started {tag}");
        let body = |skills: Value, title: &str| json!({ "job_title": title, "skills": skills });
        for bad in [
            body(json!([{ "code": a, "level": 4 }]), &title),
            body(json!([{ "code": "NOPE", "level": 2 }]), &title),
            body(json!([{ "code": a, "level": 2 }, { "code": a, "level": 3 }]), &title),
            body(json!([]), &title),
            json!({ "job_title": title, "skills": [{ "code": a, "level": 2 }], "category": "astrology" }),
        ] {
            assert_eq!(request.post("/api/role-profiles/from-sfia").json(&bad).await.status_code(), 422, "{bad}");
        }
        let roles: Value = request.get("/api/role-profiles").await.json();
        assert!(roles.as_array().unwrap().iter().all(|r| r["job_title"] != title.as_str()), "refusals made no role");
        let made: Value = request
            .post("/api/role-profiles/from-sfia")
            .json(&json!({ "job_title": title, "skills": [{ "code": a, "level": 2 }, { "code": b, "level": 4 }], "category": "domain" }))
            .await
            .json();
        let catalogue: Value = request.get("/api/skills").await.json();
        let created = catalogue.as_array().unwrap().iter().find(|s| s["name"] == format!("Invented checking {tag}").as_str()).unwrap();
        assert_eq!(created["category"], "domain", "the planner's choice, not a default");
        assert_eq!((made["requirements_created"].clone(), made["skills_created"].clone(), made["skills_linked"].clone()), (json!(2), json!(1), json!(0)), "A is already linked by code (by name at import); B is new");
        let started: Value = request.get(&format!("/api/role-profiles/{}", made["pid"].as_str().unwrap())).await.json();
        let mut levels: Vec<(i64, i64)> = started["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["source_level"].as_i64().unwrap(), r["min_proficiency"].as_i64().unwrap()))
            .collect();
        levels.sort_unstable();
        assert_eq!(levels, [(2, 2), (4, 3)], "SFIA 2 is WPM 2, SFIA 4 is WPM 3");
        assert_eq!(
            request.post("/api/role-profiles/from-sfia").json(&body(json!([{ "code": a, "level": 2 }]), &title)).await.status_code(),
            422,
            "the title is taken"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn sfiaplus_resources_are_pointers_at_the_level_a_need_counts_as_and_nothing_is_written_on_a_refusal()
 {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string()[..6].to_uppercase();
        let a = format!("P{tag}");
        let name_a = format!("Invented mending {tag}");
        let skills_csv = format!(
            "{HEAD}{a},{name_a},Cat,Sub,Invented A,2,w2,\n{a},{name_a},Cat,Sub,Invented A,3,w3,\n{a},{name_a},Cat,Sub,Invented A,5,w5,\n"
        );
        let rhead = "skill_code,level,kind,title,provider,url\n";
        let skill: Value = request.post("/api/skills").json(&json!({ "name": name_a, "category": "technical" })).await.json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        import_sfia(&ctx.db, &skills_csv, &licence("9", "2026-10-27")).await.expect("skills");

        // Refusals write nothing: no licence check, a skill not in the catalogue, a level SFIA does not define for it.
        let good = format!("{rhead}{a},5,certification,Invented certificate {tag},Invented Body,https://example.invalid/c\n");
        assert!(import_sfiaplus(&ctx.db, &good, &licence("1", "")).await.is_err());
        let unknown = format!("{rhead}NOPE{tag},2,training,t,,\n");
        assert!(import_sfiaplus(&ctx.db, &unknown, &licence("1", "2026-10-27")).await.unwrap_err().to_string().contains("not in the imported catalogue"));
        let undefined = format!("{rhead}{a},4,training,t,,\n");
        assert!(import_sfiaplus(&ctx.db, &undefined, &licence("1", "2026-10-27")).await.unwrap_err().to_string().contains("not defined at level 4"));
        let frameworks: Value = request.get("/api/capability-frameworks").await.json();
        assert!(frameworks.as_array().unwrap().iter().all(|f| f["slug"] != "sfiaplus"), "no licence record from a refused file");

        let file = format!(
            "{rhead}{a},5,certification,Invented certificate {tag},Invented Body,https://example.invalid/c\n\
             {a},5,training,Invented course {tag},,\n{a},3,membership,Invented society {tag},,\n"
        );
        let report = import_sfiaplus(&ctx.db, &file, &licence("1", "2026-10-27")).await.expect("sfiaplus");
        assert_eq!((report.resources, report.retired), (3, 0));
        let again = import_sfiaplus(&ctx.db, &file, &licence("1", "2026-10-27")).await.expect("again");
        assert_eq!((again.resources, again.retired), (3, 0), "idempotent");
        let frameworks: Value = request.get("/api/capability-frameworks").await.json();
        let plus = frameworks.as_array().unwrap().iter().find(|f| f["slug"] == "sfiaplus").unwrap();
        assert_eq!((plus["redistribution"].clone(), plus["licence_checked_on"].clone()), (json!("internal_only"), json!("2026-10-27")));

        // The skill page lists them per level.
        let shown: Value = request.get(&format!("/api/sfia/skills/{a}")).await.json();
        let at = |n: i64| shown["levels"].as_array().unwrap().iter().find(|l| l["level"] == n).unwrap()["sfiaplus"].as_array().unwrap().len();
        assert_eq!((at(2), at(3), at(5)), (0, 1, 2));

        // A person below a need that counts as WPM 4 (SFIA 5) gets the level-5 pointers, not the level-3 one (WPM 2).
        let org = an_org();
        let w = seed_worker!(&request, &org, format!("SP-{tag}"), None).await;
        activate!(&request, &w).await;
        let profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("SP role {tag}") })).await.json();
        let role = profile["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/role-profiles/{role}/requirements"))
            .json(&json!({ "skill_pid": skill, "importance": "critical", "framework": "sfia", "framework_level": 5 }))
            .await
            .assert_status_ok();
        ctx.db.execute_unprepared(&format!("UPDATE role_profiles SET framework_slug = 'uk-gdad-pcf' WHERE pid = '{role}'")).await.unwrap();
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO worker_framework_roles (pid, worker_pid, framework_slug, role_profile_pid, role_label, selected_on, started_at, on_behalf) \
                 VALUES (gen_random_uuid(), '{w}', 'uk-gdad-pcf', '{role}', 'SP role', CURRENT_DATE, now(), false)"
            ))
            .await
            .unwrap();
        request.put(&format!("/api/workers/{w}/skills")).json(&json!({ "skill_pid": skill, "proficiency": 2 })).await.assert_status_ok();

        let plan: Value = request.get(&format!("/api/workers/{w}/training-plan")).await.json();
        let pointers = plan["plan"][0]["sfiaplus_pointers"].as_array().unwrap();
        let mut titles: Vec<&str> = pointers.iter().map(|p| p["title"].as_str().unwrap()).collect();
        titles.sort_unstable();
        assert_eq!(titles, [format!("Invented certificate {tag}"), format!("Invented course {tag}")]);
        assert!(pointers.iter().all(|p| p["source"].as_str().unwrap().starts_with("SFIAplus") && p["sfia_level"] == 5));
        let gaps: Value = request.get(&format!("/api/workers/{w}/skill-gaps")).await.json();
        assert_eq!(gaps["gaps"][0]["sfiaplus_pointers"].as_array().unwrap().len(), 2);

        // A newer file without the membership and the course: kept, marked retired, and no longer pointed at.
        let newer = format!("{rhead}{a},5,certification,Invented certificate {tag},Invented Body,https://example.invalid/c\n");
        let report = import_sfiaplus(&ctx.db, &newer, &licence("2", "2026-10-28")).await.expect("newer");
        assert_eq!((report.resources, report.retired), (1, 2));
        let plan: Value = request.get(&format!("/api/workers/{w}/training-plan")).await.json();
        assert_eq!(plan["plan"][0]["sfiaplus_pointers"].as_array().unwrap().len(), 1);
    })
    .await;
}
