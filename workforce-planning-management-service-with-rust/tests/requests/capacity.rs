//! Delivery capacity (WPM-R56–R60, WPM-D45, WPM-D46, WPM-D50): skill pools, programme
//! demand, partner commitments, the capacity view, the start check and the recorded
//! start decision.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org};

fn a_programme() -> String {
    format!("thing:{}", uuid::Uuid::new_v4())
}

async fn make_pool(
    request: &loco_rs::TestServer,
    org: &str,
    name: &str,
    reservation_bp: i32,
) -> String {
    let pool: Value = request
        .post("/api/skill-pools")
        .json(&json!({
            "organization_ref": org, "name": name,
            "operations_reservation_bp": reservation_bp,
        }))
        .await
        .json();
    pool["pid"].as_str().expect("pool pid").to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Pools, members, demand and commitments refuse what is malformed, refuse URNs of the wrong
// kind, and soft-delete.
async fn capacity_records_validate_and_soft_delete() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let name = format!("Integration {tag}");
        let pool = make_pool(&request, &org, &name, 2_000).await;
        // The same name again, in any case, is refused.
        let again = request
            .post("/api/skill-pools")
            .json(&json!({ "organization_ref": org, "name": name.to_uppercase() }))
            .await;
        assert_eq!(again.status_code(), 422);
        for bad in [-1, 10_001] {
            let response = request
                .post("/api/skill-pools")
                .json(
                    &json!({ "organization_ref": org, "name": format!("x {bad} {tag}"),
                               "operations_reservation_bp": bad }),
                )
                .await;
            assert_eq!(response.status_code(), 422, "reservation {bad}");
        }
        // A pool belongs to an organization, by URN of the right kind.
        let wrong_kind = request
            .post("/api/skill-pools")
            .json(&json!({ "organization_ref": super::a_person(), "name": format!("y {tag}") }))
            .await;
        assert_eq!(wrong_kind.status_code(), 422);

        // Members: a role profile or a skill, never both, never neither, and it must exist.
        let members = format!("/api/skill-pools/{pool}/members");
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("Capacity role {tag}") }))
            .await
            .json();
        let role = profile["pid"].as_str().unwrap().to_string();
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("Capacity skill {tag}"), "category": "technical" }))
            .await
            .json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        let missing = uuid::Uuid::new_v4().to_string();
        for (bad, why) in [
            (json!({}), "neither"),
            (
                json!({ "role_profile_pid": role, "skill_pid": skill, "min_proficiency": 3 }),
                "both",
            ),
            (json!({ "skill_pid": skill }), "skill without a minimum"),
            (
                json!({ "skill_pid": skill, "min_proficiency": 6 }),
                "proficiency out of range",
            ),
            (
                json!({ "role_profile_pid": role, "min_proficiency": 3 }),
                "role with a minimum",
            ),
            (json!({ "role_profile_pid": missing }), "unknown role"),
            (
                json!({ "skill_pid": missing, "min_proficiency": 3 }),
                "unknown skill",
            ),
        ] {
            assert_eq!(
                request.post(&members).json(&bad).await.status_code(),
                422,
                "{why}"
            );
        }
        let added: Value = request
            .post(&members)
            .json(&json!({ "role_profile_pid": role }))
            .await
            .json();
        let member = added["pid"].as_str().unwrap().to_string();
        assert_eq!(
            request
                .post(&members)
                .json(&json!({ "role_profile_pid": role }))
                .await
                .status_code(),
            422,
            "the same member twice"
        );
        let shown: Value = request
            .get(&format!("/api/skill-pools/{pool}"))
            .await
            .json();
        assert_eq!(shown["members"].as_array().unwrap().len(), 1);
        request
            .delete(&format!("{members}/{member}"))
            .await
            .assert_status_ok();
        let shown: Value = request
            .get(&format!("/api/skill-pools/{pool}"))
            .await
            .json();
        assert!(shown["members"].as_array().unwrap().is_empty());

        // Demand: programme by URN (a thing), a first-of-month, a positive FTE, and never
        // `active` by a plain write.
        let programme = a_programme();
        let good = json!({ "programme_ref": programme, "pool_pid": pool,
                           "month": "2027-01-01", "fte_centi": 150 });
        for (field, value) in [
            ("programme_ref", json!(super::a_worker())),
            ("programme_ref", json!("not-a-urn")),
            ("month", json!("2027-01-15")),
            ("fte_centi", json!(0)),
            ("fte_centi", json!(-5)),
            ("fte_centi", json!(1_000_001)),
            ("status", json!("active")),
            ("status", json!("done")),
            ("pool_pid", json!(missing)),
        ] {
            let mut bad = good.clone();
            bad[field] = value.clone();
            assert_eq!(
                request
                    .put("/api/programme-demands")
                    .json(&bad)
                    .await
                    .status_code(),
                if field == "pool_pid" { 404 } else { 422 },
                "{field} = {value}"
            );
        }
        let set: Value = request
            .put("/api/programme-demands")
            .json(&good)
            .await
            .json();
        // The same programme, pool and month again updates the one claim.
        let mut changed = good.clone();
        changed["fte_centi"] = json!(175);
        let reset: Value = request
            .put("/api/programme-demands")
            .json(&changed)
            .await
            .json();
        assert_eq!(set["pid"], reset["pid"]);
        let listed: Vec<Value> = request
            .get(&format!("/api/programme-demands?pool={pool}"))
            .await
            .json();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["fte_centi"], 175);
        assert_eq!(listed[0]["status"], "proposed");
        request
            .delete(&format!(
                "/api/programme-demands/{}",
                set["pid"].as_str().unwrap()
            ))
            .await
            .assert_status_ok();
        let listed: Vec<Value> = request
            .get(&format!("/api/programme-demands?pool={pool}"))
            .await
            .json();
        assert!(listed.is_empty(), "soft-deleted");

        // Commitments: a partner organization by URN, a known status.
        let partner = an_org();
        let commit = json!({ "partner_ref": partner, "pool_pid": pool, "month": "2027-01-01",
                             "fte_centi": 50, "status": "confirmed" });
        for (field, value) in [
            ("partner_ref", json!(a_programme())),
            ("status", json!("maybe")),
            ("fte_centi", json!(0)),
            ("month", json!("2027-01-02")),
        ] {
            let mut bad = commit.clone();
            bad[field] = value.clone();
            assert_eq!(
                request
                    .put("/api/partner-commitments")
                    .json(&bad)
                    .await
                    .status_code(),
                422,
                "{field} = {value}"
            );
        }
        let set: Value = request
            .put("/api/partner-commitments")
            .json(&commit)
            .await
            .json();
        let listed: Vec<Value> = request
            .get(&format!("/api/partner-commitments?pool={pool}"))
            .await
            .json();
        assert_eq!(listed.len(), 1);
        request
            .delete(&format!(
                "/api/partner-commitments/{}",
                set["pid"].as_str().unwrap()
            ))
            .await
            .assert_status_ok();

        // Settings.
        assert_eq!(
            request
                .put("/api/capacity/settings")
                .json(&json!({ "organization_ref": org, "wip_limit": -1 }))
                .await
                .status_code(),
            422
        );

        // Soft delete of the pool: gone from the list and from a read.
        request
            .put(&format!("/api/skill-pools/{pool}"))
            .json(&json!({ "name": format!("Renamed {tag}"), "operations_reservation_bp": 500 }))
            .await
            .assert_status_ok();
        request
            .delete(&format!("/api/skill-pools/{pool}"))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .get(&format!("/api/skill-pools/{pool}"))
                .await
                .status_code(),
            404
        );
        let listed: Vec<Value> = request
            .get(&format!("/api/skill-pools?organization={org}"))
            .await
            .json();
        assert!(listed.is_empty());
        // The name is free again.
        make_pool(&request, &org, &name, 0).await;
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// One pool over-committed in February, one month (March) with a silent partner, a start
// check that fails and finds the earliest fit, the work-in-progress limit, and decisions.
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn capacity_view_start_check_and_decisions() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let title = format!("Capacity engineer {tag}");

        // Three full-time workers and one half-time worker, all with the role. The third is
        // on approved leave for the whole of February 2027.
        let mut pids = Vec::new();
        for (n, fte) in [(1, 100), (2, 100), (3, 100), (4, 50)] {
            let created: Value = request
                .post("/api/workers")
                .json(&json!({
                    "person_ref": super::a_person(), "organization_ref": org,
                    "worker_number": format!("CAP-{tag}-{n}"),
                    "display_name": format!("Test Worker CAP-{n}"),
                    "employment_type": "permanent", "fte_percent": fte,
                    "department": "engineering", "job_title": title,
                    "hired_on": "2026-01-05",
                }))
                .await
                .json();
            let pid = created["pid"].as_str().unwrap().to_string();
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        request
            .post(&format!("/api/workers/{}/leave-entitlements", pids[2]))
            .json(&json!({ "kind": "sick", "year": 2027, "entitled_days": 40 }))
            .await
            .assert_status_ok();
        let leave: Value = request
            .post(&format!("/api/workers/{}/leave-requests", pids[2]))
            .json(&json!({ "kind": "sick", "start_on": "2027-02-01", "end_on": "2027-02-28" }))
            .await
            .json();
        request
            .post(&format!("/api/leave-requests/{}/approve", leave["pid"].as_str().unwrap()))
            .await
            .assert_status_ok();
        // A requested (not approved) leave must not count.
        let pending: Value = request
            .post(&format!("/api/workers/{}/leave-requests", pids[0]))
            .json(&json!({ "kind": "sick", "start_on": "2027-03-01", "end_on": "2027-03-31" }))
            .await
            .json();
        assert!(pending["pid"].is_string());

        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": title }))
            .await
            .json();
        let pool = make_pool(&request, &org, &format!("Engineering {tag}"), 2_000).await;
        let idle = make_pool(&request, &org, &format!("Idle {tag}"), 0).await;
        request
            .post(&format!("/api/skill-pools/{pool}/members"))
            .json(&json!({ "role_profile_pid": profile["pid"] }))
            .await
            .assert_status_ok();

        // Programme A claims 2.50, 2.50 and 3.00 FTE in January, February and March 2027.
        let a = a_programme();
        for (month, fte) in [("2027-01-01", 250), ("2027-02-01", 250), ("2027-03-01", 300)] {
            request
                .put("/api/programme-demands")
                .json(&json!({ "programme_ref": a, "pool_pid": pool, "month": month, "fte_centi": fte }))
                .await
                .assert_status_ok();
        }
        // A partner: 0.50 confirmed in January, 1.00 only requested in February, nothing in
        // March.
        let partner = an_org();
        for (month, fte, status) in [("2027-01-01", 50, "confirmed"), ("2027-02-01", 100, "requested")] {
            request
                .put("/api/partner-commitments")
                .json(&json!({ "partner_ref": partner, "pool_pid": pool, "month": month,
                               "fte_centi": fte, "status": status }))
                .await
                .assert_status_ok();
        }

        // Claims are proposed, so the view does not yet count them.
        let before: Value = request
            .get(&format!("/api/capacity?organization={org}&from=2027-01-01&months=3"))
            .await
            .json();
        let cells = before["pools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["pid"] == pool.as_str())
            .unwrap()["cells"]
            .clone();
        assert_eq!(cells[0]["demand_centi"], 0);
        assert_eq!(cells[0]["proposed_centi"], 250);
        assert_eq!(before["constraint_pool"], Value::Null);

        // A proposes to start. The check fails (February is short) and a person proceeds
        // anyway, with a reason: the decision is theirs.
        let a_check: Value = request
            .post("/api/capacity/start-check")
            .json(&json!({ "organization_ref": org, "programme_ref": a }))
            .await
            .json();
        assert_eq!(a_check["fit"], "no");
        for bad in [
            json!({ "organization_ref": org, "programme_ref": a, "decision": "proceed" }),
            json!({ "organization_ref": org, "programme_ref": a, "decision": "proceed", "reason": "  " }),
            json!({ "organization_ref": org, "programme_ref": a, "decision": "maybe", "reason": "x" }),
        ] {
            assert_eq!(
                request.post("/api/capacity/start-decisions").json(&bad).await.status_code(),
                422,
                "{bad}"
            );
        }
        request
            .post("/api/capacity/start-decisions")
            .json(&json!({ "organization_ref": org, "programme_ref": a, "decision": "proceed",
                           "reason": "Contractual start; the shortfall is accepted." }))
            .await
            .assert_status_ok();

        // The view now: January fits, February is over, March cannot be said.
        let view: Value = request
            .get(&format!("/api/capacity?organization={org}&from=2027-01-01&months=3"))
            .await
            .json();
        let pools = view["pools"].as_array().unwrap();
        let engineering = pools.iter().find(|p| p["pid"] == pool.as_str()).unwrap();
        let cells = engineering["cells"].as_array().unwrap();
        // 3.5 FTE less 20% reservation = 2.80; in February one full-timer is away: 2.5 × 0.8.
        assert_eq!(cells[0]["supply_centi"], 280);
        assert_eq!(cells[1]["supply_centi"], 200);
        assert_eq!(cells[2]["supply_centi"], 280, "a requested leave does not count");
        assert_eq!(cells[0]["status"], "fits");
        assert_eq!(cells[1]["status"], "over");
        assert_eq!(cells[1]["remaining_shortfall_centi"], 50, "requested capacity does not cover it");
        assert_eq!(cells[2]["status"], "unknown", "March: short, and the partner is silent");
        let silent = cells[2]["partners"].as_array().unwrap();
        assert_eq!(silent.len(), 1);
        assert_eq!(silent[0]["status"], Value::Null);
        assert_eq!(silent[0]["fte_centi"], Value::Null, "unknown, not zero");
        assert_eq!(engineering["over_months"], 1);
        assert_eq!(engineering["unknown_months"], 1);
        assert_eq!(view["constraint_pool"], pool.as_str());
        let idle_pool = pools.iter().find(|p| p["pid"] == idle.as_str()).unwrap();
        assert_eq!(idle_pool["cells"][0]["supply_centi"], 0);
        assert_eq!(idle_pool["cells"][0]["status"], "fits");
        assert!(view["derivation"].as_str().unwrap().contains("unknown"));

        // B wants 1.00 FTE in January and February. A limit of one active programme on the
        // constraint pool is already used by A.
        let b = a_programme();
        for month in ["2027-01-01", "2027-02-01"] {
            request
                .put("/api/programme-demands")
                .json(&json!({ "programme_ref": b, "pool_pid": pool, "month": month, "fte_centi": 100 }))
                .await
                .assert_status_ok();
        }
        request
            .put("/api/capacity/settings")
            .json(&json!({ "organization_ref": org, "wip_limit": 1 }))
            .await
            .assert_status_ok();
        let settings: Value = request
            .get(&format!("/api/capacity/settings?organization={org}"))
            .await
            .json();
        assert_eq!(settings["wip_limit"], 1);
        let check: Value = request
            .post("/api/capacity/start-check")
            .json(&json!({ "organization_ref": org, "programme_ref": b }))
            .await
            .json();
        assert_eq!(check["fit"], "no");
        let lines = check["lines"].as_array().unwrap();
        // January: 0.30 free + 0.50 confirmed partner = 0.80, wants 1.00: short by 0.20.
        assert_eq!(lines[0]["outcome"], "short");
        assert_eq!(lines[0]["short_by_centi"], 20);
        // February: already 0.50 over; short by 1.50.
        assert_eq!(lines[1]["short_by_centi"], 150);
        // March is silent, so shifting to March+April is not known to fit; April+May is.
        assert_eq!(check["earliest_shift_months"], 3);
        assert_eq!(check["earliest_start_month"], "2027-04-01");
        assert_eq!(check["wip"]["active"], 1);
        assert_eq!(check["wip"]["would_be"], 2);
        assert_eq!(check["wip"]["exceeds"], true);
        assert!(check["advisory"].as_str().unwrap().contains("person decides"));

        // What-if lines store nothing, and must name a pool of this organization.
        let what_if: Value = request
            .post("/api/capacity/start-check")
            .json(&json!({ "organization_ref": org, "programme_ref": a_programme(),
                           "lines": [{ "pool_pid": pool, "month": "2027-05-01", "fte_centi": 50 }] }))
            .await
            .json();
        assert_eq!(what_if["fit"], "yes");
        assert_eq!(
            request
                .post("/api/capacity/start-check")
                .json(&json!({ "organization_ref": org, "programme_ref": a_programme(),
                               "lines": [{ "pool_pid": uuid::Uuid::new_v4(), "month": "2027-05-01",
                                           "fte_centi": 50 }] }))
                .await
                .status_code(),
            422
        );
        // A programme with nothing proposed cannot be checked.
        assert_eq!(
            request
                .post("/api/capacity/start-check")
                .json(&json!({ "organization_ref": org, "programme_ref": a_programme() }))
                .await
                .status_code(),
            422
        );
        // A decision cannot be taken on what-if lines.
        assert_eq!(
            request
                .post("/api/capacity/start-decisions")
                .json(&json!({ "organization_ref": org, "programme_ref": b, "decision": "defer",
                               "reason": "x", "lines": [] }))
                .await
                .status_code(),
            422
        );

        // Defer B: recorded, and B stays proposed.
        request
            .post("/api/capacity/start-decisions")
            .json(&json!({ "organization_ref": org, "programme_ref": b, "decision": "defer",
                           "reason": "Wait for the partner to answer for March." }))
            .await
            .assert_status_ok();
        let still: Vec<Value> = request
            .get(&format!("/api/programme-demands?programme={b}"))
            .await
            .json();
        assert!(still.iter().all(|c| c["status"] == "proposed"));
        // Then proceed anyway: recorded, and B's claims become active and now count.
        request
            .post("/api/capacity/start-decisions")
            .json(&json!({ "organization_ref": org, "programme_ref": b, "decision": "proceed",
                           "reason": "Sponsor decision; the risk is accepted." }))
            .await
            .assert_status_ok();
        let now: Vec<Value> = request
            .get(&format!("/api/programme-demands?programme={b}"))
            .await
            .json();
        assert!(now.iter().all(|c| c["status"] == "active"));
        let after: Value = request
            .get(&format!("/api/capacity?organization={org}&from=2027-01-01&months=3"))
            .await
            .json();
        let cells = after["pools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["pid"] == pool.as_str())
            .unwrap()["cells"]
            .clone();
        assert_eq!(cells[0]["demand_centi"], 350);
        assert_eq!(cells[0]["status"], "over", "280 + 50 firm partner < 350");
        assert_eq!(cells[0]["remaining_shortfall_centi"], 20);

        // The decisions are listed, newest first, with the reasons and the evidence.
        let decisions: Vec<Value> = request
            .get(&format!("/api/capacity/start-decisions?programme={b}"))
            .await
            .json();
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0]["decision"], "proceed");
        assert_eq!(decisions[1]["decision"], "defer");
        assert_eq!(decisions[1]["fit"], "no");
        assert_eq!(decisions[1]["wip_exceeds"], true);
        assert_eq!(decisions[1]["earliest_shift_months"], 3);

        // The audit trail names the decision, not the reason text.
        let audits: Vec<Value> = request
            .get(&format!("/api/audits/{}", decisions[0]["pid"].as_str().unwrap()))
            .await
            .json();
        assert!(!audits.is_empty());
        assert!(!serde_json::to_string(&audits).unwrap().contains("Sponsor decision"));
    })
    .await;
}
