//! Pay position analysis (WPM-R154): where pay sits inside its band, as an aggregate. DB-free and clock-free.
//!
//! - **Compa-ratio** is pay divided by the band's midpoint (the middle of its minimum and maximum; an intermediate step is
//!   not used), in basis points (10 000 is the midpoint). **Range penetration** is how far through the band, from its
//!   minimum (0) to its maximum (10 000), the pay sits; it is not clamped, so pay outside the band shows as under 0 or over
//!   10 000. A band with one rate has no width, so its penetration is unknown, never zero.
//! - A group is a band, a level, a department or a location. A group under the floor is withheld with no count, and a
//!   total is not shown if it would let a withheld group be worked out from it (WPM-D32).
//! - An **outlier** is a shown group whose mean compa-ratio is more than the cut-off from the mean of everyone analysed.
//!   It is a group, never a worker, and it proposes no pay change.
//!
//! Money is minor units and every ratio is integer arithmetic.

use std::collections::BTreeMap;

use serde::Serialize;

/// A band's minimum and maximum, in minor units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The entry step.
    pub min: i64,
    /// The top step.
    pub max: i64,
}

/// One worker, reduced to what the analysis needs. No name, no pid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Full-time-equivalent annual pay in the analysis currency; `None` when none is recorded (or in another currency).
    pub salary_minor: Option<i64>,
    /// The band's code, as recorded.
    pub band: String,
    /// The band's range; `None` when the band or its scale is not known.
    pub range: Option<Range>,
    /// The job level, as a label; `None` when none is recorded.
    pub level: Option<String>,
    /// The department.
    pub department: String,
    /// The location; `None` when none is recorded.
    pub location: Option<String>,
}

/// Why there is no figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// Nobody in the group has pay and a known band.
    NoPay,
    /// Showing it would let a withheld group be worked out.
    WouldReveal,
    /// The band has one rate, so there is no range to be through.
    NoWidth,
}

/// A figure, withheld, or unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Figure<T> {
    /// Shown.
    Shown {
        /// The figure.
        value: T,
    },
    /// Fewer workers than the floor: no figure and no count.
    Withheld,
    /// Cannot be worked out.
    Unknown {
        /// Why.
        reason: Reason,
    },
}

/// Mean range penetration over those whose band has a width.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Penetration {
    /// Workers behind it.
    pub workers: usize,
    /// Mean, in basis points of the band's width.
    pub mean_bp: i64,
}

/// A group's figures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Stat {
    /// Workers behind the compa-ratio.
    pub workers: usize,
    /// Mean compa-ratio in basis points.
    pub compa_mean_bp: i64,
    /// Mean range penetration.
    pub penetration: Figure<Penetration>,
}

/// Whether a group sits far from everyone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outlier {
    /// Within the cut-off.
    No,
    /// Higher than everyone by more than the cut-off.
    Above {
        /// How far, in basis points of compa-ratio.
        delta_bp: i64,
    },
    /// Lower than everyone by more than the cut-off.
    Below {
        /// How far, in basis points (positive).
        delta_bp: i64,
    },
    /// Not worked out: the group or the total is not shown.
    Unknown,
}

/// One group in a dimension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Group {
    /// The band code, level, department or location; `unrecorded` where none was recorded.
    pub key: String,
    /// The group's figures.
    pub figure: Figure<Stat>,
    /// Far from everyone?
    pub outlier: Outlier,
}

/// A way of grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    /// By band.
    Band,
    /// By job level.
    Level,
    /// By department.
    Department,
    /// By location.
    Location,
}

/// A dimension and its groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grouping {
    /// What it groups by.
    pub by: Dimension,
    /// Groups, by key.
    pub groups: Vec<Group>,
}

/// Workers left out, each shown only when it reaches the floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Left {
    /// Exactly this many.
    Exactly {
        /// The count.
        workers: usize,
    },
    /// At least one and fewer than the floor.
    UnderFloor,
}

/// What was left out and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Exclusions {
    /// No pay recorded (or pay in another currency).
    pub no_pay: Left,
    /// Pay recorded, but no band on a known scale.
    pub no_band: Left,
}

/// The analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The floor.
    pub floor: usize,
    /// The outlier cut-off, in basis points of compa-ratio.
    pub cutoff_bp: i64,
    /// Everyone analysed.
    pub overall: Figure<Stat>,
    /// The four groupings.
    pub groupings: Vec<Grouping>,
    /// Who was left out.
    pub excluded: Exclusions,
}

/// The least and most the cut-off may be, in basis points.
pub const CUTOFF_RANGE: (i64, i64) = (100, 5000);
/// The cut-off when none is asked for: ten points either side of the midpoint.
pub const DEFAULT_CUTOFF_BP: i64 = 1000;

/// Check a cut-off.
///
/// # Errors
/// When it is outside [`CUTOFF_RANGE`].
pub fn check_cutoff(bp: i64) -> Result<i64, String> {
    if (CUTOFF_RANGE.0..=CUTOFF_RANGE.1).contains(&bp) {
        Ok(bp)
    } else {
        Err(format!(
            "the cut-off must be between {} and {} basis points",
            CUTOFF_RANGE.0, CUTOFF_RANGE.1
        ))
    }
}

/// Divide, rounding half up, for any sign. `den` is positive.
fn div_round(num: i128, den: i128) -> i128 {
    (2 * num + den).div_euclid(2 * den)
}

fn clamp_i64(v: i128) -> i64 {
    i64::try_from(v).unwrap_or(if v < 0 { i64::MIN } else { i64::MAX })
}

/// Compa-ratio in basis points: pay over the midpoint. `None` when the midpoint is not above zero.
#[must_use]
pub fn compa_bp(salary: i64, r: Range) -> Option<i64> {
    let twice_mid = i128::from(r.min) + i128::from(r.max);
    (twice_mid > 0).then(|| clamp_i64(div_round(i128::from(salary) * 20_000, twice_mid)))
}

/// Range penetration in basis points; `None` when the band has one rate (or none).
#[must_use]
pub fn penetration_bp(salary: i64, r: Range) -> Option<i64> {
    let width = i128::from(r.max) - i128::from(r.min);
    (width > 0).then(|| {
        clamp_i64(div_round(
            (i128::from(salary) - i128::from(r.min)) * 10_000,
            width,
        ))
    })
}

#[derive(Clone, Copy)]
struct Point {
    compa: i64,
    pen: Option<i64>,
}

fn mean(values: impl Iterator<Item = i64>) -> Option<i64> {
    let (sum, n) = values.fold((0_i128, 0_i128), |(s, n), v| (s + i128::from(v), n + 1));
    (n > 0).then(|| clamp_i64(div_round(sum, n)))
}

fn stat(points: &[Point], floor: usize) -> Figure<Stat> {
    if points.is_empty() {
        return Figure::Unknown {
            reason: Reason::NoPay,
        };
    }
    if points.len() < floor {
        return Figure::Withheld;
    }
    let widths: Vec<i64> = points.iter().filter_map(|p| p.pen).collect();
    let penetration = if widths.is_empty() {
        Figure::Unknown {
            reason: Reason::NoWidth,
        }
    } else if widths.len() < floor {
        Figure::Withheld
    } else {
        Figure::Shown {
            value: Penetration {
                workers: widths.len(),
                mean_bp: mean(widths.iter().copied()).unwrap_or(0),
            },
        }
    };
    Figure::Shown {
        value: Stat {
            workers: points.len(),
            compa_mean_bp: mean(points.iter().map(|p| p.compa)).unwrap_or(0),
            penetration,
        },
    }
}

fn left(count: usize, floor: usize) -> Left {
    if count >= floor {
        Left::Exactly { workers: count }
    } else {
        Left::UnderFloor
    }
}

fn key_of(by: Dimension, e: &Entry) -> String {
    let named = match by {
        Dimension::Band => Some(e.band.trim().to_lowercase()),
        Dimension::Level => e.level.clone(),
        Dimension::Department => Some(e.department.trim().to_string()),
        Dimension::Location => e.location.clone(),
    };
    named
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| "unrecorded".to_string())
}

fn grouping(by: Dimension, rows: &[(&Entry, Point)]) -> Vec<(String, Vec<Point>)> {
    let mut map: BTreeMap<String, Vec<Point>> = BTreeMap::new();
    for (e, p) in rows {
        map.entry(key_of(by, e)).or_default().push(*p);
    }
    map.into_iter().collect()
}

/// A dimension and its groups with their figures.
type Built = (Dimension, Vec<(String, Figure<Stat>)>);

fn outlier(figure: &Figure<Stat>, reference: Option<i64>, cutoff_bp: i64) -> Outlier {
    match (figure, reference) {
        (Figure::Shown { value }, Some(all)) => {
            let delta = value.compa_mean_bp - all;
            if delta > cutoff_bp {
                Outlier::Above { delta_bp: delta }
            } else if delta < -cutoff_bp {
                Outlier::Below { delta_bp: -delta }
            } else {
                Outlier::No
            }
        }
        _ => Outlier::Unknown,
    }
}

/// Work out the analysis. `floor` is the smallest group that may be shown (at least 1); `cutoff_bp` has been checked.
#[must_use]
pub fn analyse(entries: &[Entry], floor: usize, cutoff_bp: i64) -> Report {
    let floor = floor.max(1);
    let no_pay = entries.iter().filter(|e| e.salary_minor.is_none()).count();
    let rows: Vec<(&Entry, Point)> = entries
        .iter()
        .filter_map(|e| {
            let (salary, range) = (e.salary_minor?, e.range?);
            Some((
                e,
                Point {
                    compa: compa_bp(salary, range)?,
                    pen: penetration_bp(salary, range),
                },
            ))
        })
        .collect();
    // Pay with no usable band (none known, or a midpoint not above zero) is left out as having no band.
    let no_band = entries.len() - no_pay - rows.len();
    let everyone: Vec<Point> = rows.iter().map(|(_, p)| *p).collect();
    let overall_mean = mean(everyone.iter().map(|p| p.compa));

    let mut built: Vec<Built> = Vec::new();
    // The total is safe to show only if, in every grouping, the groups it would let a reader subtract away are not small.
    let mut total_safe = everyone.len() >= floor;
    for by in [
        Dimension::Band,
        Dimension::Level,
        Dimension::Department,
        Dimension::Location,
    ] {
        let groups = grouping(by, &rows);
        let held_back: usize = groups
            .iter()
            .filter(|(_, pts)| pts.len() < floor)
            .map(|(_, pts)| pts.len())
            .sum();
        if held_back > 0 && held_back < floor {
            total_safe = false;
        }
        built.push((
            by,
            groups
                .into_iter()
                .map(|(k, pts)| (k, stat(&pts, floor)))
                .collect(),
        ));
    }
    let overall = if everyone.is_empty() {
        Figure::Unknown {
            reason: Reason::NoPay,
        }
    } else if everyone.len() < floor {
        Figure::Withheld
    } else if !total_safe {
        Figure::Unknown {
            reason: Reason::WouldReveal,
        }
    } else {
        stat(&everyone, floor)
    };
    let reference = matches!(overall, Figure::Shown { .. })
        .then_some(overall_mean)
        .flatten();

    let groupings = built
        .into_iter()
        .map(|(by, groups)| Grouping {
            by,
            groups: groups
                .into_iter()
                .map(|(key, figure)| Group {
                    outlier: outlier(&figure, reference, cutoff_bp),
                    key,
                    figure,
                })
                .collect(),
        })
        .collect();
    Report {
        floor,
        cutoff_bp,
        overall,
        groupings,
        excluded: Exclusions {
            no_pay: left(no_pay, floor),
            no_band: left(no_band, floor),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: Range = Range {
        min: 4_000_000,
        max: 6_000_000,
    };

    fn e(salary: Option<i64>, band: &str, dept: &str) -> Entry {
        Entry {
            salary_minor: salary,
            band: band.to_string(),
            range: Some(R),
            level: Some("L4".to_string()),
            department: dept.to_string(),
            location: Some("Leeds".to_string()),
        }
    }

    fn many(n: usize, salary: i64, dept: &str) -> Vec<Entry> {
        (0..n).map(|_| e(Some(salary), "6", dept)).collect()
    }

    #[test]
    fn the_midpoint_is_ten_thousand_and_the_ends_are_zero_and_ten_thousand() {
        assert_eq!(compa_bp(5_000_000, R), Some(10_000));
        assert_eq!(compa_bp(4_000_000, R), Some(8_000));
        assert_eq!(compa_bp(6_000_000, R), Some(12_000));
        assert_eq!(penetration_bp(4_000_000, R), Some(0));
        assert_eq!(penetration_bp(5_000_000, R), Some(5_000));
        assert_eq!(penetration_bp(6_000_000, R), Some(10_000));
    }

    #[test]
    fn pay_outside_the_band_is_shown_outside_it_and_not_clamped() {
        assert_eq!(penetration_bp(3_000_000, R), Some(-5_000));
        assert_eq!(penetration_bp(7_000_000, R), Some(15_000));
    }

    #[test]
    fn a_single_rate_band_has_a_compa_ratio_but_no_penetration() {
        let one = Range {
            min: 2_630_000,
            max: 2_630_000,
        };
        assert_eq!(compa_bp(2_630_000, one), Some(10_000));
        assert_eq!(penetration_bp(2_630_000, one), None);
    }

    #[test]
    fn a_band_with_no_midpoint_gives_no_ratio() {
        let zero = Range { min: 0, max: 0 };
        assert_eq!(compa_bp(1, zero), None);
    }

    #[test]
    fn rounding_is_half_up_for_either_sign() {
        assert_eq!(div_round(1, 2), 1);
        assert_eq!(div_round(-1, 2), 0);
        assert_eq!(div_round(-3, 2), -1);
        assert_eq!(div_round(5, 3), 2);
    }

    #[test]
    fn a_group_under_the_floor_is_withheld_with_no_count() {
        let mut rows = many(10, 5_000_000, "Care");
        rows.extend(many(3, 5_000_000, "Admin"));
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        let dept = &r.groupings[2];
        assert_eq!(dept.by, Dimension::Department);
        let admin = dept.groups.iter().find(|g| g.key == "Admin").unwrap();
        assert_eq!(admin.figure, Figure::Withheld);
        assert_eq!(admin.outlier, Outlier::Unknown);
        let json = serde_json::to_string(admin).unwrap();
        assert!(!json.contains('3'), "no count leaks: {json}");
    }

    #[test]
    fn one_under_the_floor_is_withheld_and_the_floor_itself_is_shown() {
        let mut rows = many(10, 5_000_000, "Care");
        rows.extend(many(9, 5_000_000, "Admin"));
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        let dept = &r.groupings[2].groups;
        assert_eq!(
            dept.iter().find(|g| g.key == "Admin").unwrap().figure,
            Figure::Withheld
        );
        assert!(matches!(
            dept.iter().find(|g| g.key == "Care").unwrap().figure,
            Figure::Shown { .. }
        ));
    }

    #[test]
    fn a_total_that_would_reveal_a_single_small_group_is_not_shown() {
        // 10 in Care, 3 in Admin: the total of 13 less the 10 shown would give Admin away.
        let mut rows = many(10, 5_000_000, "Care");
        rows.extend(many(3, 5_000_000, "Admin"));
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        assert_eq!(
            r.overall,
            Figure::Unknown {
                reason: Reason::WouldReveal
            }
        );
        // With no outlier test against a total that is not shown.
        assert!(
            r.groupings
                .iter()
                .flat_map(|g| &g.groups)
                .all(|g| g.outlier == Outlier::Unknown)
        );
    }

    #[test]
    fn small_groups_that_together_reach_the_floor_do_not_hold_the_total_back() {
        let mut rows = many(10, 5_000_000, "Care");
        rows.extend(many(6, 5_000_000, "Admin"));
        rows.extend(many(6, 5_000_000, "Estates"));
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        assert!(matches!(r.overall, Figure::Shown { .. }));
    }

    #[test]
    fn a_group_far_from_everyone_is_an_outlier_and_one_near_is_not() {
        // 20 at the midpoint and 10 at 20% above it: the mean is 10 667, the high group is 1 333 above it.
        let mut rows = many(20, 5_000_000, "Care");
        rows.extend(many(10, 6_000_000, "Legal"));
        let r = analyse(&rows, 10, 1000);
        let dept = &r.groupings[2];
        let legal = dept.groups.iter().find(|g| g.key == "Legal").unwrap();
        assert_eq!(legal.outlier, Outlier::Above { delta_bp: 1_333 });
        let care = dept.groups.iter().find(|g| g.key == "Care").unwrap();
        assert_eq!(care.outlier, Outlier::No, "667 below is inside the cut-off");
    }

    #[test]
    fn nothing_is_an_outlier_at_exactly_the_cut_off() {
        let mut rows = many(10, 5_000_000, "A");
        rows.extend(many(10, 6_000_000, "B"));
        // Mean 11 000; A is 1 000 below, B is 1 000 above.
        let r = analyse(&rows, 10, 1000);
        assert!(
            r.groupings[2]
                .groups
                .iter()
                .all(|g| g.outlier == Outlier::No)
        );
        let tighter = analyse(&rows, 10, 999);
        assert!(
            tighter.groupings[2]
                .groups
                .iter()
                .all(|g| g.outlier != Outlier::No)
        );
    }

    #[test]
    fn no_pay_and_no_band_are_left_out_and_never_counted_as_zero() {
        let mut rows = many(10, 5_000_000, "Care");
        rows.push(e(None, "6", "Care"));
        let mut unbanded = e(Some(5_000_000), "?", "Care");
        unbanded.range = None;
        rows.push(unbanded);
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        let Figure::Shown { value } = &r.overall else {
            panic!("shown")
        };
        assert_eq!(value.workers, 10);
        assert_eq!(value.compa_mean_bp, 10_000);
        assert_eq!(r.excluded.no_pay, Left::UnderFloor);
        assert_eq!(r.excluded.no_band, Left::UnderFloor);
    }

    #[test]
    fn exclusions_show_their_count_only_from_the_floor() {
        let mut rows = many(10, 5_000_000, "Care");
        rows.extend((0..10).map(|_| e(None, "6", "Care")));
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        assert_eq!(r.excluded.no_pay, Left::Exactly { workers: 10 });
    }

    #[test]
    fn a_missing_level_or_location_is_a_group_of_its_own() {
        let mut rows = many(10, 5_000_000, "Care");
        for r in &mut rows {
            r.level = None;
            r.location = None;
        }
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        assert_eq!(r.groupings[1].groups[0].key, "unrecorded");
        assert_eq!(r.groupings[3].groups[0].key, "unrecorded");
    }

    #[test]
    fn penetration_is_unknown_for_single_rate_bands_and_withheld_under_the_floor() {
        let one = Range {
            min: 2_630_000,
            max: 2_630_000,
        };
        let rows: Vec<Entry> = (0..10)
            .map(|_| Entry {
                range: Some(one),
                ..e(Some(2_630_000), "2", "Care")
            })
            .collect();
        let r = analyse(&rows, 10, DEFAULT_CUTOFF_BP);
        let Figure::Shown { value } = &r.overall else {
            panic!("shown")
        };
        assert_eq!(
            value.penetration,
            Figure::Unknown {
                reason: Reason::NoWidth
            }
        );
        // Two with a range among ten without: enough people to show, too few with a width to show a penetration.
        let mut mixed = rows;
        mixed.extend((0..2).map(|_| e(Some(5_000_000), "2", "Care")));
        let r = analyse(&mixed, 10, DEFAULT_CUTOFF_BP);
        let Figure::Shown { value } = &r.overall else {
            panic!("shown")
        };
        assert_eq!(value.penetration, Figure::Withheld);
    }

    #[test]
    fn nobody_analysed_is_unknown_not_zero() {
        let r = analyse(&[e(None, "6", "Care")], 10, DEFAULT_CUTOFF_BP);
        assert_eq!(
            r.overall,
            Figure::Unknown {
                reason: Reason::NoPay
            }
        );
    }

    #[test]
    fn the_cut_off_is_bounded() {
        assert!(check_cutoff(99).is_err());
        assert!(check_cutoff(100).is_ok());
        assert!(check_cutoff(5000).is_ok());
        assert!(check_cutoff(5001).is_err());
    }

    #[test]
    fn an_entry_holds_no_name_and_the_report_names_no_worker() {
        let src = include_str!("pay_range.rs");
        let entry = &src[src.find("pub struct Entry").unwrap()..];
        let entry = &entry[..entry.find("\n}\n").unwrap()];
        for word in ["pid", "name", "person", "worker_ref"] {
            assert!(!entry.contains(word), "Entry must not carry {word}");
        }
    }
}
