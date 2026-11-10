//! A request the service cannot read is refused as a problem that says what kind of thing was wrong and quotes nothing the
//! caller sent (WPM-R238, WPM-T475): the handlers use `axa::extract`'s `Json`, `Path` and `Query`, whose rejections say only
//! which part of the request was at fault. Axum's own quote the value, so a token or a name pasted by mistake would come back.

use serde_json::Value;
use serial_test::serial;

const SECRET: &str = "s3cret-value";

fn assert_problem(status: u16, body: &str, expected: u16, code: &str, detail: &str) {
    assert_eq!(status, expected, "{body}");
    assert!(
        !body.contains(SECRET),
        "the refusal quotes the request: {body}"
    );
    let problem: Value = serde_json::from_str(body).unwrap();
    assert_eq!(problem["status"], expected, "{body}");
    assert_eq!(problem["code"], code, "{body}");
    assert_eq!(problem["detail"], detail, "{body}");
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_request_that_cannot_be_read_is_a_problem_that_quotes_nothing() {
    crate::requests::request_open(|request, _ctx| async move {
        // A body that is not JSON at all.
        let response = request
            .post("/api/workers")
            .content_type("application/json")
            .bytes(format!("{{\"{SECRET}\"").into())
            .await;
        assert_eq!(response.header("content-type"), "application/problem+json");
        assert_problem(
            response.status_code().as_u16(),
            &response.text(),
            400,
            "bad_request",
            "The request body is not valid JSON",
        );

        // JSON of the wrong shape.
        let response = request
            .post("/api/workers")
            .json(&serde_json::json!({ "person_ref": [SECRET] }))
            .await;
        assert_problem(
            response.status_code().as_u16(),
            &response.text(),
            422,
            "unprocessable_entity",
            "The request body does not fit this route",
        );

        // A path segment that is not an identifier.
        let response = request
            .get(&format!("/api/consultations/{SECRET}/timeline"))
            .await;
        assert_problem(
            response.status_code().as_u16(),
            &response.text(),
            400,
            "bad_request",
            "A value in the URL is not valid",
        );

        // A query value of the wrong type.
        let response = request
            .get(&format!(
                "/api/org-chart?organization=organization:x&limit={SECRET}"
            ))
            .await;
        assert_problem(
            response.status_code().as_u16(),
            &response.text(),
            400,
            "bad_request",
            "A query parameter is not valid",
        );
    })
    .await;
}
