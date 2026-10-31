//! **Whose each default is** (WPM-T384): `GET /api/statutory-defaults` lists every `WPM_*` setting
//! whose default is, or might be read as, one regime's value, with the value in force, whether the
//! deployer set it, the default and whose the default is ([`crate::statutory_defaults`]).
//! Configuration, not personal data: any signed-in caller may read it, so a worker can see whose rule
//! dates their own request.

use loco_rs::prelude::*;
use serde_json::{Value, json};

use crate::statutory_defaults::{DEFAULTS, find};

/// Whether the deployer set `setting` (present and not blank; a value that does not check is still
/// "set", and the value in force is then the default, with a warning in the log).
fn is_set(setting: &str) -> bool {
    crate::compat::env_var(setting).is_some_and(|v| !v.trim().is_empty())
}

/// The value in force for one listed setting, read the way its controller reads it.
fn value_of(setting: &str) -> Value {
    use super::{flexible_working, pay_assessment, pay_information, pay_transparency, payroll};
    match setting {
        "WPM_AI_ACT_APPLIES_FROM" => json!(super::ai_register::applies_from()),
        "WPM_PAY_GAP_SEX_CATEGORY" => json!(pay_transparency::sex_category()),
        "WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS" => json!(flexible_working::appeal_calendar_days()),
        "WPM_FLEXIBLE_DECISION_MONTHS" => json!(flexible_working::decision_months()),
        "WPM_FLEXIBLE_REFUSAL_REASONS" => json!(flexible_working::refusal_reasons()),
        "WPM_FLEXIBLE_REQUESTS_PER_YEAR" => json!(flexible_working::requests_per_year()),
        "WPM_JOINT_ASSESSMENT_MONTHS" => json!(pay_assessment::configured().remedy_months),
        "WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT" => {
            json!(pay_assessment::configured().threshold_bp / 100)
        }
        "WPM_PAY_GAP_TIERS" => json!(pay_transparency::reporting_tiers()),
        "WPM_PAY_INFORMATION_MONTHS" => json!(pay_information::answer_months()),
        "WPM_REQUIRE_PAY_RANGE" => json!(pay_information::pay_range_required()),
        "WPM_CONTRACTED_DAY_MINUTES" => json!(payroll::contracted_day()),
        "WPM_PAYROLL_MONTH_MINUTES" => json!(payroll::tax_stub().0.month_minutes),
        "WPM_PAYROLL_STUB_ALLOWANCE_MINOR" => json!(payroll::tax_stub().0.allowance_minor),
        "WPM_PAYROLL_STUB_CURRENCY" => json!(payroll::tax_stub().0.currency),
        "WPM_PAYROLL_STUB_RATE_PERCENT" => json!(payroll::tax_stub().0.rate_percent),
        "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS" => json!(super::resignations::notice_days()),
        other => super::privacy::horizons()
            .into_iter()
            .find(|(kind, _)| kind.env_name() == other)
            .map_or(Value::Null, |(_, horizon)| json!(horizon.days)),
    }
}

/// `{setting, set, default_from}` for an answer that uses one listed setting, so the answer itself
/// says whose its number is. `Null` for a setting the table does not list.
#[must_use]
pub fn basis(setting: &str) -> Value {
    find(setting).map_or(
        Value::Null,
        |d| json!({ "setting": d.setting, "set": is_set(d.setting), "default_from": d.origin }),
    )
}

/// `GET /api/statutory-defaults` — each listed setting, its value in force, whether it was set, its
/// default and whose that default is.
#[debug_handler]
async fn list() -> Result<Response> {
    let settings: Vec<Value> = DEFAULTS
        .iter()
        .map(|d| {
            json!({
                "setting": d.setting,
                "area": d.area,
                "value": value_of(d.setting),
                "set": is_set(d.setting),
                "default": d.default,
                "default_from": d.origin,
            })
        })
        .collect();
    format::json(json!({
        "settings": settings,
        "derivation": "Each setting's value in force is read as its feature reads it: the deployer's value where it is set \
                       and checks, else the default. default_from says whose the default is: a jurisdiction code (EU, GB) \
                       and the source, read or unverified, an example in a regime's shape, or a starting point that is no \
                       regime's. A default is a starting point for the deployer to confirm, never a statement that the \
                       deployment is under that regime.",
    }))
}

/// Routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/statutory-defaults", get(list))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules;

    /// The listed defaults are the code's own defaults, so the table cannot drift from the rules.
    #[test]
    fn the_listed_defaults_are_the_codes_defaults() {
        let text = |s: &str| find(s).unwrap().default.to_string();
        let fw = rules::flexible_working::DEFAULT_DECISION_MONTHS;
        assert_eq!(text("WPM_FLEXIBLE_DECISION_MONTHS"), fw.to_string());
        let fw = rules::flexible_working::DEFAULT_REQUESTS_PER_YEAR;
        assert_eq!(text("WPM_FLEXIBLE_REQUESTS_PER_YEAR"), fw.to_string());
        let fw = rules::flexible_working::DEFAULT_APPEAL_CALENDAR_DAYS;
        assert_eq!(text("WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS"), fw.to_string());
        assert_eq!(
            text("WPM_RESIGNATION_NOTICE_CALENDAR_DAYS"),
            rules::resignation::DEFAULT_NOTICE_CALENDAR_DAYS.to_string()
        );
        assert_eq!(
            text("WPM_PAY_INFORMATION_MONTHS"),
            rules::pay_information::DEFAULT_ANSWER_MONTHS.to_string()
        );
        assert_eq!(
            text("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT"),
            (rules::joint_assessment::DEFAULT_THRESHOLD_BP / 100).to_string()
        );
        assert_eq!(
            text("WPM_JOINT_ASSESSMENT_MONTHS"),
            rules::joint_assessment::DEFAULT_REMEDY_MONTHS.to_string()
        );
        assert_eq!(
            rules::pay_gap::parse_tiers(&text("WPM_PAY_GAP_TIERS")).unwrap(),
            rules::pay_gap::directive_tiers()
        );
        assert_eq!(
            text("WPM_AI_ACT_APPLIES_FROM"),
            rules::ai_register::default_applies_from().to_string()
        );
        assert_eq!(
            text("WPM_PAY_GAP_SEX_CATEGORY"),
            crate::controllers::pay_transparency::DEFAULT_SEX_CATEGORY
        );
        let stub = rules::payroll::TaxStub::example();
        assert_eq!(text("WPM_PAYROLL_STUB_CURRENCY"), stub.currency);
        assert_eq!(
            text("WPM_PAYROLL_STUB_ALLOWANCE_MINOR"),
            stub.allowance_minor.to_string()
        );
        assert_eq!(
            text("WPM_PAYROLL_STUB_RATE_PERCENT"),
            stub.rate_percent.to_string()
        );
        assert_eq!(
            text("WPM_PAYROLL_MONTH_MINUTES"),
            stub.month_minutes.to_string()
        );
        assert_eq!(
            text("WPM_CONTRACTED_DAY_MINUTES"),
            rules::workforce::CONTRACTED_DAY_MINUTES.to_string()
        );
        // Every retention kind is listed with its own default.
        for kind in rules::privacy::RecordKind::ALL {
            assert_eq!(
                text(&kind.env_name()),
                kind.default_days().to_string(),
                "{kind:?}"
            );
        }
        // Nine refusal reasons: the eight of the regime the entry names, and other_recorded.
        assert_eq!(rules::flexible_working::DEFAULT_REFUSAL_REASONS.len(), 9);
    }

    /// Sorted by area then setting, no duplicate, every entry a `WPM_` name with a known status, and a
    /// regime only where the status is not a starting point.
    #[test]
    fn the_table_is_sorted_unique_and_says_whose() {
        let keys: Vec<(&str, &str)> = DEFAULTS.iter().map(|d| (d.area, d.setting)).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted);
        for d in DEFAULTS {
            assert!(d.setting.starts_with("WPM_"), "{}", d.setting);
            assert!(
                ["read", "unverified", "example", "starting_point"].contains(&d.origin.status),
                "{}",
                d.setting
            );
            assert_eq!(
                d.origin.regime.is_none(),
                d.origin.status == "starting_point",
                "{}: a regime is named exactly when the default is one regime's",
                d.setting
            );
            assert_ne!(d.origin.reference, "");
            // The value in force is readable for every listed setting.
            assert_ne!(value_of(d.setting), Value::Null, "{}", d.setting);
        }
        assert_eq!(basis("WPM_NOT_LISTED"), Value::Null);
        assert_eq!(basis("WPM_PAY_GAP_TIERS")["default_from"]["regime"], "EU");
    }
}
