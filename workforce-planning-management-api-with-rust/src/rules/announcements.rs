//! Pure rules for the **announcement feed**: what makes a post valid, when it
//! is visible, and how the feed is ordered. DB-free and clock-free (the day is
//! always supplied).
//!
//! Posts are plain text — the UI shows them with line breaks and never as
//! markup, so a post can never carry script or styling.

use chrono::NaiveDate;
use std::cmp::Ordering;

/// Longest title.
pub const MAX_TITLE: usize = 200;
/// Longest body.
pub const MAX_BODY: usize = 5000;

/// Roles that may post, edit and see scheduled or expired announcements.
pub const EDITOR_ROLES: &[&str] = &["hr_admin", "org_admin"];

/// Validate a post.
///
/// # Errors
///
/// A message naming the first problem: a blank or over-long title or body, or
/// an expiry before the publish day.
pub fn validate(
    title: &str,
    body: &str,
    publish_on: NaiveDate,
    expires_on: Option<NaiveDate>,
) -> Result<(), String> {
    if title.trim().is_empty() || title.chars().count() > MAX_TITLE {
        return Err(format!("title is required (up to {MAX_TITLE} characters)"));
    }
    if body.trim().is_empty() || body.chars().count() > MAX_BODY {
        return Err(format!("body is required (up to {MAX_BODY} characters)"));
    }
    if expires_on.is_some_and(|e| e < publish_on) {
        return Err("expires_on must not be before publish_on".to_string());
    }
    Ok(())
}

/// Where a post stands on a given day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Not yet published (`publish_on` is after the day).
    Scheduled,
    /// Published and not expired.
    Live,
    /// Past its `expires_on`.
    Expired,
}

/// A post's status on `today`: live from `publish_on` through `expires_on`
/// inclusive.
#[must_use]
pub fn status(publish_on: NaiveDate, expires_on: Option<NaiveDate>, today: NaiveDate) -> Status {
    if publish_on > today {
        Status::Scheduled
    } else if expires_on.is_some_and(|e| e < today) {
        Status::Expired
    } else {
        Status::Live
    }
}

/// The order of the feed: pinned posts first, then newest published first,
/// then newest created (higher `id`) first.
#[must_use]
pub fn feed_order(
    a: (bool, NaiveDate, i32),
    b: (bool, NaiveDate, i32),
) -> Ordering {
    b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(b.2.cmp(&a.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    #[test]
    fn a_post_needs_a_title_a_body_and_a_sane_window() {
        assert!(validate("Office closed", "Monday is a holiday.", d(5), None).is_ok());
        assert!(validate("  ", "x", d(5), None).is_err());
        assert!(validate("x", "", d(5), None).is_err());
        assert!(validate(&"t".repeat(201), "x", d(5), None).is_err());
        assert!(validate("x", &"b".repeat(5001), d(5), None).is_err());
        assert!(validate("x", "y", d(5), Some(d(4))).is_err());
        assert!(validate("x", "y", d(5), Some(d(5))).is_ok(), "same day is fine");
    }

    #[test]
    fn a_post_is_live_from_publish_day_through_expiry_day() {
        assert_eq!(status(d(6), None, d(5)), Status::Scheduled);
        assert_eq!(status(d(5), None, d(5)), Status::Live);
        assert_eq!(status(d(1), Some(d(5)), d(5)), Status::Live, "expiry day inclusive");
        assert_eq!(status(d(1), Some(d(4)), d(5)), Status::Expired);
        assert_eq!(status(d(1), None, d(30)), Status::Live, "no expiry");
    }

    #[test]
    fn the_feed_is_pinned_first_then_newest() {
        let mut items = [
            (false, d(3), 1),
            (true, d(1), 2),
            (false, d(4), 3),
            (false, d(4), 4),
            (true, d(2), 5),
        ];
        items.sort_by(|a, b| feed_order(*a, *b));
        let ids: Vec<i32> = items.iter().map(|i| i.2).collect();
        assert_eq!(ids, [5, 2, 4, 3, 1]);
    }
}
