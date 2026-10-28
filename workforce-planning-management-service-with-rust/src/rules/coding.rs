//! Cost centres, financial coding and subjective coding (WPM-R163 to R165, WPM-D116). DB-free and clock-free.
//!
//! Codes are configuration and **uncoded is unknown**: a segment with nothing to fill it makes the full code *incomplete* and is
//! named, never padded with zeros or a default; an assignment that no subjective rule matches is *uncoded*, never given a default
//! code. The most specific subjective rule wins, and two equally specific rules that could match the same assignment are refused
//! when the second is saved, so the answer never depends on order.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

/// The longest segment.
pub const SEGMENT_MAX: usize = 20;
/// The most segments in a structure.
pub const SEGMENTS_MAX: usize = 12;

/// Where a segment's value comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The assignment's cost centre.
    CostCentre,
    /// The subjective code derived for the assignment.
    Subjective,
    /// The same value on every code.
    Fixed(String),
    /// A category of the assignment (a kind of the vocabulary, such as `site`).
    Vocabulary(String),
}

/// One segment of a coding structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// The segment's name, as the missing-segment answer shows it.
    pub name: String,
    /// The exact length of its value.
    pub length: usize,
    /// Where the value comes from.
    pub source: Source,
}

/// Problems with a coding structure: some segments, each named once with a length that fits, at most one cost centre and one
/// subjective segment, and a fixed value of exactly its length.
#[must_use]
pub fn structure_problems(segments: &[Segment]) -> Vec<String> {
    let mut out = Vec::new();
    if segments.is_empty() || segments.len() > SEGMENTS_MAX {
        out.push(format!(
            "a coding structure has 1 to {SEGMENTS_MAX} segments"
        ));
    }
    let mut names = BTreeSet::new();
    for s in segments {
        let name = s.name.trim();
        if name.is_empty() {
            out.push("a segment needs a name".to_string());
        } else if !names.insert(name.to_string()) {
            out.push(format!("segment {name} appears twice"));
        }
        if !(1..=SEGMENT_MAX).contains(&s.length) {
            out.push(format!("segment {name} must be 1 to {SEGMENT_MAX} long"));
        }
        if let Source::Fixed(v) = &s.source
            && v.chars().count() != s.length
        {
            out.push(format!(
                "segment {name} is fixed at {v:?}, which is not {} long",
                s.length
            ));
        }
    }
    for (what, source) in [
        ("cost centre", Source::CostCentre),
        ("subjective", Source::Subjective),
    ] {
        if segments.iter().filter(|s| s.source == source).count() > 1 {
            out.push(format!("at most one {what} segment"));
        }
    }
    out
}

/// What an assignment has to fill the segments with. `None` is not recorded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Values {
    /// The assignment's cost centre code.
    pub cost_centre: Option<String>,
    /// The derived subjective code.
    pub subjective: Option<String>,
    /// Category values by kind.
    pub vocabulary: BTreeMap<String, Option<String>>,
}

/// The assembled code, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assembled {
    /// The full code when every segment is filled and fits; otherwise `None`.
    pub code: Option<String>,
    /// Segments with nothing to fill them.
    pub missing: Vec<String>,
    /// Segments whose value is not exactly the segment's length.
    pub wrong_length: Vec<String>,
}

/// Assemble the full code. A value is never padded, trimmed to fit or defaulted.
#[must_use]
pub fn assemble(segments: &[Segment], values: &Values) -> Assembled {
    let mut parts = Vec::new();
    let (mut missing, mut wrong_length) = (Vec::new(), Vec::new());
    for s in segments {
        let value: Option<&str> = match &s.source {
            Source::CostCentre => values.cost_centre.as_deref(),
            Source::Subjective => values.subjective.as_deref(),
            Source::Fixed(v) => Some(v.as_str()),
            Source::Vocabulary(kind) => values.vocabulary.get(kind).and_then(|v| v.as_deref()),
        }
        .map(str::trim)
        .filter(|v| !v.is_empty());
        match value {
            None => missing.push(s.name.clone()),
            Some(v) if v.chars().count() != s.length => wrong_length.push(s.name.clone()),
            Some(v) => parts.push(v.to_string()),
        }
    }
    let complete = missing.is_empty() && wrong_length.is_empty() && !segments.is_empty();
    Assembled {
        code: complete.then(|| parts.concat()),
        missing,
        wrong_length,
    }
}

/// A subjective rule: the fields it matches (at least one) and the code it gives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rule {
    /// The rule's id, for naming a conflict.
    pub id: i64,
    /// Staff group to match.
    pub staff_group: Option<String>,
    /// Job role to match.
    pub job_role: Option<String>,
    /// Occupation code to match.
    pub occupation_code: Option<String>,
    /// Pay scale band to match.
    pub pay_band: Option<String>,
    /// Position number to match.
    pub position_number: Option<String>,
    /// The code it gives.
    pub code: String,
}

/// What a rule is matched against.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// Staff group.
    pub staff_group: Option<String>,
    /// Job role.
    pub job_role: Option<String>,
    /// Occupation code.
    pub occupation_code: Option<String>,
    /// Pay scale band.
    pub pay_band: Option<String>,
    /// Position number.
    pub position_number: Option<String>,
}

impl Rule {
    fn criteria(&self) -> [&Option<String>; 5] {
        [
            &self.staff_group,
            &self.job_role,
            &self.occupation_code,
            &self.pay_band,
            &self.position_number,
        ]
    }

    /// How many fields the rule matches on.
    #[must_use]
    pub fn specificity(&self) -> usize {
        self.criteria().iter().filter(|c| c.is_some()).count()
    }

    /// Whether the rule takes in the facts: every field it names is recorded and equal. A field not recorded matches nothing.
    #[must_use]
    pub fn matches(&self, facts: &Facts) -> bool {
        self.specificity() > 0
            && self
                .criteria()
                .iter()
                .zip([
                    &facts.staff_group,
                    &facts.job_role,
                    &facts.occupation_code,
                    &facts.pay_band,
                    &facts.position_number,
                ])
                .all(|(rule, fact)| rule.is_none() || rule == &fact)
    }
}

/// Problems with a rule on its own: it names some field, and the code is 1 to [`SEGMENT_MAX`] long.
#[must_use]
pub fn rule_problems(rule: &Rule) -> Vec<String> {
    let mut out = Vec::new();
    if rule.specificity() == 0 {
        out.push("a rule must match on at least one field".to_string());
    }
    if !(1..=SEGMENT_MAX).contains(&rule.code.trim().chars().count()) {
        out.push(format!("code must be 1 to {SEGMENT_MAX} characters"));
    }
    out
}

/// The id of an existing rule that `new` cannot be saved beside: equally specific and able to match one assignment together
/// (every field both name is equal), which includes an exact duplicate. The derivation then would depend on order.
#[must_use]
pub fn conflict(existing: &[Rule], new: &Rule) -> Option<i64> {
    existing
        .iter()
        .filter(|r| r.id != new.id && r.specificity() == new.specificity())
        .find(|r| {
            r.criteria()
                .iter()
                .zip(new.criteria())
                .all(|(a, b)| a.is_none() || b.is_none() || a == &b)
        })
        .map(|r| r.id)
}

/// What the rules give an assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Derived {
    /// The code of the most specific rule that matches.
    Code(String),
    /// No rule matches: listed for finance, never given a default.
    Uncoded,
    /// Two equally specific rules match (saved before the conflict check existed); nothing is chosen.
    Ambiguous(Vec<i64>),
}

/// Derive the subjective code: the most specific matching rule wins.
#[must_use]
pub fn derive(rules: &[Rule], facts: &Facts) -> Derived {
    let matching: Vec<&Rule> = rules.iter().filter(|r| r.matches(facts)).collect();
    let Some(top) = matching.iter().map(|r| r.specificity()).max() else {
        return Derived::Uncoded;
    };
    let best: Vec<&&Rule> = matching.iter().filter(|r| r.specificity() == top).collect();
    match best.as_slice() {
        [one] => Derived::Code(one.code.trim().to_string()),
        many => Derived::Ambiguous(many.iter().map(|r| r.id).collect()),
    }
}

/// A cost centre in the register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostCentre {
    /// Its code.
    pub code: String,
    /// The code of its parent, if any.
    pub parent: Option<String>,
    /// The first day it may be used.
    pub valid_from: NaiveDate,
    /// The last day it may be used; a cost centre is closed, never deleted.
    pub valid_to: Option<NaiveDate>,
}

impl CostCentre {
    /// Open on `on`: from its first day to its last, inclusive.
    #[must_use]
    pub fn open_on(&self, on: NaiveDate) -> bool {
        self.valid_from <= on && self.valid_to.is_none_or(|e| e >= on)
    }
}

/// Whether a write naming `code` on `on` is allowed. A code not in the register, or not open that day, is refused.
///
/// # Errors
/// A sentence saying which.
pub fn check_open(register: &[CostCentre], code: &str, on: NaiveDate) -> Result<(), String> {
    match register.iter().find(|c| c.code == code) {
        None => Err(format!("cost centre {code} is not in the register")),
        Some(c) if !c.open_on(on) => Err(format!("cost centre {code} is not open on {on}")),
        Some(_) => Ok(()),
    }
}

/// The chain from `code` up through its parents, nearest first, stopping at the root, an unknown parent or a repeat.
#[must_use]
pub fn chain(register: &[CostCentre], code: &str) -> Vec<String> {
    let mut out = vec![code.to_string()];
    let mut here = code.to_string();
    while let Some(parent) = register
        .iter()
        .find(|c| c.code == here)
        .and_then(|c| c.parent.clone())
    {
        if out.contains(&parent) || !register.iter().any(|c| c.code == parent) {
            break;
        }
        out.push(parent.clone());
        here = parent;
    }
    out
}

/// Whether giving `code` the parent `parent` would make a loop (including a cost centre that is its own parent).
#[must_use]
pub fn creates_cycle(register: &[CostCentre], code: &str, parent: &str) -> bool {
    parent == code || chain(register, parent).iter().any(|c| c == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn seg(name: &str, length: usize, source: Source) -> Segment {
        Segment {
            name: name.to_string(),
            length,
            source,
        }
    }

    fn structure() -> Vec<Segment> {
        vec![
            seg("entity", 2, Source::Fixed("01".into())),
            seg("cost centre", 4, Source::CostCentre),
            seg("subjective", 5, Source::Subjective),
            seg("site", 2, Source::Vocabulary("site".into())),
        ]
    }

    #[test]
    fn a_structure_is_checked_before_it_is_used() {
        assert_eq!(structure_problems(&structure()), Vec::<String>::new());
        assert_eq!(structure_problems(&[]).len(), 1);
        let long = [seg("a", 21, Source::CostCentre)];
        assert_eq!(structure_problems(&long).len(), 1);
        let twice = [
            seg("a", 2, Source::CostCentre),
            seg("a", 2, Source::Subjective),
        ];
        assert_eq!(structure_problems(&twice).len(), 1, "a name appears once");
        let two_cc = [
            seg("a", 2, Source::CostCentre),
            seg("b", 2, Source::CostCentre),
        ];
        assert_eq!(structure_problems(&two_cc).len(), 1);
        let bad_fixed = [seg("a", 3, Source::Fixed("01".into()))];
        assert_eq!(structure_problems(&bad_fixed).len(), 1);
        assert_eq!(
            structure_problems(&[seg(" ", 2, Source::CostCentre)]).len(),
            1
        );
    }

    #[test]
    fn a_full_code_joins_the_segments_in_order() {
        let mut v = Values {
            cost_centre: Some("N123".to_string()),
            subjective: Some("12345".to_string()),
            ..Values::default()
        };
        v.vocabulary.insert("site".into(), Some("RX".to_string()));
        let a = assemble(&structure(), &v);
        assert_eq!(a.code, Some("01N12312345RX".to_string()));
        assert!(a.missing.is_empty() && a.wrong_length.is_empty());
    }

    #[test]
    fn a_missing_segment_makes_the_code_incomplete_and_is_named_never_padded() {
        let v = Values {
            cost_centre: Some("N123".to_string()),
            ..Values::default()
        };
        let a = assemble(&structure(), &v);
        assert_eq!(a.code, None);
        assert_eq!(
            a.missing,
            vec!["subjective".to_string(), "site".to_string()]
        );
        let blank = Values {
            cost_centre: Some("   ".to_string()),
            ..Values::default()
        };
        assert!(
            assemble(&structure(), &blank)
                .missing
                .contains(&"cost centre".to_string()),
            "blank is missing"
        );
    }

    #[test]
    fn a_value_of_the_wrong_length_is_not_padded_or_cut() {
        let mut v = Values {
            cost_centre: Some("N12".to_string()),
            subjective: Some("123456".to_string()),
            ..Values::default()
        };
        v.vocabulary.insert("site".into(), Some("RX".to_string()));
        let a = assemble(&structure(), &v);
        assert_eq!(a.code, None);
        assert_eq!(
            a.wrong_length,
            vec!["cost centre".to_string(), "subjective".to_string()]
        );
        assert_eq!(a.missing, Vec::<String>::new());
    }

    #[test]
    fn no_segments_is_no_code() {
        assert_eq!(assemble(&[], &Values::default()).code, None);
    }

    fn rule(
        id: i64,
        group: Option<&str>,
        role: Option<&str>,
        band: Option<&str>,
        code: &str,
    ) -> Rule {
        Rule {
            id,
            staff_group: group.map(String::from),
            job_role: role.map(String::from),
            pay_band: band.map(String::from),
            code: code.to_string(),
            ..Rule::default()
        }
    }

    fn facts(group: Option<&str>, role: Option<&str>, band: Option<&str>) -> Facts {
        Facts {
            staff_group: group.map(String::from),
            job_role: role.map(String::from),
            pay_band: band.map(String::from),
            ..Facts::default()
        }
    }

    #[test]
    fn the_most_specific_matching_rule_wins() {
        let rules = [
            rule(1, Some("nursing"), None, None, "10000"),
            rule(2, Some("nursing"), None, Some("5"), "10005"),
            rule(3, Some("nursing"), Some("rn"), Some("5"), "10055"),
        ];
        assert_eq!(
            derive(&rules, &facts(Some("nursing"), Some("rn"), Some("5"))),
            Derived::Code("10055".into())
        );
        assert_eq!(
            derive(&rules, &facts(Some("nursing"), Some("hca"), Some("5"))),
            Derived::Code("10005".into())
        );
        assert_eq!(
            derive(&rules, &facts(Some("nursing"), None, None)),
            Derived::Code("10000".into())
        );
    }

    #[test]
    fn no_matching_rule_is_uncoded_and_a_blank_fact_matches_nothing() {
        let rules = [rule(1, Some("nursing"), None, None, "10000")];
        assert_eq!(
            derive(&rules, &facts(Some("admin"), None, None)),
            Derived::Uncoded
        );
        assert_eq!(
            derive(&rules, &facts(None, Some("rn"), None)),
            Derived::Uncoded,
            "staff group not recorded"
        );
        assert_eq!(derive(&[], &Facts::default()), Derived::Uncoded);
        let none = rule(9, None, None, None, "99999");
        assert_eq!(
            derive(&[none], &Facts::default()),
            Derived::Uncoded,
            "a rule that names nothing matches nothing"
        );
    }

    #[test]
    fn two_equally_specific_matches_choose_nothing() {
        let rules = [
            rule(1, Some("nursing"), None, None, "A"),
            rule(2, None, Some("rn"), None, "B"),
        ];
        assert_eq!(
            derive(&rules, &facts(Some("nursing"), Some("rn"), None)),
            Derived::Ambiguous(vec![1, 2])
        );
    }

    #[test]
    fn a_rule_that_could_match_with_an_equal_one_is_refused_when_saved() {
        let existing = [rule(1, Some("nursing"), None, None, "A")];
        let same = rule(0, Some("nursing"), None, None, "B");
        assert_eq!(conflict(&existing, &same), Some(1), "an exact duplicate");
        let other_field = rule(0, None, Some("rn"), None, "B");
        assert_eq!(
            conflict(&existing, &other_field),
            Some(1),
            "a nurse who is also an RN matches both"
        );
        let other_value = rule(0, Some("admin"), None, None, "B");
        assert_eq!(
            conflict(&existing, &other_value),
            None,
            "different values of one field never meet"
        );
        let more_specific = rule(0, Some("nursing"), Some("rn"), None, "B");
        assert_eq!(
            conflict(&existing, &more_specific),
            None,
            "specificity settles it"
        );
        assert_eq!(
            conflict(&existing, &existing[0]),
            None,
            "a rule is not its own conflict"
        );
    }

    #[test]
    fn a_rule_on_its_own_needs_a_field_and_a_code() {
        assert_eq!(
            rule_problems(&rule(0, Some("a"), None, None, "X")),
            Vec::<String>::new()
        );
        assert_eq!(rule_problems(&rule(0, None, None, None, "X")).len(), 1);
        assert_eq!(rule_problems(&rule(0, Some("a"), None, None, " ")).len(), 1);
    }

    fn register() -> Vec<CostCentre> {
        let cc = |code: &str, parent: Option<&str>, from, to| CostCentre {
            code: code.to_string(),
            parent: parent.map(String::from),
            valid_from: from,
            valid_to: to,
        };
        vec![
            cc("TOP", None, d(2020, 1, 1), None),
            cc("DIV", Some("TOP"), d(2020, 1, 1), None),
            cc("WARD", Some("DIV"), d(2023, 4, 1), Some(d(2025, 3, 31))),
        ]
    }

    #[test]
    fn a_cost_centre_must_be_open_on_the_day() {
        let r = register();
        assert!(
            check_open(&r, "WARD", d(2023, 4, 1)).is_ok(),
            "first day is in"
        );
        assert!(
            check_open(&r, "WARD", d(2025, 3, 31)).is_ok(),
            "last day is in"
        );
        assert!(
            check_open(&r, "WARD", d(2025, 4, 1))
                .unwrap_err()
                .contains("not open")
        );
        assert!(check_open(&r, "WARD", d(2023, 3, 31)).is_err());
        assert!(
            check_open(&r, "NOPE", d(2024, 1, 1))
                .unwrap_err()
                .contains("not in the register")
        );
    }

    #[test]
    fn the_chain_runs_up_to_the_root_and_a_loop_is_found() {
        let r = register();
        assert_eq!(chain(&r, "WARD"), vec!["WARD", "DIV", "TOP"]);
        assert_eq!(chain(&r, "TOP"), vec!["TOP"]);
        assert!(
            creates_cycle(&r, "TOP", "WARD"),
            "the root under its own descendant"
        );
        assert!(creates_cycle(&r, "DIV", "DIV"), "its own parent");
        assert!(!creates_cycle(&r, "WARD", "TOP"));
        let looped = [
            CostCentre {
                code: "A".into(),
                parent: Some("B".into()),
                valid_from: d(2020, 1, 1),
                valid_to: None,
            },
            CostCentre {
                code: "B".into(),
                parent: Some("A".into()),
                valid_from: d(2020, 1, 1),
                valid_to: None,
            },
        ];
        assert_eq!(
            chain(&looped, "A"),
            vec!["A", "B"],
            "a stored loop does not run forever"
        );
    }
}
