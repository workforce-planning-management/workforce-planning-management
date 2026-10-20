//! One audit entry as one line of JSON for a security monitoring system (WPM-R122).
//!
//! The line carries what a monitor needs to see who did what to which record and when, and to prove nothing was
//! removed (the chain hashes). It does **not** carry the `snapshot`: that holds record content (a department, a status,
//! sometimes more) and has no place in a second system with its own access rules.

use serde_json::json;

use crate::models::_entities::audit_logs::Model;

/// The entry as a JSON object followed by a newline.
#[must_use]
pub fn line(entry: &Model) -> String {
    let mut text = json!({
        "chain_seq": entry.chain_seq,
        "at": entry.created_at.to_rfc3339(),
        "entity": entry.entity,
        "entity_pid": entry.entity_pid,
        "action": entry.action,
        "actor": entry.actor,
        "prev_hash": entry.prev_hash,
        "entry_hash": entry.entry_hash,
    })
    .to_string();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Model {
        Model {
            created_at: chrono::DateTime::parse_from_rfc3339("2026-10-20T09:30:00+00:00").unwrap(),
            updated_at: chrono::DateTime::parse_from_rfc3339("2026-10-20T09:30:00+00:00").unwrap(),
            id: 7,
            entity: "worker".into(),
            entity_pid: uuid::Uuid::nil(),
            action: "created".into(),
            actor: Some("user-1".into()),
            snapshot: Some(serde_json::json!({ "department": "secret-department" })),
            chain_seq: 3,
            prev_hash: "a".repeat(64),
            entry_hash: "b".repeat(64),
        }
    }

    #[test]
    fn a_line_is_one_json_object_ending_in_a_newline() {
        let text = line(&entry());
        assert!(text.ends_with('\n') && text.matches('\n').count() == 1);
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["chain_seq"], 3);
        assert_eq!(value["action"], "created");
        assert_eq!(value["actor"], "user-1");
        assert_eq!(value["at"], "2026-10-20T09:30:00+00:00");
    }

    #[test]
    fn the_snapshot_never_leaves() {
        let text = line(&entry());
        assert!(!text.contains("snapshot") && !text.contains("secret-department"));
    }
}
