//! Pure rules for **groups**: communities of practice (people who share a
//! craft), communities of interest (people who share a topic), and other
//! self-organised sets. A worker can belong to many at once.
//!
//! Membership is a statement of who is in, not an assessment of anyone.

/// What a group is for.
pub const KINDS: &[&str] = &["practice", "interest", "other"];

/// A person's part in a group.
pub const ROLES: &[&str] = &["member", "lead"];

/// The longest accepted name.
pub const MAX_NAME_LEN: usize = 120;

/// The longest accepted description.
pub const MAX_DESCRIPTION_LEN: usize = 1000;

/// Validate a group's fields.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_group(name: &str, kind: &str, description: Option<&str>) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("name is required".to_string());
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(format!("name is longer than {MAX_NAME_LEN} characters"));
    }
    if !KINDS.contains(&kind) {
        return Err(format!("kind must be one of {}", KINDS.join(", ")));
    }
    if description.is_some_and(|d| d.chars().count() > MAX_DESCRIPTION_LEN) {
        return Err(format!(
            "description is longer than {MAX_DESCRIPTION_LEN} characters"
        ));
    }
    Ok(())
}

/// Validate a membership role.
///
/// # Errors
/// A message listing the choices.
pub fn validate_role(role: &str) -> Result<(), String> {
    if ROLES.contains(&role) {
        Ok(())
    } else {
        Err(format!("role must be one of {}", ROLES.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_are_validated() {
        assert!(validate_group("Rust guild", "practice", Some("weekly")).is_ok());
        assert!(validate_group("  ", "practice", None).is_err());
        assert!(validate_group(&"x".repeat(121), "interest", None).is_err());
        assert!(validate_group("Chess", "club", None).is_err());
        assert!(validate_group("Chess", "other", Some(&"x".repeat(1001))).is_err());
    }

    #[test]
    fn roles_are_validated() {
        assert!(validate_role("lead").is_ok() && validate_role("member").is_ok());
        assert!(validate_role("owner").is_err());
    }
}
