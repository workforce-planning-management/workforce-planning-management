//! The AI and scoring deployer register (WPM-R148, WPM-D103): it records what a deployer says, shows `unknown` where nothing
//! was said, and refuses a record whose audit cannot be written (WPM-D107).

use serde_json::{Value, json};
use serial_test::serial;

fn feature<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["feature"]["id"] == id)
        .unwrap_or_else(|| panic!("{id}"))
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn the_register_records_what_is_said_and_shows_unknown_for_the_rest() {
    crate::requests::request_open(|request, ctx| async move {
        let first: Value = request.get("/api/ai-register").await.json();
        assert_eq!(first["features"].as_array().unwrap().len(), 8);
        assert_eq!(first["applies_from"], "2027-12-02");
        for f in first["features"].as_array().unwrap() {
            assert_eq!(f["overseer_role"], json!({ "status": "unknown" }), "nothing is overseen until said");
            assert_eq!(f["log_retention_months"], json!({ "status": "unknown" }));
            assert_eq!(f["retention_below_minimum"], false);
        }
        assert!(first["note"].as_str().unwrap().contains("not evidence of compliance"));

        // Record S3.
        let ok = request
            .put("/api/ai-register/S3")
            .json(&json!({
                "annex_category": "recruitment_selection", "overseer_role": "Head of Resourcing",
                "workers_informed_on": "2026-06-01", "log_retention_months": 12,
            }))
            .await;
        assert_eq!(ok.status_code(), 200, "{}", ok.text());
        let after: Value = request.get("/api/ai-register").await.json();
        let s3 = feature(&after, "S3");
        assert_eq!(s3["overseer_role"], json!({ "status": "recorded", "value": "Head of Resourcing" }));
        assert_eq!(s3["representatives_informed_on"], json!({ "status": "unknown" }), "only what was said");
        assert_eq!(feature(&after, "S4")["overseer_role"]["status"], "unknown", "another feature is untouched");

        // A second put replaces, and a field left out is cleared.
        request.put("/api/ai-register/S3").json(&json!({ "overseer_role": "HR Director" })).await.assert_status_ok();
        let replaced: Value = request.get("/api/ai-register").await.json();
        assert_eq!(feature(&replaced, "S3")["log_retention_months"]["status"], "unknown");
        assert_eq!(feature(&replaced, "S3")["overseer_role"]["value"], "HR Director");

        // Faults are refused with the reason; an unknown feature is not found; S9 does not exist.
        let bad = |body: Value| {
            let request = &request;
            async move { request.put("/api/ai-register/S1").json(&body).await }
        };
        assert_eq!(bad(json!({ "annex_category": "high_risk" })).await.status_code(), 422);
        assert_eq!(bad(json!({ "overseer_role": "a.person@example.org" })).await.status_code(), 422);
        assert_eq!(bad(json!({ "workers_informed_on": "2999-01-01" })).await.status_code(), 422);
        assert_eq!(bad(json!({ "log_retention_months": 3 })).await.status_code(), 422, "needs the other law named");
        assert_eq!(
            bad(json!({ "log_retention_months": 3, "retention_note": "Data protection law requires deletion at three months." })).await.status_code(),
            200
        );
        let short: Value = request.get("/api/ai-register").await.json();
        assert_eq!(feature(&short, "S1")["retention_below_minimum"], true);
        assert_eq!(request.put("/api/ai-register/S9").json(&json!({})).await.status_code(), 404);

        // No audit, no write (WPM-D107).
        crate::requests::ddl(&ctx, "CREATE FUNCTION refuse_audit() RETURNS trigger AS $$ BEGIN RAISE EXCEPTION 'audit refused'; END $$ LANGUAGE plpgsql;
                 CREATE TRIGGER refuse_audit BEFORE INSERT ON audit_logs FOR EACH ROW EXECUTE FUNCTION refuse_audit()",).await;
        let refused = request.put("/api/ai-register/S5").json(&json!({ "overseer_role": "Head of Mobility" })).await;
        assert!(refused.status_code().is_server_error(), "refused: {}", refused.status_code());
        crate::requests::ddl(&ctx, "DROP TRIGGER refuse_audit ON audit_logs; DROP FUNCTION refuse_audit()").await;
        let kept: Value = request.get("/api/ai-register").await.json();
        assert_eq!(feature(&kept, "S5")["overseer_role"]["status"], "unknown", "nothing was kept");
        request.put("/api/ai-register/S5").json(&json!({ "overseer_role": "Head of Mobility" })).await.assert_status_ok();
    })
    .await;
}
