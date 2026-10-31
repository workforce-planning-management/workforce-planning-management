//! Whose each default is (WPM-T384): `GET /api/statutory-defaults` names, for every listed `WPM_*`
//! setting, the value in force, whether the deployer set it, the default and whose it is.

use serde_json::Value;
use serial_test::serial;

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
        assert_eq!(months["default_from"]["status"], "read");
        assert!(
            months["default_from"]["reference"]
                .as_str()
                .unwrap()
                .contains("Article 7")
        );
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
