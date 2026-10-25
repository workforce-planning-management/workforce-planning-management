//! The shape of an organization now and as a plan intends it (WPM-R153), DB-free and clock-free. A **diff, not a record**: it
//! takes the live reporting structure and a target structure of *roles*, and says what changes: layers, span of control, and
//! roles added, removed, moved or resized. It names no person: the current side is reduced to roles and counts before anything
//! is compared, and a count under the floor is withheld. Spec: `spec/positions.md`.
//!
//! The target is made of **role nodes**, each with a headcount and a parent node, never of people. Cost and skills are not
//! compared here.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

/// A person in the live structure, reduced to what the shape needs. The key is only used to find the manager; it never leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    /// Who they are, for the manager link only.
    pub key: String,
    /// Their manager's key, when they have one among these people.
    pub manager: Option<String>,
    /// Their department.
    pub department: String,
    /// Their job title.
    pub title: String,
}

/// A role in the target structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The node's key, unique in the target.
    pub key: String,
    /// The parent node's key; absent for a top node.
    pub parent: Option<String>,
    /// The department.
    pub department: String,
    /// The role's title.
    pub title: String,
    /// How many people the plan puts in this role.
    pub headcount: i32,
}

/// Check a target structure: keys unique, parents present, no cycle, sizes in range.
///
/// # Errors
///
/// The problems found.
pub fn validate_target(nodes: &[Node]) -> Result<(), Vec<String>> {
    let mut out = Vec::new();
    if nodes.is_empty() {
        out.push("a target has at least one role".to_string());
    }
    if nodes.len() > 2_000 {
        out.push("a target has at most 2000 roles".to_string());
    }
    let mut keys = BTreeSet::new();
    for n in nodes {
        if !(1..=60).contains(&n.key.chars().count()) || n.key.chars().any(char::is_whitespace) {
            out.push(format!(
                "key {:?} is 1 to 60 characters with no spaces",
                n.key
            ));
        }
        if !keys.insert(n.key.as_str()) {
            out.push(format!("key {:?} is used twice", n.key));
        }
        if !(1..=120).contains(&n.title.trim().chars().count())
            || !(1..=80).contains(&n.department.trim().chars().count())
        {
            out.push(format!(
                "{}: title is 1 to 120 and department 1 to 80 characters",
                n.key
            ));
        }
        if !(1..=10_000).contains(&n.headcount) {
            out.push(format!("{}: headcount is 1 to 10000", n.key));
        }
    }
    let parent_of: HashMap<&str, &str> = nodes
        .iter()
        .filter_map(|n| Some((n.key.as_str(), n.parent.as_deref()?)))
        .collect();
    for n in nodes {
        if let Some(p) = &n.parent
            && !keys.contains(p.as_str())
        {
            out.push(format!(
                "{}: parent {p:?} is not a role of this target",
                n.key
            ));
        }
        // A walk up that does not end within the number of nodes is a cycle.
        let mut at = n.key.as_str();
        let mut steps = 0;
        while let Some(up) = parent_of.get(at) {
            at = up;
            steps += 1;
            if steps > nodes.len() {
                out.push(format!("{}: the parents loop back on themselves", n.key));
                break;
            }
        }
    }
    out.sort();
    out.dedup();
    if out.is_empty() { Ok(()) } else { Err(out) }
}

/// How wide the structure is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// Manager seats: people with reports now, or the headcount of nodes with children in the target.
    pub managers: i64,
    /// Mean reports per manager, in hundredths. Absent when there is no manager.
    pub mean_hundredths: Option<i64>,
    /// The largest number of reports to one manager (rounded up in the target).
    pub max: i64,
}

/// One side of the comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Side {
    /// Levels from the top to the deepest role.
    pub layers: usize,
    /// People now, or planned headcount.
    pub headcount: i64,
    /// Span of control.
    pub span: Span,
}

/// A count, or why it is not given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Count {
    /// A number.
    Shown {
        /// The count.
        value: i64,
    },
    /// A group of people smaller than the floor.
    Withheld,
}

/// What happens to a role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    /// Nobody has it now and the plan has it.
    Added,
    /// People have it now and the plan does not.
    Removed,
    /// More people in the plan.
    Larger,
    /// Fewer people in the plan.
    Smaller,
    /// The same number.
    Same,
}

/// A role across the two structures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Role {
    /// The title.
    pub title: String,
    /// People in it now: withheld when there are fewer than the floor.
    pub current: Count,
    /// People the plan puts in it.
    pub target: i64,
    /// What changes.
    pub change: Change,
    /// The change in people, absent while the current count is withheld.
    pub delta: Option<i64>,
    /// The role is in different departments now and in the plan.
    pub moved: bool,
    /// Departments it is in now.
    pub current_departments: Vec<String>,
    /// Departments it is in in the plan.
    pub target_departments: Vec<String>,
}

/// The comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diff {
    /// The live structure.
    pub current: Side,
    /// The target structure.
    pub target: Side,
    /// Roles, by title.
    pub roles: Vec<Role>,
}

fn key_of(title: &str) -> String {
    title.trim().to_lowercase()
}

/// Depths of the live people, each root at 1. A manager outside the set, or a loop, makes the person a root.
fn current_side(people: &[Person]) -> Side {
    let known: BTreeSet<&str> = people.iter().map(|p| p.key.as_str()).collect();
    let manager_of: HashMap<&str, &str> = people
        .iter()
        .filter_map(|p| {
            Some((
                p.key.as_str(),
                p.manager
                    .as_deref()
                    .filter(|m| known.contains(m) && *m != p.key)?,
            ))
        })
        .collect();
    let mut layers = 0;
    for p in people {
        let mut at = p.key.as_str();
        let mut depth = 1;
        while let Some(up) = manager_of.get(at) {
            at = up;
            depth += 1;
            if depth > people.len() {
                break;
            }
        }
        layers = layers.max(depth.min(people.len()));
    }
    let mut reports: HashMap<&str, i64> = HashMap::new();
    for m in manager_of.values() {
        *reports.entry(m).or_insert(0) += 1;
    }
    let managers = i64::try_from(reports.len()).unwrap_or(0);
    let total: i64 = reports.values().sum();
    Side {
        layers,
        headcount: i64::try_from(people.len()).unwrap_or(0),
        span: Span {
            managers,
            mean_hundredths: (managers > 0).then(|| (total * 200 + managers) / (2 * managers)),
            max: reports.values().copied().max().unwrap_or(0),
        },
    }
}

fn target_side(nodes: &[Node]) -> Side {
    let by_key: HashMap<&str, &Node> = nodes.iter().map(|n| (n.key.as_str(), n)).collect();
    let mut children: HashMap<&str, i64> = HashMap::new();
    for n in nodes {
        if let Some(p) = n.parent.as_deref() {
            *children.entry(p).or_insert(0) += i64::from(n.headcount);
        }
    }
    let mut layers = 0;
    for n in nodes {
        let (mut at, mut depth) = (n, 1);
        while let Some(up) = at.parent.as_deref().and_then(|p| by_key.get(p)) {
            at = up;
            depth += 1;
            if depth > nodes.len() {
                break;
            }
        }
        layers = layers.max(depth);
    }
    let (mut managers, mut reports, mut max) = (0_i64, 0_i64, 0_i64);
    for (key, r) in &children {
        let h = i64::from(by_key.get(key).map_or(1, |n| n.headcount));
        managers += h;
        reports += r;
        max = max.max((r + h - 1) / h);
    }
    Side {
        layers,
        headcount: nodes.iter().map(|n| i64::from(n.headcount)).sum(),
        span: Span {
            managers,
            mean_hundredths: (managers > 0).then(|| (reports * 200 + managers) / (2 * managers)),
            max,
        },
    }
}

/// Compare the live structure with a target. `floor` is the smallest number of people whose count in a role is given.
#[must_use]
pub fn diff(people: &[Person], target: &[Node], floor: usize) -> Diff {
    struct Agg {
        title: String,
        current: BTreeMap<String, i64>,
        target: BTreeMap<String, i64>,
    }
    fn entry<'a>(roles: &'a mut BTreeMap<String, Agg>, title: &str) -> &'a mut Agg {
        roles.entry(key_of(title)).or_insert_with(|| Agg {
            title: title.trim().to_string(),
            current: BTreeMap::new(),
            target: BTreeMap::new(),
        })
    }
    let mut roles: BTreeMap<String, Agg> = BTreeMap::new();
    for p in people {
        *entry(&mut roles, &p.title)
            .current
            .entry(p.department.trim().to_string())
            .or_insert(0) += 1;
    }
    for n in target {
        *entry(&mut roles, &n.title)
            .target
            .entry(n.department.trim().to_string())
            .or_insert(0) += i64::from(n.headcount);
    }
    let floor = i64::try_from(floor).unwrap_or(i64::MAX);
    let roles = roles
        .into_values()
        .map(|a| {
            let (now, plan): (i64, i64) = (a.current.values().sum(), a.target.values().sum());
            let change = match (now, plan) {
                (0, _) => Change::Added,
                (_, 0) => Change::Removed,
                _ if plan > now => Change::Larger,
                _ if plan < now => Change::Smaller,
                _ => Change::Same,
            };
            let withheld = now > 0 && now < floor;
            let current_departments: Vec<String> = a.current.keys().cloned().collect();
            let target_departments: Vec<String> = a.target.keys().cloned().collect();
            Role {
                title: a.title,
                current: if withheld {
                    Count::Withheld
                } else {
                    Count::Shown { value: now }
                },
                target: plan,
                change,
                delta: (!withheld).then_some(plan - now),
                moved: now > 0 && plan > 0 && current_departments != target_departments,
                current_departments,
                target_departments,
            }
        })
        .collect();
    Diff {
        current: current_side(people),
        target: target_side(target),
        roles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(key: &str, manager: Option<&str>, department: &str, title: &str) -> Person {
        Person {
            key: key.into(),
            manager: manager.map(Into::into),
            department: department.into(),
            title: title.into(),
        }
    }

    fn node(
        key: &str,
        parent: Option<&str>,
        department: &str,
        title: &str,
        headcount: i32,
    ) -> Node {
        Node {
            key: key.into(),
            parent: parent.map(Into::into),
            department: department.into(),
            title: title.into(),
            headcount,
        }
    }

    /// A head, two managers, six engineers: three layers, the managers' spans 3 and 3.
    fn now() -> Vec<Person> {
        let mut v = vec![
            person("h", None, "exec", "Head"),
            person("m1", Some("h"), "eng", "Manager"),
            person("m2", Some("h"), "eng", "Manager"),
        ];
        for i in 0..3 {
            v.push(person(&format!("a{i}"), Some("m1"), "eng", "Engineer"));
            v.push(person(&format!("b{i}"), Some("m2"), "eng", "Engineer"));
        }
        v
    }

    #[test]
    fn the_current_side_has_layers_and_spans_from_the_live_tree() {
        let s = diff(&now(), &[node("x", None, "exec", "Head", 1)], 1).current;
        assert_eq!(s.layers, 3);
        assert_eq!(s.headcount, 9);
        assert_eq!(
            s.span,
            Span {
                managers: 3,
                mean_hundredths: Some(267),
                max: 3
            },
            "(2 + 3 + 3) / 3"
        );
    }

    #[test]
    fn the_target_side_counts_manager_seats_and_rounds_the_widest_span_up() {
        let t = vec![
            node("head", None, "exec", "Head", 1),
            node("mgr", Some("head"), "eng", "Manager", 2),
            node("eng", Some("mgr"), "eng", "Engineer", 7),
        ];
        let s = diff(&[], &t, 1).target;
        assert_eq!(s.layers, 3);
        assert_eq!(s.headcount, 10);
        // Head has 2 reports (one seat); managers have 7 between 2 seats: 3.5 each, rounded up to 4 for the widest.
        assert_eq!(s.span.managers, 3);
        assert_eq!(s.span.max, 4);
        assert_eq!(s.span.mean_hundredths, Some(300), "(2 + 7) / 3 seats");
    }

    #[test]
    fn roles_are_added_removed_resized_and_the_unchanged_are_listed_too() {
        let target = vec![
            node("h", None, "exec", "Head", 1),
            node("m", Some("h"), "eng", "Manager", 1), // one manager instead of two
            node("e", Some("m"), "eng", "Engineer", 8), // two more
            node("p", Some("h"), "product", "Product Lead", 1), // new
        ];
        let d = diff(&now(), &target, 1);
        let role = |t: &str| d.roles.iter().find(|r| r.title == t).unwrap();
        assert_eq!(role("Head").change, Change::Same);
        assert_eq!(role("Manager").change, Change::Smaller);
        assert_eq!(role("Manager").delta, Some(-1));
        assert_eq!(role("Engineer").change, Change::Larger);
        assert_eq!(role("Engineer").delta, Some(2));
        assert_eq!(role("Product Lead").change, Change::Added);
        assert_eq!(role("Product Lead").current, Count::Shown { value: 0 });
        let gone = diff(&now(), &[node("h", None, "exec", "Head", 1)], 1);
        assert_eq!(
            gone.roles
                .iter()
                .find(|r| r.title == "Engineer")
                .unwrap()
                .change,
            Change::Removed
        );
    }

    #[test]
    fn titles_match_without_regard_to_case_or_spaces() {
        let d = diff(
            &[person("a", None, "eng", "Support Engineer")],
            &[node("x", None, "eng", "  support engineer ", 2)],
            1,
        );
        assert_eq!(d.roles.len(), 1);
        assert_eq!(d.roles[0].change, Change::Larger);
    }

    #[test]
    fn a_role_in_other_departments_in_the_plan_is_moved() {
        let d = diff(
            &[person("a", None, "support", "Analyst")],
            &[node("x", None, "operations", "Analyst", 1)],
            1,
        );
        assert!(d.roles[0].moved);
        assert_eq!(d.roles[0].change, Change::Same);
        assert_eq!(d.roles[0].current_departments, ["support"]);
        let same = diff(
            &[person("a", None, "support", "Analyst")],
            &[node("x", None, "support", "Analyst", 1)],
            1,
        );
        assert!(!same.roles[0].moved);
        let added = diff(&[], &[node("x", None, "operations", "Analyst", 1)], 1);
        assert!(
            !added.roles[0].moved,
            "a role that did not exist has not moved"
        );
    }

    #[test]
    fn a_small_current_count_is_withheld_and_so_is_the_change_that_would_give_it_back() {
        let people: Vec<Person> = (0..3)
            .map(|i| person(&format!("p{i}"), None, "eng", "Architect"))
            .collect();
        let d = diff(&people, &[node("x", None, "eng", "Architect", 5)], 5);
        let r = &d.roles[0];
        assert_eq!(r.current, Count::Withheld);
        assert_eq!(r.delta, None, "5 - delta would reveal 3");
        assert_eq!(r.target, 5, "a plan's number is not about people");
        let big: Vec<Person> = (0..5)
            .map(|i| person(&format!("p{i}"), None, "eng", "Architect"))
            .collect();
        let r = &diff(&big, &[node("x", None, "eng", "Architect", 5)], 5).roles[0];
        assert_eq!(r.current, Count::Shown { value: 5 });
        assert_eq!(r.delta, Some(0));
        let none = &diff(&[], &[node("x", None, "eng", "Architect", 2)], 5).roles[0];
        assert_eq!(
            none.current,
            Count::Shown { value: 0 },
            "nobody is not a small group"
        );
    }

    #[test]
    fn a_loop_or_a_manager_outside_the_set_cannot_hang_or_deepen_the_tree() {
        let people = vec![
            person("a", Some("b"), "x", "T"),
            person("b", Some("a"), "x", "T"),
            person("c", Some("nobody"), "x", "T"),
            person("d", Some("d"), "x", "T"),
        ];
        let s = diff(&people, &[node("n", None, "x", "T", 1)], 1).current;
        assert!(s.layers <= 4);
        assert_eq!(s.headcount, 4);
    }

    #[test]
    fn the_output_names_no_one() {
        let d = diff(&now(), &[node("h", None, "exec", "Head", 1)], 1);
        let text = serde_json::to_string(&d).unwrap();
        for key in ["\"h\"", "\"m1\"", "a0", "b2"] {
            assert!(!text.contains(key), "{key} in {text}");
        }
    }

    #[test]
    fn a_target_is_checked() {
        let ok = vec![
            node("a", None, "x", "T", 1),
            node("b", Some("a"), "x", "U", 3),
        ];
        assert!(validate_target(&ok).is_ok());
        let bad = |nodes: Vec<Node>| validate_target(&nodes).unwrap_err().join("; ");
        assert!(bad(vec![]).contains("at least one"));
        assert!(
            bad(vec![
                node("a", None, "x", "T", 1),
                node("a", None, "x", "U", 1)
            ])
            .contains("twice")
        );
        assert!(bad(vec![node("a", Some("zzz"), "x", "T", 1)]).contains("not a role"));
        assert!(
            bad(vec![
                node("a", Some("b"), "x", "T", 1),
                node("b", Some("a"), "x", "U", 1)
            ])
            .contains("loop")
        );
        assert!(bad(vec![node("a", Some("a"), "x", "T", 1)]).contains("loop"));
        assert!(bad(vec![node("a", None, "x", "T", 0)]).contains("headcount"));
        assert!(bad(vec![node("a b", None, "x", "T", 1)]).contains("no spaces"));
        assert!(bad(vec![node("a", None, "", "T", 1)]).contains("department"));
    }
}
