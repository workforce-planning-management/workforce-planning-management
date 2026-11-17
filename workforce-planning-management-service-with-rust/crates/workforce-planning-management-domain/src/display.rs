//! **Display arithmetic** (WPM-T253): the few conversions a response makes from stored integers
//! (hundredths of a person or an amount, tenths of an hour) and from two counts to a rate. Pure,
//! so a controller never divides inline. The integer is always returned alongside a display value.

/// `numerator / denominator`, or `None` when the denominator is zero: an unknown is never a zero.
#[must_use]
#[allow(clippy::cast_precision_loss)] // display ratio; the terms are returned alongside
pub fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator != 0).then(|| numerator as f64 / denominator as f64)
}

/// Hundredths as units (`1250` is `12.5`).
#[must_use]
#[allow(clippy::cast_precision_loss)] // display value; the integer is returned alongside
pub fn from_hundredths(hundredths: i64) -> f64 {
    hundredths as f64 / 100.0
}

/// Tenths as units (`75` is `7.5`).
#[must_use]
#[allow(clippy::cast_precision_loss)] // a few thousand tenths at most
pub fn from_tenths(tenths: i64) -> f64 {
    tenths as f64 / 10.0
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
        assert!((from_hundredths(1250) - 12.5).abs() < f64::EPSILON);
        assert!((from_hundredths(-5) + 0.05).abs() < f64::EPSILON);
        assert!((from_tenths(75) - 7.5).abs() < f64::EPSILON);
        assert!(from_tenths(0).abs() < f64::EPSILON);
    }
}
