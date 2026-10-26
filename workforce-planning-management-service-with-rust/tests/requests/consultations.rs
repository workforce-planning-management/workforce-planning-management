//! The consultation register (WPM-R105, WPM-D68): a record per proposal, the rules read over it, and nothing about who attends or what was said.

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use serial_test::serial;

use super::an_org;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_consultation_is_registered_read_against_the_rules_begun_and_closed() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = Utc::now().date_naive();
        let termination = today + Duration::days(100);

        let body: Value = request
            .post("/api/recognised-bodies")
            .json(&json!({ "organization_ref": org, "name": format!("Register Forum {tag}"), "kind": "works_council", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": "Head of Employee Relations" }))
            .await
            .json();
        let body_pid = body["pid"].as_str().unwrap().to_string();
        let unit: Value = request
            .post(&format!("/api/recognised-bodies/{body_pid}/units"))
            .json(&json!({ "name": "Everyone", "department": format!("reg-{tag}") }))
            .await
            .json();
        let agreement: Value = request
            .post(&format!("/api/bargaining-units/{}/agreements", unit["pid"].as_str().unwrap()))
            .json(&json!({ "title": "Redundancy procedure", "source": "Agreement of 2020, clause 9", "effective_from": "2020-01-01", "consultation_days": 60 }))
            .await
            .json();
        let agreement_pid = agreement["pid"].as_str().unwrap().to_string();

        let open = json!({
            "body_pid": body_pid, "establishment": format!("Leeds {tag}"), "profile": "great_britain", "establishment_size": 500,
            "dismissals": 25, "earliest_termination_on": termination, "topics": ["pay", "redundancy", "pay"], "document_ref": "DOC-123",
        });
        // Bad: an unknown profile or topic, a reference with spaces, a body that does not exist, a plan that is not the organization's.
        for (field, value) in [
            ("profile", json!("mars")),
            ("topics", json!(["gossip"])),
            ("document_ref", json!("two words")),
            ("dismissals", json!(0)),
            ("plan_pid", json!("00000000-0000-4000-8000-000000000000")),
        ] {
            let mut bad = open.clone();
            bad[field] = value;
            assert_eq!(request.post("/api/consultations").json(&bad).await.status_code(), 422, "{field}");
        }
        let mut stray = open.clone();
        stray["body_pid"] = json!("00000000-0000-4000-8000-000000000000");
        assert_eq!(request.post("/api/consultations").json(&stray).await.status_code(), 404);

        let made = request.post("/api/consultations").json(&open).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();

        // Before it begins: a duty may apply, and an agreement's longer period (60 days, over the law's 30) sets the latest start.
        let line: Value = request.get(&format!("/api/consultations/{pid}/timeline")).await.json();
        assert_eq!(line["consultation"]["topics"], json!(["redundancy", "pay"]), "known, unique, in a fixed order");
        assert_eq!(line["assessment"]["may_apply"], true);
        assert_eq!(line["assessment"]["agreed_period_days"], 60);
        assert_eq!(line["assessment"]["consultation_days"], 60);
        assert_eq!(line["assessment"]["latest_consultation_start"], json!(termination - Duration::days(60)));
        assert_eq!(line["assessment"]["deployer_to_confirm"], true);
        assert_eq!(line["timeliness"], json!({ "kind": "not_begun", "days_left": 40 }));

        // Another proposal at the same establishment inside the 90-day window counts with this one; one at another site does not.
        let mut same_site = open.clone();
        same_site["dismissals"] = json!(5);
        same_site["earliest_termination_on"] = json!(termination + Duration::days(10));
        assert_eq!(request.post("/api/consultations").json(&same_site).await.status_code(), 200);
        let mut other_site = open.clone();
        other_site["establishment"] = json!(format!("York {tag}"));
        other_site["dismissals"] = json!(3);
        let york: Value = request.post("/api/consultations").json(&other_site).await.json();
        let line: Value = request.get(&format!("/api/consultations/{pid}/timeline")).await.json();
        assert_eq!(line["assessment"]["evidence"]["counted"], 30, "25 and 5 at the same establishment");
        let under: Value = request.get(&format!("/api/consultations/{}/timeline", york["pid"].as_str().unwrap())).await.json();
        assert_eq!(under["assessment"]["may_apply"], false);
        assert_eq!(under["timeliness"], json!({ "kind": "nothing_due" }));
        assert!(under["assessment"]["latest_consultation_start"].is_null());

        // Beginning: not in the future, once; then it is in time.
        let begin = format!("/api/consultations/{pid}/begin");
        assert_eq!(request.post(&begin).json(&json!({ "started_on": today + Duration::days(1) })).await.status_code(), 422);
        assert_eq!(request.post(&begin).json(&json!({ "started_on": today, "document_ref": "DOC-124" })).await.status_code(), 200);
        assert_eq!(request.post(&begin).json(&json!({ "started_on": today })).await.status_code(), 422, "only once");
        let line: Value = request.get(&format!("/api/consultations/{pid}/timeline")).await.json();
        assert_eq!(line["timeliness"], json!({ "kind": "begun_in_time" }));
        assert_eq!(line["consultation"]["document_ref"], "DOC-124");

        // Closing: an agreement must be one of this body's; the outcome and days are checked; closed means closed.
        let close = format!("/api/consultations/{pid}/close");
        let foreign = "00000000-0000-4000-8000-000000000000";
        assert_eq!(request.post(&close).json(&json!({ "outcome": "agreed", "ended_on": today, "agreement_pid": foreign })).await.status_code(), 422);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "no_agreement", "ended_on": today, "agreement_pid": agreement_pid })).await.status_code(), 422);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "open", "ended_on": today })).await.status_code(), 422);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "agreed", "ended_on": today + Duration::days(1) })).await.status_code(), 422);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "agreed", "ended_on": today, "agreement_pid": agreement_pid })).await.status_code(), 200);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "withdrawn", "ended_on": today })).await.status_code(), 422, "already closed");
        // One that never began can only be withdrawn.
        let york_close = format!("/api/consultations/{}/close", york["pid"].as_str().unwrap());
        assert_eq!(request.post(&york_close).json(&json!({ "outcome": "agreed", "ended_on": today })).await.status_code(), 422);
        assert_eq!(request.post(&york_close).json(&json!({ "outcome": "withdrawn", "ended_on": today })).await.status_code(), 200);

        // The register lists them, and holds no attendee, name or minute.
        let list: Value = request.get(&format!("/api/consultations?organization={org}")).await.json();
        let ours: Vec<&Value> = list["consultations"].as_array().unwrap().iter().filter(|c| c["body_pid"] == body_pid.as_str()).collect();
        assert_eq!(ours.len(), 3);
        let closed = ours.iter().find(|c| c["pid"] == pid.as_str()).unwrap();
        assert_eq!((closed["outcome"].as_str(), closed["agreement_pid"].as_str()), (Some("agreed"), Some(agreement_pid.as_str())));
        let text = list.to_string().to_lowercase();
        for banned in ["attendee", "representative", "minute", "member"] {
            assert!(!text.contains(banned), "the register holds no {banned}: {text}");
        }
    })
    .await;
}
