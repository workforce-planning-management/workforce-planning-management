//! The human capital report (WPM-R147, WPM-D100), DB-free and clock-free: figures WPM already works out, each with
//! its definition, its source, its period and what it leaves out, laid beside two reporting frameworks by **name only**.
//! Spec: `spec/human-capital-reporting.md`.
//!
//! - **Mapped, not copied.** A figure carries WPM's own definition and the standard's *area or topic name* and
//!   requirement identifier. No text of ISO 30414 is held, and the ESRS numbers are identifiers and titles.
//! - **An unknown is never a zero.** What WPM cannot compute is [`Value::Unknown`] with a reason.
//! - **Aggregates only.** A count of 1 to `floor - 1` is [`Value::Withheld`]; nothing here names a person.
//!   Zero is shown: it says nobody, which discloses no one.

use chrono::NaiveDate;
use serde::Serialize;

/// What a figure is measured in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// People.
    Workers,
    /// Full-time equivalents, in hundredths (one full-time worker is 100).
    FteHundredths,
    /// Hundredths of a percent (10 000 is the whole).
    BasisPoints,
    /// Whole calendar days.
    CalendarDays,
}

/// Why WPM cannot give a figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// WPM holds no such data (no column, no table, or it is built not to).
    NotRecorded,
    /// WPM has no record of this kind yet.
    NoRecord,
    /// Nothing to divide by or to take a middle of.
    NothingToDivide,
    /// The figure sits behind a stricter read than this report, which is offered by its own route.
    ReadElsewhere,
}

/// A figure's value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Value {
    /// A number.
    Shown {
        /// The number.
        value: i64,
        /// What it counts.
        unit: Unit,
    },
    /// A group under the floor: not shown, and not exact.
    Withheld,
    /// WPM cannot compute it.
    Unknown {
        /// Why.
        reason: Reason,
    },
}

/// The period a figure covers: a date, or a range ending on one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Period {
    /// The first day, for a range; absent for a figure true on one date.
    pub from: Option<NaiveDate>,
    /// The last day, or the date.
    pub to: NaiveDate,
}

/// Where a figure sits in the two frameworks, by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Mapping {
    /// ISO 30414 area, in the 2018 edition's wording; **unconfirmed against the 2025 edition**.
    pub iso_area: &'static str,
    /// ESRS S1 topic; absent where S1 has no such datapoint.
    pub esrs_topic: Option<&'static str>,
    /// The requirement in Delegated Regulation 2023/2772.
    pub esrs_2023: Option<&'static str>,
    /// The requirement in the revised set, Delegated Regulation 2026/1563.
    pub esrs_2026: Option<&'static str>,
}

const EMPLOYEES: Mapping = Mapping {
    iso_area: "Workforce",
    esrs_topic: Some("Characteristics of employees"),
    esrs_2023: Some("S1-6"),
    esrs_2026: Some("S1-5"),
};
const TURNOVER: Mapping = Mapping {
    iso_area: "Recruitment, mobility and turnover",
    esrs_topic: Some("Characteristics of employees"),
    esrs_2023: Some("S1-6"),
    esrs_2026: Some("S1-5"),
};
const RECRUITMENT: Mapping = Mapping {
    iso_area: "Recruitment, mobility and turnover",
    esrs_topic: None,
    esrs_2023: None,
    esrs_2026: None,
};
const COLLECTIVE: Mapping = Mapping {
    iso_area: "Organizational culture",
    esrs_topic: Some("Collective bargaining coverage and social dialogue"),
    esrs_2023: Some("S1-8"),
    esrs_2026: Some("S1-7"),
};
const TRAINING: Mapping = Mapping {
    iso_area: "Skills and capabilities",
    esrs_topic: Some("Training and skills development"),
    esrs_2023: Some("S1-13"),
    esrs_2026: Some("S1-12"),
};
const COST: Mapping = Mapping {
    iso_area: "Costs",
    esrs_topic: None,
    esrs_2023: None,
    esrs_2026: None,
};
const PAY: Mapping = Mapping {
    iso_area: "Diversity",
    esrs_topic: Some("Remuneration metrics"),
    esrs_2023: Some("S1-16"),
    esrs_2026: Some("S1-15"),
};
const SUCCESSION: Mapping = Mapping {
    iso_area: "Succession planning",
    esrs_topic: None,
    esrs_2023: None,
    esrs_2026: None,
};

/// One figure with everything needed to read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Figure {
    /// Stable key.
    pub key: String,
    /// The metric's name.
    pub name: String,
    /// WPM's own definition, one sentence.
    pub definition: &'static str,
    /// The measure and table it reads.
    pub source: &'static str,
    /// The period.
    pub period: Period,
    /// What it leaves out.
    pub exclusions: Vec<String>,
    /// Where it sits in the two frameworks.
    pub mapping: Mapping,
    /// The value.
    #[serde(flatten)]
    pub value: Value,
}

/// A named metric WPM does not cover, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Gap {
    /// The metric.
    pub name: &'static str,
    /// Why it is absent.
    pub why: &'static str,
}

/// The named metrics WPM does not cover. An omission is shown, not silent.
pub const GAPS: &[Gap] = &[
    Gap {
        name: "Health and safety",
        why: "WPM holds no incident or injury record; health data is kept to what is needed for adjustments.",
    },
    Gap {
        name: "Work-life balance",
        why: "WPM holds no family-related leave or working-pattern measure for this report.",
    },
    Gap {
        name: "Adequate wages",
        why: "WPM holds no benchmark of an adequate wage to compare pay with.",
    },
    Gap {
        name: "Social protection",
        why: "WPM holds no record of the social protection a worker is covered by.",
    },
    Gap {
        name: "Persons with disabilities",
        why: "Disability is a declaration a worker may make under equality monitoring, off by default, and is not reported here.",
    },
    Gap {
        name: "Incidents and complaints of discrimination",
        why: "WPM holds no incident or complaint record, by design.",
    },
];

/// Employed workers of one employment type, as the database counted them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCount {
    /// The employment type.
    pub employment_type: String,
    /// Workers of this type employed on the date.
    pub workers: usize,
    /// Their FTE percentages added up.
    pub fte_percent_total: i64,
}

/// A succession plan, reduced to what the report needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// 1 to 5.
    pub criticality: i32,
    /// Whether somebody is ready now.
    pub covered_now: bool,
}

/// What the controller read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inputs {
    /// The date the report is as of.
    pub as_of: NaiveDate,
    /// The first day of the year ending on `as_of`.
    pub from: NaiveDate,
    /// Workers employed on `as_of`, by employment type.
    pub by_type: Vec<TypeCount>,
    /// Headcount the day before `from`.
    pub opening: usize,
    /// Headcount on `as_of`.
    pub closing: usize,
    /// Workers hired in the period.
    pub starters: usize,
    /// Workers who left in the period.
    pub leavers: usize,
    /// Calendar days from opening to filling, for requisitions filled in the period.
    pub fill_days: Vec<i64>,
    /// Training enrolments of workers employed on `as_of`.
    pub enrolments: usize,
    /// Of those, completed on or before `as_of`.
    pub completed: usize,
    /// Succession plans whose incumbent is employed in the organization.
    pub plans: Vec<Plan>,
    /// Succession plans that name no incumbent in the organization, so cannot be placed in it.
    pub plans_unplaced: usize,
}

/// The whole report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The date.
    pub as_of: NaiveDate,
    /// The smallest group shown.
    pub floor: usize,
    /// The figures.
    pub figures: Vec<Figure>,
    /// The named metrics WPM does not cover.
    pub gaps: Vec<Gap>,
}

/// A count under the floor and above zero is withheld.
fn count(n: usize, floor: usize) -> Value {
    if n > 0 && n < floor {
        Value::Withheld
    } else {
        Value::Shown {
            value: i64::try_from(n).unwrap_or(i64::MAX),
            unit: Unit::Workers,
        }
    }
}

/// `part / whole` in basis points, halves rounded up. `None` with nothing to divide.
fn basis_points(part: usize, whole: usize) -> Option<i64> {
    if whole == 0 {
        return None;
    }
    let (p, w) = (i64::try_from(part).ok()?, i64::try_from(whole).ok()?);
    Some((p * 20_000 + w) / (2 * w))
}

fn unknown(reason: Reason) -> Value {
    Value::Unknown { reason }
}

fn bp_value(v: Option<i64>) -> Value {
    v.map_or(unknown(Reason::NothingToDivide), |value| Value::Shown {
        value,
        unit: Unit::BasisPoints,
    })
}

/// The middle of the days, halves rounded up: `None` when there are none.
fn median(days: &[i64]) -> Option<i64> {
    let mut sorted = days.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(sorted[n / 2]),
        _ => Some((sorted[n / 2 - 1] + sorted[n / 2] + 1) / 2),
    }
}

/// The mean of the days, halves rounded up: `None` when there are none.
fn mean(days: &[i64]) -> Option<i64> {
    let n = i64::try_from(days.len()).ok().filter(|n| *n > 0)?;
    Some((days.iter().sum::<i64>() * 2 + n) / (2 * n))
}

#[allow(clippy::too_many_arguments)]
fn figure(
    key: &str,
    name: &str,
    definition: &'static str,
    source: &'static str,
    period: Period,
    exclusions: Vec<String>,
    mapping: Mapping,
    value: Value,
) -> Figure {
    Figure {
        key: key.to_string(),
        name: name.to_string(),
        definition,
        source,
        period,
        exclusions,
        mapping,
        value,
    }
}

const LEFT_OUT_SOFT: &str = "Records that were deleted.";

/// Headcount, FTE and the split by employment type.
fn workforce_figures(i: &Inputs, floor: usize, on: Period) -> Vec<Figure> {
    let mut out = vec![figure(
        "headcount",
        "Headcount",
        "Workers employed on the date: hired on or before it and not terminated on or before it.",
        "workers (the metrics layer's headcount)",
        on,
        vec![LEFT_OUT_SOFT.into()],
        EMPLOYEES,
        count(i.closing, floor),
    )];
    let mut types: Vec<&TypeCount> = i.by_type.iter().collect();
    types.sort_by(|a, b| a.employment_type.cmp(&b.employment_type));
    for t in types {
        let shown = t.workers == 0 || t.workers >= floor;
        out.push(figure(
            &format!("headcount_{}", t.employment_type),
            &format!("Headcount, {}", t.employment_type.replace('_', " ")),
            "Workers of this employment type employed on the date.",
            "workers.employment_type",
            on,
            vec![LEFT_OUT_SOFT.into()],
            EMPLOYEES,
            count(t.workers, floor),
        ));
        out.push(figure(
            &format!("fte_{}", t.employment_type),
            &format!("Full-time equivalents, {}", t.employment_type.replace('_', " ")),
            "The FTE percentages of workers of this employment type, added and shown in hundredths of an FTE.",
            "workers.fte_percent",
            on,
            vec![LEFT_OUT_SOFT.into()],
            EMPLOYEES,
            if shown {
                Value::Shown {
                    value: t.fte_percent_total,
                    unit: Unit::FteHundredths,
                }
            } else {
                Value::Withheld
            },
        ));
    }
    out
}

/// Starters, leavers and turnover.
fn movement_figures(i: &Inputs, floor: usize, over: Period) -> Vec<Figure> {
    let leavers = count(i.leavers, floor);
    let turnover = match (&leavers, i.opening + i.closing) {
        (Value::Withheld, _) => Value::Withheld,
        (_, 0) => unknown(Reason::NothingToDivide),
        (_, both) if both < 2 * floor => Value::Withheld,
        _ => bp_value(basis_points(i.leavers * 2, i.opening + i.closing)),
    };
    vec![
        figure(
            "starters",
            "Starters",
            "Workers whose hire date falls in the period.",
            "workers.hired_on",
            over,
            vec![LEFT_OUT_SOFT.into()],
            TURNOVER,
            count(i.starters, floor),
        ),
        figure(
            "leavers",
            "Leavers",
            "Workers whose termination date falls in the period.",
            "workers.terminated_on",
            over,
            vec![LEFT_OUT_SOFT.into()],
            TURNOVER,
            leavers,
        ),
        figure(
            "turnover_rate",
            "Turnover rate",
            "Leavers divided by the mean of opening and closing headcount.",
            "workers (the metrics layer's turnover rate)",
            over,
            vec![
                LEFT_OUT_SOFT.into(),
                "Withheld when leavers are withheld or the mean headcount is under the floor.".into(),
            ],
            TURNOVER,
            turnover,
        ),
    ]
}

/// Time to fill, training and succession.
fn practice_figures(i: &Inputs, floor: usize, on: Period, over: Period) -> Vec<Figure> {
    let days = |f: fn(&[i64]) -> Option<i64>| -> Value {
        match i.fill_days.len() {
            0 => unknown(Reason::NothingToDivide),
            n if n < floor => Value::Withheld,
            _ => f(&i.fill_days).map_or(unknown(Reason::NothingToDivide), |value| {
                Value::Shown {
                    value,
                    unit: Unit::CalendarDays,
                }
            }),
        }
    };
    let skipped = vec!["A requisition missing its opened or filled date.".to_string()];
    let critical: Vec<&Plan> = i.plans.iter().filter(|p| p.criticality >= 4).collect();
    let succession = match critical.len() {
        0 => unknown(Reason::NothingToDivide),
        n if n < floor => Value::Withheld,
        n => bp_value(basis_points(
            critical.iter().filter(|p| p.covered_now).count(),
            n,
        )),
    };
    let training = match i.enrolments {
        0 => unknown(Reason::NothingToDivide),
        n if n < floor => Value::Withheld,
        n => bp_value(basis_points(i.completed, n)),
    };
    vec![
        figure(
            "time_to_fill_median",
            "Time to fill, median",
            "The middle of the whole calendar days from a requisition's opening to its filling, halves rounded up.",
            "requisitions.opened_on, filled_on",
            over,
            skipped.clone(),
            RECRUITMENT,
            days(median),
        ),
        figure(
            "time_to_fill_mean",
            "Time to fill, mean",
            "The mean of the whole calendar days from a requisition's opening to its filling, halves rounded up.",
            "requisitions.opened_on, filled_on",
            over,
            skipped,
            RECRUITMENT,
            days(mean),
        ),
        figure(
            "training_completion",
            "Training completion",
            "Of the training enrolments of workers employed on the date, the share completed on or before it.",
            "training_enrollments.status, completed_on",
            on,
            vec!["Enrolments of workers who have left.".into()],
            TRAINING,
            training,
        ),
        figure(
            "training_hours",
            "Training hours",
            "Hours of training per worker.",
            "no source",
            on,
            vec![],
            TRAINING,
            unknown(Reason::NotRecorded),
        ),
        figure(
            "succession_coverage",
            "Critical roles with a successor ready now",
            "Of the succession plans of criticality 4 or 5 whose incumbent is employed in the organization, the share with a successor rated ready now.",
            "succession_plans, succession_candidates",
            on,
            vec![format!(
                "{} plan(s) that name no incumbent in this organization.",
                i.plans_unplaced
            )],
            SUCCESSION,
            succession,
        ),
    ]
}

/// The figures WPM does not work out here, kept so the mapping is complete.
fn elsewhere_figures(on: Period) -> Vec<Figure> {
    vec![
        figure(
            "collective_agreement_coverage",
            "Collective agreement coverage",
            "The share of workers covered by a collective agreement.",
            "no source",
            on,
            vec![],
            COLLECTIVE,
            unknown(Reason::NoRecord),
        ),
        figure(
            "workforce_cost",
            "Workforce cost",
            "The cost of the workforce for the period.",
            "payroll (a stricter read than this report)",
            on,
            vec![],
            COST,
            unknown(Reason::ReadElsewhere),
        ),
        figure(
            "pay_gap",
            "Gender pay gap",
            "The pay gap indicators of the Pay Transparency Directive, at /api/pay-transparency/gap-report.",
            "pay transparency report (needs an unmasked salary read)",
            on,
            vec![],
            PAY,
            unknown(Reason::ReadElsewhere),
        ),
    ]
}

/// Build the report. `floor` is the smallest group shown.
#[must_use]
pub fn report(i: &Inputs, floor: usize) -> Report {
    let on = Period {
        from: None,
        to: i.as_of,
    };
    let over = Period {
        from: Some(i.from),
        to: i.as_of,
    };
    let mut figures = workforce_figures(i, floor, on);
    figures.extend(movement_figures(i, floor, over));
    figures.extend(practice_figures(i, floor, on, over));
    figures.extend(elsewhere_figures(on));
    Report {
        as_of: i.as_of,
        floor,
        figures,
        gaps: GAPS.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn inputs() -> Inputs {
        Inputs {
            as_of: d(2026, 10, 24),
            from: d(2025, 10, 25),
            by_type: vec![
                TypeCount {
                    employment_type: "permanent".into(),
                    workers: 40,
                    fte_percent_total: 3_600,
                },
                TypeCount {
                    employment_type: "contractor".into(),
                    workers: 3,
                    fte_percent_total: 300,
                },
            ],
            opening: 40,
            closing: 43,
            starters: 6,
            leavers: 3,
            fill_days: vec![20, 30, 40, 50, 60, 70, 80, 90, 100, 110],
            enrolments: 20,
            completed: 15,
            plans: vec![],
            plans_unplaced: 0,
        }
    }

    fn find<'a>(r: &'a Report, key: &str) -> &'a Figure {
        r.figures.iter().find(|f| f.key == key).unwrap()
    }

    fn shown(f: &Figure) -> i64 {
        match f.value {
            Value::Shown { value, .. } => value,
            ref other => panic!("{} is {other:?}", f.key),
        }
    }

    #[test]
    fn a_count_under_the_floor_is_withheld_and_zero_is_shown() {
        assert_eq!(count(0, 10), Value::Shown { value: 0, unit: Unit::Workers });
        assert_eq!(count(1, 10), Value::Withheld);
        assert_eq!(count(9, 10), Value::Withheld);
        assert_eq!(count(10, 10), Value::Shown { value: 10, unit: Unit::Workers });
    }

    #[test]
    fn small_types_are_withheld_in_head_count_and_in_fte() {
        let r = report(&inputs(), 10);
        assert_eq!(shown(find(&r, "headcount")), 43);
        assert_eq!(shown(find(&r, "headcount_permanent")), 40);
        assert_eq!(shown(find(&r, "fte_permanent")), 3_600);
        assert_eq!(find(&r, "headcount_contractor").value, Value::Withheld);
        assert_eq!(
            find(&r, "fte_contractor").value,
            Value::Withheld,
            "FTE would give the count back"
        );
    }

    #[test]
    fn turnover_is_leavers_over_the_mean_headcount_and_withheld_with_its_leavers() {
        let mut i = inputs();
        i.leavers = 10;
        let r = report(&i, 10);
        // 10 / mean(40, 43) = 10 / 41.5 = 24.096..% -> 2410 bp.
        assert_eq!(shown(find(&r, "turnover_rate")), 2_410);
        // Three leavers: the count is withheld, and so is the rate that would give it back.
        let r = report(&inputs(), 10);
        assert_eq!(find(&r, "leavers").value, Value::Withheld);
        assert_eq!(find(&r, "turnover_rate").value, Value::Withheld);
    }

    #[test]
    fn nobody_leaving_is_zero_not_unknown_and_an_empty_organization_is_unknown() {
        let mut i = inputs();
        i.leavers = 0;
        let r = report(&i, 10);
        assert_eq!(shown(find(&r, "leavers")), 0);
        assert_eq!(shown(find(&r, "turnover_rate")), 0);
        i.opening = 0;
        i.closing = 0;
        i.by_type.clear();
        let r = report(&i, 10);
        assert_eq!(
            find(&r, "turnover_rate").value,
            Value::Unknown { reason: Reason::NothingToDivide }
        );
    }

    #[test]
    fn a_small_organization_withholds_its_rate_even_with_shown_leavers() {
        let mut i = inputs();
        (i.opening, i.closing, i.leavers) = (8, 9, 0);
        assert_eq!(
            find(&report(&i, 10), "turnover_rate").value,
            Value::Withheld
        );
    }

    #[test]
    fn time_to_fill_takes_the_middle_and_the_mean_and_needs_the_floor() {
        let r = report(&inputs(), 10);
        // Ten values: middle two are 60 and 70.
        assert_eq!(shown(find(&r, "time_to_fill_median")), 65);
        assert_eq!(shown(find(&r, "time_to_fill_mean")), 65);
        let mut i = inputs();
        i.fill_days = vec![5; 9];
        assert_eq!(
            find(&report(&i, 10), "time_to_fill_median").value,
            Value::Withheld
        );
        i.fill_days.clear();
        assert_eq!(
            find(&report(&i, 10), "time_to_fill_median").value,
            Value::Unknown { reason: Reason::NothingToDivide }
        );
    }

    #[test]
    fn halves_round_up_in_days() {
        assert_eq!(median(&[1, 2]), Some(2));
        assert_eq!(mean(&[1, 2]), Some(2));
        assert_eq!(median(&[3, 1, 2]), Some(2));
        assert_eq!(mean(&[]), None);
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn training_completion_is_a_share_and_hours_are_unknown() {
        let r = report(&inputs(), 10);
        assert_eq!(shown(find(&r, "training_completion")), 7_500);
        assert_eq!(
            find(&r, "training_hours").value,
            Value::Unknown { reason: Reason::NotRecorded }
        );
        let mut i = inputs();
        i.enrolments = 0;
        assert_eq!(
            find(&report(&i, 10), "training_completion").value,
            Value::Unknown { reason: Reason::NothingToDivide }
        );
    }

    #[test]
    fn succession_counts_only_critical_roles_and_states_the_plans_it_could_not_place() {
        let mut i = inputs();
        i.plans = (0..12)
            .map(|n| Plan {
                criticality: if n < 10 { 5 } else { 2 },
                covered_now: n % 2 == 0,
            })
            .collect();
        i.plans_unplaced = 2;
        let r = report(&i, 10);
        let f = find(&r, "succession_coverage");
        // Ten critical plans, five covered now.
        assert_eq!(shown(f), 5_000);
        assert!(f.exclusions[0].starts_with("2 plan"));
        i.plans.truncate(9);
        assert_eq!(
            find(&report(&i, 10), "succession_coverage").value,
            Value::Withheld
        );
    }

    #[test]
    fn what_wpm_cannot_work_out_is_unknown_with_a_reason() {
        let r = report(&inputs(), 10);
        for (key, reason) in [
            ("collective_agreement_coverage", Reason::NoRecord),
            ("workforce_cost", Reason::ReadElsewhere),
            ("pay_gap", Reason::ReadElsewhere),
        ] {
            assert_eq!(find(&r, key).value, Value::Unknown { reason }, "{key}");
        }
    }

    #[test]
    fn every_figure_is_named_defined_sourced_and_dated() {
        let r = report(&inputs(), 10);
        let keys: std::collections::BTreeSet<&str> =
            r.figures.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys.len(), r.figures.len(), "keys are unique");
        for f in &r.figures {
            assert!(!f.name.is_empty() && !f.definition.is_empty() && !f.source.is_empty());
            assert_eq!(f.period.to, r.as_of);
        }
        assert!(find(&r, "turnover_rate").period.from.is_some());
        assert!(find(&r, "headcount").period.from.is_none());
    }

    #[test]
    fn the_gaps_are_listed_not_left_out() {
        let r = report(&inputs(), 10);
        assert_eq!(r.gaps.len(), 6);
        assert!(r.gaps.iter().all(|g| !g.why.is_empty()));
    }

    /// Where a requirement number is quoted it says which set. Both are identifiers, and the 2026 set differs from the
    /// 2023 one wherever a topic moved.
    #[test]
    fn requirement_numbers_name_both_sets_and_differ_where_the_standard_moved() {
        let mut seen = 0;
        for m in [EMPLOYEES, TURNOVER, COLLECTIVE, TRAINING, PAY] {
            let (old, new) = (m.esrs_2023.unwrap(), m.esrs_2026.unwrap());
            assert!(old.starts_with("S1-") && new.starts_with("S1-"));
            assert_ne!(old, new, "{:?}", m.esrs_topic);
            seen += 1;
        }
        assert_eq!(seen, 5);
        for m in [RECRUITMENT, COST, SUCCESSION] {
            assert!(m.esrs_topic.is_none() && m.esrs_2023.is_none() && m.esrs_2026.is_none());
        }
    }

    /// WPM-D100: standards are mapped, not copied. Everything quoted from a standard is an area, a topic or an
    /// identifier: short, one line, no sentence. A licensed paragraph would fail this by its length alone.
    #[test]
    fn nothing_quoted_from_a_standard_is_more_than_a_name() {
        for m in [EMPLOYEES, TURNOVER, RECRUITMENT, COLLECTIVE, TRAINING, COST, PAY, SUCCESSION] {
            for text in [Some(m.iso_area), m.esrs_topic, m.esrs_2023, m.esrs_2026]
                .into_iter()
                .flatten()
            {
                assert!(text.len() <= 60, "{text}");
                assert!(!text.contains('.') && !text.contains('\n'), "{text}");
            }
        }
    }

    #[test]
    fn the_report_names_no_one() {
        let json = serde_json::to_string(&report(&inputs(), 10)).unwrap();
        for word in ["pid", "worker_pid", "display_name", "person_ref", "email"] {
            assert!(!json.contains(word), "{word}");
        }
    }
}
