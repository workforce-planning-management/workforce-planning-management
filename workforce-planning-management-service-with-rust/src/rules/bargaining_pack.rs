//! The information pack for bargaining (WPM-R106, WPM-D68). DB-free and clock-free.
//!
//! Aggregates about the people a bargaining unit's rules cover: headcount and FTE by group, pay bands, starters and leavers, contingent share.
//! Every count is a [`Cell`]: shown from the floor, withheld under it, **and when exactly one cell of a breakdown is withheld the smallest shown
//! one is withheld too**, because the total would otherwise give the hidden one away. No name, no salary, no one's band: only counts of groups.

use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate};
use serde::Serialize;

/// One count: shown from the floor, never under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cell {
    /// At or over the floor.
    Shown {
        /// The count.
        count: usize,
    },
    /// Withheld: under the floor, or hidden so that another cell cannot be worked out.
    Withheld,
}

/// The facts of one worker the pack counts. Nothing that names or pays them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Department.
    pub department: String,
    /// Employment type.
    pub employment_type: String,
    /// FTE percent (100 = full time).
    pub fte_percent: i32,
    /// The pay band they sit in, when one is recorded.
    pub band: Option<String>,
    /// First day.
    pub hired_on: NaiveDate,
    /// Last day, when they have left.
    pub terminated_on: Option<NaiveDate>,
}

impl Row {
    fn in_work(&self, on: NaiveDate) -> bool {
        self.hired_on <= on && self.terminated_on.is_none_or(|end| end > on)
    }
}

/// A group of the breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Group {
    /// The group's name (a department, an employment type, a band, or `not_recorded`).
    pub group: String,
    /// How many in it.
    pub headcount: Cell,
    /// Their FTE in hundredths of a person; shown only when the headcount is.
    pub fte_hundredths: Option<i64>,
}

/// Apply the floor to groups of `(name, headcount, fte hundredths)`: primary suppression under it, and secondary suppression of the smallest
/// shown group when exactly one was withheld (otherwise `total - shown` gives it away). Groups come back by name.
#[must_use]
pub fn suppress(groups: Vec<(String, usize, i64)>, floor: usize) -> Vec<Group> {
    let floor = floor.max(1);
    let mut hidden: Vec<bool> = groups.iter().map(|(_, n, _)| *n < floor).collect();
    if hidden.iter().filter(|h| **h).count() == 1 {
        let smallest = groups
            .iter()
            .enumerate()
            .filter(|(i, _)| !hidden[*i])
            .min_by_key(|(_, (name, n, _))| (*n, name.clone()))
            .map(|(i, _)| i);
        if let Some(i) = smallest {
            hidden[i] = true;
        }
    }
    groups
        .into_iter()
        .zip(hidden)
        .map(|((group, n, fte), hide)| Group {
            group,
            headcount: if hide {
                Cell::Withheld
            } else {
                Cell::Shown { count: n }
            },
            fte_hundredths: (!hide).then_some(fte),
        })
        .collect()
}

fn group_by(rows: &[&Row], key: impl Fn(&Row) -> String) -> Vec<(String, usize, i64)> {
    let mut map: BTreeMap<String, (usize, i64)> = BTreeMap::new();
    for row in rows {
        let entry = map.entry(key(row)).or_default();
        entry.0 += 1;
        entry.1 += i64::from(row.fte_percent);
    }
    map.into_iter().map(|(k, (n, fte))| (k, n, fte)).collect()
}

/// The pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pack {
    /// The day it is as of.
    pub as_of: NaiveDate,
    /// Workers in work on that day whom the unit's rules cover; `None` under the floor, and then nothing else is given.
    pub headcount: Option<usize>,
    /// Headcount and FTE by department.
    pub by_department: Vec<Group>,
    /// Headcount and FTE by employment type.
    pub by_employment_type: Vec<Group>,
    /// Headcount by pay band; the people with no band recorded are their own group, never counted into one.
    pub by_pay_band: Vec<Group>,
    /// Starters in the 365 days to `as_of`.
    pub starters_last_year: Cell,
    /// Leavers in the 365 days to `as_of`.
    pub leavers_last_year: Cell,
    /// People on a basis other than permanent, and the rest, apart.
    pub contingent: Vec<Group>,
}

fn one_cell(n: usize, floor: usize) -> Cell {
    if n < floor.max(1) {
        Cell::Withheld
    } else {
        Cell::Shown { count: n }
    }
}

/// Build the pack from `rows` (every non-deleted worker the unit's rules cover, in work or not) as of `as_of`, with `floor` the fewest that may be counted.
#[must_use]
pub fn pack(rows: &[Row], as_of: NaiveDate, floor: usize) -> Pack {
    let working: Vec<&Row> = rows.iter().filter(|r| r.in_work(as_of)).collect();
    let year_ago = as_of - Duration::days(365);
    let starters = rows
        .iter()
        .filter(|r| r.hired_on > year_ago && r.hired_on <= as_of)
        .count();
    let leavers = rows
        .iter()
        .filter(|r| r.terminated_on.is_some_and(|e| e > year_ago && e <= as_of))
        .count();
    if working.len() < floor.max(1) {
        // Too few to say anything about: the unit is a handful of people, and every figure would point at them.
        return Pack {
            as_of,
            headcount: None,
            by_department: vec![],
            by_employment_type: vec![],
            by_pay_band: vec![],
            starters_last_year: Cell::Withheld,
            leavers_last_year: Cell::Withheld,
            contingent: vec![],
        };
    }
    Pack {
        as_of,
        headcount: Some(working.len()),
        by_department: suppress(group_by(&working, |r| r.department.clone()), floor),
        by_employment_type: suppress(group_by(&working, |r| r.employment_type.clone()), floor),
        by_pay_band: suppress(
            group_by(&working, |r| {
                r.band.clone().unwrap_or_else(|| "not_recorded".to_string())
            }),
            floor,
        ),
        starters_last_year: one_cell(starters, floor),
        leavers_last_year: one_cell(leavers, floor),
        contingent: suppress(
            group_by(&working, |r| {
                if r.employment_type == "permanent" {
                    "permanent"
                } else {
                    "other_basis"
                }
                .to_string()
            }),
            floor,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn row(dept: &str, kind: &str, band: Option<&str>, hired: &str, ended: Option<&str>) -> Row {
        Row {
            department: dept.into(),
            employment_type: kind.into(),
            fte_percent: 100,
            band: band.map(Into::into),
            hired_on: d(hired),
            terminated_on: ended.map(d),
        }
    }

    fn rows(n: usize, dept: &str, kind: &str, band: Option<&str>) -> Vec<Row> {
        (0..n)
            .map(|_| row(dept, kind, band, "2020-01-01", None))
            .collect()
    }

    #[test]
    fn a_group_under_the_floor_is_withheld_and_so_is_the_next_smallest_so_it_cannot_be_subtracted()
    {
        let groups = suppress(
            vec![
                ("a".into(), 30, 3000),
                ("b".into(), 4, 400),
                ("c".into(), 12, 1200),
            ],
            10,
        );
        assert_eq!(groups[1].headcount, Cell::Withheld);
        assert_eq!(
            groups[2].headcount,
            Cell::Withheld,
            "12 is the smallest shown: hidden so 4 cannot be worked out from the total"
        );
        assert_eq!(groups[0].headcount, Cell::Shown { count: 30 });
        assert_eq!(groups[1].fte_hundredths, None);
        assert_eq!(groups[0].fte_hundredths, Some(3000));
    }

    #[test]
    fn two_withheld_groups_need_no_second_hiding_and_none_withheld_needs_none() {
        let two = suppress(
            vec![
                ("a".into(), 30, 0),
                ("b".into(), 4, 0),
                ("c".into(), 3, 0),
                ("d".into(), 12, 0),
            ],
            10,
        );
        assert_eq!(
            two.iter().filter(|g| g.headcount == Cell::Withheld).count(),
            2,
            "two hidden cannot be told apart"
        );
        let none = suppress(vec![("a".into(), 30, 0), ("b".into(), 12, 0)], 10);
        assert!(
            none.iter()
                .all(|g| matches!(g.headcount, Cell::Shown { .. }))
        );
    }

    #[test]
    fn a_single_withheld_group_with_nothing_else_to_hide_stays_alone() {
        let alone = suppress(vec![("a".into(), 4, 0)], 10);
        assert_eq!(alone[0].headcount, Cell::Withheld);
    }

    #[test]
    fn a_unit_under_the_floor_gives_nothing_at_all() {
        let p = pack(
            &rows(9, "care", "permanent", Some("B3")),
            d("2026-10-26"),
            10,
        );
        assert_eq!(p.headcount, None);
        assert!(p.by_department.is_empty() && p.by_pay_band.is_empty());
        assert_eq!(
            (p.starters_last_year, p.leavers_last_year),
            (Cell::Withheld, Cell::Withheld)
        );
    }

    #[test]
    fn the_pack_counts_groups_bands_and_basis_and_gives_no_band_its_own_group() {
        let mut all = rows(12, "care", "permanent", Some("B3"));
        all.extend(rows(11, "care", "permanent", None));
        all.extend(rows(10, "admin", "contractor", Some("B5")));
        let p = pack(&all, d("2026-10-26"), 10);
        assert_eq!(p.headcount, Some(33));
        let band = |g: &str| {
            p.by_pay_band
                .iter()
                .find(|x| x.group == g)
                .unwrap()
                .headcount
        };
        assert_eq!(band("B3"), Cell::Shown { count: 12 });
        assert_eq!(
            band("not_recorded"),
            Cell::Shown { count: 11 },
            "an unknown band is its own group, never zero or a guess"
        );
        assert_eq!(
            p.by_department
                .iter()
                .find(|g| g.group == "care")
                .unwrap()
                .fte_hundredths,
            Some(2300)
        );
        let other = p
            .contingent
            .iter()
            .find(|g| g.group == "other_basis")
            .unwrap();
        assert_eq!(other.headcount, Cell::Shown { count: 10 });
    }

    #[test]
    fn starters_and_leavers_are_the_year_to_the_day_and_counted_from_the_floor() {
        let mut all = rows(20, "care", "permanent", None);
        for _ in 0..10 {
            all.push(row("care", "permanent", None, "2026-01-05", None));
        }
        for _ in 0..9 {
            all.push(row(
                "care",
                "permanent",
                None,
                "2020-01-01",
                Some("2026-03-01"),
            ));
        }
        all.push(row("care", "permanent", None, "2025-10-26", None)); // hired exactly a year before: not a starter
        let p = pack(&all, d("2026-10-26"), 10);
        assert_eq!(p.starters_last_year, Cell::Shown { count: 10 });
        assert_eq!(
            p.leavers_last_year,
            Cell::Withheld,
            "nine is under the floor"
        );
        assert_eq!(p.headcount, Some(31), "leavers are not in work today");
    }

    #[test]
    fn the_last_day_is_in_work_until_the_day_after_it() {
        let mut all = rows(10, "care", "permanent", None);
        all.push(row(
            "care",
            "permanent",
            None,
            "2020-01-01",
            Some("2026-10-26"),
        ));
        assert_eq!(
            pack(&all, d("2026-10-26"), 10).headcount,
            Some(10),
            "terminated_on is the first day out of work"
        );
        assert_eq!(pack(&all, d("2026-10-25"), 10).headcount, Some(11));
    }

    #[test]
    fn nothing_in_a_pack_carries_a_name_a_salary_or_a_person() {
        let p = pack(
            &rows(12, "care", "permanent", Some("B3")),
            d("2026-10-26"),
            10,
        );
        let text = serde_json::to_string(&p).unwrap();
        for banned in ["name", "salary", "pid", "worker"] {
            assert!(!text.contains(banned), "{banned}: {text}");
        }
    }
}
