//! Links to the learning management system (WPM-R138, WPM-D91): built from the deployer's template
//! and the course's URN, on a CPD entry and a training enrolment; none without a valid template.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

const COURSE: &str = "course:33333333-3333-4333-8333-333333333333";

/// Sets the template for one test and clears it after, even on a failed assertion.
struct Template;

impl Template {
    fn set(value: &str) -> Self {
        // SAFETY: the tests that read this setting are `#[serial]`.
        unsafe { std::env::set_var("WORKFORCE_PLANNING_MANAGEMENT_LMS_COURSE_URL", value) };
        Self
    }
}

impl Drop for Template {
    fn drop(&mut self) {
        // SAFETY: as above.
        unsafe { std::env::remove_var("WORKFORCE_PLANNING_MANAGEMENT_LMS_COURSE_URL") };
    }
}

/// The link on the worker's first CPD entry and on their first training enrolment.
async fn links(request: &axa_test::TestServer, worker: &str) -> (Value, Value) {
    let entries: Value = request
        .get(&format!("/api/workers/{worker}/cpd-entries"))
        .await
        .json();
    let enrolments: Value = request
        .get(&format!("/api/workers/{worker}/training-enrollments"))
        .await
        .json();
    (
        entries[0]["lms_url"].clone(),
        enrolments[0]["lms_url"].clone(),
    )
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_completion_links_to_its_course_only_through_a_valid_template() {
    crate::requests::request_open(|request, _ctx| async move {
        let person = a_person();
        let created: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": person, "organization_ref": an_org(), "worker_number": "1",
                "display_name": "Linked Learner", "employment_type": "permanent",
                "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
            }))
            .await
            .json();
        let worker = created["pid"].as_str().expect("worker pid").to_string();
        request
            .post("/api/lms/completions")
            .json(&json!({ "completions": [{
                "person_ref": person, "course_ref": COURSE, "completed_on": "2026-09-01",
                "hours": 2.0, "external_ref": "evt-link-1",
            }] }))
            .await
            .assert_status_ok();
        // no template: no link, which is the default
        assert_eq!(links(&request, &worker).await, (Value::Null, Value::Null));

        // a valid template: the course id in place of the placeholder, on both
        let expected = json!("https://lms.example.org/course/33333333-3333-4333-8333-333333333333");
        {
            let _template = Template::set("https://lms.example.org/course/{course}");
            assert_eq!(links(&request, &worker).await, (expected.clone(), expected));
        }

        // templates that are not safe are ignored, not half-used
        for bad in [
            "http://lms.example.org/{course}",
            "https://u@lms.example.org/{course}",
            "https://lms.example.org/",
        ] {
            let _template = Template::set(bad);
            assert_eq!(
                links(&request, &worker).await,
                (Value::Null, Value::Null),
                "{bad}"
            );
        }
    })
    .await;
}

/// The batch limit, an invalid event, a learner with workers in two organizations, and a completion that finishes an
/// enrolment that already exists (updated, not doubled).
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_batch_is_bounded_and_each_event_says_what_happened() {
    crate::requests::request_open(|request, _ctx| async move {
        let event = |person: &str, external: &str| {
            json!({ "person_ref": person, "course_ref": COURSE, "completed_on": "2026-09-01", "hours": 1.0, "external_ref": external })
        };
        let many: Vec<Value> = (0..501).map(|i| event(&a_person(), &format!("evt-{i}"))).collect();
        assert_eq!(request.post("/api/lms/completions").json(&json!({ "completions": many })).await.status_code(), 422);

        // One person with a worker in each of two organizations, and one with an enrolment already open.
        let (shared, single) = (a_person(), a_person());
        let make = |person: String, number: String| {
            let request = &request;
            async move {
                let created: Value = request
                    .post("/api/workers")
                    .json(&json!({
                        "person_ref": person, "organization_ref": an_org(), "worker_number": number,
                        "display_name": "Learner", "employment_type": "permanent",
                        "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
                    }))
                    .await
                    .json();
                created["pid"].as_str().unwrap().to_string()
            }
        };
        make(shared.clone(), "S1".into()).await;
        make(shared.clone(), "S2".into()).await;
        let worker = make(single.clone(), "L1".into()).await;
        request
            .post(&format!("/api/workers/{worker}/training-enrollments"))
            .json(&json!({ "course_ref": COURSE }))
            .await
            .assert_status_ok();

        let mut bad_credit = event(&single, "evt-credit");
        bad_credit["hours"] = json!(-1.0);
        let mut bad_course = event(&single, "evt-course");
        bad_course["course_ref"] = json!("person:33333333-3333-4333-8333-333333333333");
        let batch = json!({ "completions": [
            event(&single, " "),
            bad_credit,
            bad_course,
            event(&shared, "evt-shared"),
            event(&single, "evt-update"),
        ] });
        let answer: Value = request.post("/api/lms/completions").json(&batch).await.json();
        let results = answer["results"].as_array().unwrap();
        let result = |i: usize| results[i]["result"].as_str().unwrap().to_string();
        assert_eq!((result(0), result(1), result(2)), ("invalid".into(), "invalid".into(), "invalid".into()), "{answer}");
        assert_eq!(result(3), "ambiguous", "{answer}");
        assert_eq!(result(4), "applied", "{answer}");
        assert_eq!(results[4]["enrollment"], "updated", "the open enrolment is completed, not a second one made: {answer}");
        let enrolments: Value = request.get(&format!("/api/workers/{worker}/training-enrollments")).await.json();
        assert_eq!(enrolments.as_array().map(Vec::len), Some(1), "{enrolments}");
    })
    .await;
}
