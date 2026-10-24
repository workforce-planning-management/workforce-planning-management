//! **Pay information** (WPM-R145, WPM-D99): pure rules for a worker asking what they are paid and what their category
//! of workers is paid on average by sex (Article 7 of the EU Pay Transparency Directive), and for stating a pay range
//! before a vacancy is opened (Article 5). Spec: `spec/pay-transparency.md`.
//!
//! - **The clock is computed.** The answer is due a configured number of calendar months after the request (the
//!   Directive says a reasonable period and in any event two months). A request past that date is *overdue*, never
//!   quietly closed.
//! - **An average is withheld under the floor**, for each sex on its own, and a side with nobody is unknown.
//! - **The figures are not stored.** Only the dates are; the answer is worked out when it is given.

use chrono::{Months, NaiveDate};
use serde::Serialize;

use super::pay_gap::{Figure, Reason, mean_minor};

/// The default time to answer, in calendar months: the Directive's two months.
pub const DEFAULT_ANSWER_MONTHS: u32 = 2;
/// The longest a deployment may set.
pub const MAX_ANSWER_MONTHS: u32 = 12;

/// The day an answer is due: `months` calendar months after the request (a month-end clamps to the shorter month).
#[must_use]
pub fn due_on(asked_on: NaiveDate, months: u32) -> Option<NaiveDate> {
    asked_on.checked_add_months(Months::new(months))
}

/// Where a request stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Standing {
    /// Waiting, and not yet past its due date.
    Open,
    /// Waiting, and past its due date.
    Overdue,
    /// Answered.
    Answered,
}

/// A request's standing on `today`. The due day itself is still open; the day after is overdue.
#[must_use]
pub fn standing(answered_on: Option<NaiveDate>, due_on: NaiveDate, today: NaiveDate) -> Standing {
    if answered_on.is_some() {
        Standing::Answered
    } else if today > due_on {
        Standing::Overdue
    } else {
        Standing::Open
    }
}

/// A worker has one request waiting at a time.
///
/// # Errors
///
/// When one is already waiting.
pub fn may_ask(has_open_request: bool) -> Result<(), String> {
    if has_open_request {
        Err("you already have a request waiting for an answer".to_string())
    } else {
        Ok(())
    }
}

/// What a worker is told: their own pay, and the average for their category by sex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Answer {
    /// The worker's own pay on the comparable basis, in minor units; `None` when none is recorded.
    pub own_pay_minor: Option<i64>,
    /// The average of the female workers in the category, or why there is none.
    pub female_average_minor: Figure<i64>,
    /// The average of the male workers in the category.
    pub male_average_minor: Figure<i64>,
}

/// One sex's average, withheld under the floor, unknown with nobody.
fn average(values: &[i64], floor: usize) -> Figure<i64> {
    if values.is_empty() {
        Figure::Unknown(Reason::NoWorkers)
    } else if values.len() < floor {
        Figure::Withheld
    } else {
        mean_minor(values).map_or(Figure::Unknown(Reason::NoBase), Figure::Shown)
    }
}

/// Work out the answer. `female` and `male` are the pay of the workers of each sex in the worker's category (the
/// asker included in their own sex); `None` for either means the category or the sex data is missing, with the
/// reason.
#[must_use]
pub fn answer(
    own_pay_minor: Option<i64>,
    pools: Result<(&[i64], &[i64]), Reason>,
    floor: usize,
) -> Answer {
    match pools {
        Ok((female, male)) => Answer {
            own_pay_minor,
            female_average_minor: average(female, floor),
            male_average_minor: average(male, floor),
        },
        Err(reason) => Answer {
            own_pay_minor,
            female_average_minor: Figure::Unknown(reason),
            male_average_minor: Figure::Unknown(reason),
        },
    }
}

/// A vacancy states its pay before it is opened (Article 5): a minimum and a maximum, the minimum above zero and not
/// above the maximum. One stated figure is a minimum equal to the maximum.
///
/// # Errors
///
/// A message saying what to state.
pub fn require_range(min: Option<i64>, max: Option<i64>) -> Result<(), String> {
    match (min, max) {
        (Some(low), Some(high)) if low > 0 && low <= high => Ok(()),
        (Some(_), Some(_)) => Err("the pay range must start above zero and not run backwards".to_string()),
        _ => Err("state the pay range (a minimum and a maximum) before the vacancy is opened; applicants are owed it before the interview".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn the_answer_is_due_a_whole_number_of_calendar_months_later() {
        assert_eq!(due_on(day(2026, 10, 24), 2), Some(day(2026, 12, 24)));
        assert_eq!(
            due_on(day(2026, 12, 31), 2),
            Some(day(2027, 2, 28)),
            "a month-end clamps"
        );
        assert_eq!(
            due_on(day(2027, 12, 31), 2),
            Some(day(2028, 2, 29)),
            "to a leap day when there is one"
        );
        assert_eq!(due_on(day(2026, 1, 15), 0), Some(day(2026, 1, 15)));
        assert_eq!(due_on(NaiveDate::MAX, 1), None, "past the calendar");
    }

    #[test]
    fn a_request_is_open_on_its_due_day_and_overdue_after_and_answered_stays_answered() {
        let due = day(2026, 12, 24);
        assert_eq!(standing(None, due, day(2026, 10, 24)), Standing::Open);
        assert_eq!(standing(None, due, due), Standing::Open);
        assert_eq!(standing(None, due, day(2026, 12, 25)), Standing::Overdue);
        assert_eq!(
            standing(Some(day(2027, 3, 1)), due, day(2027, 6, 1)),
            Standing::Answered,
            "answered late is still answered"
        );
    }

    #[test]
    fn one_request_at_a_time() {
        assert!(may_ask(false).is_ok());
        assert!(may_ask(true).unwrap_err().contains("already"));
    }

    #[test]
    fn each_sexs_average_is_withheld_under_the_floor_on_its_own() {
        let f: Vec<i64> = vec![90, 90, 90, 90, 90];
        let m: Vec<i64> = vec![100, 100];
        let a = answer(Some(90), Ok((&f, &m)), 5);
        assert_eq!(a.female_average_minor, Figure::Shown(90));
        assert_eq!(
            a.male_average_minor,
            Figure::Withheld,
            "two men are under the floor"
        );
        assert_eq!(
            a.own_pay_minor,
            Some(90),
            "the worker's own pay is theirs to see"
        );
    }

    #[test]
    fn a_sex_with_nobody_in_the_category_is_unknown_not_zero() {
        let f: Vec<i64> = vec![1, 2, 3, 4, 5];
        let a = answer(Some(3), Ok((&f, &[])), 5);
        assert_eq!(a.male_average_minor, Figure::Unknown(Reason::NoWorkers));
        assert!(matches!(a.female_average_minor, Figure::Shown(3)));
    }

    #[test]
    fn no_category_or_no_sex_data_says_why_and_still_gives_the_workers_own_pay() {
        let a = answer(Some(7), Err(Reason::NoCategory), 5);
        assert_eq!(a.female_average_minor, Figure::Unknown(Reason::NoCategory));
        assert_eq!(a.male_average_minor, Figure::Unknown(Reason::NoCategory));
        assert_eq!(a.own_pay_minor, Some(7));
        let b = answer(None, Err(Reason::NoSexRecorded), 5);
        assert_eq!(b.own_pay_minor, None, "no recorded pay is not zero");
        assert_eq!(b.male_average_minor, Figure::Unknown(Reason::NoSexRecorded));
    }

    #[test]
    fn an_average_rounds_half_away_from_zero() {
        let f: Vec<i64> = vec![1, 2, 2, 2, 2, 2];
        // 11 / 6 = 1.83 -> 2
        assert_eq!(
            answer(None, Ok((&f, &f)), 5).female_average_minor,
            Figure::Shown(2)
        );
        let g: Vec<i64> = vec![1, 1, 1, 1, 2, 2];
        // 8 / 6 = 1.33 -> 1
        assert_eq!(
            answer(None, Ok((&g, &g)), 5).female_average_minor,
            Figure::Shown(1)
        );
    }

    #[test]
    fn a_vacancy_states_a_range_before_it_opens() {
        assert!(require_range(Some(30_000), Some(40_000)).is_ok());
        assert!(
            require_range(Some(35_000), Some(35_000)).is_ok(),
            "one stated figure"
        );
        for (min, max) in [(None, None), (Some(1), None), (None, Some(1))] {
            assert!(
                require_range(min, max)
                    .unwrap_err()
                    .contains("state the pay range"),
                "{min:?}/{max:?}"
            );
        }
        for (min, max) in [
            (Some(0), Some(10)),
            (Some(-5), Some(10)),
            (Some(20), Some(10)),
        ] {
            assert!(
                require_range(min, max).unwrap_err().contains("above zero"),
                "{min:?}/{max:?}"
            );
        }
    }
}
