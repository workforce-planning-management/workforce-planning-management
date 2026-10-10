//! Pure rules for **merging two catalogue skills** into one (a duplicate or a
//! near-synonym): how two records about the same person, role, or plan
//! reconcile when both skills already appear there. The merge keeps the
//! stronger statement and loses no information a planner would miss.
//!
//! DB-free; the controller repoints rows and applies these where both exist.

use crate::rules::roles::IMPORTANCES;

/// The proficiency to keep when a worker declared both skills: the higher.
#[must_use]
pub fn merge_level(a: i32, b: i32) -> i32 {
    a.max(b)
}

/// The target level to keep: the higher when both are set, else whichever
/// is.
#[must_use]
pub fn merge_target(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, None) => x,
        (None, y) => y,
    }
}

/// The stronger of two requirement importances (`critical` > `important` >
/// `useful`); an unknown token loses to a known one.
#[must_use]
pub fn stronger_importance<'a>(a: &'a str, b: &'a str) -> &'a str {
    let rank = |v: &str| {
        IMPORTANCES
            .iter()
            .position(|i| *i == v)
            .unwrap_or(IMPORTANCES.len())
    };
    if rank(b) < rank(a) { b } else { a }
}

/// Where a skill is in use — a skill in use cannot simply be deleted.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Usage {
    /// Live worker declarations.
    pub declared_by: usize,
    /// Role-profile requirements.
    pub required_by_profiles: usize,
    /// Development-plan items.
    pub in_development_plans: usize,
    /// Change-initiative skill shifts.
    pub in_initiatives: usize,
}

impl Usage {
    /// Total uses.
    #[must_use]
    pub fn total(&self) -> usize {
        self.declared_by
            + self.required_by_profiles
            + self.in_development_plans
            + self.in_initiatives
    }

    /// Whether the skill can be deleted outright: nothing uses it.
    #[must_use]
    pub fn deletable(&self) -> bool {
        self.total() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_and_targets_keep_the_higher() {
        assert_eq!(merge_level(2, 4), 4);
        assert_eq!(merge_level(5, 3), 5);
        assert_eq!(merge_target(Some(3), Some(5)), Some(5));
        assert_eq!(merge_target(Some(3), None), Some(3));
        assert_eq!(merge_target(None, Some(4)), Some(4));
        assert_eq!(merge_target(None, None), None);
    }

    #[test]
    fn importance_keeps_the_stronger() {
        assert_eq!(stronger_importance("useful", "critical"), "critical");
        assert_eq!(stronger_importance("critical", "important"), "critical");
        assert_eq!(stronger_importance("important", "useful"), "important");
        assert_eq!(stronger_importance("useful", "useful"), "useful");
        assert_eq!(
            stronger_importance("???", "useful"),
            "useful",
            "unknown loses to known"
        );
    }

    #[test]
    fn usage_decides_deletability() {
        assert!(Usage::default().deletable());
        let used = Usage {
            declared_by: 1,
            ..Usage::default()
        };
        assert!(!used.deletable());
        assert_eq!(
            Usage {
                declared_by: 2,
                required_by_profiles: 3,
                in_development_plans: 1,
                in_initiatives: 4
            }
            .total(),
            10
        );
    }
}
