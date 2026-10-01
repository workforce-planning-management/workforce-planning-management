//! Pure role→capability mapping for organization-scoped grant/revoke
//! (multi-organization, WPM-Rxx): "does holding `acting_role` in an
//! organization permit granting or revoking a membership of
//! `target_role` in that same organization". DB-free and
//! clock-free, per this crate's pure-core convention
//! (`crate::rules`) — the DB-touching half (whether the *caller*
//! actually holds `acting_role` in the org in question) lives in
//! [`crate::models::memberships`].
//!
//! Also the pure graph algorithms for **org confederation** (a parent
//! organization transitively containing member organizations,
//! WPM-Rxx): [`descendants_of`] and [`would_create_cycle`] operate on
//! a plain edge list — the DB-touching half (fetching the live edges)
//! lives in [`crate::models::confederations`].

/// Whether `acting_role`, held in an organization, is sufficient to
/// grant or revoke a `target_role` membership in that same
/// organization.
///
/// `org_admin` can manage every role, including other `org_admin`
/// grants. `hr_admin` can only manage the plain employment role
/// (`member`) — granting or revoking `hr_admin`/`payroll_admin`/
/// `org_admin` themselves requires `org_admin`, so an HR admin cannot
/// escalate a colleague (or themselves) to a more privileged role.
/// Every other role (`payroll_admin`, `member`, `viewer`) cannot
/// manage membership at all — holding a role is not the same as
/// being allowed to hand it out.
#[must_use]
pub fn can_manage_membership(acting_role: &str, target_role: &str) -> bool {
    match acting_role {
        "org_admin" => true,
        "hr_admin" => target_role == "member",
        _ => false,
    }
}

/// Every transitive descendant of `root` in the confederation graph
/// `edges` (each `(parent_organization_ref, child_organization_ref)`),
/// walked breadth-first so a cycle in bad data can't loop forever.
/// `root` itself is never included.
#[must_use]
pub fn descendants_of(edges: &[(String, String)], root: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    seen.insert(root.to_string());
    let mut frontier = vec![root.to_string()];
    let mut result = Vec::new();
    while let Some(node) = frontier.pop() {
        for (parent, child) in edges {
            if parent == &node && seen.insert(child.clone()) {
                result.push(child.clone());
                frontier.push(child.clone());
            }
        }
    }
    result
}

/// Whether declaring `parent → child` on top of the existing
/// confederation graph `edges` would create a cycle: true when
/// `parent == child` (a self-loop), or when `parent` is already a
/// descendant of `child` (so the new edge would close a loop back to
/// `child`).
#[must_use]
pub fn would_create_cycle(edges: &[(String, String)], parent: &str, child: &str) -> bool {
    parent == child || descendants_of(edges, child).iter().any(|d| d == parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn org_admin_manages_every_role() {
        for role in ["member", "hr_admin", "payroll_admin", "org_admin", "viewer"] {
            assert!(can_manage_membership("org_admin", role), "{role}");
        }
    }

    #[test]
    fn hr_admin_manages_only_plain_membership() {
        assert!(can_manage_membership("hr_admin", "member"));
        assert!(!can_manage_membership("hr_admin", "hr_admin"), "no self-escalation");
        assert!(!can_manage_membership("hr_admin", "org_admin"), "no escalation to org_admin");
        assert!(!can_manage_membership("hr_admin", "payroll_admin"));
    }

    #[test]
    fn other_roles_manage_nothing() {
        for acting in ["payroll_admin", "member", "viewer"] {
            for target in ["member", "hr_admin", "payroll_admin", "org_admin", "viewer"] {
                assert!(!can_manage_membership(acting, target), "{acting} -> {target}");
            }
        }
    }

    fn edge(parent: &str, child: &str) -> (String, String) {
        (parent.to_string(), child.to_string())
    }

    #[test]
    fn descendants_of_walks_multiple_levels() {
        let edges = [
            edge("a", "b"),
            edge("a", "c"),
            edge("b", "d"),
            edge("d", "e"),
        ];
        let mut descendants = descendants_of(&edges, "a");
        descendants.sort();
        assert_eq!(descendants, vec!["b", "c", "d", "e"]);
        // A leaf has no descendants.
        assert!(descendants_of(&edges, "e").is_empty());
    }

    #[test]
    fn descendants_of_ignores_unrelated_edges() {
        let edges = [edge("a", "b"), edge("x", "y")];
        assert_eq!(descendants_of(&edges, "a"), vec!["b".to_string()]);
    }

    #[test]
    fn descendants_of_survives_a_cycle_in_bad_data() {
        // Not a state `would_create_cycle` should ever let happen, but
        // the traversal must not hang if it does (e.g. pre-existing data
        // from before this check existed). `root` is seeded into the
        // seen-set up front, so the cycle back to it is silently
        // dropped rather than re-added — it terminates, and "root
        // itself is never included" holds even under a cycle.
        let edges = [edge("a", "b"), edge("b", "a")];
        assert_eq!(descendants_of(&edges, "a"), vec!["b".to_string()]);
    }

    #[test]
    fn would_create_cycle_rejects_self_loop() {
        assert!(would_create_cycle(&[], "a", "a"));
    }

    #[test]
    fn would_create_cycle_rejects_closing_an_existing_chain() {
        let edges = [edge("a", "b"), edge("b", "c")];
        // c -> a would close a -> b -> c -> a.
        assert!(would_create_cycle(&edges, "c", "a"));
        // A fresh, unrelated edge is fine.
        assert!(!would_create_cycle(&edges, "c", "d"));
        // Extending the chain further (c -> d, no path back to c) is fine.
        assert!(!would_create_cycle(&edges, "a", "c"));
    }
}
