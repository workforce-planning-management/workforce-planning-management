//! The assessment round-trip: the instrument catalog and its
//! category↔scale rule, a sitting's lifecycle, per-scale results, the
//! derived profile, and the aggregate analytics.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one worker, the whole assessment surface
async fn assessment_round_trip() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "A-1", None).await;
        activate!(&request, &worker).await;

        // ── The instrument catalog enforces the category↔scale rule.
        assert_eq!(
            request
                .post("/api/assessment-instruments")
                .json(&json!({
                    "name": "Mis-filed", "category": "aptitude",
                    "scales": ["work_style"],
                }))
                .await
                .status_code(),
            422,
            "a personality scale does not belong on an aptitude instrument"
        );
        assert_eq!(
            request
                .post("/api/assessment-instruments")
                .json(&json!({ "name": "Nonsense", "category": "astrology" }))
                .await
                .status_code(),
            422,
            "unknown category refused"
        );

        let aptitude: Value = request
            .post("/api/assessment-instruments")
            .json(&json!({
                "name": "SHL Verify G+", "category": "aptitude", "provider": "SHL",
                "scales": ["numerical_reasoning", "logical_thinking"],
                "duration_minutes": 36, "validity_months": 24,
            }))
            .await
            .json();
        let aptitude_pid = aptitude["pid"]
            .as_str()
            .expect("instrument pid")
            .to_string();

        // Psychometric spans aptitude and personality — the one overlap.
        let psychometric: Value = request
            .post("/api/assessment-instruments")
            .json(&json!({
                "name": "Battery", "category": "psychometric",
                "scales": ["emotional_intelligence", "numerical_reasoning", "team_compatibility"],
            }))
            .await
            .json();
        assert!(psychometric["pid"].is_string(), "psychometric spans both");

        let catalog: Value = request
            .get("/api/assessment-instruments?category=aptitude")
            .await
            .json();
        assert_eq!(catalog.as_array().expect("catalog").len(), 1, "filtered");

        // ── Schedule a sitting.
        let sitting: Value = request
            .post("/api/assessments")
            .json(&json!({
                "instrument_pid": aptitude_pid,
                "subject_kind": "worker",
                "subject_pid": worker,
                "administered_by": "hr-ops",
            }))
            .await
            .json();
        let sitting_pid = sitting["pid"].as_str().expect("assessment pid").to_string();

        // Completing with no results is refused: "completed" must not
        // assert a scoring that never happened.
        assert_eq!(
            request
                .post(&format!("/api/assessments/{sitting_pid}/status"))
                .json(&json!({ "to": "completed" }))
                .await
                .status_code(),
            422,
            "no results ⇒ cannot complete"
        );

        // ── Results: the scale must suit the instrument.
        assert_eq!(
            request
                .post(&format!("/api/assessments/{sitting_pid}/results"))
                .json(&json!({ "scale": "work_style", "percentile": 50 }))
                .await
                .status_code(),
            422,
            "a personality scale on an aptitude sitting"
        );
        assert_eq!(
            request
                .post(&format!("/api/assessments/{sitting_pid}/results"))
                .json(&json!({ "scale": "verbal_reasoning", "percentile": 50 }))
                .await
                .status_code(),
            422,
            "the instrument does not report verbal reasoning"
        );
        assert_eq!(
            request
                .post(&format!("/api/assessments/{sitting_pid}/results"))
                .json(&json!({ "scale": "numerical_reasoning", "percentile": 101 }))
                .await
                .status_code(),
            422,
            "percentile is 0-100"
        );
        assert_eq!(
            request
                .post(&format!("/api/assessments/{sitting_pid}/results"))
                .json(&json!({
                    "scale": "numerical_reasoning", "raw_score": 30, "max_score": 20,
                }))
                .await
                .status_code(),
            422,
            "raw score above the maximum"
        );

        request
            .post(&format!("/api/assessments/{sitting_pid}/results"))
            .json(&json!({
                "scale": "numerical_reasoning", "percentile": 95,
                "raw_score": 19, "max_score": 20,
            }))
            .await
            .assert_status_ok();
        // Upsert: re-recording the same scale keeps one row.
        request
            .post(&format!("/api/assessments/{sitting_pid}/results"))
            .json(&json!({ "scale": "numerical_reasoning", "percentile": 92 }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/assessments/{sitting_pid}/results"))
            .json(&json!({ "scale": "logical_thinking", "percentile": 45 }))
            .await
            .assert_status_ok();

        let detail: Value = request
            .get(&format!("/api/assessments/{sitting_pid}"))
            .await
            .json();
        let results = detail["results"].as_array().expect("results");
        assert_eq!(results.len(), 2, "the re-recorded scale is one row");
        let numerical = results
            .iter()
            .find(|r| r["scale"] == "numerical_reasoning")
            .expect("numerical result");
        assert_eq!(numerical["percentile"], 92, "the later recording wins");
        assert_eq!(
            numerical["band"], "high",
            "band derived from the percentile"
        );

        // ── Lifecycle: an illegal move is refused; completing derives
        // the expiry from the instrument's validity.
        assert_eq!(
            request
                .post(&format!("/api/assessments/{sitting_pid}/status"))
                .json(&json!({ "to": "expired" }))
                .await
                .status_code(),
            422,
            "cannot expire a scheduled sitting"
        );
        request
            .post(&format!("/api/assessments/{sitting_pid}/status"))
            .json(&json!({ "to": "completed" }))
            .await
            .assert_status_ok();
        let completed: Value = request
            .get(&format!("/api/assessments/{sitting_pid}"))
            .await
            .json();
        assert!(
            completed["assessment"]["expires_on"].is_string(),
            "24-month validity derived an expiry"
        );

        // ── The derived profile.
        let profile: Value = request
            .get(&format!("/api/workers/{worker}/assessment-profile"))
            .await
            .json();
        let aptitude_slice = profile["categories"]
            .as_array()
            .expect("categories")
            .iter()
            .find(|c| c["category"] == "aptitude")
            .expect("aptitude slice")
            .clone();
        assert_eq!(aptitude_slice["recorded"], 1);
        assert_eq!(aptitude_slice["current"], 1, "completed and unexpired");
        assert_eq!(
            aptitude_slice["scales"].as_array().expect("scales").len(),
            2
        );
        let not_assessed = aptitude_slice["scales_not_assessed"]
            .as_array()
            .expect("gaps");
        assert!(
            not_assessed.iter().any(|s| s == "verbal_reasoning"),
            "the unmeasured scales are named"
        );
        assert!(
            profile["selection_suitability"].is_null(),
            "no selection sitting ⇒ no suitability figure, not zero"
        );
        // Every category appears, even with nothing recorded
        // (five since the cognitive category landed, WPM-T35).
        assert_eq!(
            profile["categories"].as_array().expect("categories").len(),
            5
        );

        // ── Aggregate analytics carry no individual score.
        let analytics: Value = request.get("/api/assessments/analytics").await.json();
        let bands = &analytics["band_distribution"]["numerical_reasoning"];
        assert_eq!(bands["high"], 1);
        assert_eq!(
            analytics["categories"][0]["completed"], 1,
            "one completed aptitude sitting"
        );

        // ── Withdrawal removes the sitting from the profile.
        request
            .delete(&format!("/api/assessments/{sitting_pid}"))
            .await
            .assert_status_ok();
        let after: Value = request
            .get(&format!("/api/workers/{worker}/assessment-profile"))
            .await
            .json();
        let aptitude_after = after["categories"]
            .as_array()
            .expect("categories")
            .iter()
            .find(|c| c["category"] == "aptitude")
            .expect("aptitude slice")
            .clone();
        assert_eq!(aptitude_after["recorded"], 0, "withdrawn sittings are gone");
    })
    .await;
}

/// A candidate's sitting tied to an application: the subject must be that application's candidate; the application's sittings
/// and the candidate's profile read; and the catalogue's and the results' remaining refusals.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one candidate, read top to bottom
async fn a_candidate_sitting_belongs_to_its_own_application() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("AC-{tag}"), None).await;
        activate!(&request, &worker).await;

        // The catalogue: a duration or validity that is not positive, and an unknown category to filter by.
        for (key, why) in [("duration_minutes", "duration"), ("validity_months", "validity")] {
            let r = request
                .post("/api/assessment-instruments")
                .json(&json!({ "name": format!("Bad {key}"), "category": "aptitude", "scales": ["numerical_reasoning"], key: 0 }))
                .await;
            assert_eq!(r.status_code(), 422, "{why}");
        }
        assert_eq!(request.get("/api/assessment-instruments?category=astrology").await.status_code(), 422);
        let instrument: Value = request
            .post("/api/assessment-instruments")
            .json(&json!({ "name": format!("Numbers {tag}"), "category": "aptitude", "scales": ["numerical_reasoning"] }))
            .await
            .json();
        let instrument = instrument["pid"].as_str().unwrap().to_string();

        // A requisition, two candidates, and an application for the first.
        let requisition: Value = request
            .post("/api/requisitions")
            .json(&json!({ "organization_ref": org, "department": "engineering", "job_title": format!("Role {tag}"), "headcount": 1,
                            "salary_min_minor": 3_000_000, "salary_max_minor": 4_000_000, "salary_currency": "GBP" }))
            .await
            .json();
        let requisition = requisition["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/requisitions/{requisition}/status")).json(&json!({ "to": "open" })).await.assert_status_ok();
        let candidate = |n: u8| {
            let (request, tag) = (&request, tag.clone());
            async move {
                let c: Value = request
                    .post("/api/candidates")
                    .json(&json!({ "display_name": format!("Candidate {n} {tag}"), "email": format!("{n}{tag}@example.com"),
                                    "source": "web", "person_ref": crate::requests::a_person() }))
                    .await
                    .json();
                c["pid"].as_str().unwrap().to_string()
            }
        };
        let (first, second) = (candidate(1).await, candidate(2).await);
        let application: Value = request
            .post(&format!("/api/requisitions/{requisition}/applications"))
            .json(&json!({ "candidate_pid": first }))
            .await
            .json();
        let application = application["pid"].as_str().unwrap().to_string();

        // The subject must be a candidate, and this application's candidate.
        let sit = |kind: &str, subject: &str| {
            json!({ "instrument_pid": instrument, "subject_kind": kind, "subject_pid": subject, "application_pid": application })
        };
        assert_eq!(request.post("/api/assessments").json(&sit("worker", &worker)).await.status_code(), 422, "a worker's sitting");
        assert_eq!(request.post("/api/assessments").json(&sit("candidate", &second)).await.status_code(), 422, "another candidate");
        let made = request.post("/api/assessments").json(&sit("candidate", &first)).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let sitting = made.json::<Value>()["pid"].as_str().unwrap().to_string();

        // A raw score needs its maximum; a result is recorded; a cancelled sitting takes none.
        let results = format!("/api/assessments/{sitting}/results");
        assert_eq!(request.post(&results).json(&json!({ "scale": "numerical_reasoning", "raw_score": 12 })).await.status_code(), 422);
        request.post(&results).json(&json!({ "scale": "numerical_reasoning", "percentile": 62 })).await.assert_status_ok();
        request.post(&format!("/api/assessments/{sitting}/status")).json(&json!({ "to": "completed" })).await.assert_status_ok();
        let other: Value = request.post("/api/assessments").json(&sit("candidate", &first)).await.json();
        let other = other["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/assessments/{other}/status")).json(&json!({ "to": "cancelled" })).await.assert_status_ok();
        let late = request
            .post(&format!("/api/assessments/{other}/results"))
            .json(&json!({ "scale": "numerical_reasoning", "percentile": 70 }))
            .await;
        assert_eq!(late.status_code(), 422, "a cancelled sitting takes no results: {}", late.text());

        // The application's sittings and the candidate's profile.
        let listed: Value = request.get(&format!("/api/applications/{application}/assessments")).await.json();
        let text = listed.to_string();
        assert!(text.contains(&sitting) && text.contains(&format!("Numbers {tag}")), "{listed}");
        let profile = request.get(&format!("/api/candidates/{first}/assessment-profile")).await;
        assert_eq!(profile.status_code(), 200, "{}", profile.text());
    })
    .await;
}
