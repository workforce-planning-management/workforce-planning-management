//! Whose each default is (WPM-T384): `GET /api/statutory-defaults` names, for every listed `WPM_*`
//! setting, the value in force, whether the deployer set it, the default and whose it is.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{an_org, seed_worker};

/// Record `code` (with `layer` and `parent`) in `org`, and append one `example` row of `key` at it.
async fn profile_row(
    request: &loco_rs::TestServer,
    org: &str,
    (code, layer, parent): (&str, &str, Option<&str>),
    key: &str,
    value: Value,
) {
    request
        .post("/api/jurisdictions")
        .json(&json!({ "organization_ref": org, "code": code, "layer": layer, "parent_code": parent }))
        .await
        .assert_status_ok();
    request
        .post("/api/jurisdiction-parameters")
        .json(&json!({ "organization_ref": org, "code": code, "key": key, "value": value,
                       "effective_from": "2000-01-01", "source": "synthetic test profile", "status": "example" }))
        .await
        .assert_status_ok();
}

/// Set `org`'s home jurisdiction (`null` clears it).
async fn set_home(request: &loco_rs::TestServer, org: &str, code: Value) {
    request
        .put("/api/home-jurisdiction")
        .json(&json!({ "organization_ref": org, "home_jurisdiction": code }))
        .await
        .assert_status_ok();
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn each_default_names_its_regime_and_a_setting_replaces_it() {
    crate::requests::request_open(|request, _ctx| async move {
        let read = || async {
            let body: Value = request.get("/api/statutory-defaults").await.json();
            body["settings"].as_array().unwrap().clone()
        };
        let entry = |list: &[Value], name: &str| {
            list.iter()
                .find(|s| s["setting"] == name)
                .unwrap_or_else(|| panic!("{name} listed"))
                .clone()
        };
        let list = read().await;
        // The five areas the task names, and the payroll stub's settings.
        for area in [
            "flexible_working",
            "resignation",
            "pay_transparency",
            "retention",
            "payroll",
        ] {
            assert!(list.iter().any(|s| s["area"] == area), "{area}");
        }
        // One regime's defaults name it; a starting point names none.
        let flexible = entry(&list, "WPM_FLEXIBLE_DECISION_MONTHS");
        assert_eq!(flexible["default_from"]["regime"], "GB");
        assert_eq!(flexible["default_from"]["status"], "unverified");
        assert_eq!(flexible["value"], 2);
        assert_eq!(flexible["set"], false);
        let notice = entry(&list, "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS");
        assert_eq!(notice["default_from"]["regime"], Value::Null);
        assert_eq!(notice["default_from"]["status"], "starting_point");
        assert_eq!(notice["value"], 28);
        let months = entry(&list, "WPM_PAY_INFORMATION_MONTHS");
        assert_eq!(months["default_from"]["regime"], "EU");
        // The Directive was seen only as search excerpts (WPM-T268), so it is not `read` (WPM-T404).
        assert_eq!(months["default_from"]["status"], "unverified");
        let reference = months["default_from"]["reference"].as_str().unwrap();
        assert!(reference.contains("Article 7"));
        assert!(reference.contains("search excerpts"));
        let threshold = entry(&list, "WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT");
        assert_eq!(threshold["value"], 5);
        assert_eq!(threshold["default_from"]["regime"], "EU");
        let tiers = entry(&list, "WPM_PAY_GAP_TIERS");
        assert_eq!(tiers["value"].as_array().unwrap().len(), 3);
        assert_eq!(tiers["value"][0]["from_workers"], 250);
        let pay = entry(&list, "WPM_RETENTION_PAY_DAYS");
        assert_eq!(pay["value"], 2190);
        assert_eq!(pay["default_from"]["regime"], Value::Null);
        // The AI Act's date is the EU's, from a secondary page, so unverified; the sex category is no regime's.
        let ai = entry(&list, "WPM_AI_ACT_APPLIES_FROM");
        assert_eq!(ai["area"], "ai_register");
        assert_eq!(ai["value"], "2027-12-02");
        assert_eq!(ai["default_from"]["regime"], "EU");
        assert_eq!(ai["default_from"]["status"], "unverified");
        let sex = entry(&list, "WPM_PAY_GAP_SEX_CATEGORY");
        assert_eq!(sex["value"], "sex");
        assert_eq!(sex["default_from"]["regime"], Value::Null);
        assert_eq!(sex["default_from"]["status"], "starting_point");
        let stub = entry(&list, "WPM_PAYROLL_STUB_CURRENCY");
        assert_eq!(stub["value"], "GBP");
        assert_eq!(stub["default_from"]["status"], "example");

        // A deployer's own values replace the defaults and are shown as set; a value that does not
        // check is set but not used. The tests run serially, so the process environment is ours here.
        unsafe {
            std::env::set_var("WPM_FLEXIBLE_DECISION_MONTHS", "3");
            std::env::set_var("WPM_PAY_GAP_TIERS", "none");
            std::env::set_var("WPM_RESIGNATION_NOTICE_CALENDAR_DAYS", "junk");
            std::env::set_var("WPM_AI_ACT_APPLIES_FROM", "2028-08-02");
            std::env::set_var("WPM_PAY_GAP_SEX_CATEGORY", "gender");
        }
        let list = read().await;
        unsafe {
            std::env::remove_var("WPM_FLEXIBLE_DECISION_MONTHS");
            std::env::remove_var("WPM_PAY_GAP_TIERS");
            std::env::remove_var("WPM_RESIGNATION_NOTICE_CALENDAR_DAYS");
            std::env::remove_var("WPM_AI_ACT_APPLIES_FROM");
            std::env::remove_var("WPM_PAY_GAP_SEX_CATEGORY");
        }
        let flexible = entry(&list, "WPM_FLEXIBLE_DECISION_MONTHS");
        assert_eq!(flexible["value"], 3);
        assert_eq!(flexible["set"], true);
        assert_eq!(flexible["default"], "2");
        let tiers = entry(&list, "WPM_PAY_GAP_TIERS");
        assert_eq!(tiers["value"], serde_json::json!([]), "no reporting duty");
        assert_eq!(tiers["set"], true);
        let notice = entry(&list, "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS");
        assert_eq!(notice["value"], 28, "a junk value is not used");
        assert_eq!(notice["set"], true);
        let ai = entry(&list, "WPM_AI_ACT_APPLIES_FROM");
        assert_eq!(ai["value"], "2028-08-02");
        assert_eq!(ai["set"], true);
        assert_eq!(entry(&list, "WPM_PAY_GAP_SEX_CATEGORY")["value"], "gender");
        // And back to the defaults.
        let list = read().await;
        assert_eq!(entry(&list, "WPM_FLEXIBLE_DECISION_MONTHS")["set"], false);
    })
    .await;
}

/// A reader about one worker (WPM-D153): the refusal reasons a flexible working request may be refused for come from that
/// worker's chain first, then `WPM_FLEXIBLE_REFUSAL_REASONS`, then the defaults; never from the organization's home.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one journey, read top to bottom
async fn a_worker_reader_takes_the_workers_profile_then_the_setting_then_the_default() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let key = "flexible_working.refusal_reasons";
        profile_row(&request, &org, ("QZ", "supranational", None), "pay_transparency.reporting_tiers", json!(["none"])).await;
        profile_row(&request, &org, ("XA", "national", Some("QZ")), key, json!(["xa_reason"])).await;
        profile_row(&request, &org, ("XB", "national", None), key, json!(["home_reason"])).await;
        profile_row(&request, &org, ("XB", "national", None), "pay_transparency.reporting_tiers", json!(["50:2:2030-01-01"])).await;
        set_home(&request, &org, json!("XB")).await;
        // A row the reader could not use is refused when it is written, not ignored later.
        for (key, value) in [(key, json!(["Not A Token"])), ("pay_transparency.reporting_tiers", json!(["none", "50:2:2030-01-01"]))] {
            let r = request
                .post("/api/jurisdiction-parameters")
                .json(&json!({ "organization_ref": org, "code": "XA", "key": key, "value": value,
                               "effective_from": "2000-01-01", "source": "synthetic", "status": "example" }))
                .await;
            assert_eq!(r.status_code(), 422, "{key}: {}", r.text());
        }
        let placed = seed_worker!(&request, &org, "SD-AT", None).await;
        let nowhere = seed_worker!(&request, &org, "SD-NO", None).await;
        request.put(&format!("/api/workers/{placed}")).json(&json!({ "work_jurisdiction": "XA" })).await.assert_status_ok();
        // A request waiting for a decision, for each case (made by the worker through /api/me, which this suite cannot sign in to).
        let waiting = |worker: String| {
            let ctx = &ctx;
            async move {
                let pid = uuid::Uuid::new_v4();
                ctx.db
                    .execute_unprepared(&format!(
                        "INSERT INTO flexible_working_requests (pid, worker_pid, kind, requested_on, proposed_start, decide_by) \
                         VALUES ('{pid}', '{worker}', 'hours', CURRENT_DATE, CURRENT_DATE + 30, CURRENT_DATE + 60)"
                    ))
                    .await
                    .unwrap();
                pid.to_string()
            }
        };
        let refuse = |pid: String, reason: &'static str| {
            let request = &request;
            async move {
                request
                    .post(&format!("/api/flexible-working/{pid}/decide"))
                    .json(&json!({ "outcome": "refuse", "reason": reason }))
                    .await
                    .status_code()
                    .as_u16()
            }
        };
        let (at, none_set, none_default) = (waiting(placed.clone()).await, waiting(nowhere.clone()).await, waiting(nowhere.clone()).await);

        unsafe { std::env::set_var("WPM_FLEXIBLE_REFUSAL_REASONS", "env_reason") };
        let profile_over_setting = refuse(at.clone(), "env_reason").await;
        let profile_used = refuse(at, "xa_reason").await;
        let not_the_home = refuse(none_set.clone(), "home_reason").await;
        let setting_used = refuse(none_set, "env_reason").await;
        let listed: Value = request.get(&format!("/api/statutory-defaults?organization={org}")).await.json();
        unsafe { std::env::remove_var("WPM_FLEXIBLE_REFUSAL_REASONS") };
        assert_eq!(profile_over_setting, 422, "the worker's profile wins over the setting");
        assert_eq!(profile_used, 200);
        assert_eq!(not_the_home, 422, "a worker reader never takes the organization's home");
        assert_eq!(setting_used, 200, "no place of work: the setting");
        assert_eq!(refuse(none_default.clone(), "env_reason").await, 422, "neither: the defaults");
        assert_eq!(refuse(none_default, "additional_cost").await, 200);

        // The list for the organization says where each value came from.
        let entry = |name: &str| listed["settings"].as_array().unwrap().iter().find(|s| s["setting"] == name).unwrap().clone();
        assert_eq!(listed["home_jurisdiction"], "XB");
        let reasons = entry("WPM_FLEXIBLE_REFUSAL_REASONS");
        assert_eq!(reasons["reader"], "worker");
        assert_eq!(reasons["key"], key);
        assert_eq!(reasons["value"], json!(["env_reason"]), "a worker with no place of work gets the setting");
        assert_eq!(reasons["came_from"]["kind"], "setting");
        let place = |code: &str| reasons["by_place"].as_array().unwrap().iter().find(|p| p["code"] == code).unwrap().clone();
        assert_eq!(place("XA")["value"], json!(["xa_reason"]));
        assert_eq!(place("XA")["came_from"]["kind"], "profile");
        assert_eq!(place("XA")["came_from"]["from"][0]["code"], "XA");
        assert_eq!(place("XB")["value"], json!(["home_reason"]));
        assert_eq!(place("QZ")["came_from"]["kind"], "setting", "no row at the bloc: the setting");
        assert_eq!(place("QZ")["came_from"]["profile"]["resolution"]["reason"], "not_configured");
        // An organization reader resolves over the home's chain: XB's tier, not the bloc's `none` (XB is not under QZ).
        let tiers = entry("WPM_PAY_GAP_TIERS");
        assert_eq!(tiers["reader"], "organization");
        assert_eq!(tiers["value"][0]["from_workers"], 50);
        assert_eq!(tiers["came_from"]["kind"], "profile");
        assert_eq!(tiers["came_from"]["from"][0]["code"], "XB");
        assert_eq!(tiers.get("by_place"), None);
        // A setting that has not moved names no key and no reader.
        let stub = entry("WPM_PAYROLL_STUB_CURRENCY");
        assert_eq!(stub["key"], Value::Null);
        assert_eq!(stub["came_from"]["kind"], "default");
        // Without an organization, the deployment's: no profile is looked at.
        let plain: Value = request.get("/api/statutory-defaults").await.json();
        let tiers = plain["settings"].as_array().unwrap().iter().find(|s| s["setting"] == "WPM_PAY_GAP_TIERS").unwrap().clone();
        assert_eq!(tiers["came_from"]["kind"], "default");
        assert_eq!(tiers["came_from"].get("profile"), None);
        assert_eq!(request.get("/api/statutory-defaults?organization=nope").await.status_code(), 422);
    })
    .await;
}

/// A reader about the organization as a whole (WPM-D153): whether a vacancy must state its pay range comes from the
/// organization's home jurisdiction's chain first, then `WPM_REQUIRE_PAY_RANGE`, then the default (on); the gap report and the
/// retention schedule say where theirs came from.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn an_organization_reader_takes_the_home_profile_then_the_setting_then_the_default() {
    crate::requests::request_open(|request, _ctx| async move {
        let open_without_a_range = |org: String| {
            let request = &request;
            async move {
                let made: Value = request
                    .post("/api/requisitions")
                    .json(&json!({ "organization_ref": org, "department": "engineering", "job_title": "Engineer", "headcount": 1 }))
                    .await
                    .json();
                let pid = made["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/requisitions/{pid}/status")).json(&json!({ "to": "open" })).await.status_code().as_u16()
            }
        };
        let key = "pay_transparency.pay_range_required";
        // Required at the home: the profile wins over a setting that turns it off.
        let strict = an_org();
        profile_row(&request, &strict, ("XA", "national", None), key, json!(true)).await;
        set_home(&request, &strict, json!("XA")).await;
        // Not required at the home, with no setting: the profile wins over the default.
        let relaxed = an_org();
        profile_row(&request, &relaxed, ("XB", "national", None), key, json!(false)).await;
        set_home(&request, &relaxed, json!("XB")).await;
        // A row at a code that is not the home is not the organization's: no home, so the setting or the default.
        let homeless = an_org();
        profile_row(&request, &homeless, ("XC", "national", None), key, json!(false)).await;

        assert_eq!(open_without_a_range(relaxed.clone()).await, 200, "the home's profile: not required");
        assert_eq!(open_without_a_range(homeless.clone()).await, 422, "no home, no setting: the default, required");
        unsafe { std::env::set_var("WPM_REQUIRE_PAY_RANGE", "off") };
        let strict_with_setting = open_without_a_range(strict.clone()).await;
        let homeless_with_setting = open_without_a_range(homeless.clone()).await;
        unsafe { std::env::remove_var("WPM_REQUIRE_PAY_RANGE") };
        assert_eq!(strict_with_setting, 422, "the home's profile wins over the setting");
        assert_eq!(homeless_with_setting, 200, "no home: the setting");

        // The retention schedule for an organization: its home's range (the floor), else the deployment's.
        profile_row(&request, &relaxed, ("XB", "national", None), "retention.planning.days", json!({ "min": 40, "max": 90 })).await;
        let schedule: Value = request.get(&format!("/api/retention/schedule?organization={relaxed}")).await.json();
        let kind = |name: &str| schedule["kinds"].as_array().unwrap().iter().find(|k| k["kind"] == name).unwrap().clone();
        assert_eq!(kind("planning")["days"], 40);
        assert_eq!(kind("planning")["source"], "profile");
        assert_eq!(kind("planning")["came_from"]["from"][0]["code"], "XB");
        assert_eq!(kind("pay")["came_from"]["kind"], "default");
        // A profile shorter than the deployment's never makes the sweep delete sooner.
        let report: Value = request.get("/api/retention").await.json();
        assert!(report["horizons"]["planning"].as_i64().unwrap() >= 365, "{report}");
        // A longer one does make it keep longer, and the report says whose it is. (Cleared at once below: a home left set with a
        // long row would change every later test's horizons in the shared database.)
        profile_row(&request, &strict, ("XA", "national", None), "retention.planning.days", json!({ "min": 36_000 })).await;
        let report: Value = request.get("/api/retention").await.json();
        set_home(&request, &strict, Value::Null).await;
        assert_eq!(report["horizons"]["planning"], 36_000, "{report}");
        assert_eq!(report["horizons_from"]["planning"]["organization_ref"], strict.as_str());
        assert_eq!(report["horizons_from"]["planning"]["kind"], "profile");
        // The homes are cleared so no later test's sweep reads them.
        for org in [&strict, &relaxed] {
            set_home(&request, org, Value::Null).await;
        }
    })
    .await;
}
