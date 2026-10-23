//! Every route the service serves is called, with its method, by at least one test.
//!
//! DB-free: the route list comes from `app::app_routes()` and the calls from reading the
//! test sources. A call is `.get(`, `.post(`, `.put(`, `.patch(` or `.delete(` (or the bare
//! helper form the enforcement suites use) with a string literal in its first argument.
//! A literal matches a route when its trailing path segments match the route's, where a
//! `{name}` in either side matches any one segment.
//!
//! Where a test builds the whole URL in a variable the literal is not in the call, so the
//! route is listed in [`BUILT_FROM_A_VARIABLE`] with the test that calls it. That list may
//! only shrink: an entry that now matches by literal, or names no route, fails the test.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use workforce_planning_management::app::app_routes;

/// Routes called by a test that builds the URL in a variable, so the literal scan cannot see
/// them: `(METHOD, route, the test that calls it)`.
const BUILT_FROM_A_VARIABLE: &[(&str, &str, &str)] = &[
    (
        "PUT",
        "/api/health-requirements/{pid}",
        "enforcement_health.rs: the `restate` closure",
    ),
    (
        "DELETE",
        "/api/health-requirements/{pid}",
        "enforcement_health.rs: the `retire` closure",
    ),
    (
        "POST",
        "/api/announcements/{pid}/read",
        "requests/announcements.rs: the `read_url` closure",
    ),
    (
        "POST",
        "/api/early-warnings/sweep",
        "enforcement_oversight_pack.rs: a (path, body) table",
    ),
    (
        "POST",
        "/api/flexible-working/{pid}/decide",
        "enforcement_flexible.rs: the `decide` closure",
    ),
    (
        "POST",
        "/api/me/leave-requests",
        "enforcement_me.rs: the `mine` helper",
    ),
    (
        "POST",
        "/api/me/leave-requests/{pid}/cancel",
        "enforcement_me.rs: the `mine` helper",
    ),
    (
        "GET",
        "/api/movements/{pid}/handover/actions",
        "requests/movements.rs: `{handover}/actions`",
    ),
    (
        "POST",
        "/api/movements/{pid}/handover/all",
        "requests/movements.rs: `{handover}/all`",
    ),
    (
        "POST",
        "/api/workers/{pid}/framework-roles/{framework}/past",
        "requests/learning.rs: `{role_url}/past`",
    ),
];

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n != "fixtures" && n != "keycloak")
            {
                sources(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs")
            && path.file_name().is_some_and(|n| n != "route_coverage.rs")
        {
            out.push(path);
        }
    }
}

/// The path segments of a URL literal: query dropped, each `{...}` one `{}` segment.
fn segments(url: &str) -> Vec<String> {
    url.split('?')
        .next()
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.contains('{') {
                "{}".to_string()
            } else {
                s.to_string()
            }
        })
        .collect()
}

/// The first string literal of an argument list that starts with one, allowing a leading `&`
/// and `format!(`.
fn literal_at(args: &str) -> Option<String> {
    let args = args.trim_start();
    let args = args.strip_prefix('&').unwrap_or(args).trim_start();
    let args = args.strip_prefix("format!(").map_or(args, str::trim_start);
    let rest = args.strip_prefix('"')?;
    Some(rest[..rest.find('"')?].to_string())
}

fn ident_at(args: &str) -> Option<&str> {
    let args = args.trim_start();
    let args = args.strip_prefix('&').unwrap_or(args);
    let end = args
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(args.len());
    (end > 0).then(|| &args[..end])
}

/// `let url = format!("/api/...")` / `let url = "/api/..."`: every URL literal a name is bound to
/// anywhere in the file. A name bound to several URLs counts as all of them.
fn bindings(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = text[from..].find("let ") {
        let start = from + at + 4;
        from = start;
        let rest = text[start..].trim_start();
        let rest = rest.strip_prefix("mut ").unwrap_or(rest);
        let Some(name) = ident_at(rest) else { continue };
        let after = rest[name.len()..].trim_start();
        let Some(value) = after.strip_prefix('=') else {
            continue;
        };
        if let Some(literal) = literal_at(value)
            && literal.starts_with('/')
        {
            out.push((name.to_string(), literal));
        }
    }
    // `for url in ["/api/a", "/api/b"] { request.get(url) }`: the loop name stands for each.
    let mut from = 0;
    while let Some(at) = text[from..].find("for ") {
        let start = from + at + 4;
        from = start;
        let Some(name) = ident_at(&text[start..]) else {
            continue;
        };
        let Some(list) = text[start + name.len()..].trim_start().strip_prefix("in [") else {
            continue;
        };
        let Some(end) = list.find(']') else { continue };
        let mut rest = &list[..end];
        while let Some(open) = rest.find('"') {
            let Some(len) = rest[open + 1..].find('"') else {
                break;
            };
            let literal = &rest[open + 1..open + 1 + len];
            if literal.starts_with('/') {
                out.push((name.to_string(), literal.to_string()));
            }
            rest = &rest[open + 2 + len..];
        }
    }
    out
}

/// `(method, literal)` for every call whose first argument is a string literal (including
/// `&format!("...")`) or a name bound to one in the same file.
fn calls(text: &str) -> Vec<(&'static str, String)> {
    let bytes = text.as_bytes();
    let names = bindings(text);
    let mut found = Vec::new();
    for method in METHODS {
        let needle = format!("{method}(");
        let mut from = 0;
        while let Some(at) = text[from..].find(&needle) {
            let start = from + at;
            from = start + needle.len();
            let before = start.checked_sub(1).map(|i| bytes[i]);
            if before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
                continue; // `forget(`, `budget(`: not a request call
            }
            let args = &text[from..];
            if let Some(literal) = literal_at(args) {
                found.push((method, literal));
            } else if let Some(name) = ident_at(args) {
                for (bound, literal) in &names {
                    if bound == name {
                        found.push((method, literal.clone()));
                    }
                }
            }
        }
    }
    found
}

fn matches(route: &[String], literal: &[String], rooted: bool) -> bool {
    if rooted {
        // A rooted literal names the whole path.
        return literal.len() == route.len() && same(route, literal);
    }
    // `{base}/handover/actions`: the variable stands for an unknown prefix, so only the
    // segments after it count, and two are needed before a tail claims a route.
    let tail = &literal[literal.iter().take_while(|s| *s == "{}").count()..];
    tail.len() >= 2 && tail.len() <= route.len() && same(&route[route.len() - tail.len()..], tail)
}

fn same(route: &[String], literal: &[String]) -> bool {
    route
        .iter()
        .zip(literal)
        .all(|(a, b)| a == "{}" || b == "{}" || a == b)
}

#[test]
fn every_route_is_called_with_its_method_by_some_test() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        &mut files,
    );
    let mut seen: Vec<(&str, Vec<String>, bool)> = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap();
        for (method, literal) in calls(&text) {
            if literal.starts_with('/') || literal.starts_with('{') {
                seen.push((method, segments(&literal), literal.starts_with('/')));
            }
        }
    }

    let mut routes = BTreeSet::new();
    for route in app_routes().collect() {
        for action in &route.actions {
            routes.insert((action.as_str().to_ascii_lowercase(), route.uri.clone()));
        }
    }
    assert!(
        routes.len() > 400,
        "the route list was not read: {}",
        routes.len()
    );

    let allowed: BTreeSet<(String, String)> = BUILT_FROM_A_VARIABLE
        .iter()
        .map(|(m, r, _)| (m.to_ascii_lowercase(), (*r).to_string()))
        .collect();

    let mut uncalled = Vec::new();
    let mut stale = Vec::new();
    for (method, uri) in &routes {
        let route = segments(uri);
        let by_literal = seen
            .iter()
            .any(|(m, lit, rooted)| m == method && matches(&route, lit, *rooted));
        match (by_literal, allowed.contains(&(method.clone(), uri.clone()))) {
            (false, false) => uncalled.push(format!("{} {uri}", method.to_ascii_uppercase())),
            (true, true) => stale.push(format!(
                "{} {uri} now matches by literal",
                method.to_ascii_uppercase()
            )),
            _ => {}
        }
    }
    for (m, r, _) in BUILT_FROM_A_VARIABLE {
        if !routes.contains(&(m.to_ascii_lowercase(), (*r).to_string())) {
            stale.push(format!("{m} {r} is not a route"));
        }
    }
    assert!(
        uncalled.is_empty(),
        "{} routes no test calls (add a request test, or list the route in BUILT_FROM_A_VARIABLE \
         with the test that calls it):\n  {}",
        uncalled.len(),
        uncalled.join("\n  ")
    );
    assert!(
        stale.is_empty(),
        "stale BUILT_FROM_A_VARIABLE entries:\n  {}",
        stale.join("\n  ")
    );
}

#[test]
fn the_literal_matcher_follows_its_own_rules() {
    let route = segments("/api/movements/{pid}/handover/actions");
    let yes = |url: &str| matches(&route, &segments(url), url.starts_with('/'));
    assert!(yes("/api/movements/{mv}/handover/actions"));
    assert!(yes("/api/movements/abc/handover/actions?limit=3"));
    assert!(
        yes("{base}/handover/actions"),
        "a tail of two static segments"
    );
    assert!(
        !yes("{base}/actions"),
        "one static segment is too little to claim a route"
    );
    assert!(!yes("/api/movements/{mv}/handover"), "a different route");
    assert!(
        !yes("/api/handover/actions"),
        "a rooted literal is the whole path"
    );
    assert_eq!(
        calls("request.post(&format!(\"/api/x/{id}\"))"),
        [("post", "/api/x/{id}".to_string())]
    );
    assert!(calls("budget(\"/api/x\")").is_empty());
    assert_eq!(
        calls("let url = format!(\"/api/y/{id}\");\nrequest.put(&url).await"),
        [("put", "/api/y/{id}".to_string())],
        "a name bound to a URL stands for it"
    );
    assert!(calls("let n = 3;\nrequest.put(&n)").is_empty());
}
