//! **Self-service writes** (WPM-R130, WPM-D76): the one list of routes a signed-in worker may
//! write without HR.
//!
//! The policy lets only HR, service and administrator callers write. A worker changing their own
//! address cannot be expressed there, because the policy sees an action and not the record. So a
//! short, explicit list names the routes under `/api/me` that skip the policy's *action* check
//! (the caller must still hold a valid token) and whose handlers resolve the person from that token
//! alone. A new self-service route is added here and to its test, or it does not exist.

/// `(method, route under /api)` of every self-service write. `{}` matches one segment.
pub const WRITES: &[(&str, &str)] = &[
    ("PUT", "me/contact-details"),
    ("DELETE", "me/contact-details"),
    ("POST", "me/emergency-contacts"),
    ("PUT", "me/emergency-contacts/{}"),
    ("DELETE", "me/emergency-contacts/{}"),
    ("POST", "me/leave-requests"),
    ("POST", "me/leave-requests/{}/cancel"),
    ("POST", "me/resignation"),
    ("POST", "me/resignation/withdraw"),
    ("PUT", "me/equality-monitoring"),
    ("DELETE", "me/equality-monitoring"),
    ("DELETE", "me/equality-monitoring/{}"),
    ("POST", "me/flexible-working"),
    ("POST", "me/flexible-working/{}/withdraw"),
    ("POST", "me/flexible-working/{}/respond"),
    ("POST", "me/flexible-working/{}/appeal"),
];

/// Routes whose write passes the policy's *action* check for a token that carries an attribute,
/// where the policy has no way to name the role: `(attribute, method, route under /api)`. The
/// handler still checks the attribute itself, and a caller without it falls through to the ordinary
/// policy (which for these routes the handler then refuses). Today only occupational health, who
/// record a worker's status against a workplace health requirement (WPM-R128).
pub const ATTRIBUTE_WRITES: &[(&str, &str, &str)] = &[
    (
        "occupational_health",
        "PUT",
        "workers/{}/health-requirements/{}",
    ),
    (
        "occupational_health",
        "DELETE",
        "workers/{}/health-requirements/{}",
    ),
];

/// The attribute that lets this request past the action check, if it is on the attribute list.
#[must_use]
pub fn attribute_for_write(method: &str, path: &str) -> Option<&'static str> {
    let path = path.trim_end_matches('/');
    let rest = path.strip_prefix("/api/")?;
    ATTRIBUTE_WRITES
        .iter()
        .find(|(_, m, pattern)| {
            m.eq_ignore_ascii_case(method) && crate::rules::access::matches(pattern, rest)
        })
        .map(|(attribute, _, _)| *attribute)
}

/// Whether this request is on the list.
#[must_use]
pub fn is_write(method: &str, path: &str) -> bool {
    let path = path.trim_end_matches('/');
    let Some(rest) = path.strip_prefix("/api/") else {
        return false;
    };
    WRITES.iter().any(|(m, pattern)| {
        m.eq_ignore_ascii_case(method) && crate::rules::access::matches(pattern, rest)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_listed_routes_and_methods_pass() {
        assert!(is_write("PUT", "/api/me/contact-details"));
        assert!(is_write("put", "/api/me/contact-details/"));
        assert!(is_write("POST", "/api/me/emergency-contacts"));
        assert!(is_write(
            "DELETE",
            "/api/me/emergency-contacts/0c4f1e2a-0000-4000-8000-000000000001"
        ));
        // The wrong method, a longer path, another prefix, or the HR route do not.
        assert!(!is_write("POST", "/api/me/contact-details"));
        assert!(!is_write("PUT", "/api/me/contact-details/extra"));
        assert!(!is_write("PUT", "/api/me/emergency-contacts"));
        assert!(!is_write(
            "PUT",
            "/api/workers/0c4f1e2a-0000-4000-8000-000000000001/contact-details"
        ));
        assert!(!is_write(
            "POST",
            "/api/workers/0c4f1e2a-0000-4000-8000-000000000001/emergency-contacts"
        ));
        assert!(!is_write("GET", "/api/me/contact-details"));
        assert!(
            !is_write("PUT", "/me/contact-details"),
            "needs the /api prefix"
        );
        assert!(!is_write("PUT", "/api/me//contact-details"));
    }

    #[test]
    fn every_entry_is_under_me() {
        for (method, route) in WRITES {
            assert!(route.starts_with("me/"), "{route}");
            assert!(matches!(*method, "POST" | "PUT" | "DELETE"), "{method}");
        }
    }

    /// Every write route registered under `/api/me` is on the list, and every entry is real.
    #[test]
    fn the_list_is_exactly_the_write_routes_under_me() {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/controllers/me.rs"),
        )
        .expect("the /api/me controller");
        let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let (_, after) = flat.split_once(".prefix(\"/api\")").expect("prefix");
        let mut registered: Vec<(String, String)> = Vec::new();
        for call in after.split(".add(").skip(1) {
            let Some(call) = call.trim_start().strip_prefix('"') else {
                continue;
            };
            let Some((route, rest)) = call.split_once('"') else {
                continue;
            };
            let call_text = rest.split("))").next().unwrap_or(rest);
            for (verb, token) in [("POST", "post("), ("PUT", "put("), ("DELETE", "delete(")] {
                if call_text.contains(token) {
                    registered.push((verb.to_string(), route.trim_start_matches('/').to_string()));
                }
            }
        }
        assert!(
            !registered.is_empty(),
            "the scan found no write routes: it is broken"
        );
        let concrete = |route: &str| {
            route
                .split('/')
                .map(|s| if s.starts_with('{') { "x" } else { s })
                .collect::<Vec<_>>()
                .join("/")
        };
        for (verb, route) in &registered {
            assert!(
                is_write(verb, &format!("/api/{}", concrete(route))),
                "{verb} {route} is registered under /api/me but is not on the self-service list"
            );
        }
        for (verb, pattern) in WRITES {
            assert!(
                registered
                    .iter()
                    .any(|(v, r)| v == verb && crate::rules::access::matches(pattern, &concrete(r))),
                "{verb} {pattern} is on the list but no such route is registered"
            );
        }
    }
}
