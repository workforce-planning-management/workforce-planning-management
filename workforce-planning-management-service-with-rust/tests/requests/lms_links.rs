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
        unsafe { std::env::set_var("WPM_LMS_COURSE_URL", value) };
        Self
    }
}

impl Drop for Template {
    fn drop(&mut self) {
        // SAFETY: as above.
        unsafe { std::env::remove_var("WPM_LMS_COURSE_URL") };
    }
}

/// The link on the worker's first CPD entry and on their first training enrolment.
async fn links(request: &loco_rs::TestServer, worker: &str) -> (Value, Value) {
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
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
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
