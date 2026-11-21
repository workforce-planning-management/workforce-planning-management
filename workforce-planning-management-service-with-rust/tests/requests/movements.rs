//! Joiners and leavers: dated checklists, and a leaver's last-day handover —
//! list what they hold, reassign it, revoke access, with an audit trail.
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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
        let today = workforce_planning_management::dates::today();
        let last_day = today + workforce_planning_management::dates::days(14);
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
        assert_eq!(items[0]["due_on"], (last_day - workforce_planning_management::dates::days(28)).to_string());
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
        let starts = (workforce_planning_management::dates::start_of_day(last_day + workforce_planning_management::dates::days(2)) + jiff::SignedDuration::from_hours(9)).to_string();
        let ends = (workforce_planning_management::dates::start_of_day(last_day + workforce_planning_management::dates::days(2)) + jiff::SignedDuration::from_hours(17)).to_string();
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
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
                 VALUES (gen_random_uuid(), '{}', '{org}', 'hr_admin', CURRENT_DATE - 30)",
                person["person_ref"].as_str().unwrap()
            ))).execute(workforce_planning_management::db::pool(&ctx))
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
        assert!(done["completed_at"].is_string(), "completing stamps the time: {done}");
        let actions: Vec<String> = sqlx::query_scalar(
            "SELECT action FROM audit_logs WHERE entity_pid = $1::uuid AND action LIKE 'leaver_%' ORDER BY id",
        )
        .bind(l)
        .fetch_all(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        assert_eq!(actions, ["leaver_opened", "leaver_completed"], "audited by kind");
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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_checklist_takes_extra_items_reassignment_and_reopening_until_it_is_cancelled() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let joiner = seed_worker!(&request, &org, format!("MC-{tag}-J"), None).await;
        let buddy = seed_worker!(&request, &org, format!("MC-{tag}-B"), None).await;
        let today = workforce_planning_management::dates::today();

        let movement: Value = request
            .post(&format!("/api/workers/{joiner}/movements"))
            .json(&json!({ "kind": "joiner" }))
            .await
            .json();
        let mv = movement["pid"].as_str().unwrap().to_string();
        let before: Value = request.get(&format!("/api/movements/{mv}")).await.json();
        let count = before["items"].as_array().unwrap().len();

        // An ad-hoc item needs a title and a date; a blank title is refused.
        let items_url = format!("/api/movements/{mv}/items");
        assert_eq!(
            request
                .post(&items_url)
                .json(&json!({ "title": "  ", "due_on": today.to_string() }))
                .await
                .status_code(),
            422
        );
        let added: Value = request
            .post(&items_url)
            .json(&json!({ "title": "Collect the laptop", "due_on": today.to_string() }))
            .await
            .json();
        let item = added["pid"].as_str().unwrap().to_string();
        let after: Value = request.get(&format!("/api/movements/{mv}")).await.json();
        let items = after["items"].as_array().unwrap();
        assert_eq!(items.len(), count + 1);
        let new = items.iter().find(|i| i["pid"] == item.as_str()).unwrap();
        assert_eq!(
            new["category"], "other",
            "a missing category falls back to other"
        );

        // Assign to a real worker, then to nobody; a stranger is refused.
        let assign = format!("/api/movement-items/{item}/assign");
        let set: Value = request
            .post(&assign)
            .json(&json!({ "assignee_pid": buddy }))
            .await
            .json();
        assert_eq!(set["assignee_pid"], buddy.as_str());
        assert_eq!(
            request
                .post(&assign)
                .json(&json!({ "assignee_pid": uuid::Uuid::new_v4() }))
                .await
                .status_code(),
            422
        );
        let cleared: Value = request
            .post(&assign)
            .json(&json!({ "assignee_pid": null }))
            .await
            .json();
        assert_eq!(cleared["assignee_pid"], Value::Null);

        // Done, then reopened: the item is open again.
        request
            .post(&format!("/api/movement-items/{item}/done"))
            .await
            .assert_status_ok();
        let reopened: Value = request
            .post(&format!("/api/movement-items/{item}/reopen"))
            .await
            .json();
        assert_eq!(reopened["state"], "reopened");
        let view: Value = request.get(&format!("/api/movements/{mv}")).await.json();
        let item_now = view["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["pid"] == item.as_str())
            .unwrap()
            .clone();
        assert_ne!(item_now["state"], "done");
        assert_eq!(
            request
                .post(&format!(
                    "/api/movement-items/{}/reopen",
                    uuid::Uuid::new_v4()
                ))
                .await
                .status_code(),
            404
        );

        // Cancelled: the record closes and takes no more items.
        let cancelled: Value = request
            .post(&format!("/api/movements/{mv}/cancel"))
            .await
            .json();
        assert_eq!(cancelled["status"], "cancelled");
        let actions: Vec<String> = sqlx::query_scalar(
            "SELECT action FROM audit_logs WHERE entity_pid = $1::uuid ORDER BY id",
        )
        .bind(&joiner)
        .fetch_all(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        let movement_actions: Vec<&str> = actions
            .iter()
            .map(String::as_str)
            .filter(|a| a.starts_with("joiner_") || a.starts_with("movement_item_"))
            .collect();
        assert_eq!(
            movement_actions,
            [
                "joiner_opened",
                "movement_item_added",
                "movement_item_assigned",
                "movement_item_assigned",
                "movement_item_done",
                "movement_item_reopened",
                "joiner_cancelled",
            ],
            "each change audited once, by kind; a refused one not at all"
        );
        assert_eq!(
            request
                .post(&items_url)
                .json(&json!({ "title": "Late", "due_on": today.to_string() }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&format!("/api/movements/{mv}/cancel"))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&assign)
                .json(&json!({ "assignee_pid": null }))
                .await
                .status_code(),
            422
        );
    })
    .await;
}

/// The kinds the first scenario does not reach: leading a group, a rota seat the successor already holds, an on-call swap
/// still ahead, and the refusals for a successor who is unknown or not employed, and for a manager below their own report.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_group_lead_a_shared_rota_seat_and_a_swap_are_handed_over_or_closed() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut people = Vec::new();
        for n in ["L", "S", "R1", "R2", "T"] {
            let pid = seed_worker!(&request, &org, format!("HO-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            people.push(pid);
        }
        let (l, s, r1, r2, gone) = (&people[0], &people[1], &people[2], &people[3], &people[4]);
        for (who, boss) in [(r1, l), (r2, r1)] {
            request.put(&format!("/api/workers/{who}")).json(&json!({ "manager_pid": boss })).await.assert_status_ok();
        }
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{gone}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        let today = workforce_planning_management::dates::today();
        let day = |n: i64| (today + workforce_planning_management::dates::days(n)).to_string();

        // L leads two groups (S is a member of one), shares a rota with S, and is on call by swap after the last day.
        let group = |name: String| {
            let (request, org) = (&request, org.clone());
            async move {
                let g: Value = request
                    .post("/api/groups")
                    .json(&json!({ "organization_ref": org, "name": name, "kind": "practice" }))
                    .await
                    .json();
                g["pid"].as_str().unwrap().to_string()
            }
        };
        let g1 = group(format!("Guild {tag}")).await;
        let g2 = group(format!("Club {tag}")).await;
        for (g, who, role) in [(&g1, l, "lead"), (&g1, s, "member"), (&g2, l, "lead")] {
            request.post(&format!("/api/groups/{g}/members")).json(&json!({ "worker_pid": who, "role": role })).await.assert_status_ok();
        }
        let rota: Value = request
            .post("/api/rotas")
            .json(&json!({ "organization_ref": org, "name": format!("HO rota {tag}"), "period_days": 7,
                           "starts_on": today.to_string(), "members": [l, s] }))
            .await
            .json();
        let rota = rota["pid"].as_str().unwrap().to_string();
        request
            .post(&format!("/api/rotas/{rota}/overrides"))
            .json(&json!({ "worker_pid": l, "starts_on": day(20), "ends_on": day(21) }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/rotas/{rota}/overrides"))
            .json(&json!({ "worker_pid": l, "starts_on": day(22), "ends_on": day(23) }))
            .await
            .assert_status_ok();

        let mv: Value = request
            .post(&format!("/api/workers/{l}/movements"))
            .json(&json!({ "kind": "leaver", "effective_on": day(14), "reason": "resignation" }))
            .await
            .json();
        let mv = mv["pid"].as_str().unwrap().to_string();
        let handover = format!("/api/movements/{mv}/handover");
        let held: Value = request.get(&handover).await.json();
        let items = held["items"].as_array().unwrap().clone();
        let subject = |kind: &str, n: usize| {
            items.iter().filter(|i| i["kind"] == kind).nth(n).unwrap()["subject_pid"].as_str().unwrap().to_string()
        };
        assert_eq!(held["counts"]["group_lead"], 2, "{held}");
        assert_eq!(held["counts"]["rota_swap"], 2, "{held}");
        assert_eq!(held["counts"]["rota_membership"], 1, "{held}");

        // Refusals: an unknown successor, one no longer employed, and a manager below their own report.
        let report = subject("direct_report", 0);
        for (to, why) in [(uuid::Uuid::new_v4().to_string(), "unknown"), (gone.to_string(), "not employed"), (r2.to_string(), "a loop")] {
            let r = request
                .post(&handover)
                .json(&json!({ "kind": "direct_report", "subject_pid": report, "to_worker_pid": to }))
                .await;
            assert_eq!(r.status_code(), 422, "{why}: {}", r.text());
        }

        // A group lead passes only to a member; a group with nobody to take it closes.
        let lead_one = |n: usize| subject("group_lead", n);
        let (first, second) = (lead_one(0), lead_one(1));
        let not_member = request.post(&handover).json(&json!({ "kind": "group_lead", "subject_pid": second, "to_worker_pid": s })).await;
        assert_eq!(not_member.status_code(), 422, "{}", not_member.text());
        let moved: Value = request.post(&handover).json(&json!({ "kind": "group_lead", "subject_pid": first, "to_worker_pid": s })).await.json();
        assert_eq!(moved["action"], "reassigned", "{moved}");
        let closed: Value = request.post(&handover).json(&json!({ "kind": "group_lead", "subject_pid": second })).await.json();
        assert_eq!(closed["action"], "closed", "{closed}");

        // The rota seat: S already has one, so L's seat is closed rather than doubled.
        let seat: Value = request
            .post(&handover)
            .json(&json!({ "kind": "rota_membership", "subject_pid": subject("rota_membership", 0), "to_worker_pid": s }))
            .await
            .json();
        assert_eq!(seat["action"], "closed", "{seat}");

        // The swaps: one passes to S, the other is removed.
        let swap: Value = request
            .post(&handover)
            .json(&json!({ "kind": "rota_swap", "subject_pid": subject("rota_swap", 0), "to_worker_pid": s }))
            .await
            .json();
        assert_eq!(swap["action"], "reassigned", "{swap}");
        let dropped: Value = request.post(&handover).json(&json!({ "kind": "rota_swap", "subject_pid": subject("rota_swap", 1) })).await.json();
        assert_eq!(dropped["action"], "closed", "{dropped}");
        let swaps = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
            "SELECT count(*) FROM rota_overrides WHERE rota_pid = '{rota}' AND worker_pid = '{s}'")))
            .fetch_one(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        assert_eq!(swaps, 1, "one swap now names S");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one leaver, read top to bottom
// Handed to nobody, a dotted line, a backup role, a mentorship, a shift and a task are each closed; a dotted line handed to
// someone who already has one to that report ends rather than doubling; a cancelled record takes no hand-over.
async fn what_is_handed_to_nobody_is_closed_and_a_duplicate_line_ends() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut people = Vec::new();
        for n in ["L", "X", "Y", "S"] {
            let pid = seed_worker!(&request, &org, format!("HC-{tag}-{n}"), None).await;
            activate!(&request, &pid).await;
            people.push(pid);
        }
        let (l, x, y, s) = (&people[0], &people[1], &people[2], &people[3]);
        let last_day = workforce_planning_management::dates::today() + workforce_planning_management::dates::days(14);
        let dotted = |report: &str, manager: &str| {
            let (request, url, body) = (&request, format!("/api/workers/{report}/dotted-line-managers"), json!({ "manager_pid": manager }));
            async move { request.post(&url).json(&body).await.assert_status_ok(); }
        };
        dotted(x, l).await;
        dotted(y, l).await;
        dotted(y, s).await;
        request.post(&format!("/api/workers/{x}/backups")).json(&json!({ "backup_pid": l })).await.assert_status_ok();
        request
            .post("/api/mentorships")
            .json(&json!({ "mentor_pid": l, "mentee_pid": x, "focus": "growth" }))
            .await
            .assert_status_ok();
        let day = workforce_planning_management::dates::start_of_day(last_day + workforce_planning_management::dates::days(2));
        let shift: Value = request
            .post("/api/shifts")
            .json(&json!({ "department": "engineering", "starts_at": (day + jiff::SignedDuration::from_hours(9)).to_string(),
                           "ends_at": (day + jiff::SignedDuration::from_hours(17)).to_string(), "required_headcount": 1 }))
            .await
            .json();
        request
            .post(&format!("/api/shifts/{}/assignments", shift["pid"].as_str().unwrap()))
            .json(&json!({ "worker_pid": l }))
            .await
            .assert_status_ok();
        let leaver: Value = request
            .post(&format!("/api/workers/{l}/movements"))
            .json(&json!({ "kind": "leaver", "effective_on": last_day.to_string(), "reason": "resignation" }))
            .await
            .json();
        let mv = leaver["pid"].as_str().unwrap().to_string();
        let handover = format!("/api/movements/{mv}/handover");
        let held: Value = request.get(&handover).await.json();
        let items = held["items"].as_array().unwrap().clone();

        // Y's line goes to S, who already has one to Y: it ends, and S's own line stays.
        let y_line = items
            .iter()
            .find(|i| i["kind"] == "dotted_line" && i["label"].as_str().is_some_and(|l| l.contains("-Y")))
            .or_else(|| items.iter().filter(|i| i["kind"] == "dotted_line").nth(1))
            .unwrap();
        let moved: Value = request
            .post(&handover)
            .json(&json!({ "kind": "dotted_line", "subject_pid": y_line["subject_pid"], "to_worker_pid": s }))
            .await
            .json();
        assert_eq!(moved["action"], "reassigned", "{moved}");
        let live: Vec<String> = sqlx::query_scalar(
            "SELECT manager_pid::text FROM dotted_line_reports WHERE report_pid = $1::uuid AND ended_at IS NULL",
        )
        .bind(y)
        .fetch_all(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        assert_eq!(live, std::slice::from_ref(s), "one line to Y is left, S's own, not two");

        // Everything else that can close, handed to nobody, is closed.
        let held: Value = request.get(&handover).await.json();
        for item in held["items"].as_array().unwrap() {
            let kind = item["kind"].as_str().unwrap();
            if !["dotted_line", "backup", "mentorship", "shift", "task"].contains(&kind) {
                continue;
            }
            let closed = request.post(&handover).json(&json!({ "kind": kind, "subject_pid": item["subject_pid"] })).await;
            assert_eq!(closed.status_code(), 200, "{kind}: {}", closed.text());
            assert_eq!(closed.json::<Value>()["action"], "closed", "{kind}");
        }

        // A cancelled record takes no hand-over of everything.
        request.post(&format!("/api/movements/{mv}/cancel")).json(&json!({})).await.assert_status_ok();
        let late = request.post(&format!("{handover}/all")).json(&json!({ "to_worker_pid": s })).await;
        assert_eq!(late.status_code(), 422, "{}", late.text());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// A joiner's record keeps a trimmed note and is completed, audited as a joiner, once its checklist is done; a leaver record
// for someone who has already left is refused.
async fn a_joiner_is_completed_and_a_leaver_who_has_left_is_refused() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let joiner = seed_worker!(&request, &org, format!("JC-{tag}-J"), None).await;
        activate!(&request, &joiner).await;
        let opened: Value = request
            .post(&format!("/api/workers/{joiner}/movements"))
            .json(&json!({ "kind": "joiner", "notes": "  starts on the help desk  " }))
            .await
            .json();
        let mv = opened["pid"].as_str().unwrap().to_string();
        let view: Value = request.get(&format!("/api/movements/{mv}")).await.json();
        assert_eq!(view["notes"], "starts on the help desk", "{view}");
        for item in view["items"].as_array().unwrap() {
            request.post(&format!("/api/movement-items/{}/done", item["pid"].as_str().unwrap())).await.assert_status_ok();
        }
        let done = request.post(&format!("/api/movements/{mv}/complete")).await;
        assert_eq!(done.status_code(), 200, "{}", done.text());
        let actions: Vec<String> = sqlx::query_scalar(
            "SELECT action FROM audit_logs WHERE entity_pid = $1::uuid AND action LIKE 'joiner_%' ORDER BY id",
        )
        .bind(&joiner)
        .fetch_all(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        assert!(actions.iter().any(|a| a == "joiner_completed"), "{actions:?}");

        let gone = seed_worker!(&request, &org, format!("JC-{tag}-L"), None).await;
        activate!(&request, &gone).await;
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{gone}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        let last_day = workforce_planning_management::dates::today() + workforce_planning_management::dates::days(7);
        let late = request
            .post(&format!("/api/workers/{gone}/movements"))
            .json(&json!({ "kind": "leaver", "effective_on": last_day.to_string(), "reason": "resignation" }))
            .await;
        assert_eq!(late.status_code(), 422, "{}", late.text());
        assert!(late.text().contains("already left"), "{}", late.text());
    })
    .await;
}
