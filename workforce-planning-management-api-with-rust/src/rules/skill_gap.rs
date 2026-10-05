//! Pure rules for **skills gap analysis**: what a person needs against what
//! they have declared, merged across the sources of a need, graded, and
//! ranked — for one person, and rolled up across a workforce.
//!
//! Like every workforce-intelligence rule it compares **declarations** and
//! never turns a missing declaration into a number: an undeclared skill is
//! `undeclared` (we do not know), has no shortfall, and no priority score —
//! it is a prompt to assess, not a gap of any size.
//!
//! DB-free and clock-free.

use serde::Serialize;
use std::collections::BTreeMap;
use uuid::Uuid;

/// Where a need comes from. Ordered: a role requirement is the firmest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// A minimum the person's current role requires.
    Role,
    /// A level the person set as their own target for the skill.
    Target,
    /// A skill the person aspires to (the person's own view only — never in
    /// workforce roll-ups, since aspirations are private unless shared).
    Aspiration,
}

/// One stated need: a skill at a level, from a source.
#[derive(Debug, Clone)]
pub struct Need {
    /// The skill.
    pub skill: Uuid,
    /// The proficiency (1–5) needed.
    pub required: i32,
    /// `critical`, `important` or `useful` (anything else counts as `useful`).
    pub importance: String,
    /// Where the need comes from.
    pub source: Source,
}

/// How a need stands against the declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Declared at or above the need.
    Met,
    /// Declared, but under the need.
    Below,
    /// Not declared: unknown.
    Undeclared,
}

/// One skill's merged need and where the person stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    /// The skill.
    pub skill: Uuid,
    /// The highest level any source asks for.
    pub required: i32,
    /// The most important of the sources' importances.
    pub importance: String,
    /// The distinct sources, firmest first.
    pub sources: Vec<Source>,
    /// What the person declared, if anything.
    pub declared: Option<i32>,
    /// How they stand.
    pub status: Status,
    /// Levels short — only when declared and below.
    pub shortfall: Option<i32>,
}

/// Importance weight: how much a level of shortfall counts.
#[must_use]
pub fn weight(importance: &str) -> u32 {
    match importance {
        "critical" => 3,
        "important" => 2,
        _ => 1,
    }
}

/// The more important of two importances.
fn stronger<'a>(a: &'a str, b: &'a str) -> &'a str {
    if weight(b) > weight(a) { b } else { a }
}

/// Merge `needs` per skill (highest level, strongest importance, distinct
/// sources) and grade each against `declared`. Result is ordered by skill id,
/// so it is stable; use [`rank`] to order for reading.
#[must_use]
pub fn merge(needs: &[Need], declared: &BTreeMap<Uuid, i32>) -> Vec<Gap> {
    let mut by_skill: BTreeMap<Uuid, (i32, String, Vec<Source>)> = BTreeMap::new();
    for n in needs {
        let entry = by_skill
            .entry(n.skill)
            .or_insert_with(|| (n.required, n.importance.clone(), Vec::new()));
        entry.0 = entry.0.max(n.required);
        entry.1 = stronger(&entry.1, &n.importance).to_string();
        if !entry.2.contains(&n.source) {
            entry.2.push(n.source);
        }
    }
    by_skill
        .into_iter()
        .map(|(skill, (required, importance, mut sources))| {
            sources.sort();
            let have = declared.get(&skill).copied();
            let (status, shortfall) = match have {
                None => (Status::Undeclared, None),
                Some(level) if level >= required => (Status::Met, None),
                Some(level) => (Status::Below, Some(required - level)),
            };
            Gap { skill, required, importance, sources, declared: have, status, shortfall }
        })
        .collect()
}

/// A gap's priority: importance weight × levels short. Zero for a gap that is
/// met or unknown — an unknown has no size.
#[must_use]
pub fn priority(gap: &Gap) -> u32 {
    gap.shortfall
        .map_or(0, |s| weight(&gap.importance) * u32::try_from(s).unwrap_or(0))
}

/// Order for reading: real gaps by priority (highest first), then the
/// unknowns by importance, then met. Ties by skill id so it is stable.
pub fn rank(gaps: &mut [Gap]) {
    let tier = |g: &Gap| match g.status {
        Status::Below => 0,
        Status::Undeclared => 1,
        Status::Met => 2,
    };
    gaps.sort_by(|a, b| {
        tier(a)
            .cmp(&tier(b))
            .then(priority(b).cmp(&priority(a)))
            .then(weight(&b.importance).cmp(&weight(&a.importance)))
            .then(a.skill.cmp(&b.skill))
    });
}

/// One person's standing on one skill, for the workforce roll-up.
#[derive(Debug, Clone)]
pub struct Row {
    /// The skill.
    pub skill: Uuid,
    /// The person's department.
    pub department: String,
    /// How they stand.
    pub status: Status,
    /// Levels short, when below.
    pub shortfall: Option<i32>,
    /// The need's importance.
    pub importance: String,
}

/// A skill's roll-up across a workforce. Counts only — no one is named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillRollup {
    /// The skill.
    pub skill: Uuid,
    /// People who need the skill.
    pub needed_by: usize,
    /// …of whom meet it.
    pub met: usize,
    /// …are below it.
    pub below: usize,
    /// …have not declared it (unknown).
    pub undeclared: usize,
    /// Levels short, summed over those below.
    pub total_shortfall: i32,
    /// Those below on a `critical` need.
    pub critical_below: usize,
    /// Σ importance weight × levels short.
    pub score: u32,
    /// People below, by department, largest first.
    pub departments: Vec<(String, usize)>,
}

/// Roll `rows` up per skill, highest score first (then critical-below, then
/// skill id).
#[must_use]
pub fn rollup(rows: &[Row]) -> Vec<SkillRollup> {
    let mut by_skill: BTreeMap<Uuid, SkillRollup> = BTreeMap::new();
    let mut depts: BTreeMap<Uuid, BTreeMap<String, usize>> = BTreeMap::new();
    for r in rows {
        let e = by_skill.entry(r.skill).or_insert_with(|| SkillRollup {
            skill: r.skill,
            needed_by: 0,
            met: 0,
            below: 0,
            undeclared: 0,
            total_shortfall: 0,
            critical_below: 0,
            score: 0,
            departments: Vec::new(),
        });
        e.needed_by += 1;
        match r.status {
            Status::Met => e.met += 1,
            Status::Undeclared => e.undeclared += 1,
            Status::Below => {
                e.below += 1;
                let s = r.shortfall.unwrap_or(0);
                e.total_shortfall += s;
                e.score += weight(&r.importance) * u32::try_from(s).unwrap_or(0);
                if r.importance == "critical" {
                    e.critical_below += 1;
                }
                *depts.entry(r.skill).or_default().entry(r.department.clone()).or_default() += 1;
            }
        }
    }
    let mut out: Vec<SkillRollup> = by_skill
        .into_values()
        .map(|mut e| {
            let mut d: Vec<(String, usize)> =
                depts.remove(&e.skill).unwrap_or_default().into_iter().collect();
            d.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            e.departments = d;
            e
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(b.critical_below.cmp(&a.critical_below))
            .then(a.skill.cmp(&b.skill))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }
    fn need(skill: u128, required: i32, importance: &str, source: Source) -> Need {
        Need { skill: id(skill), required, importance: importance.to_string(), source }
    }
    fn have(pairs: &[(u128, i32)]) -> BTreeMap<Uuid, i32> {
        pairs.iter().map(|(s, l)| (id(*s), *l)).collect()
    }

    #[test]
    fn needs_merge_to_the_highest_level_strongest_importance_and_all_sources() {
        let gaps = merge(
            &[
                need(1, 3, "useful", Source::Role),
                need(1, 4, "critical", Source::Target),
                need(2, 2, "important", Source::Role),
            ],
            &have(&[(1, 2)]),
        );
        assert_eq!(gaps.len(), 2);
        let one = &gaps[0];
        assert_eq!((one.required, one.importance.as_str()), (4, "critical"));
        assert_eq!(one.sources, [Source::Role, Source::Target]);
        assert_eq!((one.status, one.shortfall), (Status::Below, Some(2)));
    }

    #[test]
    fn met_below_and_undeclared_are_distinct_and_unknown_has_no_number() {
        let gaps = merge(
            &[need(1, 3, "critical", Source::Role), need(2, 3, "critical", Source::Role), need(3, 3, "critical", Source::Role)],
            &have(&[(1, 3), (2, 1)]),
        );
        let by = |s: u128| gaps.iter().find(|g| g.skill == id(s)).unwrap();
        assert_eq!(by(1).status, Status::Met);
        assert_eq!((by(2).status, by(2).shortfall), (Status::Below, Some(2)));
        assert_eq!((by(3).status, by(3).shortfall), (Status::Undeclared, None));
        assert_eq!(priority(by(3)), 0, "an unknown has no size");
        assert_eq!(priority(by(2)), 6, "critical (3) × 2 levels");
    }

    #[test]
    fn ranking_puts_real_gaps_first_by_priority_then_unknowns_then_met() {
        let mut gaps = merge(
            &[
                need(1, 3, "useful", Source::Role),    // below by 2 → 2
                need(2, 3, "critical", Source::Role),  // below by 1 → 3
                need(3, 3, "critical", Source::Role),  // undeclared
                need(4, 3, "useful", Source::Role),    // met
            ],
            &have(&[(1, 1), (2, 2), (4, 5)]),
        );
        rank(&mut gaps);
        let order: Vec<Uuid> = gaps.iter().map(|g| g.skill).collect();
        assert_eq!(order, [id(2), id(1), id(3), id(4)]);
    }

    fn row(skill: u128, dept: &str, status: Status, shortfall: Option<i32>, imp: &str) -> Row {
        Row { skill: id(skill), department: dept.to_string(), status, shortfall, importance: imp.to_string() }
    }

    #[test]
    fn the_rollup_counts_people_never_names_them_and_ranks_by_score() {
        let rows = vec![
            row(1, "eng", Status::Below, Some(2), "critical"),
            row(1, "eng", Status::Below, Some(1), "critical"),
            row(1, "ops", Status::Below, Some(1), "critical"),
            row(1, "ops", Status::Met, None, "critical"),
            row(1, "ops", Status::Undeclared, None, "critical"),
            row(2, "eng", Status::Below, Some(3), "useful"),
        ];
        let out = rollup(&rows);
        assert_eq!(out.len(), 2);
        let first = &out[0];
        assert_eq!(first.skill, id(1));
        assert_eq!((first.needed_by, first.met, first.below, first.undeclared), (5, 1, 3, 1));
        assert_eq!(first.total_shortfall, 4);
        assert_eq!(first.critical_below, 3);
        assert_eq!(first.score, 12, "3 × (2 + 1 + 1)");
        assert_eq!(first.departments, [("eng".to_string(), 2), ("ops".to_string(), 1)]);
        assert_eq!(out[1].score, 3, "useful (1) × 3 levels");
    }

    #[test]
    fn an_all_unknown_skill_scores_zero_but_is_still_listed() {
        let out = rollup(&[row(9, "eng", Status::Undeclared, None, "critical")]);
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].score, out[0].undeclared, out[0].below), (0, 1, 0));
    }
}
