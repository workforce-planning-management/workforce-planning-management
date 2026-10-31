//! **Whose each default is** (WPM-T384): `GET /api/statutory-defaults` lists every `WPM_*` setting
//! whose default is, or might be read as, one regime's value, with the value in force, whether the
//! deployer set it, the default and whose the default is ([`crate::statutory_defaults`]).
//! Configuration, not personal data: any signed-in caller may read it, so a worker can see whose rule
//! dates their own request.
//!
//! **Profile, then setting, then default** (WPM-D153): [`pick`] is the one order every reader of a
//! moved setting follows. With `?organization=` the list says, for that organization, where each value
//! came from: its profile (the code, layer, status and source), the `WPM_*` setting, or the default.

use std::collections::BTreeSet;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};

use super::jurisdictions::{Reader, Scope};
use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid};
use crate::auth::MaybeAuthUser;
use crate::rules::jurisdiction::{Resolution, Value as ProfileValue};
use crate::statutory_defaults::{DEFAULTS, find};

/// A value in force and where it came from (WPM-D153).
#[derive(Clone, Debug, PartialEq)]
pub struct InForce<T> {
    /// The value.
    pub value: T,
    /// `{"kind": "profile", …}` with the resolved origins, `{"kind": "setting", "setting": …}` or
    /// `{"kind": "default", "setting": …, "default_from": …}`; when a profile was looked at and gave
    /// nothing usable, `profile` says why (unknown and its reason, or a conflict).
    pub came_from: Value,
}

/// Whether the deployer set `setting` (present and not blank; a value that does not check is still
/// "set", and the value in force is then the default, with a warning in the log).
pub(crate) fn is_set(setting: &str) -> bool {
    crate::compat::env_var(setting).is_some_and(|v| !v.trim().is_empty())
}

/// The value of `setting` in force: the profile's over `scope`'s chain if it resolves and
/// `from_profile` accepts it, else `from_setting` (the deployer's `WPM_*` value that checked), else
/// `default`. `scope` is `None` for a read with no organization, which has no profile to look at.
pub fn pick<T>(
    scope: Option<&Scope>,
    setting: &str,
    as_of: NaiveDate,
    from_profile: impl Fn(&ProfileValue) -> Option<T>,
    from_setting: Option<T>,
    default: T,
) -> InForce<T> {
    let entry = find(setting);
    let mut looked = Value::Null;
    if let (Some(scope), Some(key)) = (scope, entry.and_then(|d| d.key)) {
        let resolution = scope.resolve(key, as_of);
        if let Resolution::Resolved {
            value,
            status,
            from,
        } = &resolution
        {
            if let Some(found) = from_profile(value) {
                return InForce {
                    value: found,
                    came_from: json!({
                        "kind": "profile", "key": key, "reader": scope.reader, "start": scope.start,
                        "status": status, "from": from,
                    }),
                };
            }
            tracing::warn!(
                "{key}: the profile's value is not one {setting}'s reader can use; falling back"
            );
        }
        looked = json!({
            "key": key, "reader": scope.reader, "start": scope.start, "resolution": resolution,
            "used": false,
        });
    }
    let (value, mut came_from) = match from_setting {
        Some(value) => (value, json!({ "kind": "setting", "setting": setting })),
        None => (
            default,
            json!({ "kind": "default", "setting": setting, "default_from": entry.map(|d| d.origin) }),
        ),
    };
    if !looked.is_null() {
        came_from["profile"] = looked;
    }
    InForce { value, came_from }
}

/// [`pick`] for a whole number in the reader's type.
pub fn integer<T: TryFrom<i64>>(
    scope: Option<&Scope>,
    setting: &str,
    as_of: NaiveDate,
    from_setting: Option<T>,
    default: T,
) -> InForce<T> {
    pick(
        scope,
        setting,
        as_of,
        |value| value.as_integer().and_then(|n| T::try_from(n).ok()),
        from_setting,
        default,
    )
}

/// The value in force for one listed setting over `scope`, read the way its controller reads it,
/// with where it came from.
fn value_of(setting: &str, scope: Option<&Scope>, as_of: NaiveDate) -> (Value, Value) {
    use super::{flexible_working, pay_assessment, pay_information, pay_transparency, payroll};
    fn shown<T: serde::Serialize>(found: InForce<T>) -> (Value, Value) {
        (json!(found.value), found.came_from)
    }
    // A setting that has not moved into a profile is one value per deployment: the setting if set, else the default.
    let fixed = |value: Value| {
        let came_from = if is_set(setting) {
            json!({ "kind": "setting", "setting": setting })
        } else {
            json!({ "kind": "default", "setting": setting, "default_from": find(setting).map(|d| d.origin) })
        };
        (value, came_from)
    };
    match setting {
        "WPM_AI_ACT_APPLIES_FROM" => fixed(json!(super::ai_register::applies_from())),
        "WPM_PAY_GAP_SEX_CATEGORY" => fixed(json!(pay_transparency::sex_category())),
        "WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS" => {
            shown(flexible_working::appeal_calendar_days(scope, as_of))
        }
        "WPM_FLEXIBLE_DECISION_MONTHS" => shown(flexible_working::decision_months(scope, as_of)),
        "WPM_FLEXIBLE_REFUSAL_REASONS" => shown(flexible_working::refusal_reasons(scope, as_of)),
        "WPM_FLEXIBLE_REQUESTS_PER_YEAR" => {
            shown(flexible_working::requests_per_year(scope, as_of))
        }
        "WPM_JOINT_ASSESSMENT_MONTHS" => {
            let c = pay_assessment::configured(scope, as_of);
            (json!(c.config.remedy_months), c.months_from)
        }
        "WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT" => {
            let c = pay_assessment::configured(scope, as_of);
            (json!(c.config.threshold_bp / 100), c.threshold_from)
        }
        "WPM_PAY_GAP_TIERS" => shown(pay_transparency::reporting_tiers(scope, as_of)),
        "WPM_PAY_INFORMATION_MONTHS" => shown(pay_information::answer_months(scope, as_of)),
        "WPM_REQUIRE_PAY_RANGE" => shown(pay_information::pay_range_required(scope, as_of)),
        "WPM_CONTRACTED_DAY_MINUTES" => fixed(json!(payroll::contracted_day())),
        "WPM_PAYROLL_MONTH_MINUTES" => fixed(json!(payroll::tax_stub().0.month_minutes)),
        "WPM_PAYROLL_STUB_ALLOWANCE_MINOR" => fixed(json!(payroll::tax_stub().0.allowance_minor)),
        "WPM_PAYROLL_STUB_CURRENCY" => fixed(json!(payroll::tax_stub().0.currency)),
        "WPM_PAYROLL_STUB_RATE_PERCENT" => fixed(json!(payroll::tax_stub().0.rate_percent)),
        "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS" => {
            shown(super::resignations::notice_days(scope, as_of))
        }
        other => super::privacy::horizons(scope, as_of)
            .into_iter()
            .find(|(kind, _)| kind.env_name() == other)
            .map_or((Value::Null, Value::Null), |(_, found)| {
                (json!(found.value.days), found.came_from)
            }),
    }
}

/// `{setting, set, default_from, came_from}` for an answer that uses one listed setting, so the
/// answer itself says whose its number is and where it came from. `Null` for a setting the table
/// does not list.
#[must_use]
pub fn basis(setting: &str, came_from: &Value) -> Value {
    find(setting).map_or(Value::Null, |d| {
        json!({ "setting": d.setting, "set": is_set(d.setting), "default_from": d.origin, "came_from": came_from })
    })
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: Option<String>,
}

/// `GET /api/statutory-defaults?organization=` — each listed setting, its value in force, whether it
/// was set, its default, whose that default is, and where the value in force came from. Without an
/// organization, the deployment's: the setting or the default. With one: an organization-level
/// setting over its home jurisdiction's chain; a worker-level setting as a worker with no place of
/// work recorded gets it, and `by_place` for each jurisdiction code the organization keeps.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    let as_of = Utc::now().date_naive();
    let organization = match &q.organization {
        Some(org) => {
            ensure_valid(&organization_problems(org))?;
            ensure_org_in_scope(&ctx, &caller, org).await?;
            Some(Scope::for_organization(&ctx.db, org).await?)
        }
        None => None,
    };
    // The places a worker of the organization may be recorded at: the codes it keeps and the codes it has rows for.
    let places: Vec<String> = organization.as_ref().map_or_else(Vec::new, |scope| {
        scope
            .profile
            .links
            .iter()
            .map(|link| link.code.clone())
            .chain(scope.profile.rows.iter().map(|row| row.code.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    });
    let settings: Vec<Value> = DEFAULTS
        .iter()
        .map(|d| {
            let scope = match d.reader {
                Some(Reader::Organization) => organization.as_ref(),
                _ => None,
            };
            let (value, came_from) = value_of(d.setting, scope, as_of);
            let mut row = json!({
                "setting": d.setting,
                "area": d.area,
                "key": d.key,
                "reader": d.reader,
                "value": value,
                "set": is_set(d.setting),
                "default": d.default,
                "default_from": d.origin,
                "came_from": came_from,
            });
            if let (Some(Reader::Worker), Some(org)) = (d.reader, organization.as_ref()) {
                row["by_place"] = places
                    .iter()
                    .map(|code| {
                        let place = Scope {
                            reader: Reader::Worker,
                            start: Some(code.clone()),
                            ..org.clone()
                        };
                        let (value, came_from) = value_of(d.setting, Some(&place), as_of);
                        json!({ "code": code, "value": value, "came_from": came_from })
                    })
                    .collect();
            }
            row
        })
        .collect();
    format::json(json!({
        "organization_ref": q.organization,
        "home_jurisdiction": organization.as_ref().and_then(|scope| scope.start.clone()),
        "as_of": as_of,
        "settings": settings,
        "derivation": "Each setting's value in force is read as its feature reads it. A setting with a registry key is read \
                       from the jurisdiction profile first (an organization-level one over the organization's home \
                       jurisdiction, a worker-level one over the worker's place of work), then the deployer's value where it \
                       is set and checks, then the default; came_from says which. default_from says whose the default is: a \
                       jurisdiction code (EU, GB) and the source, read or unverified, an example in a regime's shape, or a \
                       starting point that is no regime's. A default is a starting point for the deployer to confirm, never \
                       a statement that the deployment is under that regime.",
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
    use crate::controllers::jurisdictions::Profile;
    use crate::rules;
    use crate::rules::jurisdiction::{Layer, Link, Row, Status};

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// An organization keeping `XA` under the bloc `QZ`, with one row for the decision period at `XA`.
    fn scope(reader: Reader, start: Option<&str>) -> Scope {
        let row = |code: &str, key: &str, value: Value| Row {
            code: code.into(),
            key: key.into(),
            value: rules::jurisdiction::validate(key, &value).unwrap(),
            effective_from: day(2020, 1, 1),
            effective_to: None,
            source: "synthetic".into(),
            status: Status::Example,
            seq: 1,
        };
        Scope {
            organization: "organization:test".into(),
            reader,
            start: start.map(Into::into),
            profile: Profile {
                links: vec![
                    Link {
                        code: "QZ".into(),
                        layer: Layer::Supranational,
                        parent: None,
                    },
                    Link {
                        code: "XA".into(),
                        layer: Layer::National,
                        parent: Some("QZ".into()),
                    },
                ],
                rows: vec![
                    row("XA", "flexible_working.decision_months", json!(1)),
                    row(
                        "QZ",
                        "pay_transparency.reporting_tiers",
                        json!(["50:2:2030-01-01"]),
                    ),
                ],
            },
        }
    }

    /// The one order (WPM-D153): the profile, then the setting, then the default; each says so.
    #[test]
    fn the_profile_wins_then_the_setting_then_the_default() {
        let on = day(2026, 10, 31);
        let name = "WPM_FLEXIBLE_DECISION_MONTHS";
        let at = scope(Reader::Worker, Some("XA"));
        let found = integer(Some(&at), name, on, Some(3_u32), 2);
        assert_eq!(found.value, 1, "the profile wins over the setting");
        assert_eq!(found.came_from["kind"], "profile");
        assert_eq!(found.came_from["from"][0]["code"], "XA");
        assert_eq!(found.came_from["status"], "example");
        // No place of work recorded: the setting, and the profile's reason is kept.
        let nowhere = scope(Reader::Worker, None);
        let found = integer(Some(&nowhere), name, on, Some(3_u32), 2);
        assert_eq!(found.value, 3);
        assert_eq!(found.came_from["kind"], "setting");
        assert_eq!(
            found.came_from["profile"]["resolution"]["reason"],
            "no_work_jurisdiction"
        );
        // Neither: the default, with whose it is.
        let found = integer(Some(&nowhere), name, on, None, 2_u32);
        assert_eq!(found.value, 2);
        assert_eq!(found.came_from["kind"], "default");
        assert_eq!(found.came_from["default_from"]["regime"], "GB");
        // No organization at all: no profile is looked at.
        let found = integer(None, name, on, None, 2_u32);
        assert_eq!(found.came_from.get("profile"), None);
        // A place with no row is not configured, and falls through.
        let other = scope(Reader::Worker, Some("XB"));
        let found = integer(Some(&other), name, on, None, 2_u32);
        assert_eq!(
            found.came_from["profile"]["resolution"]["reason"],
            "not_configured"
        );
    }

    /// An organization-level reader says when no home is recorded; a profile value the reader cannot use falls through.
    #[test]
    fn the_organization_chain_and_an_unusable_value() {
        let on = day(2026, 10, 31);
        let homeless = scope(Reader::Organization, None);
        let tiers = super::super::pay_transparency::reporting_tiers(Some(&homeless), on);
        assert_eq!(tiers.value, rules::pay_gap::directive_tiers());
        assert_eq!(
            tiers.came_from["profile"]["resolution"]["reason"],
            "no_home_jurisdiction"
        );
        let home = scope(Reader::Organization, Some("XA"));
        let tiers = super::super::pay_transparency::reporting_tiers(Some(&home), on);
        assert_eq!(
            tiers.value.len(),
            1,
            "the bloc's one tier, over the home's chain"
        );
        assert_eq!(tiers.came_from["from"][0]["code"], "QZ");
        let unusable = pick(
            Some(&home),
            "WPM_PAY_GAP_TIERS",
            on,
            |_| None::<u8>,
            None,
            7,
        );
        assert_eq!(unusable.value, 7);
        assert_eq!(unusable.came_from["profile"]["used"], false);
    }

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
            // The value in force is readable for every listed setting, and says where it came from.
            let (value, came_from) = value_of(d.setting, None, day(2026, 10, 31));
            assert_ne!(value, Value::Null, "{}", d.setting);
            assert!(
                ["setting", "default"].contains(&came_from["kind"].as_str().unwrap()),
                "{}",
                d.setting
            );
            // A moved setting names a registry key and a reader; one that has not moved names neither.
            assert_eq!(d.key.is_some(), d.reader.is_some(), "{}", d.setting);
            if let Some(key) = d.key {
                assert!(
                    rules::jurisdiction::parameter(key).is_some(),
                    "{key} is in the registry"
                );
            }
        }
        assert_eq!(basis("WPM_NOT_LISTED", &Value::Null), Value::Null);
        let tiers = basis("WPM_PAY_GAP_TIERS", &json!({ "kind": "default" }));
        assert_eq!(tiers["default_from"]["regime"], "EU");
        assert_eq!(tiers["came_from"]["kind"], "default");
    }

    /// The Pay Transparency Directive was seen only as search excerpts of its EUR-Lex text and the
    /// Commission's FAQ (WPM-T268), so no entry resting on it says `read`, and each says so (WPM-T404).
    #[test]
    fn the_directive_is_not_claimed_as_read() {
        let directive: Vec<_> = DEFAULTS
            .iter()
            .filter(|d| d.origin.reference.contains("2023/970"))
            .collect();
        assert_eq!(directive.len(), 5);
        for d in directive {
            assert_eq!(d.origin.status, "unverified", "{}", d.setting);
            assert!(
                d.origin.reference.contains("search excerpts"),
                "{}",
                d.setting
            );
            assert!(d.origin.reference.contains("Article "), "{}", d.setting);
        }
    }
}
