//! Grades: a worker's job level (person and HR only, audited without the level,
//! exported and erased with the person) and a role's job level and pay band (a
//! link made by the editor, validated against the reference ladders).

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_workers_level_is_validated_audited_without_the_level_and_erased() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "GR-1", None).await;
        activate!(&request, &worker).await;
        let url = format!("/api/workers/{worker}/job-level");

        // Nothing recorded yet: null, not a guess.
        let none: Value = request.get(&url).await.json();
        assert_eq!(none["job_level"], Value::Null);

        // Unknown framework, unknown level, a future date: all refused.
        for bad in [
            json!({ "framework": "meta-levels", "level": "L5" }),
            json!({ "framework": "google-levels", "level": "L2" }),
            json!({ "framework": "google-levels", "level": "L5", "effective_on": "2999-01-01" }),
        ] {
            assert_eq!(
                request.put(&url).json(&bad).await.status_code(),
                422,
                "{bad}"
            );
        }

        // Set, read back (level spelt `l5`), change, then clear.
        let set: Value = request
            .put(&url)
            .json(&json!({ "framework": "google-levels", "level": "l5",
                           "effective_on": "2024-03-01" }))
            .await
            .json();
        assert_eq!(set["job_level"]["level"]["code"], "L5");
        assert_eq!(set["job_level"]["effective_on"], "2024-03-01");
        let got: Value = request.get(&url).await.json();
        assert_eq!(
            got["job_level"]["level"]["title"],
            "Senior Software Engineer"
        );
        let changed: Value = request
            .put(&url)
            .json(&json!({ "framework": "google-levels", "level": "6" }))
            .await
            .json();
        assert_eq!(changed["job_level"]["level"]["code"], "L6");

        // The audit trail records that it changed, never to what.
        let audit = serde_json::to_string(
            &request
                .get(&format!("/api/audits/{worker}"))
                .await
                .json::<Value>(),
        )
        .unwrap();
        assert!(audit.contains("job_level_set"), "{audit}");
        assert!(
            !audit.contains("google-levels") && !audit.contains("L6"),
            "{audit}"
        );

        // It travels with the person: the subject-access export includes it.
        let export: Value = request
            .get(&format!("/api/workers/{worker}/subject-access"))
            .await
            .json();
        assert_eq!(export["job_level"][0]["level_number"], 6);

        // Erasure deletes it (after the lifecycle's offboarding path).
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        let erased = request.post(&format!("/api/workers/{worker}/erase")).await;
        assert_eq!(erased.status_code(), 200);
        use sea_orm::{ConnectionTrait, Statement};
        let row = ctx
            .db
            .query_one_raw(Statement::from_string(
                ctx.db.get_database_backend(),
                format!("SELECT count(*) FROM worker_job_levels WHERE worker_pid = '{worker}'"),
            ))
            .await
            .unwrap()
            .unwrap();
        let count: i64 = row.try_get_by_index(0).unwrap();
        assert_eq!(count, 0, "the level is erased with the person");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_role_links_a_level_to_a_pay_band_by_the_editors_say_so() {
    crate::requests::request_open(|request, _ctx| async move {
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": "Grade Test Engineer", "source_ref": "test:grade" }))
            .await
            .json();
        let url = format!(
            "/api/role-profiles/{}/grade",
            profile["pid"].as_str().unwrap()
        );

        let empty: Value = request.get(&url).await.json();
        assert_eq!(empty["job_level"], Value::Null);
        assert_eq!(empty["pay_band"], Value::Null);

        // Nothing, an unknown band, an unknown scale, an unknown level: refused.
        for bad in [
            json!({}),
            json!({ "pay_band": { "scale": "national-2026-27", "band": "12" } }),
            json!({ "pay_band": { "scale": "national-2025-26", "band": "5" } }),
            json!({ "job_level": { "framework": "google-levels", "level": "L1" } }),
        ] {
            assert_eq!(
                request.put(&url).json(&bad).await.status_code(),
                422,
                "{bad}"
            );
        }

        // Both set: the role is "L5 / Band 7", the link being the editor's.
        let set: Value = request
            .put(&url)
            .json(
                &json!({ "job_level": { "framework": "google-levels", "level": "L5" },
                           "pay_band": { "scale": "national-2026-27", "band": "7" } }),
            )
            .await
            .json();
        assert_eq!(set["job_level"]["level"]["code"], "L5");
        assert_eq!(set["pay_band"]["band"], "7");
        assert_eq!(set["pay_band"]["entry_minor"], 5_012_900);
        assert_eq!(set["pay_band"]["top_minor"], 5_736_500);
        assert_eq!(set["pay_band"]["currency"], "GBP");

        // PUT replaces: a level alone clears the band.
        let only: Value = request
            .put(&url)
            .json(&json!({ "job_level": { "framework": "google-levels", "level": "L6" } }))
            .await
            .json();
        assert_eq!(only["job_level"]["level"]["code"], "L6");
        assert_eq!(only["pay_band"], Value::Null);

        request.delete(&url).await.assert_status_success();
        let cleared: Value = request.get(&url).await.json();
        assert_eq!(cleared["job_level"], Value::Null);
        assert_eq!(
            request
                .get("/api/role-profiles/00000000-0000-4000-8000-000000000000/grade")
                .await
                .status_code(),
            404
        );
    })
    .await;
}
