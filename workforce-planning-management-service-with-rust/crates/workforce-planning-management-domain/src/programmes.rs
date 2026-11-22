//! **Programme roll-up** (WPM-R83, WPM-D59): pure, database-free and clock-free. Per programme and
//! month: the FTE the programme funds, how much of it is fixed-term or contractor, and the
//! engagements and the funding that end in the month. Programmes are linked through posts, not
//! through time recording. Aggregate only: a programme with fewer people than the floor is not shown,
//! a figure for one to nine people is withheld, and **no total across programmes** is given, so a
//! withheld one cannot be worked out.

use jiff::civil::Date;

/// One person on a programme. No name, no pay.
#[derive(Debug, Clone)]
pub struct OnProgramme {
    /// The programme (`thing:` URN).
    pub programme: String,
    /// FTE in percent.
    pub fte_percent: i32,
    /// The engagement basis (`employment_type`).
    pub basis: String,
    /// When the funding ends; `None` is open-ended.
    pub funding_ends: Option<Date>,
    /// When the engagement ends; `None` is open-ended or unknown.
    pub engagement_ends: Option<Date>,
}

/// One month of one programme. A figure is `None` when withheld.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgrammeMonth {
    /// `YYYY-MM`.
    pub month: String,
    /// FTE funded that month; `None` when one to nine people are funded.
    pub funded_fte: Option<f64>,
    /// Contractor share of that FTE; `None` when withheld or nothing is funded.
    pub contractor_percent: Option<f64>,
    /// Fixed-term share of that FTE; `None` when withheld or nothing is funded.
    pub fixed_term_percent: Option<f64>,
    /// Engagements ending in the month; `None` when one to nine.
    pub engagements_ending: Option<usize>,
    /// Funding ending in the month; `None` when one to nine.
    pub funding_ending: Option<usize>,
}

/// One programme over the coming months.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgrammeView {
    /// The programme.
    pub programme: String,
    /// The months, oldest first, starting with the month of `today`.
    pub months: Vec<ProgrammeMonth>,
}

/// The result: the programmes shown, and how many were left out for being small.
#[derive(Debug, Clone, PartialEq)]
pub struct Rollup {
    /// Programmes with at least the floor of people, in URN order.
    pub programmes: Vec<ProgrammeView>,
    /// Programmes left out because fewer people than the floor are on them.
    pub withheld_programmes: usize,
}

/// Months since year 0 of the month holding `date`.
fn month_index(date: Date) -> Option<i64> {
    i64::from(date.year())
        .checked_mul(12)?
        .checked_add(i64::from(date.month()))?
        .checked_sub(1)
}

/// The first day of month number `index` (see [`month_index`]), or `None` when there is no such date.
fn first_of_month(index: i64) -> Option<Date> {
    let year = i32::try_from(index.div_euclid(12)).ok()?;
    let month = u32::try_from(index.rem_euclid(12).checked_add(1)?).ok()?;
    crate::dates::ymd(year, month, 1)
}

/// Sum of figures, `None` when the total does not fit an `i64`.
fn checked_sum(mut values: impl Iterator<Item = i64>) -> Option<i64> {
    values.try_fold(0_i64, i64::checked_add)
}

fn percent(part: i64, whole: i64) -> Option<f64> {
    if whole <= 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    Some(((part as f64) * 1000.0 / (whole as f64)).round() / 10.0)
}

fn shown(n: usize, floor: usize) -> Option<usize> {
    (n == 0 || n >= floor).then_some(n)
}

/// Roll the people on programmes up by programme and month for `months` months from the month of
/// `today`. Funding counts in a month when it has no end or ends on or after the first of that month. `None`
/// when a month falls outside the dates there are or a total does not fit: unknown, not a figure made up.
#[must_use]
pub fn rollup(rows: &[OnProgramme], today: Date, months: u32, floor: usize) -> Option<Rollup> {
    use std::collections::BTreeMap;
    let start = month_index(today)?;
    let mut by: BTreeMap<&str, Vec<&OnProgramme>> = BTreeMap::new();
    for r in rows {
        by.entry(&r.programme).or_default().push(r);
    }
    let withheld = by.values().filter(|people| people.len() < floor).count();
    let mut programmes = Vec::new();
    for (programme, people) in by {
        if people.len() < floor {
            continue;
        }
        let months = (0..i64::from(months))
            .map(|offset| {
                let at = start.checked_add(offset)?;
                let from = first_of_month(at)?;
                let in_month = |d: Option<Date>| d.is_some_and(|d| month_index(d) == Some(at));
                let funded: Vec<&&OnProgramme> = people
                    .iter()
                    .filter(|p| p.funding_ends.is_none_or(|e| e >= from))
                    .collect();
                let of = |basis: &str| {
                    checked_sum(
                        funded
                            .iter()
                            .filter(|p| p.basis == basis)
                            .map(|p| i64::from(p.fte_percent)),
                    )
                };
                let fte = checked_sum(funded.iter().map(|p| i64::from(p.fte_percent)))?;
                let (contractor, fixed_term) = (of("contractor")?, of("fixed_term")?);
                let funded_shown = shown(funded.len(), floor).is_some();
                #[allow(clippy::cast_precision_loss)]
                Some(ProgrammeMonth {
                    month: format!("{:04}-{:02}", from.year(), from.month()),
                    funded_fte: funded_shown.then_some(fte as f64 / 100.0),
                    contractor_percent: if funded_shown {
                        percent(contractor, fte)
                    } else {
                        None
                    },
                    fixed_term_percent: if funded_shown {
                        percent(fixed_term, fte)
                    } else {
                        None
                    },
                    engagements_ending: shown(
                        people
                            .iter()
                            .filter(|p| in_month(p.engagement_ends))
                            .count(),
                        floor,
                    ),
                    funding_ending: shown(
                        people.iter().filter(|p| in_month(p.funding_ends)).count(),
                        floor,
                    ),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        programmes.push(ProgrammeView {
            programme: programme.to_string(),
            months,
        });
    }
    Some(Rollup {
        programmes,
        withheld_programmes: withheld,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> Date {
        crate::dates::ymd(y, m, day).unwrap()
    }

    fn rollup(rows: &[OnProgramme], today: Date, months: u32, floor: usize) -> Rollup {
        super::rollup(rows, today, months, floor).unwrap()
    }

    #[test]
    fn a_month_outside_the_dates_there_are_is_unknown_and_so_is_a_total_that_does_not_fit() {
        let rows = [person("P1", 100, "permanent", None, None)];
        assert_eq!(super::rollup(&rows, Date::MAX, 12, 1), None);
        assert!(super::rollup(&rows, Date::MIN, 12, 1).is_some());
        let big = [
            person("P1", i32::MAX, "contractor", None, None),
            person("P1", i32::MAX, "contractor", None, None),
        ];
        assert!(super::rollup(&big, d(2026, 6, 1), 1, 1).is_some());
        assert_eq!(checked_sum([i64::MAX, 1].into_iter()), None);
        assert_eq!(first_of_month(i64::MAX), None);
        assert_eq!(first_of_month(i64::MIN), None);
    }

    fn person(
        programme: &str,
        fte: i32,
        basis: &str,
        funding: Option<Date>,
        engagement: Option<Date>,
    ) -> OnProgramme {
        OnProgramme {
            programme: programme.into(),
            fte_percent: fte,
            basis: basis.into(),
            funding_ends: funding,
            engagement_ends: engagement,
        }
    }

    fn crowd(n: usize, funding: Option<Date>) -> Vec<OnProgramme> {
        (0..n)
            .map(|i| {
                let basis = if i < 2 {
                    "contractor"
                } else if i < 5 {
                    "fixed_term"
                } else {
                    "permanent"
                };
                person("thing:a", 100, basis, funding, None)
            })
            .collect()
    }

    #[test]
    fn a_programme_below_the_floor_is_not_shown_and_is_counted_apart() {
        let mut rows = crowd(9, None);
        rows.extend(crowd(10, None).into_iter().map(|mut p| {
            p.programme = "thing:b".into();
            p
        }));
        let r = rollup(&rows, d(2026, 6, 15), 3, 10);
        assert_eq!(r.programmes.len(), 1);
        assert_eq!(r.programmes[0].programme, "thing:b");
        assert_eq!(r.withheld_programmes, 1);
    }

    #[test]
    fn funded_fte_and_the_shares_follow_the_funding_end() {
        // Ten people; the funding of all of them ends on 20 July.
        let r = rollup(&crowd(10, Some(d(2026, 7, 20))), d(2026, 6, 15), 3, 10);
        let m = &r.programmes[0].months;
        assert_eq!(m[0].month, "2026-06");
        assert_eq!(m[0].funded_fte, Some(10.0));
        assert_eq!(m[0].contractor_percent, Some(20.0));
        assert_eq!(m[0].fixed_term_percent, Some(30.0));
        assert_eq!(
            m[1].funded_fte,
            Some(10.0),
            "ends in July, so still funded in July"
        );
        assert_eq!(m[2].funded_fte, Some(0.0), "nothing is funded in August");
        assert_eq!(m[2].contractor_percent, None, "no share of nothing");
        assert_eq!(m[1].funding_ending, Some(10));
        assert_eq!(m[0].funding_ending, Some(0), "zero is shown");
    }

    #[test]
    fn a_month_of_one_to_nine_is_withheld() {
        let mut rows = crowd(10, None);
        for p in rows.iter_mut().take(4) {
            p.funding_ends = Some(d(2026, 6, 30));
            p.engagement_ends = Some(d(2026, 7, 31));
        }
        let m = &rollup(&rows, d(2026, 6, 15), 3, 10).programmes[0].months;
        assert_eq!(m[0].funding_ending, None, "four ending is withheld");
        assert_eq!(m[1].engagements_ending, None);
        assert_eq!(
            m[1].funded_fte, None,
            "six funded is withheld, with its shares"
        );
        assert_eq!(m[1].contractor_percent, None);
        assert_eq!(m[0].funded_fte, Some(10.0));
    }

    #[test]
    fn months_cross_a_year_and_the_month_of_today_comes_first() {
        let r = rollup(&crowd(10, None), d(2026, 11, 30), 4, 10);
        let names: Vec<_> = r.programmes[0]
            .months
            .iter()
            .map(|m| m.month.as_str())
            .collect();
        assert_eq!(names, ["2026-11", "2026-12", "2027-01", "2027-02"]);
    }

    #[test]
    fn no_rows_means_no_programmes() {
        assert_eq!(
            rollup(&[], d(2026, 6, 1), 12, 10),
            Rollup {
                programmes: vec![],
                withheld_programmes: 0
            }
        );
    }

    #[test]
    fn funding_that_ends_on_the_first_of_a_month_still_funds_that_day() {
        let r = rollup(&crowd(10, Some(d(2026, 7, 1))), d(2026, 6, 15), 3, 10);
        let m = &r.programmes[0].months;
        assert_eq!(
            m[1].funded_fte,
            Some(10.0),
            "the last day of funding is 1 July"
        );
        assert_eq!(m[2].funded_fte, Some(0.0));
        assert_eq!(m[1].funding_ending, Some(10));
    }
}
