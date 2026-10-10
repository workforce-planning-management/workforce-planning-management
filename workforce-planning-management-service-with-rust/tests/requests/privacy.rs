//! Subject rights & retention (WPM-R30 / WPM-D22): the subject-access
//! export gathers and names its exclusions, erasure anonymises without
//! touching payroll rows and is refused on open employment, and the
//! retention report/sweep honour the floored horizon.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one seeded life, the whole rights surface
async fn subject_rights_round_trip() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "SR-1", Some(3_600_000)).await;
        activate!(&request, &worker).await;

        // Give the worker a footprint: time (with a note), leave,
        // and a wellbeing acknowledgement.
        request
            .post(&format!("/api/workers/{worker}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-20", "minutes": 480,
                           "notes": "client visit in Leeds" }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/workers/{worker}/leave-entitlements"))
            .json(&json!({ "kind": "annual", "year": 2026, "entitled_days": 25 }))
            .await
            .assert_status_ok();
        let rule: Value = request
            .post("/api/wellbeing-entitlements")
            .json(&json!({ "name": "SR flu", "description": "x" }))
            .await
            .json();
        request
            .post(&format!("/api/workers/{worker}/wellbeing-acknowledgements"))
            .json(&json!({ "entitlement_pid": rule["pid"], "response": "done" }))
            .await
            .assert_status_ok();

        // ── Subject access: the footprint is present, exclusions named,
        // and the export is audited.
        let export: Value = request
            .get(&format!("/api/workers/{worker}/subject-access"))
            .await
            .json();
        assert_eq!(export["worker"]["worker_number"], "SR-1");
        assert_eq!(export["time_entries"][0]["notes"], "client visit in Leeds");
        assert_eq!(export["leave_entitlements"][0]["entitled_days"], 25);
        assert_eq!(export["wellbeing_acknowledgements"][0]["response"], "done");
        let exclusions = serde_json::to_string(&export["exclusions"]).unwrap();
        assert!(exclusions.contains("pulse"), "structural exclusion named");
        assert!(
            exclusions.contains("identity services"),
            "upstream duty named"
        );
        let audits: Value = request.get("/api/audits/recent").await.json();
        assert!(
            serde_json::to_string(&audits)
                .unwrap()
                .contains("subject_access_exported")
        );

        // ── Erasure is refused while employment is open.
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/erase"))
                .await
                .status_code(),
            422,
            "an active employment is the lawful basis"
        );
        // Terminate (via offboarding — the lifecycle's path), then erase.
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        let erased: Value = request
            .post(&format!("/api/workers/{worker}/erase"))
            .await
            .json();
        assert_eq!(erased["erased"], worker.as_str());
        // The worker is gone from reads (soft-deleted) …
        assert_eq!(
            request
                .get(&format!("/api/workers/{worker}"))
                .await
                .status_code(),
            404
        );
        // … their acknowledgements are deleted, and the audit snapshot
        // carries the counts.
        let audits: Value = request.get("/api/audits/recent").await.json();
        let erase_row = audits
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["action"] == "erased")
            .expect("erasure audited")
            .clone();
        assert_eq!(erase_row["snapshot"]["acknowledgements_deleted"], 1);
        assert_eq!(erase_row["snapshot"]["notes_scrubbed"], 1);

        // ── Retention: the report is readable; a fresh soft-delete is
        // inside the horizon, so nothing is listed or swept.
        let report: Value = request.get("/api/retention").await.json();
        for (kind, days) in report["horizons"].as_object().unwrap() {
            assert!(days.as_i64().unwrap() >= 30, "{kind}: floored horizon");
        }
        assert_eq!(report["horizons"].as_object().unwrap().len(), 8);
        assert!(
            report["soft_deleted_past_horizon"]
                .as_object()
                .unwrap()
                .is_empty(),
            "fresh soft-deletes are inside the horizon"
        );
        let sweep: Value = request.post("/api/retention/sweep").await.json();
        assert_eq!(sweep["rows_deleted"], 0, "nothing past the horizon yet");
        assert_eq!(sweep["candidates_scrubbed"], 0);
        let audits: Value = request.get("/api/audits/recent").await.json();
        assert!(
            serde_json::to_string(&audits)
                .unwrap()
                .contains("retention_swept")
        );
    })
    .await;
}
