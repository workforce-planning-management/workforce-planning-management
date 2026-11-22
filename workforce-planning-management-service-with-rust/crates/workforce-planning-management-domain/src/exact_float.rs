//! **Exact integer to float conversion** (RCA-CONV-2, RCA-CONV-4): a count or an amount becomes an `f64` only when the
//! float holds it exactly (magnitude at most 2^53). Beyond that the conversion is unknown (`None`), never a rounded number.

/// 2^53: every integer of at most this magnitude is exactly representable as an `f64`.
const MAX_EXACT: u64 = 9_007_199_254_740_992;
/// 2^32, to rebuild a 64-bit magnitude from two 32-bit halves.
const TWO_POW_32: f64 = 4_294_967_296.0;
/// The mask for the low 32 bits.
const LOW_MASK: u64 = 0xFFFF_FFFF;

/// `n` as an `f64`, or `None` when it is above 2^53 (RCA-CONV-4).
#[must_use]
pub fn from_u64(n: u64) -> Option<f64> {
    if n > MAX_EXACT {
        return None;
    }
    let high = u32::try_from(n >> 32).ok()?;
    let low = u32::try_from(n & LOW_MASK).ok()?;
    Some(f64::from(high) * TWO_POW_32 + f64::from(low))
}

/// `n` as an `f64`, or `None` when its magnitude is above 2^53 (RCA-CONV-4).
#[must_use]
pub fn from_i64(n: i64) -> Option<f64> {
    let magnitude = from_u64(n.unsigned_abs())?;
    Some(if n < 0 { -magnitude } else { magnitude })
}

/// `n` as an `f64`, or `None` when it is above 2^53 (RCA-CONV-4).
#[must_use]
pub fn from_usize(n: usize) -> Option<f64> {
    u64::try_from(n).ok().and_then(from_u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_values_convert_exactly() {
        assert_eq!(from_u64(0), Some(0.0));
        assert_eq!(from_u64(4_294_967_297), Some(4_294_967_297.0));
        assert_eq!(from_i64(-5), Some(-5.0));
        assert_eq!(from_i64(7), Some(7.0));
        assert_eq!(from_usize(12), Some(12.0));
    }

    #[test]
    fn two_pow_53_is_the_last_exact_value() {
        assert_eq!(from_u64(MAX_EXACT), Some(9_007_199_254_740_992.0));
        assert_eq!(from_u64(MAX_EXACT + 1), None);
        assert_eq!(
            from_i64(-9_007_199_254_740_992),
            Some(-9_007_199_254_740_992.0)
        );
        assert_eq!(from_i64(i64::MIN), None);
        assert_eq!(from_i64(i64::MAX), None);
        assert_eq!(from_usize(usize::MAX), None);
    }
}
