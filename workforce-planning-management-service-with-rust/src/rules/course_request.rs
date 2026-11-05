//! Starting a course (WPM-R236, WPM-D182). DB-free and clock-free.
//!
//! A recorded fee of zero is free, so the person enrols at once. Anything else needs a manager's approval, and that includes a course
//! with no fee recorded, because unknown is never free (WPM-D172), and a course whose catalogue rows disagree, which the spend already
//! treats as no fee (WPM-R227).

/// How a person may start a course.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Every catalogue row says the fee is zero: enrol now.
    Direct,
    /// A fee, an unknown fee or a disagreement: a manager decides.
    Approval,
}

/// The route for a course from the fee on each of its catalogue rows, or `None` for a course that is not in the catalogue.
#[must_use]
pub fn route(fees: &[Option<i64>]) -> Option<Route> {
    if fees.is_empty() {
        None
    } else if fees.iter().all(|fee| *fee == Some(0)) {
        Some(Route::Direct)
    } else {
        Some(Route::Approval)
    }
}

/// One fee for a course whose rows may disagree: the fee and currency all rows share, else nothing, so a reader sees "no fee recorded"
/// and never one of two figures.
#[must_use]
pub fn shared_fee(fees: &[(Option<i64>, Option<String>)]) -> Option<(i64, Option<String>)> {
    let (first, rest) = fees.split_first()?;
    let amount = first.0?;
    rest.iter()
        .all(|row| row == first)
        .then(|| (amount, first.1.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_recorded_zero_is_free() {
        assert_eq!(route(&[Some(0)]), Some(Route::Direct));
        assert_eq!(route(&[Some(0), Some(0)]), Some(Route::Direct));
        assert_eq!(route(&[Some(1)]), Some(Route::Approval));
        assert_eq!(route(&[None]), Some(Route::Approval), "unknown is not free");
    }

    #[test]
    fn rows_that_disagree_need_approval() {
        assert_eq!(route(&[Some(0), Some(5000)]), Some(Route::Approval));
        assert_eq!(route(&[Some(0), None]), Some(Route::Approval));
    }

    #[test]
    fn a_course_outside_the_catalogue_has_no_route() {
        assert_eq!(route(&[]), None);
    }

    #[test]
    fn rows_that_disagree_show_no_fee() {
        let gbp = |fee| (Some(fee), Some("GBP".to_string()));
        assert_eq!(
            shared_fee(&[gbp(100), gbp(100)]),
            Some((100, Some("GBP".to_string())))
        );
        assert_eq!(shared_fee(&[gbp(100), gbp(200)]), None);
        assert_eq!(shared_fee(&[gbp(100), (None, None)]), None);
        assert_eq!(shared_fee(&[(None, None)]), None);
        assert_eq!(shared_fee(&[]), None);
    }
}
