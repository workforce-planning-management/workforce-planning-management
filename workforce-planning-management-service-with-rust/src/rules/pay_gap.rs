//! **Pay gap indicators** (WPM-R144, WPM-D99): the seven items of Article 9(1) of the EU Pay Transparency
//! Directive, computed from pay the caller has already put on a comparable basis. DB-free and clock-free.
//! Spec: `spec/pay-transparency.md`.
//!
//! What this module refuses to do, on purpose:
//!
//! - **It never guesses.** A pay component nobody recorded is not zero ([`Variable::Unknown`]); a worker with no
//!   recorded pay, or no declared female or male sex, is **excluded and counted**, never inferred.
//! - **It names no one and publishes no small count.** A cell with fewer workers than the floor on either side is
//!   [`Figure::Withheld`]; a side with nobody in it is [`Figure::Unknown`] (a gap needs two sides). No figure in a
//!   [`Report`] is a head count, so a withheld cell cannot be worked out from the others; an exclusion count under
//!   the floor is shown as under the floor.
//! - **The floor is never below** [`equality_monitoring::MIN_FLOOR`].
//!
//! The comparable basis (for example gross hourly pay, or full-time-equivalent annual pay) is the caller's choice and
//! is recorded with the report by the layer above; every amount here is in minor units on that one basis.

use std::collections::BTreeMap;

use chrono::{Months, NaiveDate};
use serde::Serialize;

use super::equality_monitoring;

/// The sex a worker has chosen to declare, as far as the indicators can use it. The Directive compares female and
/// male workers; any other answer is [`Declared::Other`] and the worker is counted as excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declared {
    /// Declared female.
    Female,
    /// Declared male.
    Male,
    /// Declared anything else, including "prefer not to say".
    Other,
}

/// A complementary or variable component of pay (a bonus, an allowance, a payment in kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variable {
    /// Nobody recorded it. Not zero.
    Unknown,
    /// Recorded, in minor units. `Known(0)` is "recorded: none received".
    Known(i64),
}

/// One worker's pay for the reporting period, with no name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPay {
    /// A stable key (the worker pid), used only to order ties; it never leaves this module.
    pub key: String,
    /// The declared sex, if any. `None` when the worker has not answered or equality monitoring is off.
    pub sex: Option<Declared>,
    /// The category of workers this worker's role is in, if the employer has put it in one.
    pub category: Option<String>,
    /// Ordinary basic pay on the comparable basis, in minor units; `None` when not recorded.
    pub basic_minor: Option<i64>,
    /// Complementary or variable pay on the same basis.
    pub variable: Variable,
}

/// Which workers a gap in variable components is taken over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VariableBasis {
    /// Every worker whose variable pay is known, one who received none counting as zero.
    #[default]
    AllKnown,
    /// Only workers who received some.
    RecipientsOnly,
}

/// Why a figure cannot be given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// Nobody of either sex is in the cell.
    NoWorkers,
    /// Nobody of one sex is in the cell, so there is nothing to compare.
    OneSided,
    /// The male figure is zero or less, so a gap as a share of it has no meaning.
    NoBase,
}

/// A figure, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum Figure<T> {
    /// Shown.
    Shown(T),
    /// Fewer workers than the floor on at least one side: no figure, and no count.
    Withheld,
    /// Cannot be worked out.
    Unknown(Reason),
}

/// A gap between female and male pay, with the two figures it comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gap {
    /// The female mean or median, in minor units, rounded half away from zero.
    pub female_minor: i64,
    /// The male mean or median.
    pub male_minor: i64,
    /// `(male - female) / male` in basis points, rounded half away from zero. Negative when women are paid more.
    pub gap_bp: i64,
}

/// The share of each sex that received a component, in basis points of that sex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Shares {
    /// Of female workers.
    pub female_bp: i64,
    /// Of male workers.
    pub male_bp: i64,
}

/// Who is in one pay quartile band, in basis points of the band (the two add to 10,000).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Composition {
    /// Female share of the band.
    pub female_bp: i64,
    /// Male share of the band.
    pub male_bp: i64,
}

/// The gaps for one category of workers (Article 9(1)(g)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CategoryGap {
    /// The category's name as the employer set it.
    pub category: String,
    /// The mean gap in ordinary basic pay.
    pub basic: Figure<Gap>,
    /// The mean gap in complementary or variable components.
    pub variable: Figure<Gap>,
}

/// A count of excluded workers: exact when it reaches the floor (or is zero), otherwise only "under the floor".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum Excluded {
    /// This many.
    Exactly(usize),
    /// At least one and fewer than the floor.
    UnderFloor,
}

/// Who was left out of the indicators, and why. A worker is counted once, under the first reason that applies, in
/// the order of the fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Exclusions {
    /// No sex declared (or monitoring off): not in any indicator.
    pub no_sex_declared: Excluded,
    /// A declaration other than female or male.
    pub sex_not_female_or_male: Excluded,
    /// No basic pay recorded.
    pub no_pay_recorded: Excluded,
    /// In the indicators on basic pay only, because the variable component was not recorded.
    pub variable_unknown: Excluded,
    /// Eligible, but in no category: left out of the per-category gaps.
    pub uncategorised: Excluded,
}

/// The seven indicators and what was left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The floor used.
    pub floor: usize,
    /// (a) The mean gender pay gap, on total pay (basic plus variable).
    pub mean_gap: Figure<Gap>,
    /// (b) The mean gap in complementary or variable components.
    pub mean_variable_gap: Figure<Gap>,
    /// (c) The median gender pay gap, on total pay.
    pub median_gap: Figure<Gap>,
    /// (d) The median gap in complementary or variable components.
    pub median_variable_gap: Figure<Gap>,
    /// (e) The share of female and of male workers receiving a variable component.
    pub receiving_variable: Figure<Shares>,
    /// (f) The lowest to the highest quartile pay band, in that order.
    pub quartiles: Vec<Figure<Composition>>,
    /// (g) The gap by category of workers, in name order.
    pub categories: Vec<CategoryGap>,
    /// Who was left out.
    pub exclusions: Exclusions,
}

/// The workers that can enter an indicator at all: female or male, with basic pay.
struct Eligible<'a> {
    worker: &'a WorkerPay,
    female: bool,
    basic: i64,
}

/// Compute the indicators.
///
/// # Errors
///
/// A floor below the equality monitoring minimum.
pub fn report(workers: &[WorkerPay], floor: usize, basis: VariableBasis) -> Result<Report, String> {
    equality_monitoring::validate_floor(floor)?;

    let mut tally = Tally::default();
    let mut eligible: Vec<Eligible> = Vec::new();
    for worker in workers {
        let female = match worker.sex {
            None => {
                tally.no_sex += 1;
                continue;
            }
            Some(Declared::Other) => {
                tally.not_binary += 1;
                continue;
            }
            Some(Declared::Female) => true,
            Some(Declared::Male) => false,
        };
        let Some(pay) = worker.basic_minor else {
            tally.no_pay += 1;
            continue;
        };
        if worker.variable == Variable::Unknown {
            tally.variable_unknown += 1;
        }
        if worker.category.is_none() {
            tally.uncategorised += 1;
        }
        eligible.push(Eligible {
            worker,
            female,
            basic: pay,
        });
    }

    // Total pay needs both parts; a worker whose variable part is unknown is in the basic-pay figures only.
    let total: Vec<(&Eligible, i64)> = eligible
        .iter()
        .filter_map(|e| match e.worker.variable {
            Variable::Known(v) => e.basic.checked_add(v).map(|t| (e, t)),
            Variable::Unknown => None,
        })
        .collect();
    let (total_f, total_m) = split(&total);
    let (var_f, var_m) = split(&variable_pairs(eligible.iter(), basis));

    // (e) is over everyone whose variable pay is known, however the gap is taken.
    let all_known = variable_pairs(eligible.iter(), VariableBasis::AllKnown);

    Ok(Report {
        floor,
        mean_gap: gap_figure(&total_f, &total_m, floor, Statistic::Mean),
        mean_variable_gap: gap_figure(&var_f, &var_m, floor, Statistic::Mean),
        median_gap: gap_figure(&total_f, &total_m, floor, Statistic::Median),
        median_variable_gap: gap_figure(&var_f, &var_m, floor, Statistic::Median),
        receiving_variable: receiving_figure(&all_known, floor),
        quartiles: quartile_figures(&total, floor),
        categories: category_gaps(&eligible, floor, basis),
        exclusions: tally.exclusions(floor),
    })
}

/// Female and male amounts apart.
fn split(pairs: &[(&Eligible, i64)]) -> (Vec<i64>, Vec<i64>) {
    let mut female = Vec::new();
    let mut male = Vec::new();
    for (e, amount) in pairs {
        if e.female {
            female.push(*amount);
        } else {
            male.push(*amount);
        }
    }
    (female, male)
}

/// Each worker whose variable pay is known, with it; under [`VariableBasis::RecipientsOnly`] only those who got some.
fn variable_pairs<'a>(
    workers: impl Iterator<Item = &'a Eligible<'a>>,
    basis: VariableBasis,
) -> Vec<(&'a Eligible<'a>, i64)> {
    workers
        .filter_map(|e| match e.worker.variable {
            Variable::Known(v) if basis == VariableBasis::AllKnown || v > 0 => Some((e, v)),
            _ => None,
        })
        .collect()
}

/// (g): the mean gap in basic pay and in variable pay for each category, in name order.
fn category_gaps(eligible: &[Eligible], floor: usize, basis: VariableBasis) -> Vec<CategoryGap> {
    let mut by_category: BTreeMap<&str, Vec<&Eligible>> = BTreeMap::new();
    for e in eligible {
        if let Some(category) = &e.worker.category {
            by_category.entry(category.as_str()).or_default().push(e);
        }
    }
    by_category
        .into_iter()
        .map(|(category, members)| {
            let basic_pairs: Vec<(&Eligible, i64)> =
                members.iter().map(|e| (*e, e.basic)).collect();
            let (bf, bm) = split(&basic_pairs);
            let (vf, vm) = split(&variable_pairs(members.iter().copied(), basis));
            CategoryGap {
                category: category.to_string(),
                basic: gap_figure(&bf, &bm, floor, Statistic::Mean),
                variable: gap_figure(&vf, &vm, floor, Statistic::Mean),
            }
        })
        .collect()
}

#[derive(Default)]
struct Tally {
    no_sex: usize,
    not_binary: usize,
    no_pay: usize,
    variable_unknown: usize,
    uncategorised: usize,
}

impl Tally {
    fn exclusions(&self, floor: usize) -> Exclusions {
        let show = |n: usize| {
            if n == 0 || n >= floor {
                Excluded::Exactly(n)
            } else {
                Excluded::UnderFloor
            }
        };
        Exclusions {
            no_sex_declared: show(self.no_sex),
            sex_not_female_or_male: show(self.not_binary),
            no_pay_recorded: show(self.no_pay),
            variable_unknown: show(self.variable_unknown),
            uncategorised: show(self.uncategorised),
        }
    }
}

#[derive(Clone, Copy)]
enum Statistic {
    Mean,
    Median,
}

/// A rational number `numerator / denominator` with a positive denominator, so a median of two middle values
/// and a mean stay exact until the last rounding.
#[derive(Clone, Copy)]
struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    fn of(values: &[i64], statistic: Statistic) -> Option<Self> {
        let n = i128::try_from(values.len()).ok()?;
        if n == 0 {
            return None;
        }
        match statistic {
            Statistic::Mean => Some(Self {
                numerator: values.iter().map(|v| i128::from(*v)).sum(),
                denominator: n,
            }),
            Statistic::Median => {
                let mut sorted = values.to_vec();
                sorted.sort_unstable();
                let middle = sorted.len() / 2;
                if sorted.len() % 2 == 1 {
                    Some(Self {
                        numerator: i128::from(sorted[middle]),
                        denominator: 1,
                    })
                } else {
                    Some(Self {
                        numerator: i128::from(sorted[middle - 1]) + i128::from(sorted[middle]),
                        denominator: 2,
                    })
                }
            }
        }
    }

    fn rounded(self) -> Option<i64> {
        i64::try_from(round_div(self.numerator, self.denominator)).ok()
    }
}

/// Integer division rounding half away from zero.
fn round_div(numerator: i128, denominator: i128) -> i128 {
    let negative = (numerator < 0) != (denominator < 0);
    let n = numerator.abs();
    let d = denominator.abs();
    let quotient = (2 * n + d) / (2 * d);
    if negative { -quotient } else { quotient }
}

fn gap_figure(female: &[i64], male: &[i64], floor: usize, statistic: Statistic) -> Figure<Gap> {
    if female.is_empty() && male.is_empty() {
        return Figure::Unknown(Reason::NoWorkers);
    }
    if female.is_empty() || male.is_empty() {
        return Figure::Unknown(Reason::OneSided);
    }
    if female.len() < floor || male.len() < floor {
        return Figure::Withheld;
    }
    let (Some(f), Some(m)) = (
        Rational::of(female, statistic),
        Rational::of(male, statistic),
    ) else {
        return Figure::Unknown(Reason::NoWorkers);
    };
    if m.numerator <= 0 {
        return Figure::Unknown(Reason::NoBase);
    }
    // (M - F) / M with M = mn/md and F = fn/fd, kept exact: (mn*fd - fn*md) / (mn*fd).
    let top = m.numerator * f.denominator - f.numerator * m.denominator;
    let bottom = m.numerator * f.denominator;
    let (Some(female_minor), Some(male_minor), Ok(gap_bp)) = (
        f.rounded(),
        m.rounded(),
        i64::try_from(round_div(top * 10_000, bottom)),
    ) else {
        return Figure::Unknown(Reason::NoBase);
    };
    Figure::Shown(Gap {
        female_minor,
        male_minor,
        gap_bp,
    })
}

fn receiving_figure(known: &[(&Eligible, i64)], floor: usize) -> Figure<Shares> {
    let count = |female: bool| known.iter().filter(|(e, _)| e.female == female).count();
    let received = |female: bool| {
        known
            .iter()
            .filter(|(e, v)| e.female == female && *v > 0)
            .count()
    };
    let (f, m) = (count(true), count(false));
    if f == 0 && m == 0 {
        return Figure::Unknown(Reason::NoWorkers);
    }
    if f == 0 || m == 0 {
        return Figure::Unknown(Reason::OneSided);
    }
    if f < floor || m < floor {
        return Figure::Withheld;
    }
    let share = |part: usize, whole: usize| {
        i64::try_from(round_div(
            i128::try_from(part).unwrap_or(0) * 10_000,
            i128::try_from(whole).unwrap_or(1),
        ))
        .unwrap_or(0)
    };
    Figure::Shown(Shares {
        female_bp: share(received(true), f),
        male_bp: share(received(false), m),
    })
}

/// Four equal groups by total pay, lowest first; a tie across a boundary is broken by the worker key so the split
/// is stable.
fn quartile_figures(total: &[(&Eligible, i64)], floor: usize) -> Vec<Figure<Composition>> {
    let mut ranked: Vec<(i64, &str, bool)> = total
        .iter()
        .map(|(e, pay)| (*pay, e.worker.key.as_str(), e.female))
        .collect();
    ranked.sort_unstable();
    let n = ranked.len();
    let mut female = [0_usize; 4];
    let mut male = [0_usize; 4];
    for (rank, (_, _, is_female)) in ranked.iter().enumerate() {
        let band = rank * 4 / n.max(1);
        if *is_female {
            female[band] += 1;
        } else {
            male[band] += 1;
        }
    }
    (0..4)
        .map(|band| {
            let (f, m) = (female[band], male[band]);
            if f + m == 0 {
                return Figure::Unknown(Reason::NoWorkers);
            }
            if f < floor || m < floor {
                return Figure::Withheld;
            }
            let female_bp = i64::try_from(round_div(
                i128::try_from(f).unwrap_or(0) * 10_000,
                i128::try_from(f + m).unwrap_or(1),
            ))
            .unwrap_or(0);
            Figure::Shown(Composition {
                female_bp,
                male_bp: 10_000 - female_bp,
            })
        })
        .collect()
}

/// How often an employer of a given size reports, and from when (Article 9 of the Directive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Obligation {
    /// The smallest headcount this rule covers.
    pub from_workers: usize,
    /// Years between reports (1 = every year).
    pub every_years: u32,
    /// The date of the first report.
    pub first_report_on: NaiveDate,
}

/// The Directive's own tiers, as defaults a deployer's national law may replace: 250 or more report every year from
/// 7 June 2027; 150 to 249 every three years from 7 June 2027; 100 to 149 every three years from 7 June 2031.
#[must_use]
pub fn directive_tiers() -> Vec<Obligation> {
    let date = |year| NaiveDate::from_ymd_opt(year, 6, 7).unwrap_or(NaiveDate::MIN);
    vec![
        Obligation {
            from_workers: 250,
            every_years: 1,
            first_report_on: date(2027),
        },
        Obligation {
            from_workers: 150,
            every_years: 3,
            first_report_on: date(2027),
        },
        Obligation {
            from_workers: 100,
            every_years: 3,
            first_report_on: date(2031),
        },
    ]
}

/// The tier that covers `headcount`: the one with the largest `from_workers` it reaches. `None` below every tier.
#[must_use]
pub fn tier_for(headcount: usize, tiers: &[Obligation]) -> Option<Obligation> {
    tiers
        .iter()
        .filter(|t| headcount >= t.from_workers)
        .max_by_key(|t| t.from_workers)
        .copied()
}

/// The first report date on or after `today`: the first report, then one every `every_years`.
#[must_use]
pub fn next_due(obligation: Obligation, today: NaiveDate) -> NaiveDate {
    let mut due = obligation.first_report_on;
    let mut step = 0_u32;
    while due < today {
        step += 1;
        let months = obligation
            .every_years
            .saturating_mul(12)
            .saturating_mul(step);
        match obligation
            .first_report_on
            .checked_add_months(Months::new(months))
        {
            Some(next) => due = next,
            None => break,
        }
    }
    due
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worker(
        n: usize,
        sex: Option<Declared>,
        basic: Option<i64>,
        variable: Variable,
    ) -> WorkerPay {
        WorkerPay {
            key: format!("w{n:04}"),
            sex,
            category: Some("engineers".into()),
            basic_minor: basic,
            variable,
        }
    }

    /// `n` workers of one sex at the same basic pay and variable pay.
    fn group(start: usize, n: usize, sex: Declared, basic: i64, variable: i64) -> Vec<WorkerPay> {
        (start..start + n)
            .map(|i| worker(i, Some(sex), Some(basic), Variable::Known(variable)))
            .collect()
    }

    fn shown<T: Clone + std::fmt::Debug>(figure: &Figure<T>) -> T {
        match figure {
            Figure::Shown(v) => v.clone(),
            other => panic!("expected a figure, got {other:?}"),
        }
    }

    #[test]
    fn a_floor_below_the_minimum_is_refused() {
        let err = report(
            &[],
            equality_monitoring::MIN_FLOOR - 1,
            VariableBasis::AllKnown,
        )
        .unwrap_err();
        assert!(err.contains("floor"), "{err}");
        assert!(report(&[], equality_monitoring::MIN_FLOOR, VariableBasis::AllKnown).is_ok());
    }

    #[test]
    fn the_mean_gap_is_exact_and_signed() {
        let mut w = group(0, 5, Declared::Female, 9_000, 0);
        w.extend(group(10, 5, Declared::Male, 10_000, 0));
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        let gap = shown(&r.mean_gap);
        assert_eq!(
            (gap.female_minor, gap.male_minor, gap.gap_bp),
            (9_000, 10_000, 1_000)
        );
        // Women paid more: negative.
        let mut w = group(0, 5, Declared::Female, 11_000, 0);
        w.extend(group(10, 5, Declared::Male, 10_000, 0));
        assert_eq!(
            shown(&report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap).gap_bp,
            -1_000
        );
        // Equal: zero, which is a real figure.
        let mut w = group(0, 5, Declared::Female, 10_000, 0);
        w.extend(group(10, 5, Declared::Male, 10_000, 0));
        assert_eq!(
            shown(&report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap).gap_bp,
            0
        );
    }

    #[test]
    fn total_pay_is_basic_plus_variable() {
        let mut w = group(0, 5, Declared::Female, 9_000, 1_000);
        w.extend(group(10, 5, Declared::Male, 9_000, 0));
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        // Female 10,000 against male 9,000: women ahead by 1/9 of the male figure, 1,111 bp rounded.
        assert_eq!(shown(&r.mean_gap).gap_bp, -1_111);
        // Basic only (the category figure) is equal.
        assert_eq!(shown(&r.categories[0].basic).gap_bp, 0);
    }

    #[test]
    fn the_median_of_an_even_count_is_the_midpoint_and_stays_exact() {
        // Female pay 1,2,3,4,5,6 -> median 3.5; male all 10 -> gap 6.5/10 = 6,500 bp.
        let f: Vec<WorkerPay> = (1..=6_i64)
            .enumerate()
            .map(|(i, p)| worker(i, Some(Declared::Female), Some(p), Variable::Known(0)))
            .collect();
        let mut w = f;
        w.extend(group(10, 6, Declared::Male, 10, 0));
        let g = shown(&report(&w, 5, VariableBasis::AllKnown).unwrap().median_gap);
        assert_eq!(g.gap_bp, 6_500);
        // 3.5 rounds half away from zero to 4.
        assert_eq!(g.female_minor, 4);
    }

    #[test]
    fn a_median_resists_an_outlier_that_moves_the_mean() {
        let mut f = group(0, 5, Declared::Female, 100, 0);
        f.extend(group(5, 1, Declared::Female, 1_000_000, 0));
        let mut w = f;
        w.extend(group(10, 6, Declared::Male, 100, 0));
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        assert_eq!(shown(&r.median_gap).gap_bp, 0);
        assert!(shown(&r.mean_gap).gap_bp < 0);
    }

    #[test]
    fn a_cell_under_the_floor_on_either_side_is_withheld_not_shown() {
        // A cell of one woman: never a figure, even though ten men are in it.
        let mut w = group(0, 1, Declared::Female, 9_000, 0);
        w.extend(group(10, 10, Declared::Male, 10_000, 0));
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        assert_eq!(r.mean_gap, Figure::Withheld);
        assert_eq!(r.median_gap, Figure::Withheld);
        assert_eq!(r.categories[0].basic, Figure::Withheld);
        // Four against ten, and ten against four.
        let mut w = group(0, 4, Declared::Female, 9_000, 0);
        w.extend(group(10, 10, Declared::Male, 10_000, 0));
        assert_eq!(
            report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap,
            Figure::Withheld
        );
        let mut w = group(0, 10, Declared::Female, 9_000, 0);
        w.extend(group(20, 4, Declared::Male, 10_000, 0));
        assert_eq!(
            report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap,
            Figure::Withheld
        );
        // Exactly at the floor shows.
        let mut w = group(0, 5, Declared::Female, 9_000, 0);
        w.extend(group(10, 5, Declared::Male, 10_000, 0));
        assert!(matches!(
            report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap,
            Figure::Shown(_)
        ));
    }

    #[test]
    fn a_one_sided_cell_is_unknown_and_an_empty_one_says_nobody() {
        let w = group(0, 10, Declared::Male, 10_000, 0);
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        assert_eq!(r.mean_gap, Figure::Unknown(Reason::OneSided));
        assert_eq!(r.receiving_variable, Figure::Unknown(Reason::OneSided));
        let r = report(&[], 5, VariableBasis::AllKnown).unwrap();
        assert_eq!(r.mean_gap, Figure::Unknown(Reason::NoWorkers));
        assert_eq!(r.quartiles, vec![Figure::Unknown(Reason::NoWorkers); 4]);
        assert_eq!(r.categories, vec![]);
    }

    #[test]
    fn a_male_figure_of_zero_has_no_base_for_a_share() {
        let mut w = group(0, 5, Declared::Female, 100, 0);
        w.extend(group(10, 5, Declared::Male, 0, 0));
        assert_eq!(
            report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap,
            Figure::Unknown(Reason::NoBase)
        );
    }

    #[test]
    fn exclusions_are_counted_by_first_reason_and_small_counts_are_not_exact() {
        let mut w = group(0, 5, Declared::Female, 9_000, 0);
        w.extend(group(10, 5, Declared::Male, 10_000, 0));
        // Two with no sex declared, one "other", one with no pay, three with variable unknown, one uncategorised.
        w.push(worker(100, None, Some(1), Variable::Known(0)));
        w.push(worker(101, None, None, Variable::Unknown));
        w.push(worker(
            102,
            Some(Declared::Other),
            Some(1),
            Variable::Known(0),
        ));
        w.push(worker(
            103,
            Some(Declared::Female),
            None,
            Variable::Known(0),
        ));
        for i in 104..107 {
            w.push(worker(
                i,
                Some(Declared::Male),
                Some(10_000),
                Variable::Unknown,
            ));
        }
        let mut loose = worker(110, Some(Declared::Female), Some(9_000), Variable::Known(0));
        loose.category = None;
        w.push(loose);
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        let e = &r.exclusions;
        assert_eq!(
            e.no_sex_declared,
            Excluded::UnderFloor,
            "2 is under the floor of 5"
        );
        assert_eq!(e.sex_not_female_or_male, Excluded::UnderFloor);
        assert_eq!(e.no_pay_recorded, Excluded::UnderFloor);
        assert_eq!(e.variable_unknown, Excluded::UnderFloor);
        assert_eq!(e.uncategorised, Excluded::UnderFloor);
        // Enough of them and the count is exact.
        let mut many = w.clone();
        many.extend((200..205).map(|i| worker(i, None, Some(1), Variable::Known(0))));
        assert_eq!(
            report(&many, 5, VariableBasis::AllKnown)
                .unwrap()
                .exclusions
                .no_sex_declared,
            Excluded::Exactly(7)
        );
        // None at all is an exact zero, not "under the floor".
        let clean = group(0, 10, Declared::Female, 1, 0);
        assert_eq!(
            report(&clean, 5, VariableBasis::AllKnown)
                .unwrap()
                .exclusions
                .sex_not_female_or_male,
            Excluded::Exactly(0)
        );
    }

    #[test]
    fn a_worker_whose_variable_pay_is_unknown_is_in_basic_pay_only() {
        let mut w = group(0, 5, Declared::Female, 9_000, 0);
        w.extend(group(10, 5, Declared::Male, 10_000, 0));
        // Five more men, variable unknown: they cannot move the total-pay gap, and they do count in basic pay.
        w.extend(
            (20..25).map(|i| worker(i, Some(Declared::Male), Some(20_000), Variable::Unknown)),
        );
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        assert_eq!(
            shown(&r.mean_gap).male_minor,
            10_000,
            "total pay ignores them"
        );
        assert_eq!(
            shown(&r.categories[0].basic).male_minor,
            15_000,
            "basic pay includes them"
        );
    }

    #[test]
    fn the_variable_gap_follows_the_chosen_basis() {
        // Women: five receive 100, five receive none. Men: ten each receive 100.
        let mut w = group(0, 5, Declared::Female, 1_000, 100);
        w.extend(group(5, 5, Declared::Female, 1_000, 0));
        w.extend(group(20, 10, Declared::Male, 1_000, 100));
        let all = report(&w, 5, VariableBasis::AllKnown).unwrap();
        assert_eq!(
            shown(&all.mean_variable_gap).gap_bp,
            5_000,
            "women averaged 50 against 100"
        );
        let recipients = report(&w, 5, VariableBasis::RecipientsOnly).unwrap();
        assert_eq!(
            shown(&recipients.mean_variable_gap).gap_bp,
            0,
            "those who received any got the same"
        );
        // The share receiving does not depend on the basis.
        assert_eq!(all.receiving_variable, recipients.receiving_variable);
        let shares = shown(&all.receiving_variable);
        assert_eq!((shares.female_bp, shares.male_bp), (5_000, 10_000));
    }

    #[test]
    fn nobody_known_to_have_received_anything_is_a_real_zero_but_unknown_is_not() {
        let mut w = group(0, 5, Declared::Female, 1_000, 0);
        w.extend(group(10, 5, Declared::Male, 1_000, 0));
        let shares = shown(
            &report(&w, 5, VariableBasis::AllKnown)
                .unwrap()
                .receiving_variable,
        );
        assert_eq!((shares.female_bp, shares.male_bp), (0, 0));
        let unknown: Vec<WorkerPay> = (0..10)
            .map(|i| {
                worker(
                    i,
                    Some(if i < 5 {
                        Declared::Female
                    } else {
                        Declared::Male
                    }),
                    Some(1_000),
                    Variable::Unknown,
                )
            })
            .collect();
        assert_eq!(
            report(&unknown, 5, VariableBasis::AllKnown)
                .unwrap()
                .receiving_variable,
            Figure::Unknown(Reason::NoWorkers)
        );
    }

    #[test]
    fn quartile_bands_are_four_equal_groups_lowest_first() {
        // 40 workers: pay 1..=40; the bottom ten are mostly women and the top ten mostly men.
        let w: Vec<WorkerPay> = (1..=40_i64)
            .enumerate()
            .map(|(i, p)| {
                let band = i / 10;
                let female = match band {
                    0 => i % 10 < 8,
                    1 | 2 => i % 2 == 0,
                    _ => i % 10 < 2,
                };
                worker(
                    i,
                    Some(if female {
                        Declared::Female
                    } else {
                        Declared::Male
                    }),
                    Some(p),
                    Variable::Known(0),
                )
            })
            .collect();
        let q = report(&w, 5, VariableBasis::AllKnown).unwrap().quartiles;
        // Band 0 has 8 women and 2 men: the men are under the floor of 5, so it is withheld, not shown.
        assert_eq!(q[0], Figure::Withheld);
        assert_eq!(q[3], Figure::Withheld);
        let mid = shown(&q[1]);
        assert_eq!((mid.female_bp, mid.male_bp), (5_000, 5_000));
        assert_eq!(shown(&q[2]).female_bp + shown(&q[2]).male_bp, 10_000);
    }

    #[test]
    fn a_tie_across_a_quartile_boundary_is_split_by_key_and_stable() {
        // Everyone is paid the same: the bands are still four equal groups, and the same on every run.
        let mut w = group(0, 10, Declared::Female, 100, 0);
        w.extend(group(10, 10, Declared::Male, 100, 0));
        let a = report(&w, 5, VariableBasis::AllKnown).unwrap().quartiles;
        let mut reversed = w.clone();
        reversed.reverse();
        let b = report(&reversed, 5, VariableBasis::AllKnown)
            .unwrap()
            .quartiles;
        assert_eq!(a, b, "input order does not matter");
        // 20 workers, five per band; keys w0000..w0009 are the women, so the two lowest bands are all women:
        // men are zero in them, which is under the floor.
        assert_eq!(a[0], Figure::Withheld);
        assert_eq!(a[3], Figure::Withheld);
    }

    #[test]
    fn fewer_than_four_workers_leave_empty_bands_unknown_rather_than_invented() {
        // Below the floor there would be no figure anyway; this pins that an empty band is not "0 and 0".
        let w = group(0, 2, Declared::Female, 100, 0);
        let q = report(&w, 5, VariableBasis::AllKnown).unwrap().quartiles;
        assert_eq!(q.len(), 4);
        assert!(q.contains(&Figure::Unknown(Reason::NoWorkers)));
        assert!(q.iter().all(|f| !matches!(f, Figure::Shown(_))));
    }

    #[test]
    fn categories_are_separate_sorted_and_each_has_its_own_floor() {
        let in_category = |name: &str, w: Vec<WorkerPay>| -> Vec<WorkerPay> {
            w.into_iter()
                .map(|mut x| {
                    x.category = Some(name.to_string());
                    x
                })
                .collect()
        };
        let mut w = in_category("nurses", group(0, 5, Declared::Female, 9_000, 0));
        w.extend(in_category(
            "nurses",
            group(10, 5, Declared::Male, 10_000, 0),
        ));
        // A category with four of each: under the floor, withheld, even though the organization overall is large.
        w.extend(in_category(
            "porters",
            group(20, 4, Declared::Female, 5_000, 0),
        ));
        w.extend(in_category(
            "porters",
            group(30, 4, Declared::Male, 5_000, 0),
        ));
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        let names: Vec<&str> = r.categories.iter().map(|c| c.category.as_str()).collect();
        assert_eq!(names, vec!["nurses", "porters"]);
        assert_eq!(shown(&r.categories[0].basic).gap_bp, 1_000);
        assert_eq!(r.categories[1].basic, Figure::Withheld);
        assert!(
            matches!(r.mean_gap, Figure::Shown(_)),
            "the whole workforce is large enough"
        );
    }

    #[test]
    fn amounts_near_the_ends_of_the_range_do_not_overflow() {
        let mut w = group(0, 5, Declared::Female, i64::MAX / 2, 0);
        w.extend(group(10, 5, Declared::Male, i64::MAX / 2, 0));
        assert_eq!(
            shown(&report(&w, 5, VariableBasis::AllKnown).unwrap().mean_gap).gap_bp,
            0
        );
        // Total pay that would overflow i64 is left out, not wrapped.
        let mut w = group(0, 5, Declared::Female, i64::MAX, 1);
        w.extend(group(10, 5, Declared::Male, 100, 0));
        let r = report(&w, 5, VariableBasis::AllKnown).unwrap();
        assert!(
            matches!(r.mean_gap, Figure::Unknown(_) | Figure::Withheld),
            "{:?}",
            r.mean_gap
        );
    }

    #[test]
    fn rounding_is_half_away_from_zero_in_both_directions() {
        assert_eq!(round_div(5, 2), 3);
        assert_eq!(round_div(-5, 2), -3);
        assert_eq!(round_div(4, 3), 1);
        assert_eq!(round_div(-4, 3), -1);
        assert_eq!(round_div(7, -2), -4);
        assert_eq!(round_div(0, 5), 0);
    }

    #[test]
    fn a_report_carries_no_head_count_and_no_name() {
        let mut w = group(0, 6, Declared::Female, 9_000, 100);
        w.extend(group(10, 7, Declared::Male, 10_000, 200));
        let json = serde_json::to_value(report(&w, 5, VariableBasis::AllKnown).unwrap()).unwrap();
        let text = json.to_string();
        assert!(!text.contains("w00"), "no worker key: {text}");
        for banned in [
            "headcount",
            "workers",
            "count",
            "n_female",
            "n_male",
            "female_workers",
        ] {
            assert!(!text.contains(banned), "{banned} in {text}");
        }
    }

    #[test]
    fn the_directive_tiers_pick_the_largest_that_is_reached() {
        let tiers = directive_tiers();
        let pick = |n| tier_for(n, &tiers).map(|t| (t.every_years, t.first_report_on.to_string()));
        assert_eq!(pick(99), None);
        assert_eq!(pick(100), Some((3, "2031-06-07".into())));
        assert_eq!(pick(149), Some((3, "2031-06-07".into())));
        assert_eq!(pick(150), Some((3, "2027-06-07".into())));
        assert_eq!(pick(249), Some((3, "2027-06-07".into())));
        assert_eq!(pick(250), Some((1, "2027-06-07".into())));
        assert_eq!(pick(10_000), Some((1, "2027-06-07".into())));
        assert_eq!(
            tier_for(500, &[]),
            None,
            "a deployer with no tiers has none"
        );
    }

    #[test]
    fn the_next_report_falls_on_the_first_date_then_every_period() {
        let day = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let yearly = Obligation {
            from_workers: 250,
            every_years: 1,
            first_report_on: day(2027, 6, 7),
        };
        assert_eq!(
            next_due(yearly, day(2026, 1, 1)),
            day(2027, 6, 7),
            "before the first"
        );
        assert_eq!(
            next_due(yearly, day(2027, 6, 7)),
            day(2027, 6, 7),
            "on the day"
        );
        assert_eq!(next_due(yearly, day(2027, 6, 8)), day(2028, 6, 7));
        assert_eq!(next_due(yearly, day(2030, 1, 1)), day(2030, 6, 7));
        let triennial = Obligation {
            from_workers: 150,
            every_years: 3,
            first_report_on: day(2027, 6, 7),
        };
        assert_eq!(next_due(triennial, day(2027, 6, 8)), day(2030, 6, 7));
        assert_eq!(next_due(triennial, day(2030, 6, 8)), day(2033, 6, 7));
        // A date that does not exist in the later year (29 February) lands on the last day of the month.
        let leap = Obligation {
            from_workers: 1,
            every_years: 1,
            first_report_on: day(2028, 2, 29),
        };
        assert_eq!(next_due(leap, day(2028, 3, 1)), day(2029, 2, 28));
    }
}
