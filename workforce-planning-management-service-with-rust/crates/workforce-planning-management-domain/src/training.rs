//! Pure rules for **training time recommendations**: how many hours it would
//! take a person to close a skill gap, which catalogue courses would do it,
//! and — at a stated weekly pace — when they would finish.
//!
//! These are **planning estimates, not promises**. A recommendation says what
//! it rests on: catalogue courses (their stated hours), or a per-level
//! planning default (`hours_per_level`, set per skill or the service
//! default). It never invents a number for a skill the person has not
//! declared — that needs assessing first, not training.
//!
//! DB-free and clock-free (the start day is supplied).
use jiff::Span;
use jiff::civil::Date;
use serde::Serialize;
use std::collections::HashSet;

/// Hours to gain one proficiency level when a skill has no figure of its own.
pub const DEFAULT_HOURS_PER_LEVEL: u32 = 30;

/// Weekly training hours for a full-time worker when none is asked for.
pub const DEFAULT_WEEKLY_HOURS: f64 = 4.0;

/// The most weekly hours a plan may assume.
pub const MAX_WEEKLY_HOURS: u32 = 40;

/// A catalogue course that builds a skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Course {
    /// The upstream course reference.
    pub course_ref: String,
    /// Display title.
    pub title: String,
    /// Hours it takes.
    pub hours: u32,
    /// Levels it typically adds (1–4).
    pub levels: u32,
}

/// What a recommendation rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    /// Catalogue courses cover the whole gap.
    Courses,
    /// Catalogue courses cover part; the rest is the per-level estimate.
    Mixed,
    /// No (remaining) catalogue course: the per-level planning estimate.
    Estimate,
}

/// The training that would close one gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recommendation {
    /// Courses to take, cheapest per level first.
    pub courses: Vec<Course>,
    /// Levels the courses are expected to add.
    pub course_levels: u32,
    /// Levels left to the per-level estimate.
    pub estimated_levels: u32,
    /// Total hours: the courses' hours plus the estimate for what they leave.
    pub hours: u32,
    /// What it rests on.
    pub basis: Basis,
}

/// The levels left to the per-level estimate when the courses cover the whole shortfall: none.
const NO_LEVELS_LEFT: u32 = 0;

/// Recommend training to close a gap of `shortfall` levels. Catalogue courses
/// the person has already `completed` are skipped; the rest are taken
/// cheapest-per-level first (ties by title, so it is stable) until the gap
/// is covered. Whatever the courses leave is the per-level estimate.
///
/// `None` when the levels or hours add up to more than a `u32` holds: there is then no recommendation to make, and
/// not a smaller one.
#[must_use]
pub fn recommend<S: std::hash::BuildHasher>(
    shortfall: u32,
    catalogue: &[Course],
    completed: &HashSet<String, S>,
    hours_per_level: u32,
) -> Option<Recommendation> {
    let mut available: Vec<&Course> = catalogue
        .iter()
        .filter(|c| !completed.contains(&c.course_ref) && c.levels > 0)
        .collect();
    // hours/levels ascending, compared by cross-multiplication (no floats).
    available.sort_by(|a, b| {
        cross(a.hours, b.levels)
            .cmp(&cross(b.hours, a.levels))
            .then(a.title.cmp(&b.title))
            .then(a.course_ref.cmp(&b.course_ref))
    });
    let mut courses = Vec::new();
    let mut course_levels: u32 = 0;
    let mut hours: u32 = 0;
    for c in available {
        if course_levels >= shortfall {
            break;
        }
        course_levels = course_levels.checked_add(c.levels)?;
        hours = hours.checked_add(c.hours)?;
        courses.push(c.clone());
    }
    // Courses that cover the gap, or more, leave no levels to the per-level estimate ("whatever the courses leave" above).
    let estimated_levels = shortfall
        .checked_sub(course_levels)
        .unwrap_or(NO_LEVELS_LEFT);
    hours = hours.checked_add(estimated_levels.checked_mul(hours_per_level)?)?;
    let basis = if courses.is_empty() {
        Basis::Estimate
    } else if estimated_levels > 0 {
        Basis::Mixed
    } else {
        Basis::Courses
    };
    Some(Recommendation {
        courses,
        course_levels,
        estimated_levels,
        hours,
        basis,
    })
}

/// `hours * levels` as a `u64`, for comparing two courses' hours per level without division.
fn cross(hours: u32, levels: u32) -> u64 {
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "RCA-EXC-2: the product of two u32 values is under 2^64"
    )]
    let product = u64::from(hours) * u64::from(levels);
    product
}

/// The weekly hours a plan assumes: the request if given and within 1–40,
/// else [`DEFAULT_WEEKLY_HOURS`] scaled by the worker's FTE (at least 1) —
/// a half-time worker has half the time to train.
///
/// # Errors
///
/// A message when the request is outside 1–40, or the worker's FTE is outside 1–100 (`spec/domain-model.md`).
pub fn weekly_hours(requested: Option<i64>, fte_percent: i32) -> Result<u32, String> {
    if let Some(n) = requested {
        return u32::try_from(n)
            .ok()
            .filter(|h| (1..=MAX_WEEKLY_HOURS).contains(h))
            .ok_or_else(|| format!("weekly_hours must be between 1 and {MAX_WEEKLY_HOURS}"));
    }
    if !(1..=100).contains(&fte_percent) {
        return Err("the worker's FTE must be between 1 and 100 percent".to_string());
    }
    let fte = f64::from(fte_percent) / 100.0;
    let scaled = crate::metrics::rounded_i64(DEFAULT_WEEKLY_HOURS * fte)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| "the weekly hours do not fit a count".to_string())?;
    Ok(scaled.max(1))
}

/// One item's place in the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    /// First day (`None` for an item that needs no hours).
    pub starts_on: Option<Date>,
    /// Last day.
    pub ends_on: Option<Date>,
    /// Whole weeks at the weekly pace.
    pub weeks: u32,
    /// Hours up to and including this item.
    pub cumulative_hours: u32,
}

/// Lay items out one after another, in the order given, at `weekly_hours`:
/// each takes `ceil(hours / weekly_hours)` whole weeks starting where the last
/// ended. An item with no hours takes no time and is not placed.
///
/// `None` when the hours add up to more than a `u32` holds, or the plan runs past the last date the calendar has.
#[must_use]
pub fn schedule(hours: &[u32], weekly_hours: u32, start: Date) -> Option<Vec<Slot>> {
    let pace = weekly_hours.max(1);
    let mut cursor = start;
    let mut total: u32 = 0;
    hours
        .iter()
        .map(|h| {
            total = total.checked_add(*h)?;
            if *h == 0 {
                return Some(Slot {
                    starts_on: None,
                    ends_on: None,
                    weeks: 0,
                    cumulative_hours: total,
                });
            }
            let weeks = h.div_ceil(pace);
            let starts_on = cursor;
            let next = cursor
                .checked_add(Span::new().try_weeks(i64::from(weeks)).ok()?)
                .ok()?;
            let ends_on = next.yesterday().ok()?;
            cursor = next;
            Some(Slot {
                starts_on: Some(starts_on),
                ends_on: Some(ends_on),
                weeks,
                cumulative_hours: total,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn course(r: &str, hours: u32, levels: u32) -> Course {
        Course {
            course_ref: r.to_string(),
            title: r.to_uppercase(),
            hours,
            levels,
        }
    }
    fn none() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn no_catalogue_means_the_per_level_estimate_and_says_so() {
        let r = recommend(3, &[], &none(), 30).unwrap();
        assert_eq!(
            (r.hours, r.basis, r.estimated_levels),
            (90, Basis::Estimate, 3)
        );
        assert_eq!(r.courses, Vec::<Course>::new());
    }

    #[test]
    fn courses_are_taken_cheapest_per_level_first_until_the_gap_is_covered() {
        let cat = [
            course("big", 60, 2),
            course("fast", 10, 1),
            course("slow", 40, 1),
        ];
        // fast = 10/level, big = 30/level, slow = 40/level.
        let r = recommend(2, &cat, &none(), 99).unwrap();
        let refs: Vec<&str> = r.courses.iter().map(|c| c.course_ref.as_str()).collect();
        assert_eq!(refs, ["fast", "big"]);
        assert_eq!((r.course_levels, r.hours, r.basis), (3, 70, Basis::Courses));
    }

    #[test]
    fn what_courses_leave_is_estimated_and_the_basis_is_mixed() {
        let r = recommend(3, &[course("a", 20, 1)], &none(), 30).unwrap();
        assert_eq!((r.course_levels, r.estimated_levels), (1, 2));
        assert_eq!(r.hours, 20 + 60);
        assert_eq!(r.basis, Basis::Mixed);
    }

    #[test]
    fn a_completed_course_is_not_recommended_again() {
        let cat = [course("done", 10, 1), course("next", 25, 1)];
        let completed: HashSet<String> = ["done".to_string()].into();
        let r = recommend(1, &cat, &completed, 30).unwrap();
        assert_eq!(r.courses.len(), 1);
        assert_eq!(r.courses[0].course_ref, "next");
        // Everything completed: back to the estimate.
        let all: HashSet<String> = ["done".to_string(), "next".to_string()].into();
        assert_eq!(recommend(1, &cat, &all, 30).unwrap().basis, Basis::Estimate);
    }

    #[test]
    fn no_shortfall_needs_no_training() {
        let r = recommend(0, &[course("a", 20, 1)], &none(), 30).unwrap();
        assert_eq!((r.hours, r.courses.len()), (0, 0));
    }

    #[test]
    fn weekly_hours_default_scales_with_fte_and_a_request_is_bounded() {
        assert_eq!(weekly_hours(None, 100), Ok(4));
        assert_eq!(weekly_hours(None, 50), Ok(2));
        assert_eq!(weekly_hours(None, 10), Ok(1), "at least one hour");
        assert_eq!(weekly_hours(Some(6), 50), Ok(6), "a request wins");
        assert!(weekly_hours(Some(0), 100).is_err());
        assert!(weekly_hours(Some(41), 100).is_err());
        assert!(weekly_hours(Some(-3), 100).is_err());
        assert_eq!(weekly_hours(None, 1), Ok(1));
        for fte in [0, -1, 101, i32::MIN, i32::MAX] {
            assert_eq!(
                weekly_hours(None, fte),
                Err("the worker's FTE must be between 1 and 100 percent".to_string())
            );
        }
        assert_eq!(weekly_hours(Some(6), 0), Ok(6), "a request needs no FTE");
    }

    #[test]
    fn items_run_one_after_another_in_whole_weeks() {
        let start = crate::dates::ymd(2026, 10, 5).unwrap();
        let slots = schedule(&[10, 0, 3], 4, start).unwrap();
        // 10h at 4/week = 3 weeks: 5–25 Oct. Then a zero-hour item: not placed.
        assert_eq!(slots[0].weeks, 3);
        assert_eq!(slots[0].starts_on, Some(start));
        assert_eq!(
            slots[0].ends_on,
            Some(crate::dates::ymd(2026, 10, 25).unwrap())
        );
        assert_eq!(
            (
                slots[1].starts_on,
                slots[1].weeks,
                slots[1].cumulative_hours
            ),
            (None, 0, 10)
        );
        // 3h = 1 week, starting the day after the first ended.
        assert_eq!(
            slots[2].starts_on,
            Some(crate::dates::ymd(2026, 10, 26).unwrap())
        );
        assert_eq!(slots[2].cumulative_hours, 13);
    }

    #[test]
    fn totals_that_do_not_fit_are_no_recommendation_and_no_plan() {
        let big = |r: &str, h, l| course(r, h, l);
        // Levels add up past u32.
        let cat = [big("a", 1, u32::MAX - 1), big("b", 1, 2)];
        assert!(recommend(u32::MAX, &cat, &none(), 1).is_none());
        // Hours add up past u32.
        let cat = [big("a", u32::MAX, 1), big("b", 1, 1)];
        assert!(recommend(2, &cat, &none(), 1).is_none());
        // The estimate overflows, or tips the hours over.
        assert!(recommend(10, &[], &none(), u32::MAX).is_none());
        assert!(recommend(2, &[big("a", u32::MAX, 1)], &none(), 1).is_none());
        let start = crate::dates::ymd(2026, 10, 5).unwrap();
        assert!(schedule(&[u32::MAX, 1], 1, start).is_none(), "hours total");
        assert!(
            schedule(&[u32::MAX], 1, start).is_none(),
            "weeks beyond a span"
        );
        assert!(schedule(&[1], 1, Date::MAX).is_none(), "past the calendar");
    }
}
