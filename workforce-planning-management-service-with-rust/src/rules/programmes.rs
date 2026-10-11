//! **Programme roll-up** (WPM-R83, WPM-D59): pure, database-free and clock-free. Per programme and
//! month: the FTE the programme funds, how much of it is fixed-term or contractor, and the
//! engagements and the funding that end in the month. Programmes are linked through posts, not
//! through time recording. Aggregate only: a programme with fewer people than the floor is not shown,
//! a figure for one to nine people is withheld, and **no total across programmes** is given, so a
//! withheld one cannot be worked out.

use chrono::{Datelike, NaiveDate};

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
    pub funding_ends: Option<NaiveDate>,
    /// When the engagement ends; `None` is open-ended or unknown.
    pub engagement_ends: Option<NaiveDate>,
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

fn first_of_month(index: i64) -> NaiveDate {
    let year = i32::try_from(index.div_euclid(12)).unwrap_or(1970);
    let month = u32::try_from(index.rem_euclid(12)).unwrap_or(0) + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(NaiveDate::MIN)
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
/// `today`. Funding counts in a month when it has no end or ends on or after the first of that month.
#[must_use]
pub fn rollup(rows: &[OnProgramme], today: NaiveDate, months: u32, floor: usize) -> Rollup {
    use std::collections::BTreeMap;
    let index = |d: NaiveDate| i64::from(d.year()) * 12 + i64::from(d.month0());
    let start = index(today);
    let mut by: BTreeMap<&str, Vec<&OnProgramme>> = BTreeMap::new();
    for r in rows {
        by.entry(&r.programme).or_default().push(r);
    }
    let mut programmes = Vec::new();
    let mut withheld = 0;
    for (programme, people) in by {
        if people.len() < floor {
            withheld += 1;
            continue;
        }
        let months = (0..i64::from(months))
            .map(|offset| {
                let at = start + offset;
                let from = first_of_month(at);
                let in_month = |d: Option<NaiveDate>| d.is_some_and(|d| index(d) == at);
                let funded: Vec<&&OnProgramme> = people
                    .iter()
                    .filter(|p| p.funding_ends.is_none_or(|e| e >= from))
                    .collect();
                let fte: i64 = funded.iter().map(|p| i64::from(p.fte_percent)).sum();
                let of = |basis: &str| -> i64 {
                    funded
                        .iter()
                        .filter(|p| p.basis == basis)
                        .map(|p| i64::from(p.fte_percent))
                        .sum()
                };
                let funded_shown = shown(funded.len(), floor).is_some();
                #[allow(clippy::cast_precision_loss)]
                ProgrammeMonth {
                    month: format!("{:04}-{:02}", at.div_euclid(12), at.rem_euclid(12) + 1),
                    funded_fte: funded_shown.then_some(fte as f64 / 100.0),
                    contractor_percent: if funded_shown {
                        percent(of("contractor"), fte)
                    } else {
                        None
                    },
                    fixed_term_percent: if funded_shown {
                        percent(of("fixed_term"), fte)
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
                }
            })
            .collect();
        programmes.push(ProgrammeView {
            programme: programme.to_string(),
            months,
        });
    }
    Rollup {
        programmes,
        withheld_programmes: withheld,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn person(
        programme: &str,
        fte: i32,
        basis: &str,
        funding: Option<NaiveDate>,
        engagement: Option<NaiveDate>,
    ) -> OnProgramme {
        OnProgramme {
            programme: programme.into(),
            fte_percent: fte,
            basis: basis.into(),
            funding_ends: funding,
            engagement_ends: engagement,
        }
    }

    fn crowd(n: usize, funding: Option<NaiveDate>) -> Vec<OnProgramme> {
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
