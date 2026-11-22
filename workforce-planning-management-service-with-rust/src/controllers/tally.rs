//! Counting and date offsets without a plain `+` or `-` (`spec/rust-checked-arithmetic/index.md`).

use std::ops::AddAssign;

use axa::prelude::*;
use jiff::civil::Date;

/// Count one more item.
///
/// For a tally with one step per item of a collection already held in memory (a row, a worker, a plan): the tally cannot pass the
/// number of items, which is far below the largest value of any counter type used here. A figure that comes from outside the code
/// (an amount, a minute count, a version number) is added with `checked_add` instead.
pub(crate) fn bump<T: AddAssign + From<u8>>(counter: &mut T) {
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "RCA-EXC-2: one step per item held in memory: the count cannot reach the type's largest value"
    )]
    {
        *counter += T::from(1);
    }
}

/// An integer type a total can be kept in.
pub(crate) trait Total: Copy {
    /// `self + other`, or `None` when it does not fit.
    fn plus(self, other: Self) -> Option<Self>;
}

macro_rules! total_of {
    ($($int:ty),*) => {$(
        impl Total for $int {
            fn plus(self, other: Self) -> Option<Self> {
                self.checked_add(other)
            }
        }
    )*};
}

total_of!(usize, u64, u32, i64, i32);

/// Add `amount` to `total`: a figure that comes from outside the code (a stored count, an amount, minutes).
///
/// # Errors
///
/// A `500` when the sum does not fit the type; `total` is left as it was. A stored figure that large is a broken invariant, and a
/// wrapped or capped sum would be a number nobody recorded.
pub(crate) fn add<T: Total>(total: &mut T, amount: T) -> Result<()> {
    *total = total
        .plus(amount)
        .ok_or_else(|| super::internal("a total overflowed"))?;
    Ok(())
}

/// The date `days` whole days after `date` (before it, when `days` is negative).
///
/// # Errors
///
/// A `500` when the result is a date the calendar cannot hold: the date and the offset are the code's own (the clock, a
/// clamped parameter), so none is expected.
pub(crate) fn date_plus_days(date: Date, days: i64) -> Result<Date> {
    jiff::Span::new()
        .try_days(days)
        .and_then(|span| date.checked_add(span))
        .map_err(|_| super::internal("a date offset left the calendar"))
}

/// The date `days` whole days before `date`.
///
/// # Errors
///
/// As [`date_plus_days`].
pub(crate) fn date_minus_days(date: Date, days: i64) -> Result<Date> {
    jiff::Span::new()
        .try_days(days)
        .and_then(|span| date.checked_sub(span))
        .map_err(|_| super::internal("a date offset left the calendar"))
}

#[cfg(test)]
mod tests {
    use super::{add, bump, date_minus_days, date_plus_days};
    use jiff::civil::Date;

    #[test]
    fn counts_one_per_call() {
        let (mut small, mut wide) = (0_usize, 41_i64);
        bump(&mut small);
        bump(&mut wide);
        bump(&mut wide);
        assert_eq!((small, wide), (1, 43));
    }

    #[test]
    fn adds_a_figure_and_refuses_a_sum_that_does_not_fit() {
        let mut total = 40_usize;
        assert!(add(&mut total, 2).is_ok());
        assert_eq!(total, 42);
        assert!(add(&mut total, usize::MAX).is_err());
        assert_eq!(total, 42);
        let mut wide = i64::MAX;
        assert!(add(&mut wide, 1).is_err());
        let (mut a, mut b, mut c) = (u64::MAX, u32::MAX, i32::MAX);
        assert!(add(&mut a, 1).is_err() && add(&mut b, 1).is_err() && add(&mut c, 1).is_err());
    }

    #[test]
    fn moves_a_date_and_refuses_one_off_the_calendar() {
        let day = Date::constant(2026, 3, 1);
        assert_eq!(
            date_plus_days(day, 30).ok(),
            Some(Date::constant(2026, 3, 31))
        );
        assert_eq!(
            date_minus_days(day, 1).ok(),
            Some(Date::constant(2026, 2, 28))
        );
        assert!(date_plus_days(Date::MAX, 1).is_err());
        assert!(date_minus_days(Date::MIN, 1).is_err());
        assert!(date_plus_days(day, i64::MAX).is_err());
        assert!(date_minus_days(day, i64::MIN).is_err());
    }
}
