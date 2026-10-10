//! Pure rules for **skills matching and internal mobility** (WPM-R37,
//! WPM-D28). Matching is *employee-facing*: a worker sees which roles fit
//! **their own** declared skills and may express interest. Nothing here
//! ranks people against each other or selects anyone for an outcome — the
//! only ordering is of roles, for one worker, by that worker's own fit.

use std::cmp::Ordering;

use crate::rules::gap::Readiness;

/// The most text one interest note may hold.
pub const MAX_NOTE_LEN: usize = 500;

/// `(numerator, denominator)` a role is ordered by: critical
/// requirements met when the role has any, otherwise all requirements.
/// `None` when the role has no requirements at all — there is no
/// evidence to rank on.
#[must_use]
pub fn fit_score(readiness: &Readiness) -> Option<(usize, usize)> {
    if readiness.total == 0 {
        None
    } else if readiness.critical_total > 0 {
        Some((readiness.critical_met, readiness.critical_total))
    } else {
        Some((readiness.met, readiness.total))
    }
}

/// Order two roles for one worker: better fit first (critical, then
/// overall, compared by cross-multiplication so no float is involved),
/// and roles with no requirements last.
#[must_use]
pub fn compare_fit(a: &Readiness, b: &Readiness) -> Ordering {
    match (fit_score(a), fit_score(b)) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some((an, ad)), Some((bn, bd))) => (bn * ad)
            .cmp(&(an * bd))
            .then_with(|| (b.met * a.total).cmp(&(a.met * b.total))),
    }
}

/// Which target an interest points at.
#[derive(Debug, PartialEq, Eq)]
pub enum Target {
    /// A role profile.
    Role,
    /// An open requisition.
    Requisition,
}

/// Exactly one of role / requisition must be named.
///
/// # Errors
/// A message when neither or both are given.
pub fn interest_target(has_role: bool, has_requisition: bool) -> Result<Target, String> {
    match (has_role, has_requisition) {
        (true, false) => Ok(Target::Role),
        (false, true) => Ok(Target::Requisition),
        _ => Err("name exactly one of role_profile_pid or requisition_pid".to_string()),
    }
}

/// Whether two job titles are the same role (trimmed, case-insensitive).
#[must_use]
pub fn same_title(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(critical_met: usize, critical_total: usize, met: usize, total: usize) -> Readiness {
        Readiness {
            critical_met,
            critical_total,
            met,
            total,
        }
    }

    /// A role ordered by critical fit first, then overall; no-requirement
    /// roles sort last because there is no evidence.
    #[test]
    fn roles_order_by_the_workers_own_fit() {
        let strong = r(2, 2, 3, 5);
        let partial = r(1, 2, 5, 5);
        let none = r(0, 0, 0, 0);
        assert_eq!(
            compare_fit(&strong, &partial),
            Ordering::Less,
            "critical beats overall"
        );
        assert_eq!(compare_fit(&partial, &strong), Ordering::Greater);
        assert_eq!(compare_fit(&none, &strong), Ordering::Greater);
        assert_eq!(compare_fit(&strong, &none), Ordering::Less);
        assert_eq!(compare_fit(&none, &none), Ordering::Equal);
        // Equal critical fit falls back to overall.
        assert_eq!(compare_fit(&r(1, 1, 4, 4), &r(1, 1, 2, 4)), Ordering::Less);
        // No critical requirements: ordered on all requirements.
        assert_eq!(compare_fit(&r(0, 0, 3, 4), &r(0, 0, 1, 4)), Ordering::Less);
    }

    /// Fractions compare exactly, not by rounding (2/3 > 3/5).
    #[test]
    fn fractions_compare_exactly() {
        assert_eq!(compare_fit(&r(2, 3, 2, 3), &r(3, 5, 3, 5)), Ordering::Less);
    }

    /// Exactly one target is accepted.
    #[test]
    fn exactly_one_target() {
        assert_eq!(interest_target(true, false), Ok(Target::Role));
        assert_eq!(interest_target(false, true), Ok(Target::Requisition));
        assert!(interest_target(true, true).is_err());
        assert!(interest_target(false, false).is_err());
    }

    #[test]
    fn titles_match_loosely() {
        assert!(same_title(" Nurse ", "nurse"));
        assert!(!same_title("Nurse", "Doctor"));
    }
}
