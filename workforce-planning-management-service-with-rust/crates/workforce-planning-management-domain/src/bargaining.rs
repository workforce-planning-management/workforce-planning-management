//! Recognised bodies and bargaining units (WPM-R102, WPM-D68). DB-free and clock-free.
//!
//! A body is a thing the **organization** recognises; a unit is the part of the workforce it covers, as rules over facts WPM already
//! holds. Whether a worker is *covered* is derived from those facts, never recorded, and the only thing said about it is a count with
//! a floor. Nothing here is about who belongs to a body or represents one, and nothing could be: there is no such input.

/// What a body is.
pub const BODY_KINDS: &[&str] = &["union", "works_council", "staff_forum"];

/// On what footing it is recognised.
pub const BASES: &[&str] = &["voluntary_agreement", "statutory", "other"];

/// The longest name of a body or unit.
pub const NAME_MAX: usize = 120;
/// The longest contact role or rule value.
pub const SHORT_MAX: usize = 80;

fn length_problem(name: &str, text: &str, max: usize) -> Option<String> {
    let n = text.trim().chars().count();
    (n == 0 || n > max).then(|| format!("{name} must be 1 to {max} characters"))
}

/// Problems with a body, one sentence each. The contact is a **role** ("Head of Employee Relations"), never a person.
#[must_use]
pub fn body_problems(name: &str, kind: &str, basis: &str, contact_role: &str) -> Vec<String> {
    let mut out: Vec<String> = [
        length_problem("name", name, NAME_MAX),
        length_problem("contact_role", contact_role, SHORT_MAX),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !BODY_KINDS.contains(&kind) {
        out.push(format!("kind must be one of: {}", BODY_KINDS.join(", ")));
    }
    if !BASES.contains(&basis) {
        out.push(format!("basis must be one of: {}", BASES.join(", ")));
    }
    out
}

/// The rules that define a unit. Each one set must match; at least one must be set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnitRule<'a> {
    /// The worker's department.
    pub department: Option<&'a str>,
    /// The worker's location.
    pub location: Option<&'a str>,
    /// The worker's employment type.
    pub employment_type: Option<&'a str>,
    /// The worker's job title.
    pub job_title: Option<&'a str>,
}

/// Problems with a unit: a name, at least one rule, and each rule a sensible length.
#[must_use]
pub fn unit_problems(name: &str, rule: &UnitRule<'_>) -> Vec<String> {
    let mut out: Vec<String> = length_problem("name", name, NAME_MAX).into_iter().collect();
    let set = [
        ("department", rule.department),
        ("location", rule.location),
        ("employment_type", rule.employment_type),
        ("job_title", rule.job_title),
    ];
    if set.iter().all(|(_, v)| v.is_none()) {
        out.push(
            "a unit needs at least one rule: department, location, employment_type or job_title"
                .to_string(),
        );
    }
    for (field, value) in set {
        if let Some(value) = value {
            out.extend(length_problem(field, value, SHORT_MAX));
        }
    }
    out
}

/// The facts about a worker a unit is defined over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facts<'a> {
    /// Department.
    pub department: &'a str,
    /// Location, when held.
    pub location: Option<&'a str>,
    /// Employment type.
    pub employment_type: &'a str,
    /// Job title.
    pub job_title: &'a str,
}

fn same(rule: &str, fact: &str) -> bool {
    rule.trim().to_lowercase() == fact.trim().to_lowercase()
}

/// Whether the unit's rules cover a worker with these facts: every rule set must match (ignoring case and edge spaces); a location
/// rule never matches a worker whose location is not held.
#[must_use]
pub fn covers(rule: &UnitRule<'_>, facts: &Facts<'_>) -> bool {
    let all_set = rule.department.is_some()
        || rule.location.is_some()
        || rule.employment_type.is_some()
        || rule.job_title.is_some();
    all_set
        && rule.department.is_none_or(|r| same(r, facts.department))
        && rule
            .location
            .is_none_or(|r| facts.location.is_some_and(|f| same(r, f)))
        && rule
            .employment_type
            .is_none_or(|r| same(r, facts.employment_type))
        && rule.job_title.is_none_or(|r| same(r, facts.job_title))
}

/// What is said about how many a unit covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Coverage {
    /// A count at or over the floor.
    Shown {
        /// Workers in work today whom the unit's rules cover.
        covered: usize,
    },
    /// Some, but under the floor: no count.
    Withheld,
    /// The rules cover no one in work (a rule that matches nothing, which the deployer should see).
    NoOne,
}

/// The coverage for `covered` workers, with `floor` the fewest that may be counted.
#[must_use]
pub fn coverage(covered: usize, floor: usize) -> Coverage {
    if covered == 0 {
        Coverage::NoOne
    } else if covered < floor.max(1) {
        Coverage::Withheld
    } else {
        Coverage::Shown { covered }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts<'static> {
        Facts {
            department: "Care",
            location: Some("Leeds"),
            employment_type: "permanent",
            job_title: "Carer",
        }
    }

    #[test]
    fn a_body_needs_a_known_kind_and_basis_and_a_contact_role() {
        assert_eq!(
            body_problems(
                "Staff Forum",
                "staff_forum",
                "voluntary_agreement",
                "Head of Employee Relations"
            ),
            Vec::<String>::new()
        );
        assert_eq!(body_problems("", "union", "statutory", "HR").len(), 1);
        assert_eq!(body_problems("X", "club", "statutory", "HR").len(), 1);
        assert_eq!(body_problems("X", "union", "custom", "HR").len(), 1);
        assert_eq!(body_problems("X", "union", "statutory", "  ").len(), 1);
        assert_eq!(
            body_problems("X", "union", "statutory", &"r".repeat(81)).len(),
            1
        );
        assert_eq!(
            body_problems("", "club", "custom", "").len(),
            4,
            "every problem is named"
        );
    }

    #[test]
    fn a_unit_needs_at_least_one_rule() {
        assert_eq!(unit_problems("Care staff", &UnitRule::default()).len(), 1);
        assert_eq!(
            unit_problems(
                "Care staff",
                &UnitRule {
                    department: Some("Care"),
                    ..UnitRule::default()
                }
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            unit_problems(
                "",
                &UnitRule {
                    department: Some("Care"),
                    ..UnitRule::default()
                }
            )
            .len(),
            1
        );
        assert_eq!(
            unit_problems(
                "U",
                &UnitRule {
                    job_title: Some(" "),
                    ..UnitRule::default()
                }
            )
            .len(),
            1
        );
    }

    #[test]
    fn every_rule_set_must_match_and_case_and_edge_spaces_do_not_matter() {
        let f = facts();
        assert!(covers(
            &UnitRule {
                department: Some(" care "),
                ..UnitRule::default()
            },
            &f
        ));
        assert!(covers(
            &UnitRule {
                department: Some("CARE"),
                location: Some("leeds"),
                ..UnitRule::default()
            },
            &f
        ));
        assert!(!covers(
            &UnitRule {
                department: Some("Care"),
                location: Some("York"),
                ..UnitRule::default()
            },
            &f
        ));
        assert!(!covers(
            &UnitRule {
                employment_type: Some("contractor"),
                ..UnitRule::default()
            },
            &f
        ));
        assert!(covers(
            &UnitRule {
                employment_type: Some("permanent"),
                job_title: Some("carer"),
                ..UnitRule::default()
            },
            &f
        ));
    }

    #[test]
    fn a_location_rule_never_matches_a_worker_with_no_location() {
        let mut f = facts();
        f.location = None;
        assert!(!covers(
            &UnitRule {
                location: Some("Leeds"),
                ..UnitRule::default()
            },
            &f
        ));
        assert!(
            covers(
                &UnitRule {
                    department: Some("Care"),
                    ..UnitRule::default()
                },
                &f
            ),
            "other rules still work"
        );
    }

    #[test]
    fn a_unit_with_no_rule_covers_no_one_rather_than_everyone() {
        assert!(!covers(&UnitRule::default(), &facts()));
    }

    #[test]
    fn coverage_is_a_count_only_from_the_floor() {
        assert_eq!(coverage(0, 10), Coverage::NoOne);
        assert_eq!(coverage(1, 10), Coverage::Withheld);
        assert_eq!(coverage(9, 10), Coverage::Withheld);
        assert_eq!(coverage(10, 10), Coverage::Shown { covered: 10 });
        assert_eq!(
            coverage(1, 0),
            Coverage::Shown { covered: 1 },
            "a floor of nought is a floor of one"
        );
    }
}
