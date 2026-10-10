//! Pure rules for **strategic workforce planning** (WPM-R36, WPM-R37,
//! WPM-D26–D28): workforce plans, demand lines, and the transparent
//! arithmetic behind the supply projection and the headcount gap.
//!
//! DB-free and clock-free (dates are always supplied). Forecasting here is
//! **stated assumptions plus arithmetic** — no model, nothing imputed: with
//! too little history the answer is `None` (`insufficient_history`), never
//! an extrapolation (WPM-D27).

use chrono::NaiveDate;

use crate::rules::lifecycle;

/// Plan lifecycle statuses.
pub const STATUSES: &[&str] = &["draft", "active", "archived"];

/// The legal plan transitions; `archived` is terminal.
pub const MACHINE: &[(&str, &str)] = &[
    ("draft", "active"),
    ("draft", "archived"),
    ("active", "archived"),
];

/// The most headcount one demand line may ask for.
pub const MAX_LINE_HEADCOUNT: i32 = 100_000;

/// Snapshots needed before an observed attrition rate is offered (the first
/// has no leavers figure, so this is `MIN_SNAPSHOTS - 1` measured windows).
pub const MIN_SNAPSHOTS: usize = 3;

/// The shortest span of history (days) an observed rate may rest on.
pub const MIN_WINDOW_DAYS: i64 = 60;

/// Check a plan status transition.
///
/// # Errors
/// A message naming the current state.
pub fn check_transition(from: &str, to: &str) -> Result<(), String> {
    lifecycle::check("workforce plan", MACHINE, from, to)
}

/// Validate a plan: a name, an ordered horizon, and an attrition
/// assumption (basis points per year, 0–10000) when given.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_plan(
    name: &str,
    horizon_start: NaiveDate,
    horizon_end: NaiveDate,
    attrition_bp: Option<i32>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("name is required".to_string());
    }
    if horizon_end < horizon_start {
        return Err("horizon_end must not be before horizon_start".to_string());
    }
    if attrition_bp.is_some_and(|bp| !(0..=10_000).contains(&bp)) {
        return Err("attrition_bp must be between 0 and 10000".to_string());
    }
    Ok(())
}

/// Validate a demand line: a department, a target date inside the plan's
/// horizon, and a sane headcount.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_line(
    department: &str,
    target_on: NaiveDate,
    horizon_start: NaiveDate,
    horizon_end: NaiveDate,
    target_headcount: i32,
) -> Result<(), String> {
    if department.trim().is_empty() {
        return Err("department is required".to_string());
    }
    if target_on < horizon_start || target_on > horizon_end {
        return Err("target_on must fall within the plan's horizon".to_string());
    }
    if !(0..=MAX_LINE_HEADCOUNT).contains(&target_headcount) {
        return Err(format!(
            "target_headcount must be between 0 and {MAX_LINE_HEADCOUNT}"
        ));
    }
    Ok(())
}

/// One recorded headcount snapshot, aggregated over the series being rated.
#[derive(Debug, Clone, Copy)]
pub struct Snap {
    /// Snapshot date.
    pub as_of: NaiveDate,
    /// Employed headcount on that date.
    pub headcount: i64,
    /// Leavers since the previous snapshot; `None` for the first.
    pub leavers: Option<i64>,
}

/// Observed annual attrition in basis points from a series of snapshots:
/// leavers over the window, annualised, divided by the mean headcount.
/// `None` — *insufficient history* — when there are fewer than
/// [`MIN_SNAPSHOTS`] snapshots with known leavers, the span is shorter than
/// [`MIN_WINDOW_DAYS`], or the mean headcount is zero.
#[must_use]
pub fn observed_attrition_bp(snaps: &[Snap]) -> Option<i32> {
    let mut sorted = snaps.to_vec();
    sorted.sort_by_key(|s| s.as_of);
    if sorted.len() < MIN_SNAPSHOTS {
        return None;
    }
    // Every snapshot after the first must say how many people left.
    let known: Vec<&Snap> = sorted.iter().skip(1).collect();
    if known.iter().any(|s| s.leavers.is_none()) {
        return None;
    }
    let first = sorted.first()?;
    let last = sorted.last()?;
    let days = (last.as_of - first.as_of).num_days();
    if days < MIN_WINDOW_DAYS {
        return None;
    }
    let leavers: i64 = known.iter().filter_map(|s| s.leavers).sum();
    let mean_headcount_x = sorted.iter().map(|s| s.headcount).sum::<i64>();
    let n = i64::try_from(sorted.len()).ok()?;
    if mean_headcount_x == 0 {
        return None;
    }
    // bp = leavers * (365 / days) / (sum / n) * 10000, in integer arithmetic.
    let numerator = i128::from(leavers) * 365 * i128::from(n) * 10_000;
    let denominator = i128::from(days) * i128::from(mean_headcount_x);
    let bp = (numerator + denominator / 2) / denominator;
    i32::try_from(bp).ok()
}

/// Projected supply after `days` at an annual attrition of `attrition_bp`
/// basis points, from `opening` employed workers; **no hires are assumed**.
/// Rounded to the nearest whole person and never negative.
#[must_use]
pub fn project_supply(opening: usize, attrition_bp: i32, days: i64) -> i64 {
    let opening = i128::try_from(opening).unwrap_or(i128::MAX);
    let days = i128::from(days.max(0));
    let bp = i128::from(attrition_bp.max(0));
    let denominator = 10_000 * 365;
    let leavers = (opening * bp * days + denominator / 2) / denominator;
    i64::try_from((opening - leavers).max(0)).unwrap_or(i64::MAX)
}

/// One employed worker, as the basis-aware projection sees them (WPM-R85). No name and no
/// reference to the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Person {
    /// How they are engaged.
    pub basis: crate::rules::engagement::Basis,
    /// Contracted FTE as a percentage (`100` is full time).
    pub fte_percent: i32,
    /// The last day of the engagement; `None` where none is recorded. An extension moves
    /// this date, so a recorded extension is already in it.
    pub ends_on: Option<NaiveDate>,
}

/// Supply from one basis at the target date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasisSupply {
    /// People engaged on the target date. `None` when it cannot be said (permanent staff
    /// exist and there is not enough history for an attrition rate).
    pub headcount: Option<i64>,
    /// Their FTE in hundredths, under the same condition.
    pub fte_centi: Option<i64>,
    /// People of a basis that needs an end date, or an intern, with none recorded; they
    /// are counted as continuing and named in the assumptions.
    pub no_end_date: usize,
    /// People whose recorded end falls before the target date, so are not counted.
    pub leaving_before: usize,
}

/// The supply projection for every basis at one target date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    /// Permanent staff, projected by attrition.
    pub permanent: BasisSupply,
    /// Fixed-term staff, projected by their known end dates.
    pub fixed_term: BasisSupply,
    /// Contractors, projected by their known end dates.
    pub contractor: BasisSupply,
    /// Interns, projected by their end dates where they have one.
    pub intern: BasisSupply,
    /// All bases together; `None` when any part cannot be said.
    pub total_headcount: Option<i64>,
    /// All bases together, in FTE hundredths.
    pub total_fte_centi: Option<i64>,
    /// Each assumption, named.
    pub assumptions: Vec<String>,
}

/// Apply an annual attrition (basis points) over `days` calendar days to an amount,
/// rounded to the nearest whole unit and never negative.
fn attrit(opening: i64, attrition_bp: i32, days: i64) -> i64 {
    let days = i128::from(days.max(0));
    let bp = i128::from(attrition_bp.max(0));
    let opening = i128::from(opening);
    let denominator = 10_000 * 365;
    let leavers = (opening * bp * days + denominator / 2) / denominator;
    i64::try_from((opening - leavers).max(0)).unwrap_or(i64::MAX)
}

/// Project supply at `target`, each basis on its own terms (WPM-R85, WPM-D57):
///
/// - **fixed-term, contractor and intern** leave on their known end date: engaged on that
///   day, gone the day after. Someone with no end date recorded is counted as continuing
///   and the assumption says how many;
/// - **permanent** staff leave at the observed or stated annual attrition, which applies to
///   them **only**. With permanent staff and no rate (`insufficient_history`) their supply,
///   and the totals, are `None`, never zero. With no permanent staff the permanent supply
///   is a known zero.
///
/// No hires are assumed. Headcount and FTE are both given.
#[must_use]
pub fn project_by_basis(
    people: &[Person],
    as_of: NaiveDate,
    target: NaiveDate,
    permanent_attrition_bp: Option<i32>,
    attrition_source: &str,
) -> Projection {
    use crate::rules::engagement::Basis;
    let days = (target - as_of).num_days().max(0);
    let supply_of = |basis: Basis| -> BasisSupply {
        let group: Vec<&Person> = people.iter().filter(|p| p.basis == basis).collect();
        let opening = i64::try_from(group.len()).unwrap_or(i64::MAX);
        let opening_fte: i64 = group.iter().map(|p| i64::from(p.fte_percent.max(0))).sum();
        if basis == Basis::Permanent {
            return match (opening, permanent_attrition_bp) {
                (0, _) => BasisSupply {
                    headcount: Some(0),
                    fte_centi: Some(0),
                    no_end_date: 0,
                    leaving_before: 0,
                },
                (_, Some(bp)) => BasisSupply {
                    headcount: Some(attrit(opening, bp, days)),
                    fte_centi: Some(attrit(opening_fte, bp, days)),
                    no_end_date: 0,
                    leaving_before: 0,
                },
                (_, None) => BasisSupply {
                    headcount: None,
                    fte_centi: None,
                    no_end_date: 0,
                    leaving_before: 0,
                },
            };
        }
        let mut headcount = 0;
        let mut fte = 0;
        let mut no_end = 0;
        let mut leaving = 0;
        for person in &group {
            match person.ends_on {
                None => {
                    no_end += 1;
                    headcount += 1;
                    fte += i64::from(person.fte_percent.max(0));
                }
                Some(end) if end >= target => {
                    headcount += 1;
                    fte += i64::from(person.fte_percent.max(0));
                }
                Some(_) => leaving += 1,
            }
        }
        BasisSupply {
            headcount: Some(headcount),
            fte_centi: Some(fte),
            no_end_date: no_end,
            leaving_before: leaving,
        }
    };
    let permanent = supply_of(Basis::Permanent);
    let fixed_term = supply_of(Basis::FixedTerm);
    let contractor = supply_of(Basis::Contractor);
    let intern = supply_of(Basis::Intern);
    let parts = [permanent, fixed_term, contractor, intern];
    let sum = |pick: fn(&BasisSupply) -> Option<i64>| -> Option<i64> {
        parts.iter().map(pick).sum::<Option<i64>>()
    };
    let mut assumptions = vec!["No hires are assumed.".to_string()];
    match permanent_attrition_bp {
        Some(bp) => assumptions.push(format!(
            "Permanent staff leave at {bp} basis points a year ({attrition_source}); the rate is applied to permanent staff only."
        )),
        None => assumptions.push(
            "Permanent staff: insufficient history for an attrition rate, so their supply is not projected.".to_string(),
        ),
    }
    assumptions.push(
        "Fixed-term, contractor and intern engagements leave the day after their recorded end date; a recorded extension has already moved it.".to_string(),
    );
    for (label, part) in [
        ("fixed-term", fixed_term),
        ("contractor", contractor),
        ("intern", intern),
    ] {
        if part.no_end_date > 0 {
            assumptions.push(format!(
                "{} {label} engagement(s) have no end date recorded and are counted as continuing.",
                part.no_end_date
            ));
        }
    }
    Projection {
        permanent,
        fixed_term,
        contractor,
        intern,
        total_headcount: sum(|p| p.headcount),
        total_fte_centi: sum(|p| p.fte_centi),
        assumptions,
    }
}

/// Headcount gap: demand minus projected supply. Positive is a shortfall,
/// negative a surplus.
#[must_use]
pub fn headcount_gap(demand: i64, supply: i64) -> i64 {
    demand - supply
}

/// Suggested levers for a headcount gap — suggestions with evidence, never
/// decisions (WPM-D28).
#[must_use]
pub fn suggested_levers(gap: i64) -> Vec<&'static str> {
    match gap.cmp(&0) {
        std::cmp::Ordering::Greater => vec!["hire", "reskill", "promote"],
        std::cmp::Ordering::Less => vec!["redeploy"],
        std::cmp::Ordering::Equal => Vec::new(),
    }
}

/// People still needed with a skill: the line's headcount less those
/// already proficient (never negative).
#[must_use]
pub fn competency_shortfall(needed: usize, proficient: usize) -> usize {
    needed.saturating_sub(proficient)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(basis: crate::rules::engagement::Basis, fte: i32, ends: Option<NaiveDate>) -> Person {
        Person {
            basis,
            fte_percent: fte,
            ends_on: ends,
        }
    }

    #[test]
    fn each_basis_is_projected_on_its_own_terms() {
        use crate::rules::engagement::Basis::{Contractor, FixedTerm, Intern, Permanent};
        let as_of = day(2026, 1, 1);
        let target = day(2026, 7, 1);
        let people = [
            person(Permanent, 100, None),
            person(Permanent, 50, None),
            person(FixedTerm, 100, Some(day(2026, 7, 1))), // ends on the target date: still engaged
            person(FixedTerm, 100, Some(day(2026, 6, 30))), // ends the day before: gone
            person(Contractor, 100, Some(day(2026, 12, 31))), // an extension past the target
            person(Contractor, 80, None),                  // no end date: counted, and named
            person(Intern, 100, Some(day(2026, 3, 1))),
        ];
        let p = project_by_basis(&people, as_of, target, Some(0), "stated");
        assert_eq!(p.fixed_term.headcount, Some(1));
        assert_eq!(p.fixed_term.fte_centi, Some(100));
        assert_eq!(p.fixed_term.leaving_before, 1);
        assert_eq!(p.contractor.headcount, Some(2));
        assert_eq!(p.contractor.fte_centi, Some(180));
        assert_eq!(p.contractor.no_end_date, 1);
        assert_eq!(p.intern.headcount, Some(0));
        // With no attrition every permanent worker stays.
        assert_eq!(p.permanent.headcount, Some(2));
        assert_eq!(p.permanent.fte_centi, Some(150));
        assert_eq!(p.total_headcount, Some(5));
        assert_eq!(p.total_fte_centi, Some(100 + 150 + 180));
        assert!(
            p.assumptions
                .iter()
                .any(|a| a.contains("1 contractor engagement(s) have no end date"))
        );
    }

    #[test]
    fn attrition_applies_to_permanent_staff_only() {
        use crate::rules::engagement::Basis::{Contractor, Permanent};
        let as_of = day(2026, 1, 1);
        let target = day(2027, 1, 1); // 365 calendar days
        let mut people = vec![person(Contractor, 100, Some(day(2028, 1, 1))); 10];
        people.extend(vec![person(Permanent, 100, None); 10]);
        // 10% a year: one of ten permanent staff leaves; the ten contractors are untouched.
        let p = project_by_basis(&people, as_of, target, Some(1_000), "observed");
        assert_eq!(p.permanent.headcount, Some(9));
        assert_eq!(p.permanent.fte_centi, Some(900));
        assert_eq!(p.contractor.headcount, Some(10));
        assert_eq!(p.total_headcount, Some(19));
        assert!(
            p.assumptions
                .iter()
                .any(|a| a.contains("permanent staff only"))
        );
    }

    #[test]
    fn permanent_without_a_rate_is_unknown_never_zero() {
        use crate::rules::engagement::Basis::{Contractor, Permanent};
        let people = [
            person(Permanent, 100, None),
            person(Contractor, 100, Some(day(2027, 1, 1))),
        ];
        let p = project_by_basis(&people, day(2026, 1, 1), day(2026, 7, 1), None, "none");
        assert_eq!(p.permanent.headcount, None, "insufficient history");
        assert_eq!(p.permanent.fte_centi, None);
        assert_eq!(
            p.contractor.headcount,
            Some(1),
            "known end dates need no history"
        );
        assert_eq!(
            p.total_headcount, None,
            "a total with an unknown part is unknown"
        );
        assert_eq!(p.total_fte_centi, None);
        assert!(
            p.assumptions
                .iter()
                .any(|a| a.contains("insufficient history"))
        );
    }

    #[test]
    fn no_permanent_staff_is_a_known_zero_even_without_a_rate() {
        use crate::rules::engagement::Basis::Contractor;
        let people = [person(Contractor, 100, Some(day(2027, 1, 1)))];
        let p = project_by_basis(&people, day(2026, 1, 1), day(2026, 7, 1), None, "none");
        assert_eq!(p.permanent.headcount, Some(0));
        assert_eq!(p.total_headcount, Some(1));
    }

    #[test]
    fn a_target_in_the_past_projects_no_attrition() {
        use crate::rules::engagement::Basis::Permanent;
        let people = [person(Permanent, 100, None)];
        let p = project_by_basis(
            &people,
            day(2026, 6, 1),
            day(2026, 1, 1),
            Some(5_000),
            "stated",
        );
        assert_eq!(p.permanent.headcount, Some(1));
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// Plans are one-way: draft → active → archived.
    #[test]
    fn lifecycle_machine() {
        assert!(check_transition("draft", "active").is_ok());
        assert!(check_transition("active", "archived").is_ok());
        assert!(check_transition("draft", "archived").is_ok());
        assert!(check_transition("archived", "active").is_err());
        assert!(check_transition("active", "draft").is_err());
        for (from, to) in MACHINE {
            assert!(STATUSES.contains(from) && STATUSES.contains(to));
        }
    }

    /// A plan needs a name, an ordered horizon, and an in-range assumption.
    #[test]
    fn plans_are_validated() {
        let (a, b) = (day(2027, 1, 1), day(2027, 12, 31));
        assert!(validate_plan("FY27", a, b, Some(1200)).is_ok());
        assert!(validate_plan("FY27", a, b, None).is_ok());
        assert!(validate_plan(" ", a, b, None).is_err());
        assert!(validate_plan("x", b, a, None).is_err());
        assert!(validate_plan("x", a, b, Some(10_001)).is_err());
        assert!(validate_plan("x", a, b, Some(-1)).is_err());
    }

    /// A demand line must sit inside the horizon with a sane headcount.
    #[test]
    fn lines_are_validated() {
        let (a, b) = (day(2027, 1, 1), day(2027, 12, 31));
        assert!(validate_line("eng", day(2027, 6, 30), a, b, 12).is_ok());
        assert!(
            validate_line("eng", a, a, b, 0).is_ok(),
            "zero is a real target"
        );
        assert!(validate_line("", day(2027, 6, 30), a, b, 1).is_err());
        assert!(validate_line("eng", day(2028, 1, 1), a, b, 1).is_err());
        assert!(validate_line("eng", day(2027, 6, 30), a, b, -1).is_err());
        assert!(validate_line("eng", day(2027, 6, 30), a, b, 100_001).is_err());
    }

    fn snap(d: NaiveDate, headcount: i64, leavers: Option<i64>) -> Snap {
        Snap {
            as_of: d,
            headcount,
            leavers,
        }
    }

    /// An observed rate needs enough snapshots over enough time; otherwise
    /// it is `None`, not a guess.
    #[test]
    fn observed_attrition_needs_history() {
        assert_eq!(observed_attrition_bp(&[]), None);
        let two = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 7, 1), 100, Some(5)),
        ];
        assert_eq!(
            observed_attrition_bp(&two),
            None,
            "two snapshots are not enough"
        );
        let gap = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 7, 1), 100, None),
            snap(day(2027, 1, 1), 100, Some(5)),
        ];
        assert_eq!(
            observed_attrition_bp(&gap),
            None,
            "a snapshot without a leavers figure breaks the window"
        );
        let short = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 1, 8), 100, Some(1)),
            snap(day(2026, 1, 15), 100, Some(1)),
            snap(day(2026, 1, 22), 100, Some(1)),
        ];
        assert_eq!(
            observed_attrition_bp(&short),
            None,
            "three weeks is too short"
        );
    }

    /// 10 leavers over 365 days at a mean headcount of 100 is 1000 bp.
    #[test]
    fn observed_attrition_is_annualised() {
        let series = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 7, 1), 100, Some(5)),
            snap(day(2027, 1, 1), 100, Some(5)),
        ];
        // 365 days: 10 leavers / mean 100 = 10% = 1000 bp.
        assert_eq!(observed_attrition_bp(&series), Some(1000));
        // Order of input does not matter.
        let shuffled = [series[2], series[0], series[1]];
        assert_eq!(observed_attrition_bp(&shuffled), Some(1000));
        // No leavers is a real 0, not absent.
        let none = [
            snap(day(2026, 1, 1), 50, None),
            snap(day(2026, 7, 1), 50, Some(0)),
            snap(day(2027, 1, 1), 50, Some(0)),
        ];
        assert_eq!(observed_attrition_bp(&none), Some(0));
        let empty = [
            snap(day(2026, 1, 1), 0, None),
            snap(day(2026, 7, 1), 0, Some(0)),
            snap(day(2027, 1, 1), 0, Some(0)),
        ];
        assert_eq!(observed_attrition_bp(&empty), None, "no base ⇒ no rate");
    }

    /// Supply falls by expected leavers, rounds to a person, never goes
    /// below zero, and assumes no hires.
    #[test]
    fn supply_projection() {
        assert_eq!(project_supply(100, 1000, 365), 90);
        assert_eq!(project_supply(100, 1000, 0), 100, "today is today");
        assert_eq!(project_supply(100, 0, 365), 100);
        assert_eq!(
            project_supply(100, 1000, 182),
            95,
            "half a year ≈ 5 leavers"
        );
        assert_eq!(project_supply(3, 10_000, 365 * 5), 0, "never negative");
        assert_eq!(project_supply(0, 1000, 365), 0);
        assert_eq!(
            project_supply(100, 1000, -5),
            100,
            "past dates do not grow supply"
        );
    }

    /// Gap sign and lever suggestions.
    #[test]
    fn gap_and_levers() {
        assert_eq!(headcount_gap(12, 9), 3);
        assert_eq!(headcount_gap(8, 9), -1);
        assert_eq!(suggested_levers(3), ["hire", "reskill", "promote"]);
        assert_eq!(suggested_levers(-1), ["redeploy"]);
        assert_eq!(suggested_levers(0), Vec::<&str>::new());
        assert_eq!(competency_shortfall(10, 4), 6);
        assert_eq!(competency_shortfall(3, 8), 0);
    }
}
