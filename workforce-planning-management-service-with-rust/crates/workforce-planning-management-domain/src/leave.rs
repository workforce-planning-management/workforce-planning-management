//! Leave balance arithmetic (WPM-R5), DB-free.
//!
//! Balances are whole **days** per worker / kind / year. Annual
//! leave over the remaining balance is refused; sick leave may go
//! negative but the result is flagged; other kinds behave like
//! annual (balance-enforced) except `unpaid`, which has no balance.

use jiff::civil::Date;

/// The verdict on taking `days` against a balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BalanceCheck {
    /// Within balance — proceed.
    Ok {
        /// Days remaining after the request.
        remaining_after: i32,
    },
    /// Allowed, but the balance goes negative (sick leave) — flag it.
    NegativeFlagged {
        /// The (negative) days remaining after the request.
        remaining_after: i32,
    },
    /// Refused: the request exceeds the remaining balance.
    OverBalance {
        /// Days remaining before the request.
        remaining: i32,
        /// Days requested.
        requested: i32,
    },
}

/// A sick-leave request records that it is sick leave and its dates, never why (WPM-D73): a
/// diagnosis is special-category health data the service has no need to hold. Other kinds may
/// give a reason. A blank reason is no reason.
///
/// # Errors
///
/// A sick-leave request that carries a reason.
pub fn validate_reason(kind: &str, reason: Option<&str>) -> Result<(), String> {
    if !super::absence_category::reason_allowed(kind)
        && reason.is_some_and(|r| !r.trim().is_empty())
    {
        return Err(
            "sick leave records that it is sick leave and its dates, never why: leave the reason out"
                .to_string(),
        );
    }
    Ok(())
}

/// Check taking `requested` days of `kind` leave against
/// `entitled - used`. A category with no balance (`unpaid`, `public_holiday`) always passes.
///
/// # Errors
///
/// A refusal when the balance does not fit an `i32`: a balance nobody recorded is unknown, never a capped number.
pub fn check_balance(
    kind: &str,
    entitled: i32,
    used: i32,
    requested: i32,
) -> Result<BalanceCheck, String> {
    use super::absence_category::{Balance, balance_of};
    let balance = balance_of(kind);
    let remaining = entitled
        .checked_sub(used)
        .ok_or_else(|| BALANCE_TOO_LARGE.to_string())?;
    let remaining_after = remaining
        .checked_sub(requested)
        .ok_or_else(|| BALANCE_TOO_LARGE.to_string())?;
    Ok(if balance == Balance::None || remaining_after >= 0 {
        BalanceCheck::Ok { remaining_after }
    } else if balance == Balance::MayGoNegative {
        BalanceCheck::NegativeFlagged { remaining_after }
    } else {
        BalanceCheck::OverBalance {
            remaining,
            requested,
        }
    })
}

/// The refusal when a leave balance does not fit its type.
const BALANCE_TOO_LARGE: &str = "the leave balance is too large to work out";

/// Inclusive day span of a leave request (`start..=end`), refusing a
/// reversed range or a span over a year.
///
/// # Errors
///
/// A human-readable refusal (`422` material) for a reversed or
/// oversized range.
pub fn day_span(start: Date, end: Date) -> Result<i32, String> {
    if end < start {
        return Err(format!("end_on {end} is before start_on {start}"));
    }
    let days = crate::dates::days_between(start, end)
        .checked_add(1)
        .ok_or_else(|| "leave spans more days than can be counted".to_string())?;
    if days > 366 {
        return Err(format!(
            "leave spans {days} calendar days; the cap is 366 calendar days"
        ));
    }
    i32::try_from(days).map_err(|_| "leave span overflows".to_string())
}

/// Whether two inclusive date ranges overlap (shift-vs-leave and
/// leave-vs-leave conflict checks).
#[must_use]
pub fn ranges_overlap(a_start: Date, a_end: Date, b_start: Date, b_end: Date) -> bool {
    a_start <= b_end && b_start <= a_end
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_sick_leave_request_carries_no_reason() {
        use super::validate_reason;
        assert!(validate_reason("sick", None).is_ok());
        assert!(validate_reason("sick", Some("")).is_ok(), "blank is none");
        assert!(validate_reason("sick", Some("   ")).is_ok());
        assert!(validate_reason("sick", Some("chemotherapy")).is_err());
        assert!(validate_reason("sick", Some(" x ")).is_err());
        for kind in ["annual", "parental", "unpaid", "other"] {
            assert!(validate_reason(kind, Some("holiday")).is_ok(), "{kind}");
        }
    }

    use super::*;

    fn d(y: i32, m: u32, day: u32) -> Date {
        crate::dates::ymd(y, m, day).unwrap()
    }

    /// Annual leave: within balance passes with the new remainder;
    /// over balance is refused with both figures.
    #[test]
    fn annual_balance_is_enforced() {
        assert_eq!(
            check_balance("annual", 25, 20, 5).unwrap(),
            BalanceCheck::Ok { remaining_after: 0 }
        );
        assert_eq!(
            check_balance("annual", 25, 20, 6).unwrap(),
            BalanceCheck::OverBalance {
                remaining: 5,
                requested: 6
            }
        );
    }

    /// Sick leave may go negative but is flagged; within-balance sick
    /// leave is a plain Ok.
    #[test]
    fn sick_negative_is_flagged_not_refused() {
        assert_eq!(
            check_balance("sick", 10, 8, 2).unwrap(),
            BalanceCheck::Ok { remaining_after: 0 }
        );
        assert_eq!(
            check_balance("sick", 10, 8, 5).unwrap(),
            BalanceCheck::NegativeFlagged {
                remaining_after: -3
            }
        );
    }

    /// Unpaid leave has no balance to enforce.
    #[test]
    fn unpaid_always_passes() {
        assert!(matches!(
            check_balance("unpaid", 0, 0, 30).unwrap(),
            BalanceCheck::Ok { .. }
        ));
    }

    /// A balance that does not fit an `i32` is refused, never capped.
    #[test]
    fn a_balance_past_i32_is_refused() {
        let refusal = Err(BALANCE_TOO_LARGE.to_string());
        assert_eq!(check_balance("annual", i32::MIN, 1, 0), refusal);
        assert_eq!(check_balance("unpaid", 0, 0, i32::MIN), refusal);
        assert_eq!(
            check_balance("annual", i32::MAX, -1, 0),
            refusal,
            "the first overflow"
        );
        assert_eq!(
            check_balance("annual", i32::MAX, 0, 0),
            Ok(BalanceCheck::Ok {
                remaining_after: i32::MAX
            })
        );
    }

    /// Day spans are inclusive; reversed and oversized ranges are
    /// refused with readable messages.
    #[test]
    fn day_span_rules() {
        assert_eq!(day_span(d(2026, 8, 3), d(2026, 8, 7)).unwrap(), 5);
        assert_eq!(day_span(d(2026, 8, 3), d(2026, 8, 3)).unwrap(), 1);
        assert!(
            day_span(d(2026, 8, 7), d(2026, 8, 3))
                .unwrap_err()
                .contains("before")
        );
        assert!(
            day_span(d(2026, 1, 1), d(2028, 1, 1))
                .unwrap_err()
                .contains("cap")
        );
    }

    /// Range overlap: touching endpoints overlap; disjoint ranges do
    /// not.
    #[test]
    fn overlap_rules() {
        assert!(ranges_overlap(
            d(2026, 8, 1),
            d(2026, 8, 5),
            d(2026, 8, 5),
            d(2026, 8, 9)
        ));
        assert!(ranges_overlap(
            d(2026, 8, 1),
            d(2026, 8, 9),
            d(2026, 8, 3),
            d(2026, 8, 4)
        ));
        assert!(!ranges_overlap(
            d(2026, 8, 1),
            d(2026, 8, 4),
            d(2026, 8, 5),
            d(2026, 8, 9)
        ));
    }
}
