//! The personal-data inventory (WPM-T353): every table that is keyed to a person is in the subject-access export, and is either
//! erased or on the retention schedule, or is named here with the reason it is not. A new table that holds personal data cannot be
//! added without one of the two, which is the rule in `AGENTS/privacy-and-data-rules.md` made checkable.
//!
//! This reads the migrations and `src/controllers/privacy.rs` (no database), and keeps `spec/governance/personal-data-inventory.md`
//! current: run `WPM_WRITE_INVENTORY=1 cargo test --test personal_data_inventory` to rewrite it.
//!
//! What it **cannot** see: a person-keyed table whose link is not one of the column names below, personal data in a column of a table
//! that is not keyed to a person, and whether an export or an erasure statement does the right thing. A reviewer still reads those.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use workforce_planning_management::rules::privacy::kind_of;

/// Columns that tie a row to a person. A table with one of these is in the inventory.
const PERSON_COLUMNS: &[&str] = &[
    "worker_pid",
    "person_ref",
    "holder_worker_pid",
    "from_worker",
    "to_worker",
    "requester_pid",
    "taker_pid",
    "backup_pid",
    "assignee_pid",
    "candidate_pid",
    "report_pid",
    "manager_pid",
    "rater_pid",
];

/// Person-keyed tables that are **not** in the subject-access export, each with the reason. A table leaves this list when it is
/// exported (the test fails if one is listed and is in the export).
const NOT_EXPORTED: &[(&str, &str)] = &[
    (
        "erasure_ledger",
        "Records only that an erasure happened (a pid and a time, WPM-D63); it is not data about the person.",
    ),
    (
        "succession_candidates",
        "Readiness and rank are withheld from the person by design (WPM-D128); a redacted view of the person's own entry is not built yet. Erased with the worker.",
    ),
    (
        "movement_items",
        "A task assigned to a worker about someone else's move; the worker's own joiner and leaver record is exported. Erasure unassigns it.",
    ),
    (
        "appraisal_nominations",
        "Says who was asked to rate whom. Raters' answers are third-party content withheld from the subject by design, and a nominee's export carries the report aggregate instead.",
    ),
    (
        "applications",
        "Applicants are data subjects too, but there is no subject-access route for them yet: a gap (WPM-T353). Removed by the retention horizon.",
    ),
    (
        "candidates",
        "As for applications: no subject-access route for applicants yet (WPM-T353). Removed when consent expires and by the retention horizon.",
    ),
];

/// Person-keyed tables that are neither erased by a statement nor on the retention schedule, each with the reason.
const NOT_ERASED_OR_RETAINED: &[(&str, &str)] = &[
    (
        "appraisal_nominations",
        "The row (that a person was asked to rate) stays with the appraisal it belongs to; erasure scrubs the rater's free-text comments on it. Not yet on the retention schedule: a gap (WPM-T353).",
    ),
    (
        "erasure_ledger",
        "Must stay: it is what makes a restore erase the person again (WPM-D63). It holds a pid and a time.",
    ),
];

struct Table {
    name: String,
    linked_by: Vec<&'static str>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Tables created by raw SQL in the migrations, with the person columns each declares.
fn person_tables() -> Vec<Table> {
    let mut seen: BTreeMap<String, Vec<&'static str>> = BTreeMap::new();
    let mut dir: Vec<_> = fs::read_dir(root().join("migration/src"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    dir.sort();
    for path in dir {
        let src = fs::read_to_string(&path).unwrap();
        let mut rest = src.as_str();
        while let Some(i) = rest.find("CREATE TABLE IF NOT EXISTS ") {
            rest = &rest[i + "CREATE TABLE IF NOT EXISTS ".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let body_start = rest.find('(').unwrap_or(0);
            let mut depth = 0;
            let mut end = rest.len();
            for (k, ch) in rest[body_start..].char_indices() {
                if ch == '(' {
                    depth += 1;
                }
                if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end = body_start + k;
                        break;
                    }
                }
            }
            let body = &rest[body_start..end];
            let linked: Vec<&'static str> = PERSON_COLUMNS
                .iter()
                .copied()
                .filter(|c| {
                    body.lines().any(|l| {
                        l.trim_start()
                            .strip_prefix(c)
                            .is_some_and(|r| r.starts_with(char::is_whitespace))
                    })
                })
                .collect();
            if !linked.is_empty() {
                seen.entry(name).or_insert(linked);
            }
        }
    }
    seen.into_iter()
        .map(|(name, linked_by)| Table { name, linked_by })
        .collect()
}

fn privacy_source() -> String {
    fs::read_to_string(root().join("src/controllers/privacy.rs")).unwrap()
}

fn exported(source: &str, table: &str) -> bool {
    source.contains(&format!("rows_for!(db, {table},"))
        || source.contains(&format!("rows_for!(\n            db,\n            {table},"))
        || source.contains(&format!("\"{table}\":"))
        || (table == "workers" && source.contains("\"worker\": worker"))
        // `rows_for!` split over lines by rustfmt: `rows_for!(db, <table>, Column, epid)` after a wrapped key.
        || regex_like(source, table)
}

/// `rows_for!(` followed by whitespace, `db,`, whitespace and the table name, which rustfmt produces for a long line.
fn regex_like(source: &str, table: &str) -> bool {
    source.match_indices("rows_for!(").any(|(i, _)| {
        let after = source[i + "rows_for!(".len()..].trim_start();
        after
            .strip_prefix("db,")
            .map(str::trim_start)
            .is_some_and(|r| r.starts_with(&format!("{table},")))
    })
}

fn erased(source: &str, table: &str) -> bool {
    ["DELETE FROM ", "UPDATE "].iter().any(|verb| {
        source
            .match_indices(&format!("{verb}{table}"))
            .any(|(i, m)| {
                !source[i + m.len()..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            })
    })
}

fn reason<'a>(list: &'a [(&str, &str)], table: &str) -> Option<&'a str> {
    list.iter().find(|(t, _)| *t == table).map(|(_, r)| *r)
}

#[test]
fn every_person_keyed_table_is_exported_and_erased_or_retained_or_explained() {
    let source = privacy_source();
    let tables = person_tables();
    assert!(
        tables.len() > 50,
        "found only {} person-keyed tables",
        tables.len()
    );
    let mut bad = Vec::new();
    for t in &tables {
        let in_export = exported(&source, &t.name);
        let kept = erased(&source, &t.name) || kind_of(&t.name).is_some();
        if !in_export && reason(NOT_EXPORTED, &t.name).is_none() {
            bad.push(format!("{}: not in the subject-access export (add it to privacy.rs, or to NOT_EXPORTED with the reason)", t.name));
        }
        if in_export && reason(NOT_EXPORTED, &t.name).is_some() {
            bad.push(format!(
                "{}: is exported now; remove it from NOT_EXPORTED",
                t.name
            ));
        }
        if !kept && reason(NOT_ERASED_OR_RETAINED, &t.name).is_none() {
            bad.push(format!("{}: neither erased nor on the retention schedule (add a statement, add it to rules::privacy::kind_of, or to NOT_ERASED_OR_RETAINED with the reason)", t.name));
        }
        if kept && reason(NOT_ERASED_OR_RETAINED, &t.name).is_some() {
            bad.push(format!(
                "{}: is erased or retained now; remove it from NOT_ERASED_OR_RETAINED",
                t.name
            ));
        }
    }
    for (name, why) in NOT_EXPORTED.iter().chain(NOT_ERASED_OR_RETAINED) {
        assert!(why.len() > 20, "{name}: the reason is too short to be one");
        if !tables.iter().any(|t| t.name == *name) {
            bad.push(format!(
                "{name}: is listed but is not a person-keyed table any more"
            ));
        }
    }
    assert!(bad.is_empty(), "\n{}\n", bad.join("\n"));
}

fn inventory() -> String {
    let source = privacy_source();
    let mut out = String::from(
        "# Personal-data inventory\n\n\
         Every table that is keyed to a person, and what happens to it on a subject-access request and on erasure. **Generated** by\n\
         `workforce-planning-management-service-with-rust/tests/personal_data_inventory.rs` from the migrations and `src/controllers/privacy.rs`;\n\
         do not edit by hand. Rewrite it with `WPM_WRITE_INVENTORY=1 cargo test --test personal_data_inventory`; the test fails when it is out of date.\n\n\
         - **Export**: the table is in `GET /api/workers/{pid}/subject-access`.\n\
         - **Erasure**: a statement in `erasure_statements` deletes or scrubs it when a worker is erased (WPM-D22).\n\
         - **Retention**: the table is on the retention schedule ([retention-schedule.md](retention-schedule.md)); a soft-deleted row is removed for good after its horizon.\n\
         - A table with neither erasure nor retention, or no export, carries the reason in the last column.\n\n\
         **What this does not cover.** A table is listed because it has a column that names a person (`worker_pid`, `person_ref` and the others in the test). Personal data in a\n\
         free-text column of a table that is not keyed to a person, a table linked by some other name, and whether an export or erasure statement is *right* are for a reviewer.\n\
         Applicants (`candidates`, `applications`) have no subject-access route yet.\n\n\
         | Table | Linked by | Export | Erasure | Retention | Reason, where something is missing |\n\
         | --- | --- | --- | --- | --- | --- |\n",
    );
    for t in person_tables() {
        let ex = exported(&source, &t.name);
        let er = erased(&source, &t.name);
        let kind = kind_of(&t.name)
            .map_or_else(|| "none".to_string(), |k| format!("{k:?}").to_lowercase());
        let mut why = Vec::new();
        if let Some(r) = reason(NOT_EXPORTED, &t.name) {
            why.push(format!("Not exported: {r}"));
        }
        if let Some(r) = reason(NOT_ERASED_OR_RETAINED, &t.name) {
            why.push(format!("Not erased or retained: {r}"));
        }
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            t.name,
            t.linked_by
                .iter()
                .map(|c| format!("`{c}`"))
                .collect::<Vec<_>>()
                .join(", "),
            if ex { "yes" } else { "no" },
            if er { "yes" } else { "no" },
            kind,
            why.join(" "),
        ));
    }
    out
}

#[test]
fn the_inventory_document_is_current() {
    let path = root().join("../spec/governance/personal-data-inventory.md");
    let fresh = inventory();
    if std::env::var("WPM_WRITE_INVENTORY").is_ok() {
        fs::write(&path, &fresh).unwrap();
        return;
    }
    let have = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        have, fresh,
        "spec/governance/personal-data-inventory.md is out of date: run `WPM_WRITE_INVENTORY=1 cargo test --test personal_data_inventory`"
    );
}
