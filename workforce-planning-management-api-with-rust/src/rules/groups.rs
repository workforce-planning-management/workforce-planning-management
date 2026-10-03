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

/// The fewest people a skill roll-up may describe. Below this, a level
/// distribution could point at one person, so it is withheld.
pub const MIN_ROLLUP: usize = 3;

/// One skill's roll-up across a group: how many members declare it and the
/// spread of their levels. Aggregate only; never names anyone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRollup {
    /// Members who have declared the skill.
    pub declared: usize,
    /// Of the group's members.
    pub members: usize,
    /// Members at each level 1–5 (index 0 is level 1).
    pub levels: [usize; 5],
}

impl SkillRollup {
    /// Share of members who declare it, or `None` for an empty group — never
    /// a zero standing in for "unknown".
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // member counts are tiny
    pub fn coverage(&self) -> Option<f64> {
        (self.members > 0).then(|| self.declared as f64 / self.members as f64)
    }
}

/// Roll declared `levels` (one per member who declares the skill, each 1–5)
/// up for a group of `members`. Out-of-range levels are ignored. Returns
/// `None` when fewer than [`MIN_ROLLUP`] members declare it (withheld), or the
/// group itself is smaller than the floor.
#[must_use]
pub fn rollup(members: usize, levels: &[i32]) -> Option<SkillRollup> {
    let mut counts = [0usize; 5];
    for level in levels {
        if let Some(slot) = usize::try_from(*level - 1)
            .ok()
            .and_then(|i| counts.get_mut(i))
        {
            *slot += 1;
        }
    }
    let declared: usize = counts.iter().sum();
    (members >= MIN_ROLLUP && declared >= MIN_ROLLUP).then_some(SkillRollup {
        declared,
        members,
        levels: counts,
    })
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
    fn rollups_are_aggregate_and_floored() {
        let r = rollup(5, &[2, 4, 4, 5]).unwrap();
        assert_eq!((r.declared, r.members), (4, 5));
        assert_eq!(r.levels, [0, 1, 0, 2, 1]);
        assert_eq!(r.coverage(), Some(0.8));
        assert!(
            rollup(5, &[3, 3]).is_none(),
            "two people could be identified"
        );
        assert!(rollup(2, &[3, 3, 3]).is_none(), "a group below the floor");
        assert_eq!(
            rollup(3, &[1, 9, 0, 3, 3]).unwrap().declared,
            3,
            "out-of-range ignored"
        );
        assert_eq!(
            SkillRollup {
                declared: 0,
                members: 0,
                levels: [0; 5]
            }
            .coverage(),
            None
        );
    }

    #[test]
    fn roles_are_validated() {
        assert!(validate_role("lead").is_ok() && validate_role("member").is_ok());
        assert!(validate_role("owner").is_err());
    }
}
