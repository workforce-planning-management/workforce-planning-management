//! Outbound webhooks (WPM-R151, WPM-D106), DB-free and clock-free. Spec: `spec/integrations.md`.
//!
//! An event leaves as a **reference**: its id, its kind, the URN of the record it is about and when it happened. Never a name,
//! a number, a date of birth or any other personal field; a subscriber that needs more reads it through the API under its own
//! access. A hire or an ending also carries `data`, exactly the handoff of [`crate::rules::bridge::handoff_data`] (a `person:` and an
//! `organization:` reference and a date, with `from` and `to` for an ending), so a subscriber can tell *who* by reference (proposed
//! WPM-D145, consistent with WPM-D106 and WPM-D134). The body is signed with HMAC-SHA256 under a per-subscription secret that is held **by reference** (the name of an
//! environment variable), so the secret is never stored or returned.

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// The events a subscriber may ask for: `(entity, kind)`. Anything else is not offered.
pub const KINDS: &[(&str, &str)] = &[
    ("worker", "worker_hired"),
    ("worker", "worker_activated"),
    ("worker", "worker_terminated"),
    ("worker", "worker_retired"),
    ("leave_request", "leave_approved"),
];

/// Whether an emitted event is one a subscriber may be told about.
#[must_use]
pub fn offered(entity: &str, kind: &str) -> bool {
    KINDS.iter().any(|(e, k)| *e == entity && *k == kind)
}

/// The kinds, as a subscriber names them.
#[must_use]
pub fn kind_names() -> Vec<&'static str> {
    KINDS.iter().map(|(_, k)| *k).collect()
}

/// The fields every payload has. Adding one is a decision (WPM-D106), and a test holds this list against the payload.
pub const PAYLOAD_FIELDS: [&str; 4] = ["id", "kind", "occurred_at", "subject"];

/// The fields of a payload that carries `data` (a hire or an ending, [`data_keys`]), sorted.
pub const PAYLOAD_FIELDS_WITH_DATA: [&str; 5] = ["data", "id", "kind", "occurred_at", "subject"];

/// The keys of a hire's `data`, sorted: [`crate::rules::bridge::HANDOFF_KEYS`].
pub const HIRED_DATA_KEYS: [&str; 3] = crate::rules::bridge::HANDOFF_KEYS;

/// The keys of an ending's `data`, sorted: the handoff and the status change.
pub const ENDED_DATA_KEYS: [&str; 5] = [
    "effective_on",
    "from",
    "organization_ref",
    "person_ref",
    "to",
];

/// The keys a kind's `data` holds, sorted; empty for a kind that carries no `data` (proposed WPM-D145).
#[must_use]
pub fn data_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "worker_hired" => &HIRED_DATA_KEYS,
        "worker_terminated" | "worker_retired" => &ENDED_DATA_KEYS,
        _ => &[],
    }
}

/// The top-level fields of a payload of this kind, sorted.
#[must_use]
pub fn payload_fields(kind: &str) -> &'static [&'static str] {
    if data_keys(kind).is_empty() {
        &PAYLOAD_FIELDS
    } else {
        &PAYLOAD_FIELDS_WITH_DATA
    }
}

/// What a delivery of this kind keeps of the event's data: exactly [`data_keys`], or nothing. A kind with no keys keeps nothing
/// whatever the event carried (a status change's `from` and `to` on an activation stay behind), and an event that lacks one of the
/// keys keeps nothing rather than a part, so a subscriber is never handed half a handoff.
#[must_use]
pub fn delivery_data(kind: &str, event_data: Option<&Value>) -> Option<Value> {
    let keys = data_keys(kind);
    let source = event_data?.as_object()?;
    if keys.is_empty() || !keys.iter().all(|k| source.contains_key(*k)) {
        return None;
    }
    Some(Value::Object(
        keys.iter()
            .map(|k| ((*k).to_string(), source[*k].clone()))
            .collect(),
    ))
}

/// The URN of the record an event is about.
#[must_use]
pub fn subject(entity: &str, pid: &str) -> String {
    format!("{entity}:{pid}")
}

/// The payload: a reference and a kind, and for a hire or an ending the handoff `data` the delivery kept ([`delivery_data`]). `data` is
/// sent only for a kind that carries it, so a stray value for another kind is dropped here too.
#[must_use]
pub fn payload(
    event_id: &str,
    kind: &str,
    subject: &str,
    occurred_at: DateTime<Utc>,
    data: Option<&Value>,
) -> Value {
    let mut body = json!({
        "id": event_id,
        "kind": kind,
        "subject": subject,
        "occurred_at": occurred_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    });
    if let (Some(data), Some(object)) = (delivery_data(kind, data), body.as_object_mut()) {
        object.insert("data".to_string(), data);
    }
    body
}

const BLOCK: usize = 64;

/// HMAC-SHA256 (RFC 2104).
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k = [0_u8; BLOCK];
    if key.len() > BLOCK {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut inner = Sha256::new();
    inner.update(k.map(|b| b ^ 0x36));
    inner.update(message);
    let mut outer = Sha256::new();
    outer.update(k.map(|b| b ^ 0x5c));
    outer.update(inner.finalize());
    outer.finalize().into()
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|b| [DIGITS[usize::from(b >> 4)], DIGITS[usize::from(b & 15)]])
        .map(char::from)
        .collect()
}

/// The `X-WPM-Signature` header: `t=<unix seconds>,v1=<hex HMAC of "<t>.<body>">`. The timestamp is signed, so a captured
/// delivery cannot be replayed later without the subscriber noticing the age.
#[must_use]
pub fn signature(secret: &[u8], timestamp: i64, body: &[u8]) -> String {
    let mut signed = format!("{timestamp}.").into_bytes();
    signed.extend_from_slice(body);
    format!("t={timestamp},v1={}", hex(&hmac_sha256(secret, &signed)))
}

/// What a subscriber does with a delivery: check the signature and the age. Provided so the contract is tested from both ends.
#[must_use]
pub fn verify(secret: &[u8], header: &str, body: &[u8], now: i64, tolerance_seconds: i64) -> bool {
    let mut t = None;
    let mut v1 = None;
    for part in header.split(',') {
        match part.split_once('=') {
            Some(("t", v)) => t = v.parse::<i64>().ok(),
            Some(("v1", v)) => v1 = Some(v),
            _ => {}
        }
    }
    let (Some(t), Some(v1)) = (t, v1) else {
        return false;
    };
    if (now - t).abs() > tolerance_seconds {
        return false;
    }
    let expected = signature(secret, t, body);
    let given = format!("t={t},v1={v1}");
    // Compared without stopping at the first difference.
    expected.len() == given.len()
        && expected
            .bytes()
            .zip(given.bytes())
            .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

/// How many tries a delivery gets before it is given up on.
pub const MAX_ATTEMPTS: i32 = 10;

/// What happens after an attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// Done.
    Delivered,
    /// Try again after this many seconds.
    Retry(i64),
    /// Stop: the subscriber refused it, or the tries are used up.
    GiveUp,
}

/// Decide from the HTTP status (`None` when the request did not complete) and the number of attempts made so far, this one
/// included. A 2xx is delivered; a network failure, 408, 429 and 5xx are retried with back-off; any other answer is the
/// subscriber saying no and is not repeated.
#[must_use]
pub fn after_attempt(status: Option<u16>, attempts: i32) -> Next {
    match status {
        Some(200..=299) => Next::Delivered,
        None | Some(408 | 429 | 500..=599) if attempts < MAX_ATTEMPTS => Next::Retry(
            crate::rules::scim::backoff_seconds(u32::try_from(attempts - 1).unwrap_or(0)),
        ),
        _ => Next::GiveUp,
    }
}

/// Whether a host is an address literal that points inside a network: loopback, private, link-local, unspecified, unique-local.
fn internal_literal(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    match bare.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(a)) => {
            a.is_loopback()
                || a.is_private()
                || a.is_link_local()
                || a.is_unspecified()
                || a.is_broadcast()
        }
        Ok(std::net::IpAddr::V6(a)) => {
            a.is_loopback()
                || a.is_unspecified()
                || (a.segments()[0] & 0xfe00) == 0xfc00
                || (a.segments()[0] & 0xffc0) == 0xfe80
        }
        Err(_) => false,
    }
}

/// Check an endpoint: `https` to a host that is not an internal address literal, or `http` to this machine (a test or a local
/// bridge); no credentials, query or fragment in it (a secret in a URL is stored and logged).
///
/// # Errors
///
/// A message naming what is wrong.
pub fn check_url(url: &str) -> Result<String, String> {
    let url = url.trim();
    if url.contains(['?', '#', '@']) || url.chars().any(char::is_whitespace) {
        return Err("the endpoint has no credentials, query, fragment or spaces".to_string());
    }
    let (scheme, rest) = url.split_once("://").ok_or("the endpoint needs a scheme")?;
    let authority = rest.split('/').next().unwrap_or("");
    let host = if authority.starts_with('[') {
        authority
            .split_once(']')
            .map_or(authority, |(h, _)| h)
            .trim_start_matches('[')
    } else {
        authority.split(':').next().unwrap_or("")
    };
    let local = matches!(host, "localhost" | "127.0.0.1" | "::1");
    match scheme {
        "https" if !host.is_empty() && !internal_literal(host) => Ok(url.to_string()),
        "http" if local => Ok(url.to_string()),
        "https" => Err("the endpoint may not be an internal address".to_string()),
        _ => Err("the endpoint must be https (or http to this machine)".to_string()),
    }
}

/// A secret reference names an environment variable of a fixed shape, so a subscription can never point the service at
/// `DATABASE_URL` or any other setting.
///
/// # Errors
///
/// A message when the name is not of the form `WPM_WEBHOOK_SECRET_<NAME>`.
pub fn check_secret_ref(name: &str) -> Result<(), String> {
    const PREFIX: &str = "WPM_WEBHOOK_SECRET_";
    let rest = name.strip_prefix(PREFIX).unwrap_or("");
    if !rest.is_empty()
        && rest.len() <= 60
        && rest
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        Ok(())
    } else {
        Err(format!(
            "secret_ref is the name of an environment variable, {PREFIX}<NAME>, in capitals"
        ))
    }
}

/// Check a subscription's name and kinds.
///
/// # Errors
///
/// The problems found.
pub fn check_subscription(name: &str, kinds: &[String]) -> Result<(), Vec<String>> {
    let mut out = Vec::new();
    if !(1..=80).contains(&name.trim().chars().count()) {
        out.push("name is 1 to 80 characters".to_string());
    }
    if kinds.is_empty() {
        out.push("kinds needs at least one event kind".to_string());
    }
    let offered = kind_names();
    for k in kinds {
        if !offered.contains(&k.as_str()) {
            out.push(format!(
                "{k} is not an event kind offered; the kinds are {}",
                offered.join(", ")
            ));
        }
    }
    let mut seen = kinds.to_vec();
    seen.sort();
    seen.dedup();
    if seen.len() != kinds.len() {
        out.push("kinds has a repeat".to_string());
    }
    if out.is_empty() { Ok(()) } else { Err(out) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-24T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn hmac_matches_the_rfc_4231_vectors() {
        // Test case 1 and test case 6 (a key longer than the block).
        assert_eq!(
            hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            hex(&hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    fn sorted_keys(v: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        keys
    }

    /// What an ending event carries on the transport: the status change and the handoff, and here one field too many.
    fn ending_event_data() -> Value {
        json!({
            "from": "offboarding", "to": "terminated",
            "person_ref": "person:0c4f1e2a-0000-4000-8000-000000000001",
            "organization_ref": "organization:0c4f1e2a-0000-4000-8000-0000000000aa",
            "effective_on": "2026-10-30",
            "display_name": "Never Sent",
        })
    }

    #[test]
    fn a_payload_is_a_reference_and_a_kind_and_nothing_else() {
        let p = payload(
            "e1",
            "worker_activated",
            &subject("worker", "11111111-2222-4333-8444-555555555555"),
            at(),
            Some(&json!({ "from": "pending", "to": "active" })),
        );
        assert_eq!(
            sorted_keys(&p),
            PAYLOAD_FIELDS,
            "a new field is a decision, not an accident; an activation's from and to stay behind"
        );
        assert_eq!(p["subject"], "worker:11111111-2222-4333-8444-555555555555");
        assert_eq!(p["occurred_at"], "2026-10-24T12:00:00Z");
        let leave = payload("e2", "leave_approved", "leave_request:x", at(), None);
        assert_eq!(sorted_keys(&leave), PAYLOAD_FIELDS);
    }

    #[test]
    fn the_key_sets_are_pinned_per_kind() {
        assert_eq!(
            data_keys("worker_hired"),
            ["effective_on", "organization_ref", "person_ref"]
        );
        assert_eq!(
            data_keys("worker_hired"),
            crate::rules::bridge::HANDOFF_KEYS
        );
        for kind in ["worker_terminated", "worker_retired"] {
            assert_eq!(
                data_keys(kind),
                [
                    "effective_on",
                    "from",
                    "organization_ref",
                    "person_ref",
                    "to"
                ]
            );
            assert_eq!(
                payload_fields(kind),
                ["data", "id", "kind", "occurred_at", "subject"]
            );
        }
        for kind in ["worker_activated", "leave_approved", "updated"] {
            assert_eq!(data_keys(kind), [] as [&str; 0]);
            assert_eq!(payload_fields(kind), PAYLOAD_FIELDS);
        }
        // Every offered kind is covered by one of the two shapes.
        for kind in kind_names() {
            assert!(
                payload_fields(kind) == PAYLOAD_FIELDS
                    || payload_fields(kind) == PAYLOAD_FIELDS_WITH_DATA
            );
        }
    }

    #[test]
    fn a_hire_and_an_ending_carry_exactly_the_handoff_and_nothing_else() {
        let handoff = crate::rules::bridge::handoff_data(
            "person:0c4f1e2a-0000-4000-8000-000000000001",
            "organization:0c4f1e2a-0000-4000-8000-0000000000aa",
            chrono::NaiveDate::from_ymd_opt(2026, 10, 30).unwrap(),
        );
        let hired = payload("e1", "worker_hired", "worker:x", at(), Some(&handoff));
        assert_eq!(sorted_keys(&hired), PAYLOAD_FIELDS_WITH_DATA);
        assert_eq!(sorted_keys(&hired["data"]), HIRED_DATA_KEYS);
        assert_eq!(hired["data"], handoff);
        for kind in ["worker_terminated", "worker_retired"] {
            let ended = payload("e2", kind, "worker:x", at(), Some(&ending_event_data()));
            assert_eq!(sorted_keys(&ended), PAYLOAD_FIELDS_WITH_DATA);
            assert_eq!(
                sorted_keys(&ended["data"]),
                ENDED_DATA_KEYS,
                "the extra field is dropped"
            );
            assert!(!ended.to_string().contains("Never Sent"));
        }
    }

    #[test]
    fn half_a_handoff_is_not_sent() {
        assert_eq!(delivery_data("worker_hired", None), None);
        assert_eq!(
            delivery_data("worker_hired", Some(&json!({ "person_ref": "person:x" }))),
            None
        );
        assert_eq!(delivery_data("worker_hired", Some(&json!("text"))), None);
        let p = payload(
            "e1",
            "worker_hired",
            "worker:x",
            at(),
            Some(&json!({ "person_ref": "person:x" })),
        );
        assert_eq!(
            sorted_keys(&p),
            PAYLOAD_FIELDS,
            "no data rather than part of it"
        );
        assert_eq!(
            delivery_data("worker_activated", Some(&ending_event_data())),
            None,
            "not a kind that carries data"
        );
    }

    #[test]
    fn only_the_offered_events_are_offered() {
        assert!(offered("worker", "worker_hired"));
        assert!(offered("leave_request", "leave_approved"));
        assert!(
            !offered("worker", "updated"),
            "an update carries a changed field and is not offered"
        );
        assert!(!offered("payslip", "worker_hired"));
        assert!(!offered("leave_request", "leave_rejected"));
    }

    #[test]
    fn a_signature_verifies_and_fails_on_any_change() {
        let body = br#"{"kind":"worker_hired"}"#;
        let header = signature(b"s3cret", 1_800_000_000, body);
        assert!(header.starts_with("t=1800000000,v1="));
        assert!(verify(b"s3cret", &header, body, 1_800_000_010, 300));
        assert!(
            !verify(b"other", &header, body, 1_800_000_010, 300),
            "another secret"
        );
        assert!(
            !verify(
                b"s3cret",
                &header,
                br#"{"kind":"worker_fired"}"#,
                1_800_000_010,
                300
            ),
            "another body"
        );
        assert!(
            !verify(b"s3cret", &header, body, 1_800_000_400, 300),
            "too old"
        );
        assert!(
            !verify(b"s3cret", &header, body, 1_799_999_000, 300),
            "from the future"
        );
        assert!(
            !verify(b"s3cret", "v1=abc", body, 1_800_000_010, 300),
            "no timestamp"
        );
        assert!(!verify(b"s3cret", "garbage", body, 1_800_000_010, 300));
        let moved = header.replace("t=1800000000", "t=1800000100");
        assert!(
            !verify(b"s3cret", &moved, body, 1_800_000_100, 300),
            "the timestamp is signed"
        );
    }

    #[test]
    fn retries_back_off_and_a_refusal_is_not_repeated() {
        assert_eq!(after_attempt(Some(204), 1), Next::Delivered);
        assert_eq!(after_attempt(Some(503), 1), Next::Retry(60));
        assert_eq!(after_attempt(Some(503), 3), Next::Retry(240));
        assert_eq!(after_attempt(None, 2), Next::Retry(120));
        assert_eq!(after_attempt(Some(429), 1), Next::Retry(60));
        assert_eq!(after_attempt(Some(408), 1), Next::Retry(60));
        assert_eq!(after_attempt(Some(400), 1), Next::GiveUp);
        assert_eq!(after_attempt(Some(410), 1), Next::GiveUp);
        assert_eq!(
            after_attempt(Some(301), 1),
            Next::GiveUp,
            "redirects are not followed"
        );
        assert_eq!(
            after_attempt(Some(503), MAX_ATTEMPTS),
            Next::GiveUp,
            "the tries are used up"
        );
    }

    #[test]
    fn an_endpoint_is_https_and_not_inside_a_network() {
        assert!(check_url("https://hooks.example/wpm").is_ok());
        assert!(check_url("http://127.0.0.1:9000/hook").is_ok());
        assert!(check_url("http://localhost/hook").is_ok());
        for bad in [
            "http://hooks.example/wpm",
            "https://169.254.169.254/latest",
            "https://10.0.0.5/hook",
            "https://192.168.1.1/hook",
            "https://127.0.0.1/hook",
            "https://[::1]/hook",
            "https://[fd00::1]/hook",
            "https://0.0.0.0/hook",
            "https://user:pw@hooks.example/wpm",
            "https://hooks.example/wpm?token=abc",
            "https://hooks.example/wpm#frag",
            "https://hooks.example/a b",
            "hooks.example",
            "ftp://hooks.example",
            "https://",
        ] {
            assert!(check_url(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_secret_reference_cannot_name_another_setting() {
        assert!(check_secret_ref("WPM_WEBHOOK_SECRET_PAYROLL").is_ok());
        for bad in [
            "DATABASE_URL",
            "WPM_SCIM_TOKEN",
            "WPM_WEBHOOK_SECRET_",
            "wpm_webhook_secret_x",
            "WPM_WEBHOOK_SECRET_A-B",
            "",
            "PATH",
        ] {
            assert!(check_secret_ref(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_subscription_names_offered_kinds_once() {
        let ok = vec!["worker_hired".to_string(), "leave_approved".to_string()];
        assert!(check_subscription("Payroll", &ok).is_ok());
        assert!(check_subscription("", &ok).is_err());
        assert!(check_subscription("x", &[]).is_err());
        assert!(check_subscription("x", &["updated".to_string()]).is_err());
        assert!(
            check_subscription(
                "x",
                &["worker_hired".to_string(), "worker_hired".to_string()]
            )
            .is_err()
        );
    }
}
