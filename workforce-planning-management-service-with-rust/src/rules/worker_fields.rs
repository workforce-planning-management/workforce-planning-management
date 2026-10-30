//! **Which fields of a worker a list returns** (WPM-R216, WPM-D147): pure, database-free and clock-free.
//!
//! `GET /api/workers?fields=pid,department,person_ref` lets a caller that needs a few fields of every worker (a bridge, such as CRM's
//! teams import) ask for those and no more. The names are an allowlist of the record's own fields; an unknown name is refused rather
//! than ignored, so a typo cannot quietly widen or empty the answer. The policy, the scope and the salary mask are unchanged: a field
//! asked for is still masked where the single read would mask it. Whether the answer **discloses salary** decides whether the list
//! audits a salary read, as the single read does.

use serde_json::{Map, Value};

/// The fields a caller may name, in the record's order. The serial `id` and `deleted_at` are not offered: the one is internal and the
/// other is always null on a listed row.
pub const LIST_FIELDS: [&str; 20] = [
    "pid",
    "person_ref",
    "upstream_worker_ref",
    "organization_ref",
    "worker_number",
    "display_name",
    "status",
    "employment_type",
    "fte_percent",
    "department",
    "job_title",
    "location",
    "manager_pid",
    "salary_minor",
    "salary_currency",
    "hired_on",
    "terminated_on",
    "engagement_ends_on",
    "created_at",
    "updated_at",
];

/// The fields that disclose pay. Either one asked for (or no `fields` at all) makes the answer a salary read.
pub const SALARY_FIELDS: [&str; 2] = ["salary_minor", "salary_currency"];

/// The most names `fields` may carry (more than [`LIST_FIELDS`] is a mistake, and the text is bounded).
const MAX_NAMES: usize = 64;

/// Parse `?fields=`: `None` (absent) is the whole record; otherwise a comma-separated list of names from [`LIST_FIELDS`], spaces
/// around a name ignored and a repeat kept once, in the order given.
///
/// # Errors
///
/// An empty list, an empty name, a name not on the list, or more than 64 names.
pub fn parse_fields(text: Option<&str>) -> Result<Option<Vec<&'static str>>, String> {
    let Some(text) = text else {
        return Ok(None);
    };
    let names: Vec<&str> = text.split(',').map(str::trim).collect();
    if names.len() > MAX_NAMES {
        return Err(format!("fields may name at most {MAX_NAMES} fields"));
    }
    let mut out: Vec<&'static str> = Vec::new();
    let mut unknown: Vec<&str> = Vec::new();
    for name in names {
        match LIST_FIELDS.iter().find(|f| **f == name) {
            Some(field) if !out.contains(field) => out.push(field),
            Some(_) => {}
            None => unknown.push(name),
        }
    }
    if !unknown.is_empty() {
        let shown: Vec<String> = unknown.iter().map(|n| format!("{n:?}")).collect();
        return Err(format!(
            "fields may name only {}; not {}",
            LIST_FIELDS.join(", "),
            shown.join(", ")
        ));
    }
    if out.is_empty() {
        return Err("fields must name at least one field".to_string());
    }
    Ok(Some(out))
}

/// Whether an answer with these fields discloses pay: the whole record does, and so does a list naming a salary field.
#[must_use]
pub fn includes_salary(fields: Option<&[&str]>) -> bool {
    fields.is_none_or(|names| names.iter().any(|n| SALARY_FIELDS.contains(n)))
}

/// Keep only `fields` of a serialized record. A field the record lacks is left out, not invented.
#[must_use]
pub fn project(record: &Map<String, Value>, fields: &[&str]) -> Map<String, Value> {
    fields
        .iter()
        .filter_map(|name| record.get(*name).map(|v| ((*name).to_string(), v.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn absent_is_the_whole_record_and_a_list_is_kept_in_order_once() {
        assert_eq!(parse_fields(None), Ok(None));
        assert_eq!(
            parse_fields(Some("pid,department,person_ref")),
            Ok(Some(vec!["pid", "department", "person_ref"]))
        );
        assert_eq!(
            parse_fields(Some(" pid , pid,department ")),
            Ok(Some(vec!["pid", "department"]))
        );
    }

    #[test]
    fn an_unknown_or_empty_name_is_refused_not_ignored() {
        for bad in [
            "",
            ",",
            "pid,",
            "pid,name",
            "id",
            "deleted_at",
            "PID",
            "salary",
        ] {
            assert!(parse_fields(Some(bad)).is_err(), "{bad:?}");
        }
        let message = parse_fields(Some("pid,nmae")).unwrap_err();
        assert!(
            message.contains("\"nmae\"") && message.contains("department"),
            "{message}"
        );
        let many = vec!["pid"; MAX_NAMES + 1].join(",");
        assert!(parse_fields(Some(&many)).is_err());
    }

    #[test]
    fn salary_is_disclosed_by_the_whole_record_or_a_salary_field_only() {
        assert!(includes_salary(None));
        assert!(includes_salary(Some(&["pid", "salary_minor"])));
        assert!(includes_salary(Some(&["salary_currency"])));
        assert!(!includes_salary(Some(&["pid", "department", "person_ref"])));
        assert!(!includes_salary(Some(&[])));
    }

    #[test]
    fn every_offered_field_is_a_field_of_the_record_and_the_salary_fields_are_offered() {
        let record = json!({
            "created_at": "t", "updated_at": "t", "id": 1, "pid": "p", "person_ref": "person:x", "upstream_worker_ref": null,
            "organization_ref": "organization:x", "worker_number": "1", "display_name": "n", "status": "active",
            "employment_type": "permanent", "fte_percent": 100, "department": "d", "job_title": "j", "location": null,
            "manager_pid": null, "salary_minor": 1, "salary_currency": "GBP", "hired_on": "2026-01-05", "terminated_on": null,
            "engagement_ends_on": null, "deleted_at": null,
        });
        let record = record.as_object().unwrap();
        for field in LIST_FIELDS {
            assert!(record.contains_key(field), "{field}");
        }
        assert!(SALARY_FIELDS.iter().all(|f| LIST_FIELDS.contains(f)));
        assert_eq!(
            record.len(),
            LIST_FIELDS.len() + 2,
            "only id and deleted_at are held back"
        );
    }

    #[test]
    fn a_projection_keeps_the_named_fields_in_order_and_nothing_else() {
        let record = json!({ "pid": "p", "department": "d", "person_ref": "person:x", "display_name": "n", "salary_minor": 1 });
        let kept = project(
            record.as_object().unwrap(),
            &["person_ref", "pid", "terminated_on"],
        );
        assert_eq!(
            Value::Object(kept),
            json!({ "person_ref": "person:x", "pid": "p" })
        );
    }
}
