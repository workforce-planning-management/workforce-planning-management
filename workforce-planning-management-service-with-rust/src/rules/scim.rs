//! SCIM 2.0 outbound provisioning (WPM-R150, WPM-D102), DB-free and clock-free. Spec: `spec/integrations.md`.
//!
//! A worker becomes a SCIM `User` (RFC 7643) with the Enterprise User extension, carrying **employment facts only**: employee
//! number, display name, title, department, manager and whether the person is employed. [`Employment`] has no field for a
//! contact detail, so none can leave through the mapping; a source test also fails if this file or the push names one.

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// The SCIM schema URNs used.
pub const USER_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
/// The Enterprise User extension.
pub const ENTERPRISE_SCHEMA: &str = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User";

/// What the mapping may read about a worker. Nothing here is a contact detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Employment {
    /// The worker's public id; the stable `externalId` is `worker:<pid>`.
    pub pid: String,
    /// The employee number, which is also the `userName` (WPM holds no login; the identity provider links on it).
    pub worker_number: String,
    /// The name as the person service resolves it.
    pub display_name: String,
    /// The job title.
    pub job_title: String,
    /// The department.
    pub department: String,
    /// The first day of employment.
    pub hired_on: NaiveDate,
    /// The termination date, when there is one.
    pub terminated_on: Option<NaiveDate>,
    /// The record was deleted.
    pub deleted: bool,
}

/// Whether the person should be active on `today`, by the same definition as every headcount: hired on or before it and not
/// terminated on or before it. A deleted record is never active.
#[must_use]
pub fn active_on(e: &Employment, today: NaiveDate) -> bool {
    !e.deleted && crate::rules::metrics::is_employed_on(today, e.hired_on, e.terminated_on)
}

/// Whether the worker should be sent at all on `today`: not before the first day, so no account exists before the person joins.
#[must_use]
pub fn due(e: &Employment, today: NaiveDate) -> bool {
    e.hired_on <= today
}

/// The SCIM `externalId`: stable, and a reference rather than a name.
#[must_use]
pub fn external_id(pid: &str) -> String {
    format!("worker:{pid}")
}

/// The SCIM `User` for a worker. `manager_scim_id` is the identity provider's own id for the manager, known only once the
/// manager has been provisioned; until then the manager is left out and a later run adds it.
#[must_use]
pub fn user(e: &Employment, today: NaiveDate, manager_scim_id: Option<&str>) -> Value {
    let mut enterprise = json!({ "employeeNumber": e.worker_number, "department": e.department });
    if let Some(id) = manager_scim_id {
        enterprise["manager"] = json!({ "value": id });
    }
    json!({
        "schemas": [USER_SCHEMA, ENTERPRISE_SCHEMA],
        "externalId": external_id(&e.pid),
        "userName": e.worker_number,
        "displayName": e.display_name,
        "title": e.job_title,
        "active": active_on(e, today),
        ENTERPRISE_SCHEMA: enterprise,
    })
}

/// The query that finds the user the provider already has for an `externalId` (RFC 7644 filter), percent-encoded.
#[must_use]
pub fn lookup_query(external_id: &str) -> String {
    let filter = format!("externalId eq \"{external_id}\"");
    let encoded: String = filter
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect();
    format!("filter={encoded}")
}

/// A short fingerprint of a body, so an unchanged user is not sent again.
#[must_use]
pub fn fingerprint(body: &Value) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = serde_json::to_vec(body).unwrap_or_default();
    Sha256::digest(bytes)
        .iter()
        .flat_map(|b| [HEX[usize::from(b >> 4)], HEX[usize::from(b & 15)]])
        .map(char::from)
        .collect()
}

/// What to do for one worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    /// Not due yet, or deleted and never sent: nothing to do.
    Skip,
    /// Already sent in exactly this form.
    Unchanged,
    /// Create the user.
    Create,
    /// Replace the user.
    Replace,
}

/// Decide the step from what is wanted, what was last sent and whether the identity provider has an id for the user.
#[must_use]
pub fn step(due: bool, deleted: bool, known_id: bool, sent_fingerprint: Option<&str>, wanted: &str) -> Step {
    match (due, known_id) {
        (false, _) => Step::Skip,
        (true, false) if deleted => Step::Skip,
        (true, false) => Step::Create,
        (true, true) if sent_fingerprint == Some(wanted) => Step::Unchanged,
        (true, true) => Step::Replace,
    }
}

/// The wait before the next try after `attempts` failures: one minute doubling, at most a day.
#[must_use]
pub fn backoff_seconds(attempts: u32) -> i64 {
    60_i64.saturating_mul(1_i64 << attempts.min(20)).min(86_400)
}

/// A base URL is `https`, or plain `http` to the machine itself (a stub or a local bridge). Anything else is refused so a
/// bearer token is never sent in clear across a network.
///
/// # Errors
///
/// A message when the URL is not acceptable.
pub fn check_base_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim().trim_end_matches('/');
    let (scheme, rest) = trimmed.split_once("://").ok_or("the SCIM base URL needs a scheme")?;
    let host = rest.split(['/', ':']).next().unwrap_or("");
    let local = matches!(host, "localhost" | "127.0.0.1" | "[::1]");
    match scheme {
        "https" if !host.is_empty() => Ok(trimmed.to_string()),
        "http" if local => Ok(trimmed.to_string()),
        _ => Err("the SCIM base URL must be https (or http to this machine)".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn worker() -> Employment {
        Employment {
            pid: "11111111-2222-4333-8444-555555555555".into(),
            worker_number: "W-0042".into(),
            display_name: "Sam Example".into(),
            job_title: "Engineer".into(),
            department: "engineering".into(),
            hired_on: d(2026, 1, 5),
            terminated_on: None,
            deleted: false,
        }
    }

    #[test]
    fn a_worker_maps_to_a_user_with_the_enterprise_extension() {
        let u = user(&worker(), d(2026, 10, 24), None);
        assert_eq!(u["schemas"], json!([USER_SCHEMA, ENTERPRISE_SCHEMA]));
        assert_eq!(u["externalId"], "worker:11111111-2222-4333-8444-555555555555");
        assert_eq!(u["userName"], "W-0042");
        assert_eq!(u["title"], "Engineer");
        assert_eq!(u["active"], true);
        assert_eq!(u[ENTERPRISE_SCHEMA]["employeeNumber"], "W-0042");
        assert_eq!(u[ENTERPRISE_SCHEMA]["department"], "engineering");
        assert!(u[ENTERPRISE_SCHEMA].get("manager").is_none(), "unknown until the manager is provisioned");
        assert!(u[ENTERPRISE_SCHEMA].get("costCenter").is_none(), "WPM holds no cost centre, so none is sent");
    }

    #[test]
    fn the_manager_is_the_identity_providers_id_when_known() {
        let u = user(&worker(), d(2026, 10, 24), Some("abc-123"));
        assert_eq!(u[ENTERPRISE_SCHEMA]["manager"], json!({ "value": "abc-123" }));
    }

    #[test]
    fn the_body_holds_only_employment_facts() {
        let u = user(&worker(), d(2026, 10, 24), Some("m"));
        let keys: Vec<&str> = u.as_object().unwrap().keys().map(String::as_str).collect();
        let expected = ["schemas", "externalId", "userName", "displayName", "title", "active", ENTERPRISE_SCHEMA];
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        let mut want = expected.to_vec();
        want.sort_unstable();
        assert_eq!(sorted, want, "a new field must be a decision, not an accident");
    }

    /// WPM-D102: no contact field leaves. The input type has none; this makes the sources prove they never name one outside
    /// comments and tests.
    #[test]
    fn no_contact_field_is_reachable_from_the_mapping_or_the_push() {
        let code = |source: &'static str| -> String {
            source
                .split("#[cfg(test)]")
                .next()
                .unwrap_or("")
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase()
        };
        // The words are assembled so this test does not trip on itself.
        let words = ["contact", &["em", "ail"].concat(), "phone", "address", "emergency", "postal", "mobile"];
        for (name, source) in [
            ("rules/scim.rs", include_str!("scim.rs")),
            ("tasks/scim_push.rs", include_str!("../tasks/scim_push.rs")),
        ] {
            let text = code(source);
            for word in words {
                assert!(!text.contains(word), "{name} names {word}");
            }
        }
    }

    #[test]
    fn active_follows_the_headcount_definition_and_deleted_is_never_active() {
        let mut w = worker();
        assert!(active_on(&w, d(2026, 10, 24)));
        assert!(!active_on(&w, d(2026, 1, 4)), "before the first day");
        w.terminated_on = Some(d(2026, 11, 1));
        assert!(active_on(&w, d(2026, 10, 31)));
        assert!(!active_on(&w, d(2026, 11, 1)), "terminated on or before the date is not employed");
        assert_eq!(user(&w, d(2026, 11, 2), None)["active"], false);
        w.terminated_on = None;
        w.deleted = true;
        assert!(!active_on(&w, d(2026, 10, 24)));
    }

    #[test]
    fn nobody_is_sent_before_their_first_day() {
        let w = worker();
        assert!(!due(&w, d(2026, 1, 4)));
        assert!(due(&w, d(2026, 1, 5)));
    }

    #[test]
    fn the_step_is_idempotent_and_never_creates_for_a_deleted_unknown_worker() {
        let body = user(&worker(), d(2026, 10, 24), None);
        let f = fingerprint(&body);
        assert_eq!(step(false, false, false, None, &f), Step::Skip);
        assert_eq!(step(true, false, false, None, &f), Step::Create);
        assert_eq!(step(true, true, false, None, &f), Step::Skip, "deleted and never sent");
        assert_eq!(step(true, false, true, Some(&f), &f), Step::Unchanged);
        assert_eq!(step(true, false, true, Some("old"), &f), Step::Replace);
        assert_eq!(step(true, true, true, Some("old"), &f), Step::Replace, "a deleted user already sent is deactivated");
        assert_eq!(step(true, false, true, None, &f), Step::Replace);
    }

    #[test]
    fn a_change_changes_the_fingerprint_and_the_same_body_does_not() {
        let a = user(&worker(), d(2026, 10, 24), None);
        assert_eq!(fingerprint(&a), fingerprint(&a.clone()));
        let mut w = worker();
        w.job_title = "Senior Engineer".into();
        assert_ne!(fingerprint(&a), fingerprint(&user(&w, d(2026, 10, 24), None)));
        assert_eq!(fingerprint(&a).len(), 64);
    }

    #[test]
    fn the_lookup_filter_is_encoded_and_cannot_carry_another_parameter() {
        assert_eq!(lookup_query("worker:abc"), "filter=externalId%20eq%20%22worker%3Aabc%22");
        assert!(!lookup_query("x&y=1\"").contains('&'));
    }

    #[test]
    fn back_off_doubles_and_stops_at_a_day() {
        assert_eq!(backoff_seconds(0), 60);
        assert_eq!(backoff_seconds(1), 120);
        assert_eq!(backoff_seconds(3), 480);
        assert_eq!(backoff_seconds(30), 86_400);
    }

    #[test]
    fn a_token_is_never_sent_in_clear_across_a_network() {
        assert_eq!(check_base_url("https://idp.example/scim/v2/").unwrap(), "https://idp.example/scim/v2");
        assert!(check_base_url("http://127.0.0.1:9000/scim").is_ok());
        assert!(check_base_url("http://localhost/scim").is_ok());
        for bad in ["http://idp.example/scim", "ftp://x", "idp.example", "https://", "http://127.0.0.1.evil.example/"] {
            assert!(check_base_url(bad).is_err(), "{bad}");
        }
    }
}
