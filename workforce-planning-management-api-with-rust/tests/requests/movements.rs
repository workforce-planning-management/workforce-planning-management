//! Joiners and leavers: dated checklists, and a leaver's last-day handover —
//! list what they hold, reassign it, revoke access, with an audit trail.

use chrono::{Duration, Utc};
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_leaver_is_handed_over_item_by_item_and_completed_only_when_nothing_is_left() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut people = Vec::new();
        for n in ["M", "L", "R1", "R2", "S", "X"] {
            let pid = seed_worker!(&request, &org, format!("MV-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            people.push(pid);
        }
        let (m, l, r1, r2, s, x) = (&people[0], &people[1], &people[2], &people[3], &people[4], &people[5]);
        let today = Utc::now().date_naive();
        let last_day = today + Duration::days(14);
        for (who, boss) in [(l, m), (r1, l), (r2, l)] {
            request
                .put(&format!("/api/workers/{who}"))
                .json(&json!({ "manager_pid": boss }))
                .await
                .assert_status_ok();
        }

        // A joiner record: a dated checklist; the people items go to the manager (L).
        let joiner: Value = request
            .post(&format!("/api/workers/{r1}/movements"))
            .json(&json!({ "kind": "joiner" }))
            .await
            .json();
        let joiner_view: Value = request
            .get(&format!("/api/movements/{}", joiner["pid"].as_str().unwrap()))
            .await
            .json();
        assert_eq!(joiner_view["kind"], "joiner");
        assert_eq!(joiner_view["items"].as_array().unwrap().len(), 7);
        let to_manager = joiner_view["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["assignee_pid"] == l.as_str())
            .count();
        assert_eq!(to_manager, 4, "welcome, buddy, first week, 30-day — the people items");

        // A leaver needs a last day and a reason; one open record each.
        let leaver_url = format!("/api/workers/{l}/movements");
        for bad in [
            json!({ "kind": "leaver", "reason": "resignation" }),
            json!({ "kind": "leaver", "effective_on": last_day.to_string() }),
            json!({ "kind": "leaver", "effective_on": last_day.to_string(), "reason": "bored" }),
        ] {
            assert_eq!(request.post(&leaver_url).json(&bad).await.status_code(), 422, "{bad}");
        }
        let leaver: Value = request
            .post(&leaver_url)
            .json(&json!({ "kind": "leaver", "effective_on": last_day.to_string(), "reason": "resignation" }))
            .await
            .json();
        let mv = leaver["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post(&leaver_url)
                .json(&json!({ "kind": "leaver", "effective_on": last_day.to_string(), "reason": "resignation" }))
                .await
                .status_code(),
            422,
            "already open"
        );
        let view: Value = request.get(&format!("/api/movements/{mv}")).await.json();
        let items = view["items"].as_array().unwrap();
        assert_eq!(items.len(), 8);
        assert_eq!(items[0]["due_on"], (last_day - Duration::days(28)).to_string());
        // "Notice acknowledged" was due 28 days before the last day = 14 days ago: overdue.
        assert_eq!(items[0]["state"], "overdue");
        // The last day is 14 days away: only "−28" is past; "−14" is due today; the rest are ahead.
        assert_eq!(view["progress"]["overdue"].as_u64(), Some(1));
        assert_eq!(items[1]["state"], "due_today");
        assert_eq!(items[2]["state"], "upcoming");

        // What they hold: people, a rota seat, a backup role, a dotted line, a shift, a mentorship, tasks, access.
        request
            .post("/api/rotas")
            .json(&json!({ "organization_ref": org, "name": format!("MV rota {tag}"), "period_days": 7,
                           "starts_on": today.to_string(), "members": [l, x] }))
            .await
            .assert_status_ok();
        request.post(&format!("/api/workers/{x}/backups")).json(&json!({ "backup_pid": l })).await.assert_status_ok();
        request
            .post(&format!("/api/workers/{x}/dotted-line-managers"))
            .json(&json!({ "manager_pid": l }))
            .await
            .assert_status_ok();
        let starts = (last_day + Duration::days(2)).and_hms_opt(9, 0, 0).unwrap().and_utc().to_rfc3339();
        let ends = (last_day + Duration::days(2)).and_hms_opt(17, 0, 0).unwrap().and_utc().to_rfc3339();
        let shift: Value = request
            .post("/api/shifts")
            .json(&json!({ "department": "engineering", "starts_at": starts, "ends_at": ends, "required_headcount": 1 }))
            .await
            .json();
        request
            .post(&format!("/api/shifts/{}/assignments", shift["pid"].as_str().unwrap()))
            .json(&json!({ "worker_pid": l }))
            .await
            .assert_status_ok();
        request
            .post("/api/mentorships")
            .json(&json!({ "mentor_pid": l, "mentee_pid": x, "focus": "growth" }))
            .await
            .assert_status_ok();
        let person: Value = request.get(&format!("/api/workers/{l}")).await.json();
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
                 VALUES (gen_random_uuid(), '{}', '{org}', 'hr_admin', CURRENT_DATE - 30)",
                person["person_ref"].as_str().unwrap()
            ))
            .await
            .unwrap();

        let held: Value = request.get(&format!("/api/movements/{mv}/handover")).await.json();
        assert_eq!(
            held["counts"],
            json!({ "direct_report": 2, "dotted_line": 1, "rota_membership": 1, "backup": 1,
                    "mentorship": 1, "shift": 1, "task": 4, "access": 1 })
        );
        assert_eq!(held["remaining"], 12);

        // Not complete while anything is open or held.
        let early = request.post(&format!("/api/movements/{mv}/complete")).await;
        assert_eq!(early.status_code(), 422);
        assert!(early.text().contains("checklist") && early.text().contains("not yet reassigned"));

        // Refusals.
        let handover = format!("/api/movements/{mv}/handover");
        let report_subject = held["items"].as_array().unwrap().iter()
            .find(|i| i["kind"] == "direct_report").unwrap()["subject_pid"].as_str().unwrap().to_string();
        let access_subject = held["items"].as_array().unwrap().iter()
            .find(|i| i["kind"] == "access").unwrap()["subject_pid"].as_str().unwrap().to_string();
        for (bad, why) in [
            (json!({ "kind": "direct_report", "subject_pid": report_subject }), "a report needs a manager"),
            (json!({ "kind": "direct_report", "subject_pid": report_subject, "to_worker_pid": l }), "not themself"),
            (json!({ "kind": "access", "subject_pid": access_subject, "to_worker_pid": s }), "access is revoked"),
            (json!({ "kind": "backup", "subject_pid": uuid::Uuid::new_v4(), "to_worker_pid": s }), "not held"),
            (json!({ "kind": "bogus", "subject_pid": report_subject }), "unknown kind"),
        ] {
            assert_eq!(request.post(&handover).json(&bad).await.status_code(), 422, "{why}");
        }
        // A successor from another organization is refused.
        let outsider = seed_worker!(&request, &an_org(), format!("MV-{tag}-O"), None).await;
        activate!(&request, &outsider).await;
        assert_eq!(
            request
                .post(&handover)
                .json(&json!({ "kind": "direct_report", "subject_pid": report_subject, "to_worker_pid": outsider }))
                .await
                .status_code(),
            422
        );

        // One item by hand: R1 (or R2) to S; the new holder is told.
        let one: Value = request
            .post(&handover)
            .json(&json!({ "kind": "direct_report", "subject_pid": report_subject, "to_worker_pid": s, "note": "covering" }))
            .await
            .json();
        assert_eq!(one["action"], "reassigned");
        assert_eq!(one["remaining"], 11);
        let told: Value = request.get(&format!("/api/workers/{s}/notifications")).await.json();
        assert!(told.as_array().unwrap().iter().any(|n| n["kind"] == "handover_received"));

        // Everything else to S in one go; access is revoked, not handed over.
        let all: Value = request
            .post(&format!("{handover}/all"))
            .json(&json!({ "to_worker_pid": s }))
            .await
            .json();
        assert_eq!(all["handed_over"], 10);
        assert_eq!(all["access_revoked"], 1);
        assert_eq!(all["failed"], json!([]));
        assert_eq!(all["remaining"], 0);

        // The things really moved.
        for report in [r1, r2] {
            let w: Value = request.get(&format!("/api/workers/{report}")).await.json();
            assert_eq!(w["manager_pid"], s.as_str(), "both reports now report to S");
        }
        let backups: Vec<Value> = request.get(&format!("/api/workers/{x}/backups")).await.json();
        assert_eq!(backups[0]["backup_pid"], s.as_str());

        // The audit trail: one row per item, in order, with who it went to.
        let trail: Vec<Value> = request.get(&format!("{handover}/actions")).await.json();
        assert_eq!(trail.len(), 12);
        assert_eq!(trail[0]["note"], "covering");
        let verbs: Vec<&str> = trail.iter().map(|a| a["action"].as_str().unwrap()).collect();
        assert_eq!(verbs.iter().filter(|v| **v == "reassigned").count(), 11);
        assert_eq!(verbs.iter().filter(|v| **v == "revoked").count(), 1);
        assert!(trail.iter().filter(|a| a["action"] == "reassigned").all(|a| a["to_worker_pid"] == s.as_str()));

        // The checklist still has to be closed before the record can be.
        assert_eq!(request.post(&format!("/api/movements/{mv}/complete")).await.status_code(), 422);
        for item in items {
            let pid = item["pid"].as_str().unwrap();
            if item["title"].as_str().unwrap().contains("Exit interview") {
                assert_eq!(
                    request.post(&format!("/api/movement-items/{pid}/skip")).json(&json!({ "reason": " " })).await.status_code(),
                    422,
                    "a skip needs a reason"
                );
                request
                    .post(&format!("/api/movement-items/{pid}/skip"))
                    .json(&json!({ "reason": "declined" }))
                    .await
                    .assert_status_ok();
            } else {
                request.post(&format!("/api/movement-items/{pid}/done")).await.assert_status_ok();
            }
        }
        request.post(&format!("/api/movements/{mv}/complete")).await.assert_status_ok();
        let done: Value = request.get(&format!("/api/movements/{mv}")).await.json();
        assert_eq!(done["status"], "completed");
        // A completed record takes no more changes.
        assert_eq!(
            request.post(&handover).json(&json!({ "kind": "backup", "subject_pid": uuid::Uuid::new_v4() })).await.status_code(),
            422
        );

        // Only a leaver has a handover.
        assert_eq!(
            request.get(&format!("/api/movements/{}/handover", joiner["pid"].as_str().unwrap())).await.status_code(),
            422
        );
        // The list shows open movements only by default.
        let open: Vec<Value> = request.get("/api/movements?kind=leaver").await.json();
        assert!(open.iter().all(|m| m["pid"] != mv.as_str()));
        let completed: Vec<Value> = request.get("/api/movements?kind=leaver&status=completed").await.json();
        assert!(completed.iter().any(|m| m["pid"] == mv.as_str()));
    })
    .await;
}
