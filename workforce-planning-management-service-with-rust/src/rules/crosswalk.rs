//! The skills crosswalk (WPM-R158, WPM-D109): a statement, with a stated basis, that a skill declared under one record counts as
//! another. DB-free and clock-free.
//!
//! A mapping runs **from** the skill a person declared **to** the skill that is needed, and its confidence says how the first
//! relates to the second. Only `exact` and `close` let a declaration stand in for a need; `broader` and `narrower` are shown beside
//! it and never counted, because a broad declaration does not prove a narrow need and the reverse is not shown either. A need with
//! no declaration and no mapping stays `undeclared`, which is unknown and not "not held".

use std::collections::BTreeMap;

use serde::Serialize;
use uuid::Uuid;

use super::skill_gap::{Gap, Status};

/// How a declared skill relates to a needed one.
pub const CONFIDENCES: &[&str] = &["exact", "close", "broader", "narrower"];
/// What a mapping rests on.
pub const BASES: &[&str] = &["authority", "judgement"];

/// The longest source or note.
pub const SOURCE_MAX: usize = 300;

/// Problems with a mapping, one sentence each.
#[must_use]
pub fn problems(from: Uuid, to: Uuid, confidence: &str, basis: &str, source: &str) -> Vec<String> {
    let mut out = Vec::new();
    if from == to {
        out.push("a skill is not mapped to itself".to_string());
    }
    if !CONFIDENCES.contains(&confidence) {
        out.push(format!("confidence must be one of: {}", CONFIDENCES.join(", ")));
    }
    if !BASES.contains(&basis) {
        out.push(format!("basis must be one of: {}", BASES.join(", ")));
    }
    let n = source.trim().chars().count();
    if n == 0 || n > SOURCE_MAX {
        out.push(format!(
            "source must say, in 1 to {SOURCE_MAX} characters, which published crosswalk or whose judgement this rests on"
        ));
    }
    out
}

/// A mapping as the rule needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping<'a> {
    /// The skill a person declares.
    pub from: Uuid,
    /// The skill that is needed.
    pub to: Uuid,
    /// How the first relates to the second.
    pub confidence: &'a str,
}

/// Whether a declaration under this confidence stands in for the need.
#[must_use]
pub fn counts(confidence: &str) -> bool {
    matches!(confidence, "exact" | "close")
}

/// A need met or graded through a mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Via {
    /// The needed skill.
    pub skill: Uuid,
    /// The skill the person declared.
    pub declared_skill: Uuid,
    /// The mapping's confidence.
    pub confidence: String,
}

/// What applying the crosswalk to a person's gaps found.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Applied {
    /// Needs graded through a mapping.
    pub via: Vec<Via>,
    /// Needs still undeclared that have a mapping which does not count (`broader` or `narrower`), with the declared skill.
    pub related: Vec<Via>,
    /// Needs still undeclared that no mapping reaches: unknown, and nothing is said about them.
    pub unmapped: usize,
}

fn rank(confidence: &str) -> u8 {
    u8::from(confidence == "exact")
}

/// Grade the `undeclared` gaps through the mappings, in place. A direct declaration is never overridden. Of several counting
/// mappings the most confident wins, then the highest declared level, then the smaller skill id, so the answer is stable.
pub fn apply(gaps: &mut [Gap], declared: &BTreeMap<Uuid, i32>, mappings: &[Mapping<'_>]) -> Applied {
    let mut out = Applied::default();
    for gap in gaps.iter_mut().filter(|g| g.status == Status::Undeclared) {
        let reaching: Vec<&Mapping<'_>> = mappings
            .iter()
            .filter(|m| m.to == gap.skill && declared.contains_key(&m.from))
            .collect();
        let best = reaching
            .iter()
            .filter(|m| counts(m.confidence))
            .max_by(|a, b| {
                rank(a.confidence)
                    .cmp(&rank(b.confidence))
                    .then(declared[&a.from].cmp(&declared[&b.from]))
                    .then(b.from.cmp(&a.from))
            });
        if let Some(m) = best {
            let level = declared[&m.from];
            gap.declared = Some(level);
            if level >= gap.required {
                gap.status = Status::Met;
                gap.shortfall = None;
            } else {
                gap.status = Status::Below;
                gap.shortfall = Some(gap.required - level);
            }
            out.via.push(Via {
                skill: gap.skill,
                declared_skill: m.from,
                confidence: m.confidence.to_string(),
            });
        } else if let Some(m) = reaching.first() {
            out.related.push(Via {
                skill: gap.skill,
                declared_skill: m.from,
                confidence: m.confidence.to_string(),
            });
        } else {
            out.unmapped += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::skill_gap::{Need, Source, merge};

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn need(skill: Uuid, required: i32) -> Need {
        Need { skill, required, importance: "important".to_string(), source: Source::Role }
    }

    #[test]
    fn a_mapping_needs_two_skills_a_known_confidence_and_basis_and_a_stated_source() {
        let ok = |c: &str, b: &str, s: &str| problems(id(1), id(2), c, b, s);
        assert_eq!(ok("exact", "authority", "ESCO to SFIA crosswalk, 2025"), Vec::<String>::new());
        assert_eq!(problems(id(1), id(1), "exact", "authority", "x").len(), 1);
        assert_eq!(ok("same", "authority", "x").len(), 1);
        assert_eq!(ok("exact", "hunch", "x").len(), 1);
        assert_eq!(ok("exact", "judgement", "  ").len(), 1);
        assert_eq!(ok("exact", "judgement", &"x".repeat(301)).len(), 1);
    }

    #[test]
    fn only_exact_and_close_count() {
        assert!(counts("exact") && counts("close"));
        assert!(!counts("broader") && !counts("narrower") && !counts(""));
    }

    #[test]
    fn an_undeclared_need_is_graded_through_an_exact_or_close_mapping() {
        let (needed, held) = (id(10), id(20));
        let mut gaps = merge(&[need(needed, 3)], &BTreeMap::new());
        let declared = BTreeMap::from([(held, 4)]);
        let applied = apply(&mut gaps, &declared, &[Mapping { from: held, to: needed, confidence: "close" }]);
        assert_eq!(gaps[0].status, Status::Met);
        assert_eq!(gaps[0].declared, Some(4));
        assert_eq!(applied.via, vec![Via { skill: needed, declared_skill: held, confidence: "close".to_string() }]);
        // Exactly the level needed is met, not short.
        let mut gaps = merge(&[need(needed, 4)], &BTreeMap::new());
        apply(&mut gaps, &declared, &[Mapping { from: held, to: needed, confidence: "exact" }]);
        assert_eq!((gaps[0].status, gaps[0].shortfall), (Status::Met, None));
        // Below the need: a shortfall, graded the same way.
        let mut gaps = merge(&[need(needed, 5)], &BTreeMap::new());
        apply(&mut gaps, &declared, &[Mapping { from: held, to: needed, confidence: "exact" }]);
        assert_eq!((gaps[0].status, gaps[0].shortfall), (Status::Below, Some(1)));
    }

    #[test]
    fn broader_and_narrower_are_shown_and_never_counted() {
        let (needed, held) = (id(10), id(20));
        for confidence in ["broader", "narrower"] {
            let mut gaps = merge(&[need(needed, 1)], &BTreeMap::new());
            let applied = apply(&mut gaps, &BTreeMap::from([(held, 5)]), &[Mapping { from: held, to: needed, confidence }]);
            assert_eq!(gaps[0].status, Status::Undeclared, "{confidence}");
            assert_eq!(gaps[0].declared, None);
            assert_eq!(applied.via, Vec::<Via>::new());
            assert_eq!(applied.related.len(), 1);
            assert_eq!(applied.unmapped, 0, "it is mapped, just not counted");
        }
    }

    #[test]
    fn a_need_no_mapping_reaches_stays_unknown_and_is_counted_as_unmapped() {
        let mut gaps = merge(&[need(id(10), 3), need(id(11), 3)], &BTreeMap::new());
        let declared = BTreeMap::from([(id(20), 4)]);
        // A mapping whose declared side the person has not declared does not reach anything.
        let applied = apply(&mut gaps, &declared, &[Mapping { from: id(99), to: id(10), confidence: "exact" }]);
        assert!(gaps.iter().all(|g| g.status == Status::Undeclared && g.declared.is_none()));
        assert_eq!(applied.unmapped, 2);
    }

    #[test]
    fn a_direct_declaration_is_never_overridden() {
        let (needed, held) = (id(10), id(20));
        let declared = BTreeMap::from([(needed, 1), (held, 5)]);
        let mut gaps = merge(&[need(needed, 3)], &declared);
        apply(&mut gaps, &declared, &[Mapping { from: held, to: needed, confidence: "exact" }]);
        assert_eq!((gaps[0].status, gaps[0].declared), (Status::Below, Some(1)));
    }

    #[test]
    fn of_several_mappings_the_most_confident_then_the_highest_level_wins() {
        let needed = id(10);
        let declared = BTreeMap::from([(id(20), 5), (id(21), 2), (id(22), 3)]);
        let maps = [
            Mapping { from: id(20), to: needed, confidence: "close" },
            Mapping { from: id(21), to: needed, confidence: "exact" },
            Mapping { from: id(22), to: needed, confidence: "exact" },
        ];
        let mut gaps = merge(&[need(needed, 4)], &BTreeMap::new());
        let applied = apply(&mut gaps, &declared, &maps);
        assert_eq!(applied.via[0].declared_skill, id(22), "exact beats close; of the exact, level 3 beats 2");
        assert_eq!(gaps[0].declared, Some(3));
    }
}
