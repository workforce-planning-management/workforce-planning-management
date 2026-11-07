//! Every error is a problem document, whatever refused the request (WPM-R238). The layer's rules are in `src/problem.rs`;
//! this checks that it stands where it must in the real stack.

use serde_json::{Value, json};
use serial_test::serial;
use uuid::Uuid;

const PROBLEM_JSON: &str = "application/problem+json";

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn an_error_from_a_handler_a_layer_or_no_route_is_a_problem() {
    crate::requests::request_open(|request, _ctx| async move {
        // `axa::Error::NotFound`, from a handler.
        let missing = request
            .get(&format!("/api/workers/{}", Uuid::new_v4()))
            .await;
        assert_eq!(missing.status_code(), 404);
        assert_eq!(missing.header("content-type"), PROBLEM_JSON);
        let body: Value = missing.json();
        assert_eq!(body["type"], "about:blank");
        assert_eq!(body["title"], "Not Found");
        assert_eq!(body["status"], 404);
        assert!(body.get("error").is_none() && body.get("description").is_none());

        // A refusal from a handler: the reason is `code` and the words are `detail`.
        let offset = request.get("/api/workers?offset=999999999").await;
        assert_eq!(offset.status_code(), 422);
        assert_eq!(offset.header("content-type"), PROBLEM_JSON);
        let body: Value = offset.json();
        assert_eq!(body["code"], "unprocessable_entity");
        assert!(
            body["detail"]
                .as_str()
                .unwrap()
                .contains("offset is not taken"),
            "{body}"
        );

        // A body the handler cannot read: `axa::extract` refuses it with a reason and fixed words (WPM-T475).
        let invalid = request
            .post("/api/workers")
            .json(&json!({ "person_ref": "" }))
            .await;
        assert_eq!(invalid.status_code(), 422);
        assert_eq!(invalid.header("content-type"), PROBLEM_JSON);
        let body: Value = invalid.json();
        assert_eq!(body["code"], "unprocessable_entity", "{body}");
        assert_eq!(
            body["detail"], "The request body does not fit this route",
            "{body}"
        );

        // A layer's refusal, with its own members kept.
        let version = request
            .get("/api/workers")
            .add_header("accepts-version", "9.0")
            .await;
        assert_eq!(version.status_code(), 406);
        assert_eq!(version.header("content-type"), PROBLEM_JSON);
        let body: Value = version.json();
        assert_eq!(body["code"], "unsupported_api_version");
        assert_eq!(body["requested"], "9.0");

        // No such route.
        let nowhere = request.get("/api/no-such-thing").await;
        assert_eq!(nowhere.status_code(), 404);
        assert_eq!(nowhere.header("content-type"), PROBLEM_JSON);
        assert_eq!(nowhere.json::<Value>()["status"], 404);

        // A success is not touched.
        let fine = request.get("/_health").await;
        assert_eq!(fine.status_code(), 200);
        assert_ne!(fine.header("content-type"), PROBLEM_JSON);
    })
    .await;
}
