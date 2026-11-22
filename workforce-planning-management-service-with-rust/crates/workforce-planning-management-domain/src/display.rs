//! **Display arithmetic** (WPM-T253): the few conversions a response makes from stored integers
//! (hundredths of a person or an amount, tenths of an hour) and from two counts to a rate. Pure,
//! so a controller never divides inline. The integer is always returned alongside a display value.

use super::metrics::{count_f64, exact_f64};

/// `numerator / denominator`, or `None` when the denominator is zero or a term is past 2^53 and a float would round it
/// (RCA-CONV-4): an unknown is never a zero.
#[must_use]
pub fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    if denominator == 0 {
        return None;
    }
    Some(count_f64(numerator)? / count_f64(denominator)?)
}

/// `sum / count`, or `None` when the count is zero or a term is past 2^53 (RCA-CONV-4). The average of no values is unknown.
#[must_use]
pub fn average(sum: i64, count: usize) -> Option<f64> {
    if count == 0 {
        return None;
    }
    Some(exact_f64(sum)? / count_f64(count)?)
}

/// Hundredths as units (`1250` is `12.5`), or `None` when the figure is past 2^53 and a float would round it.
#[must_use]
pub fn from_hundredths(hundredths: i64) -> Option<f64> {
    Some(exact_f64(hundredths)? / 100.0)
}

/// Tenths as units (`75` is `7.5`), or `None` when the figure is past 2^53 and a float would round it.
#[must_use]
pub fn from_tenths(tenths: i64) -> Option<f64> {
    Some(exact_f64(tenths)? / 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ratio_of_nothing_is_unknown_not_zero() {
        assert_eq!(ratio(0, 0), None);
        assert_eq!(ratio(3, 0), None);
        assert_eq!(ratio(0, 4), Some(0.0));
        assert_eq!(ratio(1, 4), Some(0.25));
    }

    #[test]
    fn hundredths_and_tenths_become_units() {
        assert_eq!(from_hundredths(1250), Some(12.5));
        assert_eq!(from_hundredths(-5), Some(-0.05));
        assert_eq!(from_tenths(75), Some(7.5));
        assert_eq!(from_tenths(0), Some(0.0));
    }

    #[test]
    fn a_figure_a_float_would_round_is_unknown() {
        assert_eq!(from_hundredths(i64::MAX), None);
        assert_eq!(from_tenths(i64::MIN), None);
        assert_eq!(ratio(usize::MAX, 1), None);
        assert_eq!(ratio(1, usize::MAX), None);
        assert_eq!(average(i64::MAX, 1), None);
        assert_eq!(average(1, usize::MAX), None);
    }

    #[test]
    fn an_average_of_nothing_is_unknown_not_zero() {
        assert_eq!(average(0, 0), None);
        assert_eq!(average(7, 0), None);
        assert_eq!(average(-9, 3), Some(-3.0));
        assert_eq!(average(5, 2), Some(2.5));
    }
}
