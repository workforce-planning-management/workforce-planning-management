//! Backward-compatibility shims for two renames of the settings prefix: the **2026-07-23 rename** (human capital
//! management / `HCM` → workforce planning management / `WPM`) and the **2026-11-20 rename** (`WPM_` →
//! `WORKFORCE_PLANNING_MANAGEMENT_`, WPM-R278, WPM-D221).
//!
//! The first rename changed two things a deployment cannot see coming:
//!
//! 1. the environment-variable prefix (`HCM_REQUIRE_AUTH` →
//!    `WPM_REQUIRE_AUTH`, and so on; now `WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH`), and
//! 2. the ABAC resource entity (`"hcm"` → `"wpm"`), which a mounted
//!    policy may key its rules on.
//!
//! Renaming those without a shim would silently change behaviour on an
//! existing deployment — the worst failure mode available here, because
//! **both** of them fail *open-ish*: a `HCM_REQUIRE_AUTH=1` that stops
//! being read turns authentication **off**, and a policy rule keyed on
//! `entity: "hcm"` that stops matching falls through to the default
//! decision. A config that no longer applies must not look like a
//! config that says "allow".
//!
//! So both are accepted, once, with a loud deprecation warning:
//!
//! - [`env_var`] reads the `WORKFORCE_PLANNING_MANAGEMENT_*` name and falls back to the legacy `WPM_*`, then `HCM_*`,
//!   name.
//! - [`migrate_policy_entity`] rewrites `entity: "hcm"` conditions in a
//!   mounted policy to `"wpm"` before it is parsed.
//!
//! Both are **transitional**. They exist so a deployment survives the
//! rename across one release, not forever; see the removal note on each.

use std::collections::HashSet;
use std::sync::Mutex;

/// The current environment-variable prefix (WPM-R278).
pub const PREFIX: &str = "WORKFORCE_PLANNING_MANAGEMENT_";

/// The earlier environment-variable prefixes, newest first, still honoured as deprecated fallbacks (WPM-D221).
pub const LEGACY_PREFIXES: [&str; 2] = ["WPM_", "HCM_"];

/// The current ABAC resource-entity name (see
/// [`crate::auth::ENTITY`]).
pub const ENTITY: &str = "wpm";

/// The pre-rename ABAC resource-entity name, still honoured in a
/// mounted policy.
pub const LEGACY_ENTITY: &str = "hcm";

/// The current spelling of `name`: a name under a legacy prefix becomes the same name under [`PREFIX`]; any other name
/// is unchanged. A name stored before a rename (a webhook subscription's `secret_ref`) still finds its variable after the
/// operator renames it.
#[must_use]
pub fn current_name(name: &str) -> String {
    LEGACY_PREFIXES
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))
        .map_or_else(|| name.to_string(), |rest| format!("{PREFIX}{rest}"))
}

/// The legacy spellings of a current variable, newest first, or none when `name` is not one of ours (so an unrelated
/// variable is never rewritten).
///
/// Pure, so the mapping is unit-tested without touching process env.
#[must_use]
pub fn legacy_names(name: &str) -> Vec<String> {
    name.strip_prefix(PREFIX).map_or_else(Vec::new, |rest| {
        LEGACY_PREFIXES
            .iter()
            .map(|prefix| format!("{prefix}{rest}"))
            .collect()
    })
}

/// Names already warned about, so a deprecation notice is logged once
/// per variable rather than on every read (several of these are read
/// per request behind a `OnceLock`, and a per-read warning would drown
/// the log it is trying to be noticed in).
static WARNED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Log the deprecation notice for `legacy`, at most once per process.
fn warn_once(legacy: &str, current: &str) {
    let mut guard = WARNED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let seen = guard.get_or_insert_with(HashSet::new);
    if seen.insert(legacy.to_string()) {
        tracing::warn!(
            legacy_variable = legacy,
            current_variable = current,
            "using the deprecated pre-rename environment variable; \
             rename it to the current one — the fallback will be removed"
        );
    }
}

/// Read a `WORKFORCE_PLANNING_MANAGEMENT_*` environment variable, falling back to its legacy `WPM_*`, then `HCM_*`,
/// spelling.
///
/// Blank or whitespace-only counts as unset at both names, matching the
/// rest of the crate's env handling. Using the legacy name logs a
/// deprecation warning once per variable per process.
///
/// **Removal**: delete this indirection (and call `std::env::var`
/// directly) once no deployment sets `WPM_*` or `HCM_*`.
#[must_use]
pub fn env_var(name: &str) -> Option<String> {
    env_var_from(name, |name| std::env::var(name).ok())
}

/// As [`env_var`], with the source passed in. This keeps precedence and blank-value handling
/// testable without mutating the process environment.
#[must_use]
pub(crate) fn env_var_from(name: &str, get: impl Fn(&str) -> Option<String>) -> Option<String> {
    let nonblank = |v: &String| !v.trim().is_empty();
    let current = current_name(name);
    let name = current.as_str();

    let legacy = legacy_names(name);
    if let Some(value) = get(name).filter(nonblank) {
        // The current name wins; a legacy one still set beside it is ignored, and said (WPM-R278).
        for old in &legacy {
            if get(old).is_some_and(|v| nonblank(&v)) {
                warn_once(old, name);
            }
        }
        return Some(value);
    }
    legacy.iter().find_map(|old| {
        let value = get(old).filter(nonblank)?;
        warn_once(old, name);
        Some(value)
    })
}

/// Rewrite legacy `entity: "hcm"` conditions in a mounted ABAC policy
/// to the current entity name, returning `true` when anything changed.
///
/// A policy rule may key on the `entity` pseudo-attribute
/// (`authorization-attributes.md` §2). After the rename, a rule written
/// as `{"when": {"entity": ["hcm"]}}` would simply stop matching — and
/// a rule that stops matching is *invisible*: the decision quietly
/// falls through to the default instead of erroring. Rewriting it (with
/// a warning) keeps a mounted policy meaning what its author intended.
///
/// Only the `entity` key is touched, and only where its value is
/// exactly the legacy name — a subject attribute that happens to hold
/// the string `"hcm"` is left alone.
///
/// Pure (operates on the parsed JSON), so the walk is unit-tested.
///
/// **Removal**: delete once no mounted policy names the old entity.
pub fn migrate_policy_entity(policy: &mut serde_json::Value) -> bool {
    let mut changed = false;
    let Some(rules) = policy.get_mut("rules").and_then(|r| r.as_array_mut()) else {
        return false;
    };
    for rule in rules {
        let Some(when) = rule.get_mut("when").and_then(|w| w.as_object_mut()) else {
            continue;
        };
        let Some(entity) = when.get_mut("entity") else {
            continue;
        };
        match entity {
            serde_json::Value::String(value) if value == LEGACY_ENTITY => {
                *value = ENTITY.to_string();
                changed = true;
            }
            serde_json::Value::Array(values) => {
                for value in values.iter_mut() {
                    if value.as_str() == Some(LEGACY_ENTITY) {
                        *value = serde_json::Value::String(ENTITY.to_string());
                        changed = true;
                    }
                }
            }
            _ => {}
        }
    }
    if changed {
        tracing::warn!(
            legacy_entity = LEGACY_ENTITY,
            current_entity = ENTITY,
            "the mounted ABAC policy names the pre-rename entity; it was accepted and \
             rewritten — update the policy, the fallback will be removed"
        );
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Only our own prefix is mapped; an unrelated variable is never
    /// given a legacy alias.
    #[test]
    fn legacy_names_map_only_our_prefix() {
        assert_eq!(
            legacy_names("WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH"),
            ["WPM_REQUIRE_AUTH", "HCM_REQUIRE_AUTH"]
        );
        assert_eq!(
            legacy_names("WORKFORCE_PLANNING_MANAGEMENT_ABAC_POLICY_FILE"),
            ["WPM_ABAC_POLICY_FILE", "HCM_ABAC_POLICY_FILE"]
        );
        assert_eq!(
            legacy_names("WORKFORCE_PLANNING_MANAGEMENT_"),
            ["WPM_", "HCM_"]
        );

        let none: Vec<String> = Vec::new();
        assert_eq!(legacy_names("DATABASE_URL"), none);
        assert_eq!(legacy_names("RUST_LOG"), none);
        assert_eq!(legacy_names("WPM_REQUIRE_AUTH"), none, "no reverse mapping");
        assert_eq!(legacy_names("HCM_REQUIRE_AUTH"), none, "no reverse mapping");
        assert_eq!(
            legacy_names("XWORKFORCE_PLANNING_MANAGEMENT_FOO"),
            none,
            "prefix must be at the start"
        );
    }

    /// One case: the environment, the value expected, and why.
    type Case<'a> = (&'a [(&'a str, &'a str)], Option<&'a str>, &'a str);

    fn name_new() -> &'static str {
        "WORKFORCE_PLANNING_MANAGEMENT_MODE"
    }

    #[test]
    fn current_environment_name_wins_and_blanks_fall_back() {
        let read = |pairs: &[(&str, &str)], name: &str| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_string())
        };
        let name = "WORKFORCE_PLANNING_MANAGEMENT_MODE";
        let cases: [Case; 7] = [
            (
                &[(name, "current"), ("WPM_MODE", "wpm"), ("HCM_MODE", "hcm")],
                Some("current"),
                "the current name wins",
            ),
            (
                &[(name, "  "), ("WPM_MODE", "wpm"), ("HCM_MODE", "hcm")],
                Some("wpm"),
                "a blank falls back to WPM_",
            ),
            (
                &[("WPM_MODE", "wpm"), ("HCM_MODE", "hcm")],
                Some("wpm"),
                "WPM_ before HCM_",
            ),
            (
                &[("WPM_MODE", " "), ("HCM_MODE", "hcm")],
                Some("hcm"),
                "a blank WPM_ falls back to HCM_",
            ),
            (&[("HCM_MODE", "hcm")], Some("hcm"), "HCM_ alone"),
            (&[("HCM_MODE", "\t")], None, "blank everywhere is unset"),
            (&[], None, "nothing set"),
        ];
        for (pairs, expected, why) in cases {
            assert_eq!(
                env_var_from(name, |n| read(pairs, n)).as_deref(),
                expected,
                "{why}"
            );
        }
        assert_eq!(
            env_var_from("DATABASE_URL", |name| read(
                &[("DATABASE_URL", "postgres://example")],
                name
            )),
            Some("postgres://example".to_string())
        );
        assert_eq!(env_var_from("DATABASE_URL", |_| None), None);
        // A name stored before the rename is read as its current name first, then its old spellings.
        assert_eq!(
            env_var_from("WPM_MODE", |name| read(&[("WPM_MODE", "x")], name)).as_deref(),
            Some("x")
        );
        assert_eq!(
            env_var_from("WPM_MODE", |name| read(
                &[(name_new(), "new"), ("WPM_MODE", "x")],
                name
            ))
            .as_deref(),
            Some("new"),
            "a renamed variable is found from its old stored name"
        );
        assert_eq!(
            current_name("HCM_MODE"),
            "WORKFORCE_PLANNING_MANAGEMENT_MODE"
        );
        assert_eq!(current_name("DATABASE_URL"), "DATABASE_URL");
    }

    /// A rule keyed on the legacy entity is rewritten, in both the
    /// array and bare-string forms.
    #[test]
    fn policy_entity_is_migrated() {
        let mut policy = json!({
            "rules": [
                { "effect": "allow", "actions": ["read"], "when": { "entity": ["hcm"] } },
                { "effect": "deny", "actions": ["write"], "when": { "entity": "hcm" } },
            ]
        });
        assert!(migrate_policy_entity(&mut policy));
        assert_eq!(policy["rules"][0]["when"]["entity"], json!(["wpm"]));
        assert_eq!(policy["rules"][1]["when"]["entity"], json!("wpm"));
    }

    /// A mixed list keeps its other entries; a policy already on the
    /// new name is left untouched and reports no change.
    #[test]
    fn migration_is_narrow_and_idempotent() {
        let mut mixed = json!({
            "rules": [{ "when": { "entity": ["hcm", "person"] } }]
        });
        assert!(migrate_policy_entity(&mut mixed));
        assert_eq!(
            mixed["rules"][0]["when"]["entity"],
            json!(["wpm", "person"])
        );

        // Already migrated ⇒ no change, no warning.
        assert!(!migrate_policy_entity(&mut mixed));

        let mut current = json!({ "rules": [{ "when": { "entity": ["wpm"] } }] });
        assert!(!migrate_policy_entity(&mut current));
    }

    /// Only the `entity` key is rewritten — a *subject* attribute whose
    /// value happens to be `"hcm"` (a department, a tenant, a service
    /// name) must survive untouched.
    #[test]
    fn only_the_entity_key_is_touched() {
        let mut policy = json!({
            "rules": [{
                "effect": "allow",
                "when": { "dept": ["hcm"], "svc": ["hcm"], "entity": ["hcm"] }
            }]
        });
        assert!(migrate_policy_entity(&mut policy));
        let when = &policy["rules"][0]["when"];
        assert_eq!(when["entity"], json!(["wpm"]), "the entity is migrated");
        assert_eq!(when["dept"], json!(["hcm"]), "a subject attribute is not");
        assert_eq!(when["svc"], json!(["hcm"]), "a subject attribute is not");
    }

    /// Shapes that carry no entity condition are handled without
    /// panicking (the policy is operator-supplied JSON — invariant 2).
    #[test]
    fn malformed_or_empty_policies_are_safe() {
        for mut value in [
            json!({}),
            json!({ "rules": "not-an-array" }),
            json!({ "rules": [] }),
            json!({ "rules": [{}] }),
            json!({ "rules": [{ "when": "not-an-object" }] }),
            json!({ "rules": [{ "when": { "entity": 7 } }] }),
            json!(null),
        ] {
            assert!(!migrate_policy_entity(&mut value));
        }
    }
}
