//! **A requested page size** (RCA-INT-6): the number of rows a list endpoint returns. The caller may ask for fewer than the
//! endpoint's largest page; asking for more is answered with the largest page, which each endpoint declares in its documentation
//! as "at most N". Asking for none is refused, not turned into one.

/// The page size to use: `default` when none was asked for, the request when it is at least 1, and never more than `max` (the
/// declared largest page, cited by the caller). `None` when the request is 0, which no page can answer.
#[must_use]
pub fn resolve<T: Ord + Copy + From<u8>>(requested: Option<T>, default: T, max: T) -> Option<T> {
    match requested {
        None => Some(default.min(max)),
        Some(n) if n < T::from(1) => None,
        Some(n) => Some(n.min(max)),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[test]
    fn a_request_is_kept_up_to_the_declared_largest_page() {
        assert_eq!(resolve(None, 25_usize, 100), Some(25));
        assert_eq!(resolve(Some(1_usize), 25, 100), Some(1));
        assert_eq!(resolve(Some(100_usize), 25, 100), Some(100));
        assert_eq!(resolve(Some(101_usize), 25, 100), Some(100));
        assert_eq!(resolve(Some(u64::MAX), 100, 500), Some(500));
    }

    #[test]
    fn a_request_for_no_rows_is_refused_not_made_one() {
        assert_eq!(resolve(Some(0_usize), 25, 100), None);
        assert_eq!(resolve(Some(0_u64), 100, 500), None);
    }
}
