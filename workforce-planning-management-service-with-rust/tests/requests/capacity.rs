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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// The governance load register (WPM-R64, WPM-D45): commitments of a role, never a person; the senior
// hours a month; refusals; the register is scoped to its organization.
async fn governance_load_totals_senior_hours_and_refuses_the_malformed() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let other = an_org();
        let body = |name: &str, role: &str, senior: bool, frequency: &str, minutes: i32, holders: i32| {
            json!({ "organization_ref": org, "name": name, "role": role, "senior": senior, "kind": "meeting",
                    "frequency": frequency, "minutes": minutes, "holders": holders })
        };
        // Monthly, 90 minutes, 3 people: 4.5 hours a month, senior.
        let board: Value = request
            .post("/api/governance-commitments")
            .json(&body("Oversight board", "director", true, "monthly", 90, 3))
            .await
            .json();
        assert_eq!(board["hours_per_month"], 4.5);
        // Weekly, 60 minutes, 2 people: 8.7 hours a month, not senior.
        request
            .post("/api/governance-commitments")
            .json(&body("Returns", "analyst", false, "weekly", 60, 2))
            .await
            .assert_status_ok();
        let load: Value = request
            .get(&format!("/api/governance-commitments/load?organization={org}"))
            .await
            .json();
        assert_eq!(load["commitments"], 2);
        assert_eq!(load["senior_hours_per_month"], 4.5, "only the senior role counts");
        assert_eq!(load["all_hours_per_month"], 13.2);
        assert_eq!(load["roles"][0]["role"], "analyst", "heaviest first");
        assert!(load["basis"].as_str().unwrap().contains("estimate"));
        // Every refusal is a 422.
        for (why, bad) in [
            ("unknown kind", json!({ "kind": "party" })),
            ("unknown frequency", json!({ "frequency": "daily" })),
            ("no minutes", json!({ "minutes": 0 })),
            ("no holders", json!({ "holders": 0 })),
            ("blank role", json!({ "role": "  " })),
        ] {
            let mut b = body("x", "director", true, "monthly", 60, 1);
            b.as_object_mut().unwrap().extend(bad.as_object().unwrap().clone());
            let r = request.post("/api/governance-commitments").json(&b).await;
            assert_eq!(r.status_code(), 422, "{why}");
        }
        let mut wrong = body("x", "director", true, "monthly", 60, 1);
        wrong["organization_ref"] = json!("worker:00000000-0000-0000-0000-000000000000");
        assert_eq!(request.post("/api/governance-commitments").json(&wrong).await.status_code(), 422);
        // Change and remove.
        let pid = board["pid"].as_str().unwrap();
        let changed: Value = request
            .put(&format!("/api/governance-commitments/{pid}"))
            .json(&body("Oversight board", "director", true, "quarterly", 120, 2))
            .await
            .json();
        assert_eq!(changed["hours_per_month"], 1.3, "4 × 120 × 2 minutes a year, a month's share");
        request.delete(&format!("/api/governance-commitments/{pid}")).await.assert_status_ok();
        assert_eq!(request.delete(&format!("/api/governance-commitments/{pid}")).await.status_code(), 404);
        let after: Value = request
            .get(&format!("/api/governance-commitments/load?organization={org}"))
            .await
            .json();
        assert_eq!(after["senior_hours_per_month"], 0.0);
        // Another organization's register is its own.
        let theirs: Value = request.get(&format!("/api/governance-commitments?organization={other}")).await.json();
        assert_eq!(theirs.as_array().unwrap().len(), 0);
        assert_eq!(request.get("/api/governance-commitments").await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Key-person risk (WPM-R61, WPM-D47): holders, qualified backups, ready successors; the aggregate names
// no one; the named read gives the sole holder of a single point of failure.
async fn key_person_risk_counts_holders_backups_and_successors() {
    use super::{activate, seed_worker};
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let skill_name = format!("Key skill {tag}");
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": skill_name, "category": "technical" }))
            .await
            .json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("Key role {tag}") }))
            .await
            .json();
        let profile = profile["pid"].as_str().unwrap().to_string();
        request
            .put(&format!("/api/role-profiles/{profile}/requirements"))
            .json(&json!({ "skill_pid": skill, "min_proficiency": 4, "importance": "critical" }))
            .await
            .assert_status_ok();
        let holder = seed_worker!(&request, &org, format!("K-1-{tag}"), None).await;
        let other = seed_worker!(&request, &org, format!("K-2-{tag}"), None).await;
        let junior = seed_worker!(&request, &org, format!("K-3-{tag}"), None).await;
        for w in [&holder, &other, &junior] {
            activate!(&request, w).await;
        }
        let declare = |who: String, level: i32| {
            let request = &request;
            let skill = skill.clone();
            async move {
                request
                    .put(&format!("/api/workers/{who}/skills"))
                    .json(&json!({ "skill_pid": skill, "proficiency": level }))
                    .await
                    .assert_status_ok();
            }
        };
        declare(holder.clone(), 5).await;
        declare(junior.clone(), 3).await; // below the bar: neither a holder nor a qualified backup
        let row = |report: &Value, department: Option<&str>| {
            report["skills"].as_array().unwrap().iter()
                .find(|r| r["skill"] == skill_name.as_str() && r["department"] == json!(department))
                .cloned()
        };
        let report = || async { request.get("/api/workforce-intelligence/key-person").await.json::<Value>() };
        // One holder, nobody covering: a single point of failure.
        let first = report().await;
        assert_eq!(row(&first, None).unwrap()["status"], "single_point_of_failure");
        assert_eq!(row(&first, Some("engineering")).unwrap()["holders"], 1);
        assert!(!first.to_string().contains(&holder), "the aggregate names no one");
        // Read for one organization: this one has the skill, another does not.
        let mine: Value = request
            .get(&format!("/api/workforce-intelligence/key-person?organization={org}"))
            .await
            .json();
        assert_eq!(row(&mine, None).unwrap()["status"], "single_point_of_failure");
        let elsewhere: Value = request
            .get(&format!("/api/workforce-intelligence/key-person?organization=organization:{}", uuid::Uuid::new_v4()))
            .await
            .json();
        let there = row(&elsewhere, None).expect("a critical skill is listed for every organization");
        assert_eq!((there["holders"].as_i64(), there["status"].as_str()), (Some(0), Some("no_holder")), "another organization's people are not counted");
        let named: Value = request.get("/api/workforce-intelligence/key-person/named").await.json();
        assert!(named["people"].as_array().unwrap().iter().any(|p| p["skill"] == skill_name.as_str() && p["worker_pid"] == holder.as_str()));
        // A backup who is below the bar does not cover.
        request.post(&format!("/api/workers/{holder}/backups")).json(&json!({ "backup_pid": junior })).await.assert_status_ok();
        assert_eq!(row(&report().await, None).unwrap()["status"], "single_point_of_failure");
        // A person ranked ready now on the holder's succession plan covers them.
        let plan: Value = request
            .post("/api/succession-plans")
            .json(&json!({ "role_title": format!("Key plan {tag}"), "department": "engineering", "criticality": 4, "incumbent_pid": holder }))
            .await
            .json();
        let plan = plan["pid"].as_str().unwrap().to_string();
        request
            .post(&format!("/api/succession-plans/{plan}/candidates"))
            .json(&json!({ "worker_pid": junior, "readiness": "ready_2y" }))
            .await
            .assert_status_ok();
        assert_eq!(row(&report().await, None).unwrap()["status"], "single_point_of_failure", "ready in two years is not cover");
        request
            .post(&format!("/api/succession-plans/{plan}/candidates"))
            .json(&json!({ "worker_pid": other, "readiness": "ready_now" }))
            .await
            .assert_status_ok();
        let covered = report().await;
        let r = row(&covered, None).unwrap();
        assert_eq!((r["holders"].as_i64(), r["ready_successors"].as_i64()), (Some(1), Some(1)));
        assert_eq!(r["status"], "covered_single_holder");
        // A second holder shares the skill, and a backup who holds it counts as a qualified backup.
        declare(other.clone(), 4).await;
        request.post(&format!("/api/workers/{holder}/backups")).json(&json!({ "backup_pid": other })).await.assert_status_ok();
        let shared = report().await;
        let r = row(&shared, None).unwrap();
        assert_eq!((r["holders"].as_i64(), r["backups"].as_i64()), (Some(2), Some(1)));
        assert_eq!(r["status"], "shared");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Oversight (WPM-R65, WPM-R66, WPM-D48, WPM-D49): the organization's own scale, a recorded position, an exit
// criterion with a reported measure, and a sustain rule that is met only by consecutive months of evidence.
async fn oversight_status_reports_the_sustain_rule_and_never_moves_the_level() {
    use chrono::{Datelike, Months, Utc};
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let first_of_this_month = Utc::now().date_naive().with_day(1).unwrap();
        let month = |back: u32| first_of_this_month - Months::new(back); // 1 = last complete month
        let status = || async {
            request.get(&format!("/api/oversight/status?organization={org}")).await.json::<Value>()
        };
        // Nothing is pre-loaded.
        let empty = status().await;
        assert_eq!(empty["scale"].as_array().unwrap().len(), 0);
        assert!(empty["current"].is_null());
        // The scale: ordinals are unique and bounded; a name is required.
        let mut levels = Vec::new();
        for (ordinal, name) in [(1, "Routine"), (2, "Enhanced")] {
            let l: Value = request.post("/api/oversight/levels").json(&json!({ "organization_ref": org, "ordinal": ordinal, "name": name })).await.json();
            levels.push(l["pid"].as_str().unwrap().to_string());
        }
        for (why, body) in [
            ("a repeated ordinal", json!({ "organization_ref": org, "ordinal": 1, "name": "Again" })),
            ("an ordinal out of range", json!({ "organization_ref": org, "ordinal": 21, "name": "Far" })),
            ("a blank name", json!({ "organization_ref": org, "ordinal": 3, "name": " " })),
            ("a non-organization URN", json!({ "organization_ref": "worker:00000000-0000-0000-0000-000000000000", "ordinal": 3, "name": "x" })),
        ] {
            assert_eq!(request.post("/api/oversight/levels").json(&body).await.status_code(), 422, "{why}");
        }
        // A position is recorded by a person with a body and a reason.
        let position = |level: &str, body: &str, reason: &str| json!({ "organization_ref": org, "level_pid": level, "effective_on": "2026-01-01", "deciding_body": body, "reason": reason });
        assert_eq!(request.post("/api/oversight/positions").json(&position(&levels[1], "", "because")).await.status_code(), 422);
        assert_eq!(request.post("/api/oversight/positions").json(&position(&levels[1], "The panel", "")).await.status_code(), 422);
        request.post("/api/oversight/positions").json(&position(&levels[1], "The panel", "Staffing review")).await.assert_status_ok();
        let current = status().await;
        assert_eq!(current["current"]["level"], "Enhanced");
        assert_eq!(current["current"]["deciding_body"], "The panel");
        // A criterion on the level the organization is at, with a reported measure: at most 800, three months running.
        let criterion: Value = request.post("/api/oversight/criteria").json(&json!({ "level_pid": levels[1], "name": "Staffing is stable" })).await.json();
        let criterion = criterion["pid"].as_str().unwrap().to_string();
        let measure_body = |origin: &str, direction: &str, sustain: i64| json!({ "criterion_pid": criterion, "name": "Vacancy rate", "origin": origin, "unit": "basis points", "threshold": 800, "direction": direction, "sustain": sustain });
        for (why, body) in [
            ("a derived measure that names no derivation", measure_body("derived", "at_most", 3)),
            ("an unknown direction", measure_body("reported", "below", 3)),
            ("a sustain of zero", measure_body("reported", "at_most", 0)),
        ] {
            assert_eq!(request.post("/api/oversight/measures").json(&body).await.status_code(), 422, "{why}");
        }
        let measure: Value = request.post("/api/oversight/measures").json(&measure_body("reported", "at_most", 3)).await.json();
        let measure = measure["pid"].as_str().unwrap().to_string();
        let reading = |back: u32, value: Value| json!({ "period_on": month(back).to_string(), "value": value, "evidence": "Read from the monthly return", "source_document": "Monthly return" });
        let put = |body: Value| request.put(&format!("/api/oversight/measures/{measure}/readings")).json(&body);
        // A value without its evidence or source is refused.
        assert_eq!(put(json!({ "period_on": month(1).to_string(), "value": 500 })).await.status_code(), 422);
        assert_eq!(put(json!({ "period_on": month(1).to_string(), "value": 500, "evidence": "x" })).await.status_code(), 422);
        // Two months of evidence do not meet a sustain rule of three, and the answer says why.
        put(reading(1, json!(700))).await.assert_status_ok();
        put(reading(2, json!(800))).await.assert_status_ok();
        let two = status().await;
        assert_eq!(two["criteria"][0]["status"], "unknown");
        let m = &two["criteria"][0]["measures"][0];
        assert_eq!((m["run"].as_i64(), m["required"].as_i64()), (Some(2), Some(3)));
        assert_eq!(m["stopped_by"]["reason"], "no_reading");
        // A null month breaks the run and is not a pass.
        put(json!({ "period_on": month(3).to_string(), "value": null })).await.assert_status_ok();
        assert_eq!(status().await["criteria"][0]["measures"][0]["stopped_by"]["reason"], "null_reading");
        // A breach is not met.
        put(reading(3, json!(900))).await.assert_status_ok();
        let breached = status().await;
        assert_eq!(breached["criteria"][0]["status"], "not_met");
        assert_eq!(breached["criteria"][0]["measures"][0]["stopped_by"]["reason"], "breach");
        // Replacing the month with a value at the threshold meets the rule; the same month is one reading.
        put(reading(3, json!(800))).await.assert_status_ok();
        let met = status().await;
        assert_eq!(met["criteria"][0]["status"], "met");
        assert_eq!(met["criteria"][0]["measures"][0]["reported_periods"], 3);
        // The service never moves the level.
        assert_eq!(met["current"]["level"], "Enhanced");
        assert_eq!(met["positions"].as_array().unwrap().len(), 1);
        // A level in use cannot be removed; a criterion goes with its measures and readings.
        assert_eq!(request.delete(&format!("/api/oversight/levels/{}", levels[1])).await.status_code(), 422);
        request.delete(&format!("/api/oversight/criteria/{criterion}")).await.assert_status_ok();
        assert_eq!(status().await["criteria"].as_array().unwrap().len(), 0);
        assert_eq!(request.delete(&format!("/api/oversight/measures/{measure}")).await.status_code(), 404);
        request.delete(&format!("/api/oversight/levels/{}", levels[0])).await.assert_status_ok();
        // The status needs an organization.
        assert_eq!(request.get("/api/oversight/status").await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Derived oversight measures (WPM-R67, WPM-T114): the service computes the reading itself, shows the
// months it used, and a month the data cannot speak for is a null that is never a pass.
async fn derived_oversight_measures_are_computed_and_an_unknown_month_is_never_a_pass() {
    use chrono::{Datelike, Months, NaiveDate, Utc};
    use sea_orm::{ActiveValue, EntityTrait};
    use workforce_planning_management::models::_entities::{headcount_snapshots, workers};
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let first_of_this_month = Utc::now().date_naive().with_day(1).unwrap();
        let month = |back: u32| first_of_this_month - Months::new(back);
        let status = || async {
            request.get(&format!("/api/oversight/status?organization={org}")).await.json::<Value>()
        };
        // The catalogue says what each derivation counts, and leaves out what cannot be dated.
        let catalogue: Value = request.get("/api/oversight/derivations").await.json();
        let keys: Vec<&str> = catalogue["derivations"].as_array().unwrap().iter().map(|d| d["key"].as_str().unwrap()).collect();
        assert_eq!(keys, ["turnover_rate", "contingent_share", "overcommitted_pools", "time_to_report"]);
        // Twenty people hired long ago; one of them leaves four days into last month but one.
        let left_on = month(2) + chrono::Days::new(4);
        for n in 0..20 {
            workers::Entity::insert(workers::ActiveModel {
                pid: ActiveValue::set(uuid::Uuid::new_v4()),
                person_ref: ActiveValue::set(super::a_person()),
                organization_ref: ActiveValue::set(org.clone()),
                worker_number: ActiveValue::set(format!("D{n:03}")),
                display_name: ActiveValue::set(format!("Synthetic Worker {n}")),
                status: ActiveValue::set("active".into()),
                employment_type: ActiveValue::set("permanent".into()),
                fte_percent: ActiveValue::set(100),
                department: ActiveValue::set("engineering".into()),
                job_title: ActiveValue::set("Engineer".into()),
                hired_on: ActiveValue::set(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                terminated_on: ActiveValue::set((n == 0).then_some(left_on)),
                ..Default::default()
            })
            .exec(&ctx.db)
            .await
            .unwrap();
        }
        // Two snapshots in last month: the earlier shows no contingent FTE, the later (the one that
        // counts) 100 contractor and 100 fixed-term out of 20 people and 2000 FTE.
        for (days, contractor, fixed) in [(2, 0, 0), (10, 100, 100)] {
        headcount_snapshots::Entity::insert(headcount_snapshots::ActiveModel {
            pid: ActiveValue::set(uuid::Uuid::new_v4()),
            organization_ref: ActiveValue::set(org.clone()),
            department: ActiveValue::set("engineering".into()),
            as_of: ActiveValue::set(month(1) + chrono::Days::new(days)),
            headcount: ActiveValue::set(20),
            fte_percent_total: ActiveValue::set(2000),
            contractor_fte_percent_total: ActiveValue::set(Some(contractor)),
            fixed_term_fte_percent_total: ActiveValue::set(Some(fixed)),
            ..Default::default()
        })
        .exec(&ctx.db)
        .await
        .unwrap();
        }
        let level: Value = request.post("/api/oversight/levels").json(&json!({ "organization_ref": org, "ordinal": 1, "name": "Synthetic level" })).await.json();
        let level = level["pid"].as_str().unwrap().to_string();
        request.post("/api/oversight/positions").json(&json!({ "organization_ref": org, "level_pid": level, "effective_on": "2026-01-01", "deciding_body": "Synthetic panel", "reason": "Synthetic review" })).await.assert_status_ok();
        let criterion: Value = request.post("/api/oversight/criteria").json(&json!({ "level_pid": level, "name": "Synthetic criterion" })).await.json();
        let criterion = criterion["pid"].as_str().unwrap().to_string();
        let measure = |derivation: Option<&str>, unit: &str, threshold: i64, sustain: i64| json!({
            "criterion_pid": criterion, "name": "m", "origin": "derived", "derivation": derivation,
            "unit": unit, "threshold": threshold, "direction": "at_most", "sustain": sustain });
        for (why, body) in [
            ("an unknown derivation", measure(Some("sickness_absence"), "basis points", 1, 1)),
            ("no derivation", measure(None, "basis points", 1, 1)),
            ("a unit that is not the derivation's", measure(Some("turnover_rate"), "percent", 1, 1)),
        ] {
            assert_eq!(request.post("/api/oversight/measures").json(&body).await.status_code(), 422, "{why}");
        }
        let mut reported = measure(None, "basis points", 1, 1);
        reported["origin"] = json!("reported");
        reported["derivation"] = json!("turnover_rate");
        assert_eq!(request.post("/api/oversight/measures").json(&reported).await.status_code(), 422, "a reported measure names no derivation");
        // Turnover: no leaver last month, one the month before (1 / 19.5 = 513 basis points), none before.
        let turnover: Value = request.post("/api/oversight/measures").json(&measure(Some("turnover_rate"), "basis points", 600, 3)).await.json();
        let turnover_pid = turnover["pid"].as_str().unwrap().to_string();
        let contingent: Value = request.post("/api/oversight/measures").json(&measure(Some("contingent_share"), "basis points", 1000, 1)).await.json();
        let contingent_pid = contingent["pid"].as_str().unwrap().to_string();
        let earlier_contingent: Value = request.post("/api/oversight/measures").json(&measure(Some("contingent_share"), "basis points", 1000, 2)).await.json();
        let earlier_contingent = earlier_contingent["pid"].as_str().unwrap().to_string();
        let pools: Value = request.post("/api/oversight/measures").json(&measure(Some("overcommitted_pools"), "pools", 0, 1)).await.json();
        let pools_pid = pools["pid"].as_str().unwrap().to_string();
        // A derived measure takes no entered reading.
        assert_eq!(
            request.put(&format!("/api/oversight/measures/{turnover_pid}/readings"))
                .json(&json!({ "period_on": month(1).to_string(), "value": 1, "evidence": "x", "source_document": "y" })).await.status_code(),
            422
        );
        let find = |v: &Value, pid: &str| v["criteria"][0]["measures"].as_array().unwrap().iter().find(|m| m["measure_pid"] == pid).unwrap().clone();
        let first = status().await;
        let t = find(&first, &turnover_pid);
        let values: Vec<i64> = t["derived_values"].as_array().unwrap().iter().map(|v| v["value"].as_i64().unwrap()).collect();
        assert_eq!(values, [0, 513, 0], "oldest first; the leaver is counted in their month only");
        assert_eq!(t["status"], "met");
        assert_eq!(t["reported_periods"], 0);
        // The share is 200 of 2000 FTE = 1000 basis points, at the threshold, which meets an at-most.
        let c = find(&first, &contingent_pid);
        assert_eq!(c["derived_values"][0]["value"], 1000);
        assert_eq!(c["status"], "met");
        // The month before has no snapshot: null, which is never a pass, so two months running is not met.
        let e = find(&first, &earlier_contingent);
        assert_eq!(e["derived_values"][0]["value"], Value::Null);
        assert_eq!(e["status"], "unknown");
        assert_eq!(e["stopped_by"]["reason"], "null_reading");
        // No pools: nothing to count, so unknown, not zero.
        let p = find(&first, &pools_pid);
        assert_eq!(p["derived_values"][0]["value"], Value::Null);
        assert_eq!(p["status"], "unknown");
        assert_eq!(first["criteria"][0]["status"], "unknown");
        // A pool short of an agreed claim last month makes one over-committed pool-month.
        let pool = make_pool(&request, &org, "Synthetic pool", 0).await;
        let programme = a_programme();
        request.put("/api/programme-demands").json(&json!({ "programme_ref": programme, "pool_pid": pool, "month": month(1).to_string(), "fte_centi": 100 })).await.assert_status_ok();
        request.post("/api/capacity/start-decisions").json(&json!({ "organization_ref": org, "programme_ref": programme, "decision": "proceed", "reason": "Synthetic: the shortfall is accepted." })).await.assert_status_ok();
        let over = status().await;
        let p = find(&over, &pools_pid);
        assert_eq!(p["derived_values"][0]["value"], 1);
        assert_eq!(p["status"], "not_met", "one pool over is above an at-most of none");
        // Derived measures are read, never stored: the readings table holds nothing for them.
        use sea_orm::{ColumnTrait, PaginatorTrait, QueryFilter};
        use workforce_planning_management::models::_entities::oversight_readings;
        let pids: Vec<uuid::Uuid> = [&turnover_pid, &contingent_pid, &pools_pid].iter().map(|p| p.parse().unwrap()).collect();
        assert_eq!(oversight_readings::Entity::find().filter(oversight_readings::Column::MeasurePid.is_in(pids)).count(&ctx.db).await.unwrap(), 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Early warning (WPM-R68, WPM-T115): a pool over-committed and a measure in breach each open one
// warning, dated by the first sweep that saw it, tell the HR administrators without a figure, stay one
// warning however often they are swept, close when they stop, and the time to report is the longest
// time to acknowledge, unknown while any is open.
async fn early_warnings_open_once_tell_hr_without_figures_and_measure_time_to_report() {
    use chrono::{Datelike, Days, Months, Utc};
    use sea_orm::ConnectionTrait;
    use workforce_planning_management::controllers::early_warning::sweep;
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let today = Utc::now().date_naive();
        let this_month = today.with_day(1).unwrap();
        let last_month = this_month - Months::new(1);
        let hr: Value = request.post("/api/workers").json(&json!({
            "person_ref": super::a_person(), "organization_ref": org, "worker_number": "EW-HR",
            "display_name": "Synthetic HR", "employment_type": "permanent", "department": "people",
            "job_title": "Administrator", "hired_on": "2026-01-05" })).await.json();
        let hr = hr["pid"].as_str().unwrap().to_string();
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
             VALUES ('{}', 'person:{}', '{org}', '{hr}', 'hr_admin', '2023-01-05')", uuid::Uuid::new_v4(), uuid::Uuid::new_v4())).await.unwrap();
        // A measure that breaches last month (value 900 against an at-most of 800), on the level the organization is at.
        let level: Value = request.post("/api/oversight/levels").json(&json!({ "organization_ref": org, "ordinal": 1, "name": "Synthetic level" })).await.json();
        let level = level["pid"].as_str().unwrap().to_string();
        request.post("/api/oversight/positions").json(&json!({ "organization_ref": org, "level_pid": level, "effective_on": "2026-01-01", "deciding_body": "Synthetic panel", "reason": "Synthetic review" })).await.assert_status_ok();
        let criterion: Value = request.post("/api/oversight/criteria").json(&json!({ "level_pid": level, "name": "Synthetic criterion" })).await.json();
        let criterion = criterion["pid"].as_str().unwrap().to_string();
        let measure: Value = request.post("/api/oversight/measures").json(&json!({ "criterion_pid": criterion, "name": "Synthetic vacancy rate", "origin": "reported", "unit": "basis points", "threshold": 800, "direction": "at_most", "sustain": 1 })).await.json();
        let measure = measure["pid"].as_str().unwrap().to_string();
        let reading = |value: i64| request.put(&format!("/api/oversight/measures/{measure}/readings")).json(&json!({ "period_on": last_month.to_string(), "value": value, "evidence": "Synthetic return", "source_document": "Synthetic" }));
        reading(900).await.assert_status_ok();
        let derived: Value = request.post("/api/oversight/measures").json(&json!({ "criterion_pid": criterion, "name": "Synthetic time to report", "origin": "derived", "derivation": "time_to_report", "unit": "calendar days", "threshold": 1000, "direction": "at_most", "sustain": 1 })).await.json();
        let derived = derived["pid"].as_str().unwrap().to_string();
        // A pool with an agreed claim this month and nobody to deliver it.
        let pool = make_pool(&request, &org, "Synthetic pool", 0).await;
        let programme = a_programme();
        request.put("/api/programme-demands").json(&json!({ "programme_ref": programme, "pool_pid": pool, "month": this_month.to_string(), "fte_centi": 100 })).await.assert_status_ok();
        request.post("/api/capacity/start-decisions").json(&json!({ "organization_ref": org, "programme_ref": programme, "decision": "proceed", "reason": "Synthetic: the shortfall is accepted." })).await.assert_status_ok();

        // The first sweep opens one warning for each and tells HR; a second the same day changes nothing.
        let first = sweep(&ctx, &org, today).await.unwrap();
        assert_eq!((first.opened, first.cleared, first.told), (2, 0, 2));
        let again = sweep(&ctx, &org, today).await.unwrap();
        assert_eq!((again.opened, again.cleared, again.told), (0, 0, 0));
        let notes = request.get(&format!("/api/workers/{hr}/notifications")).await.json::<Value>();
        let text = serde_json::to_string(&notes).unwrap();
        assert_eq!(text.matches("early_warning").count(), 2);
        assert!(text.contains("Synthetic pool") && text.contains("Synthetic vacancy rate"));
        for figure in ["900", "800", "100"] {
            assert!(!text.contains(&format!("\"{figure}")) && !text.contains(&format!(" {figure} ")), "no figure {figure} in {text}");
        }
        let list = || async { request.get(&format!("/api/early-warnings?organization={org}")).await.json::<Value>() };
        let l = list().await;
        assert_eq!(l["warnings"].as_array().unwrap().len(), 2);
        assert_eq!(l["unacknowledged"], 2);
        assert_eq!(l["time_to_report_days"], Value::Null);
        let pool_warning = l["warnings"].as_array().unwrap().iter().find(|w| w["kind"] == "pool_over_committed").unwrap()["pid"].as_str().unwrap().to_string();
        let breach_warning = l["warnings"].as_array().unwrap().iter().find(|w| w["kind"] == "measure_breach").unwrap()["pid"].as_str().unwrap().to_string();
        // Time to report measured by the oversight status is unknown while a warning is open.
        let status = || async { request.get(&format!("/api/oversight/status?organization={org}")).await.json::<Value>() };
        let t = |s: &Value| s["criteria"][0]["measures"].as_array().unwrap().iter().find(|m| m["measure_pid"] == derived.as_str()).unwrap().clone();
        assert_eq!(t(&status().await)["derived_values"][0]["value"], Value::Null);
        // The pool warning was first seen last month (backdated here), the breach today.
        let seen = last_month + Days::new(2);
        ctx.db.execute_unprepared(&format!("UPDATE early_warnings SET first_true_on = '{seen}' WHERE pid = '{pool_warning}'")).await.unwrap();
        request.post(&format!("/api/early-warnings/{pool_warning}/acknowledge")).await.assert_status_ok();
        assert_eq!(request.post(&format!("/api/early-warnings/{pool_warning}/acknowledge")).await.status_code(), 422, "once only");
        assert_eq!(request.post(&format!("/api/early-warnings/{}/acknowledge", uuid::Uuid::new_v4())).await.status_code(), 404);
        // Still one unacknowledged, so the overall figure is unknown; last month's, all acknowledged, is known.
        let after_one = list().await;
        assert_eq!(after_one["time_to_report_days"], Value::Null);
        let expected = (today - seen).num_days();
        let w = after_one["warnings"].as_array().unwrap().iter().find(|w| w["pid"] == pool_warning.as_str()).unwrap().clone();
        assert_eq!(w["days_to_acknowledge"], expected);
        assert_eq!(t(&status().await)["derived_values"][0]["value"], expected);
        request.post(&format!("/api/early-warnings/{breach_warning}/acknowledge")).await.assert_status_ok();
        assert_eq!(list().await["time_to_report_days"], expected, "the longest, not the quickest");
        // The condition stops: the warning closes, and returning is a new warning, not a reopened one.
        reading(500).await.assert_status_ok();
        let cleared = sweep(&ctx, &org, today).await.unwrap();
        assert_eq!((cleared.opened, cleared.cleared), (0, 1));
        reading(900).await.assert_status_ok();
        let back = sweep(&ctx, &org, today).await.unwrap();
        assert_eq!((back.opened, back.cleared), (1, 0));
        let l = list().await;
        assert_eq!(l["warnings"].as_array().unwrap().len(), 3);
        assert_eq!(l["unacknowledged"], 1);
        assert_eq!(request.get("/api/early-warnings").await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// The monthly pack (WPM-R69, WPM-T116) freezes the position, criteria, measures and capacity view as of
// a date: a later change to the live data does not change a stored pack, a second pack shows the
// change, and the payload holds no worker.
async fn a_monthly_pack_is_frozen_and_a_later_change_shows_only_in_a_later_pack() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let level: Value = request.post("/api/oversight/levels").json(&json!({ "organization_ref": org, "ordinal": 1, "name": "Synthetic level" })).await.json();
        let level = level["pid"].as_str().unwrap().to_string();
        request.post("/api/oversight/positions").json(&json!({ "organization_ref": org, "level_pid": level, "effective_on": "2026-01-01", "deciding_body": "Synthetic panel", "reason": "Synthetic review" })).await.assert_status_ok();
        make_pool(&request, &org, "Synthetic pool one", 0).await;
        let first: Value = request.post("/api/oversight/packs").json(&json!({ "organization_ref": org })).await.json();
        let first_pid = first["pid"].as_str().unwrap().to_string();
        assert_eq!(first["pack"]["oversight"]["current"]["level"], "Synthetic level");
        let names = |v: &Value| v["pack"]["capacity"]["pools"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        assert_eq!(names(&first), ["Synthetic pool one"]);
        // The live data moves on; the stored pack does not.
        make_pool(&request, &org, "Synthetic pool two", 0).await;
        let stored: Value = request.get(&format!("/api/oversight/packs/{first_pid}")).await.json();
        assert_eq!(stored, first, "frozen");
        let second: Value = request.post("/api/oversight/packs").json(&json!({ "organization_ref": org })).await.json();
        assert_eq!(names(&second).len(), 2);
        let list: Value = request.get(&format!("/api/oversight/packs?organization={org}")).await.json();
        let pids: Vec<&str> = list["packs"].as_array().unwrap().iter().map(|p| p["pid"].as_str().unwrap()).collect();
        assert_eq!(pids, [second["pid"].as_str().unwrap(), first_pid.as_str()], "newest first");
        assert!(list["packs"][0].get("pack").is_none(), "the list carries no payload");
        assert_eq!(request.get(&format!("/api/oversight/packs/{}", uuid::Uuid::new_v4())).await.status_code(), 404);
        assert_eq!(request.get("/api/oversight/packs").await.status_code(), 422);
        let text = serde_json::to_string(&second).unwrap();
        assert!(!text.contains("worker:") && !text.contains("person:"));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// The scheduled freeze writes one pack for the month for each organization with a position, and a
// second run for the same month writes none.
async fn the_scheduled_freeze_is_idempotent_for_a_month() {
    use workforce_planning_management::tasks::oversight_pack_freeze::freeze_all;
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let level: Value = request.post("/api/oversight/levels").json(&json!({ "organization_ref": org, "ordinal": 1, "name": "Synthetic level" })).await.json();
        request.post("/api/oversight/positions").json(&json!({ "organization_ref": org, "level_pid": level["pid"], "effective_on": "2026-01-01", "deciding_body": "Synthetic body", "reason": "Synthetic reason" })).await.assert_status_ok();
        let in_org = || async { request.get(&format!("/api/oversight/packs?organization={org}")).await.json::<Value>()["packs"].as_array().unwrap().len() };
        assert_eq!(in_org().await, 0);
        let day = chrono::NaiveDate::from_ymd_opt(2026, 9, 2).unwrap();
        assert!(freeze_all(&ctx, day).await.unwrap() >= 1);
        assert_eq!(in_org().await, 1);
        freeze_all(&ctx, day).await.unwrap();
        assert_eq!(in_org().await, 1, "the same month is not frozen twice");
        freeze_all(&ctx, chrono::NaiveDate::from_ymd_opt(2026, 10, 2).unwrap()).await.unwrap();
        assert_eq!(in_org().await, 2, "the next month is");
        // The deciding body is for the status route; the pack does not carry it.
        let list: Value = request.get(&format!("/api/oversight/packs?organization={org}")).await.json();
        let one: Value = request.get(&format!("/api/oversight/packs/{}", list["packs"][0]["pid"].as_str().unwrap())).await.json();
        let text = serde_json::to_string(&one).unwrap();
        assert!(text.contains("Synthetic reason") && !text.contains("Synthetic body"));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Cutover readiness (WPM-R70, WPM-T117): trained staff per role at the site, hypercare cover and
// key-person risk, each ready, at risk or unknown with reasons; an unknown is never ready, and the
// answer names no one.
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn cutover_readiness_reports_the_people_side_and_never_reads_unknown_as_ready() {
    use super::{activate, seed_worker};
    use sea_orm::ConnectionTrait;
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let site = format!("Synthetic Site {tag}");
        let course = format!("SYN-COURSE-{tag}");
        // Four people with the role "Engineer" (the title seed_worker gives), and a role profile for it.
        let profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": "Engineer" })).await.json();
        let profile = profile["pid"].as_str().unwrap().to_string();
        let mut people = Vec::new();
        for n in 0..8 {
            let w = seed_worker!(&request, &org, format!("CO-{n}-{tag}"), None).await;
            activate!(&request, &w).await;
            people.push(w);
        }
        let (a, b, c, d) = (people[0].clone(), people[1].clone(), people[2].clone(), people[3].clone());
        let (late_done, expiring, gardener, unplaced_holder) = (people[4].clone(), people[5].clone(), people[6].clone(), people[7].clone());
        let annex = format!("Synthetic Annex {tag}");
        let set_site = |w: &str, loc: &str| {
            let db = &ctx.db;
            let sql = format!("UPDATE workers SET location = '{loc}' WHERE pid = '{w}'");
            async move { db.execute_unprepared(&sql).await.unwrap(); }
        };
        // A is recorded with different case and punctuation; B under the site's alias: both are at the site.
        set_site(&a, &format!("  {}. ", site.to_uppercase())).await;
        set_site(&b, &annex).await;
        set_site(&d, "Elsewhere").await; // trained, but at another site
        for w in [&late_done, &expiring, &gardener] {
            set_site(w, &site).await;
        }
        ctx.db.execute_unprepared(&format!("UPDATE workers SET job_title = 'Gardener' WHERE pid = '{gardener}'")).await.unwrap();
        let complete = |w: &str| {
            let db = &ctx.db;
            let sql = format!(
                "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status, completed_on) \
                 VALUES ('{}', '{w}', '{course}', 'completed', '{}')", uuid::Uuid::new_v4(), day(-5));
            async move { db.execute_unprepared(&sql).await.unwrap(); }
        };
        complete(&a).await;
        complete(&c).await; // trained, no site recorded
        complete(&d).await;
        complete(&gardener).await; // at the site and trained, but not in the role
        let enrol = |w: &str, completed: i64, expires: Option<i64>| {
            let db = &ctx.db;
            let expires = expires.map_or("NULL".to_string(), |n| format!("'{}'", day(n)));
            let sql = format!(
                "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status, completed_on, certificate_expires_on) \
                 VALUES ('{}', '{w}', '{course}', 'completed', '{}', {expires})", uuid::Uuid::new_v4(), day(completed));
            async move { db.execute_unprepared(&sql).await.unwrap(); }
        };
        enrol(&late_done, 11, None).await; // completes the day after the window opens: not trained for it
        enrol(&expiring, -5, Some(11)).await; // the certificate lapses before the window closes
        // A rota that covers the window, one that starts after it begins, and a pool with a critical skill.
        let rota_body = |name: &str, starts: String| json!({ "organization_ref": org, "name": format!("{name} {tag}"), "period_days": 7, "starts_on": starts, "members": [a, b] });
        let good_rota: Value = request.post("/api/rotas").json(&rota_body("Hypercare", day(0))).await.json();
        let late_rota: Value = request.post("/api/rotas").json(&rota_body("Late hypercare", day(12))).await.json();
        let skill_name = format!("Cutover skill {tag}");
        let skill: Value = request.post("/api/skills").json(&json!({ "name": skill_name, "category": "technical" })).await.json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        let key_profile: Value = request.post("/api/role-profiles").json(&json!({ "job_title": format!("Key role {tag}") })).await.json();
        request.put(&format!("/api/role-profiles/{}/requirements", key_profile["pid"].as_str().unwrap()))
            .json(&json!({ "skill_pid": skill, "min_proficiency": 4, "importance": "critical" })).await.assert_status_ok();
        // Another critical skill held by one person, in no listed pool: it must not count against this cutover.
        let other_skill: Value = request.post("/api/skills").json(&json!({ "name": format!("Unlisted skill {tag}"), "category": "technical" })).await.json();
        let other_skill = other_skill["pid"].as_str().unwrap().to_string();
        request.put(&format!("/api/role-profiles/{}/requirements", key_profile["pid"].as_str().unwrap()))
            .json(&json!({ "skill_pid": other_skill, "min_proficiency": 4, "importance": "critical" })).await.assert_status_ok();
        request.put(&format!("/api/workers/{a}/skills")).json(&json!({ "skill_pid": other_skill, "proficiency": 5 })).await.assert_status_ok();
        let pool = make_pool(&request, &org, &format!("Cutover pool {tag}"), 0).await;
        request.post(&format!("/api/skill-pools/{pool}/members")).json(&json!({ "skill_pid": skill, "min_proficiency": 4 })).await.assert_status_ok();

        let cutover = |rota: Option<&str>, start: i64, end: i64| json!({
            "organization_ref": org, "programme_ref": a_programme(), "name": "Synthetic go-live", "site": site,
            "window_start": day(start), "window_end": day(end), "rota_pid": rota, "site_aliases": [annex] });
        for (why, body) in [
            ("a window that ends before it starts", cutover(None, 12, 10)),
            ("a window longer than the on-call schedule can show", cutover(None, 10, 120)),
            ("a rota that is not the organization's", cutover(Some(&uuid::Uuid::new_v4().to_string()), 10, 12)),
        ] {
            assert_eq!(request.post("/api/cutovers").json(&body).await.status_code(), 422, "{why}");
        }
        let mut blank = cutover(None, 10, 12);
        blank["site"] = json!("  ");
        assert_eq!(request.post("/api/cutovers").json(&blank).await.status_code(), 422, "a blank site");
        let made: Value = request.post("/api/cutovers").json(&cutover(good_rota["pid"].as_str(), 10, 12)).await.json();
        let id = made["pid"].as_str().unwrap().to_string();
        let ready = || async { request.get(&format!("/api/cutovers/{id}/readiness")).await.json::<Value>() };

        // Nothing listed: roles and pools unknown, hypercare ready from the rota. Unknown is not ready.
        let first = ready().await;
        assert_eq!(first["checks"]["hypercare_cover"]["status"], "ready");
        assert_eq!(first["checks"]["trained_staff"]["status"], "unknown");
        assert_eq!(first["checks"]["key_person_risk"]["status"], "unknown");
        assert_eq!(first["readiness"], "unknown");
        assert!(first["basis"].as_str().unwrap().contains("human decision"));
        // The role needs two trained people at the site: A is; C is trained but has no site recorded.
        for (why, body) in [
            ("no headcount", json!({ "role_profile_pid": profile, "course_ref": course, "required_headcount": 0 })),
            ("a blank course", json!({ "role_profile_pid": profile, "course_ref": " ", "required_headcount": 2 })),
            ("a role profile that does not exist", json!({ "role_profile_pid": uuid::Uuid::new_v4(), "course_ref": course, "required_headcount": 2 })),
        ] {
            assert_eq!(request.post(&format!("/api/cutovers/{id}/roles")).json(&body).await.status_code(), 422, "{why}");
        }
        assert_eq!(
            request.post(&format!("/api/cutovers/{id}/roles")).json(&json!({ "role_profile_pid": profile, "course_ref": "NOT-A-COURSE", "required_headcount": 2 })).await.status_code(),
            422,
            "a course the learning records do not know"
        );
        let role = json!({ "role_profile_pid": profile, "course_ref": course, "required_headcount": 2 });
        request.post(&format!("/api/cutovers/{id}/roles")).json(&role).await.assert_status_ok();
        assert_eq!(request.post(&format!("/api/cutovers/{id}/roles")).json(&role).await.status_code(), 422, "listed once");
        let unplaced = ready().await;
        assert_eq!(unplaced["checks"]["trained_staff"]["status"], "unknown", "someone with no site could close the gap");
        assert!(unplaced["checks"]["trained_staff"]["reasons"][0].as_str().unwrap().contains("no site recorded"));
        // C is recorded at another site: now a known shortfall of one. D, trained elsewhere, never counted.
        set_site(&c, "Elsewhere").await;
        let short = ready().await;
        assert_eq!(short["checks"]["trained_staff"]["status"], "at_risk");
        assert!(short["checks"]["trained_staff"]["reasons"][0].as_str().unwrap().contains("short by 1"));
        // The same reason as data, so a client can say it in its own language.
        let detail = &short["checks"]["trained_staff"]["details"][0];
        assert_eq!(detail["code"], "trained_short");
        assert_eq!(detail["params"]["short"], "1");
        assert_eq!(short["readiness"], "at_risk");
        // The sweep opens one early warning for the cutover and the next finds nothing new.
        use workforce_planning_management::controllers::early_warning::sweep;
        let opened = sweep(&ctx, &org, today).await.unwrap();
        assert_eq!(opened.opened, 1);
        assert_eq!(sweep(&ctx, &org, today).await.unwrap().opened, 0);
        let warnings: Value = request.get(&format!("/api/early-warnings?organization={org}")).await.json();
        assert_eq!(warnings["warnings"][0]["kind"], "cutover_at_risk");
        assert_eq!(warnings["warnings"][0]["subject_name"], "Synthetic go-live");
        // B completes the course: two trained at the site.
        complete(&b).await;
        assert_eq!(ready().await["checks"]["trained_staff"]["status"], "ready");
        // The pool: one holder of a critical skill with no cover is a single point of failure.
        request.post(&format!("/api/cutovers/{id}/pools")).json(&json!({ "pool_pid": pool })).await.assert_status_ok();
        assert_eq!(request.post(&format!("/api/cutovers/{id}/pools")).json(&json!({ "pool_pid": pool })).await.status_code(), 422, "listed once");
        assert_eq!(request.post(&format!("/api/cutovers/{id}/pools")).json(&json!({ "pool_pid": uuid::Uuid::new_v4() })).await.status_code(), 422);
        let declare = |who: String, level: i32| {
            let (request, skill) = (&request, skill.clone());
            async move { request.put(&format!("/api/workers/{who}/skills")).json(&json!({ "skill_pid": skill, "proficiency": level })).await.assert_status_ok(); }
        };
        // Someone elsewhere holds it too, but not at the site: the site still has one holder.
        declare(d.clone(), 5).await;
        declare(a.clone(), 5).await;
        // A backup who holds the skill but works elsewhere does not cover the holder at the site.
        request.post(&format!("/api/workers/{a}/backups")).json(&json!({ "backup_pid": d })).await.assert_status_ok();
        let single = ready().await;
        assert_eq!(single["checks"]["key_person_risk"]["status"], "at_risk");
        assert!(single["checks"]["key_person_risk"]["reasons"][0].as_str().unwrap().contains(&skill_name));
        assert_eq!(single["readiness"], "at_risk");
        assert!(single["checks"]["key_person_risk"]["reasons"][0].as_str().unwrap().contains("at the site"));
        // A holder with no site on file might be there: the risk cannot be called.
        declare(unplaced_holder.clone(), 5).await;
        let maybe = ready().await;
        assert_eq!(maybe["checks"]["key_person_risk"]["status"], "unknown");
        assert!(maybe["checks"]["key_person_risk"]["reasons"][0].as_str().unwrap().contains("no site recorded"));
        assert_eq!(maybe["checks"]["key_person_risk"]["details"][0]["code"], "key_single_holder_unplaced");
        set_site(&unplaced_holder, "Elsewhere").await;
        assert_eq!(ready().await["checks"]["key_person_risk"]["status"], "at_risk");
        // A second holder at the site: shared. Every check is now ready, and so is the cutover.
        declare(b.clone(), 5).await;
        let all = ready().await;
        assert_eq!(all["checks"]["key_person_risk"]["status"], "ready");
        assert_eq!(all["readiness"], "ready");
        // The warning closes when the cutover is no longer at risk.
        let closed = sweep(&ctx, &org, today).await.unwrap();
        assert_eq!((closed.opened, closed.cleared), (0, 1));
        // A course renamed in the learning records cannot be checked, and is said so.
        ctx.db.execute_unprepared(&format!("UPDATE training_enrollments SET course_ref = 'RENAMED-{tag}' WHERE course_ref = '{course}'")).await.unwrap();
        let renamed = ready().await;
        assert_eq!(renamed["checks"]["trained_staff"]["status"], "unknown");
        assert!(renamed["checks"]["trained_staff"]["reasons"][0].as_str().unwrap().contains("not in the learning records"));
        assert_eq!(renamed["readiness"], "unknown");
        // Nobody is named anywhere in the answer.
        let text = all.to_string();
        for pid in [&a, &b, &c, &d] {
            assert!(!text.contains(pid.as_str()), "the report names no one");
        }
        // A rota that starts after the window begins leaves days with nobody on call.
        let late: Value = request.post("/api/cutovers").json(&cutover(late_rota["pid"].as_str(), 10, 12)).await.json();
        let late = request.get(&format!("/api/cutovers/{}/readiness", late["pid"].as_str().unwrap())).await.json::<Value>();
        assert_eq!(late["checks"]["hypercare_cover"]["status"], "at_risk");
        assert!(late["checks"]["hypercare_cover"]["reasons"][0].as_str().unwrap().contains("2 calendar days"));
        assert_eq!(late["checks"]["hypercare_cover"]["details"][0]["code"], "uncovered_days");
        assert_eq!(late["checks"]["hypercare_cover"]["details"][0]["params"]["count"], "2");
        // No rota linked: unknown, not ready.
        let none: Value = request.post("/api/cutovers").json(&cutover(None, 10, 12)).await.json();
        let none = request.get(&format!("/api/cutovers/{}/readiness", none["pid"].as_str().unwrap())).await.json::<Value>();
        assert_eq!(none["checks"]["hypercare_cover"]["status"], "unknown");
        // The list, then removal.
        let listed: Value = request.get(&format!("/api/cutovers?organization={org}")).await.json();
        assert_eq!(listed.as_array().unwrap().len(), 3);
        request.delete(&format!("/api/cutovers/{id}")).await.assert_status_ok();
        assert_eq!(request.get(&format!("/api/cutovers/{id}/readiness")).await.status_code(), 404);
        assert_eq!(request.get("/api/cutovers").await.status_code(), 422);
    })
    .await;
}
