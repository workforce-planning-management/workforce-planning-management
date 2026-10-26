//! Union membership and representative status are never stored, and nothing may reveal them (WPM-D68, WPM-T171).
//!
//! Membership of a trade union is special category data, and a past of blacklisting makes it dangerous to hold at all. So
//! no table, column, field, export, audit entry, label or kind of time may say who belongs to a union or who represents one.
//! This reads the source, the migrations, the entities, the API document and the English catalogue (no database), and
//! fails on a name or a phrase that would. The consultation rules and the recognised-body tables to come
//! (WPM-R102 to R108) name **bodies and units**, never people: they pass because nothing here is about a body.
//!
//! The deny-list is a floor. Adding a field that reveals membership under another name is still a breach of WPM-D68; the test
//! catches the obvious names, and a reviewer catches the rest (`AGENTS/privacy-and-data-rules.md`).

use std::fs;
use std::path::{Path, PathBuf};

/// Whole identifiers (split on anything that is not a letter, digit or underscore) that must not appear.
const DENIED_TOKENS: &[&str] = &[
    "steward",
    "stewards",
    "checkoff",
    "check_off",
    "union_id",
    "union_ref",
];

/// Parts of an identifier that must not appear in it.
const DENIED_PARTS: &[&str] = &[
    "union_member",
    "trade_union",
    "union_rep",
    "shop_steward",
    "union_dues",
    "union_subscription",
    "union_activity",
    "union_status",
    "union_duties",
    "representative_status",
    "is_representative",
    "is_steward",
];

/// Phrases a label, a message or a document must not contain (compared lower-case).
const DENIED_PHRASES: &[&str] = &[
    "union member",
    "member of a union",
    "member of the union",
    "trade union membership",
    "shop steward",
    "union rep",
    "union dues",
    "union subscription",
    "check-off",
    "representative status",
    "union representative",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every file under `dir` with one of `extensions`.
fn files(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, extensions, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.contains(&e))
        {
            out.push(path);
        }
    }
}

/// What in `text` breaks the rule: identifiers when `identifiers`, and phrases always.
fn breaches(text: &str, identifiers: bool) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut found: Vec<String> = DENIED_PHRASES
        .iter()
        .filter(|p| lower.contains(*p))
        .map(|p| format!("phrase `{p}`"))
        .collect();
    if identifiers {
        for token in lower
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .filter(|t| !t.is_empty())
        {
            if DENIED_TOKENS.contains(&token) {
                found.push(format!("name `{token}`"));
            }
            if let Some(part) = DENIED_PARTS.iter().find(|p| token.contains(*p)) {
                found.push(format!("name `{token}` (contains `{part}`)"));
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Check every file; one line per breach.
fn scan(paths: &[PathBuf], identifiers: bool) -> Vec<String> {
    let mut all = Vec::new();
    for path in paths {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for breach in breaches(&text, identifiers) {
            all.push(format!("{}: {breach}", path.display()));
        }
    }
    all
}

/// The guard can fail: names and phrases that would reveal membership are found, ordinary ones are not.
#[test]
fn the_guard_flags_what_it_should_and_nothing_else() {
    for bad in [
        "union_member_id",
        "worker_is_steward",
        "TradeUnionMember trade_union_id",
        "checkoff",
        "STEWARD",
    ] {
        assert!(!breaches(bad, true).is_empty(), "{bad} should be flagged");
    }
    for bad in [
        "Shop Steward",
        "a union member",
        "Check-off dues",
        "Representative status",
    ] {
        assert!(!breaches(bad, false).is_empty(), "{bad} should be flagged");
    }
    for fine in [
        "check_offset",
        "stewardship_of_data",
        "recognised_body",
        "bargaining_unit",
        "Trade Union and Labour Relations (Consolidation) Act 1992",
        "consult the union with a view to agreement",
        "headcount of a unit",
    ] {
        assert!(
            breaches(fine, true).is_empty(),
            "{fine} should pass: {:?}",
            breaches(fine, true)
        );
    }
}

/// The scan reaches the files it claims to; an empty directory would pass silently otherwise.
#[test]
fn the_guard_reads_what_it_should() {
    let mut sources = Vec::new();
    files(&root().join("src"), &["rs"], &mut sources);
    files(&root().join("migration/src"), &["rs"], &mut sources);
    assert!(
        sources.len() > 200,
        "expected the service and migration sources, found {}",
        sources.len()
    );
    assert!(
        sources
            .iter()
            .any(|p| p.ends_with("models/_entities/workers.rs")),
        "the worker entity is scanned"
    );
    assert!(
        root()
            .join("../workforce-planning-management-ui-with-svelte/content/locales/en-001/ui.json")
            .exists()
    );
    assert!(
        root()
            .join("../workforce-planning-management-ui-with-svelte/src/lib/api/openapi.json")
            .exists()
    );
}

/// No table, column, entity field, route, audit label, kind of time or deduction in the service names membership.
#[test]
fn the_service_and_its_schema_name_no_membership() {
    let mut sources = Vec::new();
    files(&root().join("src"), &["rs"], &mut sources);
    files(&root().join("migration/src"), &["rs"], &mut sources);
    files(&root().join("config"), &["yaml", "yml"], &mut sources);
    let found = scan(&sources, true);
    assert!(
        found.is_empty(),
        "WPM-D68 — membership and representative status are never stored or shown:\n{}",
        found.join("\n")
    );
}

/// The published API and the words people read say nothing about membership either.
#[test]
fn the_api_document_and_the_catalogue_say_nothing_about_membership() {
    let ui = root().join("../workforce-planning-management-ui-with-svelte");
    let mut docs = vec![ui.join("src/lib/api/openapi.json")];
    files(&ui.join("content/locales/en-001"), &["json"], &mut docs);
    let found = scan(&docs, true);
    assert!(
        found.is_empty(),
        "WPM-D68 — the API and its labels must not reveal membership:\n{}",
        found.join("\n")
    );
}

/// Facility time (WPM-R107) is minutes per **body** per period: its table has no column that could name or point at a person, and no
/// kind of leave or time names representative duties.
#[test]
fn facility_time_has_no_person_and_no_time_entry_kind() {
    let migration = fs::read_dir(root().join("migration/src"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.to_string_lossy().contains("facility_time"))
        .expect("the facility-time migration");
    let sql = fs::read_to_string(migration).unwrap().to_lowercase();
    let start = sql
        .find("create table if not exists facility_time_periods")
        .expect("the table");
    let table = &sql[start..sql[start..].find(")\",").map_or(sql.len(), |n| start + n)];
    for column in [
        "worker",
        "person",
        "representative",
        "employee",
        "user",
        "member",
        "steward",
    ] {
        assert!(
            !table.contains(column),
            "facility time is per body, never per person: `{column}` in\n{table}"
        );
    }
    for file in [
        "src/rules/leave.rs",
        "src/rules/time_off.rs",
        "src/rules/working_time.rs",
    ] {
        let text = fs::read_to_string(root().join(file))
            .unwrap()
            .to_lowercase();
        for word in ["facility", "representative", "union"] {
            assert!(
                !text.contains(word),
                "no kind of time or leave may name representative duties: `{word}` in {file}"
            );
        }
    }
}
