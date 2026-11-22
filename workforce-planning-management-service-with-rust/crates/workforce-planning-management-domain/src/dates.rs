//! Small helpers over `jiff` for the idioms the service used with `chrono` (WPM-T528).
//!
//! The service's instants are [`Timestamp`]s and its days are [`Date`]s, both in UTC. These helpers keep each call site
//! one line: today's date, a date from three integers as `chrono`'s `from_ymd_opt` took them, the whole days between two
//! dates, a timestamp a number of days on (a `Timestamp` takes no calendar units, so whole days are 24-hour durations),
//! and RFC 3339 to the second.

use jiff::tz::TimeZone;
pub use jiff::{
    SignedDuration, Span, Timestamp, ToSpan,
    civil::{Date, Weekday},
};

/// Today's date in UTC.
#[must_use]
pub fn today() -> Date {
    date_of(Timestamp::now())
}

/// The UTC calendar date of an instant (a [`Timestamp`] or a row's
/// [`SqlTimestamp`](crate::sql_time::SqlTimestamp)), as `chrono`'s `date_naive`.
#[must_use]
pub fn date_of(at: impl Into<Timestamp>) -> Date {
    at.into().to_zoned(TimeZone::UTC).date()
}

/// The date `year-month-day`, or `None` when there is no such day (as `chrono`'s `NaiveDate::from_ymd_opt`).
#[must_use]
pub fn ymd(year: i32, month: u32, day: u32) -> Option<Date> {
    let year = i16::try_from(year).ok()?;
    let month = i8::try_from(month).ok()?;
    let day = i8::try_from(day).ok()?;
    Date::new(year, month, day).ok()
}

/// The instant midnight UTC begins `date`.
///
/// Every date `jiff` can hold has a midnight in UTC, so the fallback (the nearer end of the range) is never taken.
#[must_use]
pub fn start_of_day(date: Date) -> Timestamp {
    date.to_zoned(TimeZone::UTC).map_or_else(
        |_| {
            if date.year() < 0 {
                Timestamp::MIN
            } else {
                Timestamp::MAX
            }
        },
        |zoned| zoned.timestamp(),
    )
}

/// The instant `year-month-day hour:minute:second` UTC, or `None` when it does not exist.
#[must_use]
pub fn utc(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<Timestamp> {
    let date = ymd(year, month, day)?;
    let time = jiff::civil::Time::new(
        i8::try_from(hour).ok()?,
        i8::try_from(minute).ok()?,
        i8::try_from(second).ok()?,
        0,
    )
    .ok()?;
    date.to_datetime(time)
        .to_zoned(TimeZone::UTC)
        .ok()
        .map(|z| z.timestamp())
}

/// Whole days from `earlier` to `later`: negative when `later` is before `earlier` (as `chrono`'s
/// `(later - earlier).num_days()`).
#[must_use]
pub fn days_between(earlier: Date, later: Date) -> i64 {
    // A civil duration is a whole number of days, so the hours divide exactly by 24.
    later.duration_since(earlier).as_hours() / 24
}

/// A duration of whole 24-hour days, for adding to a [`Timestamp`] (as `chrono`'s `Duration::days`).
#[must_use]
pub fn days(n: i64) -> SignedDuration {
    // `SignedDuration` holds seconds in an `i64`, so more days than that is no duration at all. No date is that far from any
    // other (a `Date` spans some 3.7 million days), so the bound changes no sum with a date, which fails either way.
    const MOST_SECS: i64 = i64::MAX / 86_400 * 86_400;
    const LEAST_SECS: i64 = -MOST_SECS;
    let secs = n
        .checked_mul(86_400)
        .unwrap_or(if n < 0 { LEAST_SECS } else { MOST_SECS });
    SignedDuration::from_secs(secs)
}

/// RFC 3339 to the whole second, in UTC with a `Z`, the fraction cut off and not rounded (as `chrono`'s
/// `to_rfc3339_opts(SecondsFormat::Secs, true)`).
#[must_use]
pub fn rfc3339_seconds(at: Timestamp) -> String {
    at.strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_is_the_current_utc_date() {
        let before = date_of(Timestamp::now());
        let actual = today();
        let after = date_of(Timestamp::now());
        assert!(actual == before || actual == after);
    }

    #[test]
    fn ymd_refuses_a_day_that_does_not_exist() {
        assert_eq!(ymd(2026, 2, 28), Some(jiff::civil::date(2026, 2, 28)));
        assert_eq!(ymd(2026, 2, 29), None);
        assert_eq!(ymd(2024, 2, 29), Some(jiff::civil::date(2024, 2, 29)));
        assert_eq!(ymd(2026, 13, 1), None);
        assert_eq!(ymd(100_000, 1, 1), None);
    }

    #[test]
    fn a_huge_number_of_days_is_a_duration_no_date_can_take_not_a_panic() {
        assert!(Date::MIN.checked_add(days(i64::MAX)).is_err());
        assert!(Date::MAX.checked_sub(days(i64::MAX)).is_err());
        assert!(Date::MAX.checked_add(days(i64::MIN)).is_err());
        assert_eq!(days(2).as_hours(), 48);
        let most = i64::MAX / 86_400;
        assert_eq!(days(most).as_secs(), most * 86_400);
        assert_eq!(days(-most).as_secs(), -most * 86_400);
        assert_eq!(days(most + 1).as_secs(), most * 86_400);
        assert_eq!(days(i64::MAX).as_secs(), most * 86_400);
        assert_eq!(days(i64::MIN).as_secs(), -most * 86_400);
    }

    #[test]
    fn days_between_is_signed_whole_days() {
        let a = jiff::civil::date(2026, 1, 31);
        let b = jiff::civil::date(2026, 3, 1);
        assert_eq!(days_between(a, b), 29);
        assert_eq!(days_between(b, a), -29);
        assert_eq!(days_between(a, a), 0);
    }

    #[test]
    fn instants_and_days_in_utc() {
        let at = utc(2026, 11, 17, 23, 59, 59).expect("an instant");
        assert_eq!(date_of(at), jiff::civil::date(2026, 11, 17));
        assert_eq!(date_of(at + days(1)), jiff::civil::date(2026, 11, 18));
        assert_eq!(
            start_of_day(jiff::civil::date(2026, 11, 17)).to_string(),
            "2026-11-17T00:00:00Z"
        );
        assert_eq!(rfc3339_seconds(at), "2026-11-17T23:59:59Z");
        let fractional: Timestamp = "2026-11-17T09:30:00.4Z".parse().expect("an instant");
        assert_eq!(rfc3339_seconds(fractional), "2026-11-17T09:30:00Z");
        let late: Timestamp = "2026-11-17T09:30:00.9Z".parse().expect("an instant");
        assert_eq!(
            rfc3339_seconds(late),
            "2026-11-17T09:30:00Z",
            "cut off, not rounded"
        );
    }

    #[test]
    fn adding_a_month_keeps_to_the_end_of_a_short_month() {
        // As `chrono`'s `checked_add_months`: 31 January plus a month is 28 February.
        let d = jiff::civil::date(2026, 1, 31);
        assert_eq!(
            d.checked_add(1.month()).ok(),
            Some(jiff::civil::date(2026, 2, 28))
        );
    }
}
