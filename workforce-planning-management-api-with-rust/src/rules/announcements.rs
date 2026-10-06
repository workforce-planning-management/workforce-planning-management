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

/// Most links one post can carry.
pub const MAX_LINKS: usize = 3;

/// Validate a post's link attachments: at most [`MAX_LINKS`], each with a
/// label (1–100 characters) and an **https** address (up to 500 characters,
/// no whitespace). Nothing else is accepted — no `javascript:`, `data:` or
/// plain `http:` — so a link can never be a script or a downgrade.
///
/// # Errors
///
/// A message naming the first problem.
pub fn validate_links(links: &[(String, String)]) -> Result<(), String> {
    if links.len() > MAX_LINKS {
        return Err(format!("at most {MAX_LINKS} links"));
    }
    for (label, url) in links {
        if label.trim().is_empty() || label.chars().count() > 100 {
            return Err("each link needs a label (up to 100 characters)".to_string());
        }
        let host_ok = url
            .strip_prefix("https://")
            .is_some_and(|rest| rest.split('/').next().is_some_and(|h| h.contains('.')));
        if !host_ok || url.chars().count() > 500 || url.chars().any(char::is_whitespace) {
            return Err("each link must be an https:// address with no spaces".to_string());
        }
    }
    Ok(())
}

/// Whether a post aimed at `audience` (a department, or `None` for everyone)
/// is for a reader in `reader_departments` — or for an editor, who sees all.
/// Department names compare ignoring ASCII case.
#[must_use]
pub fn audience_includes(
    audience: Option<&str>,
    reader_departments: &[String],
    is_editor: bool,
) -> bool {
    match audience {
        None => true,
        Some(_) if is_editor => true,
        Some(dept) => reader_departments
            .iter()
            .any(|d| d.eq_ignore_ascii_case(dept)),
    }
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
pub fn feed_order(a: (bool, NaiveDate, i32), b: (bool, NaiveDate, i32)) -> Ordering {
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
        assert!(
            validate("x", "y", d(5), Some(d(5))).is_ok(),
            "same day is fine"
        );
    }

    #[test]
    fn links_are_few_labelled_and_https_only() {
        let ok = |l: &[(&str, &str)]| {
            validate_links(
                &l.iter()
                    .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
                    .collect::<Vec<_>>(),
            )
        };
        assert!(ok(&[("Handbook", "https://intranet.example.org/handbook")]).is_ok());
        assert!(ok(&[]).is_ok());
        assert!(
            ok(&[
                ("a", "https://a.io"),
                ("b", "https://b.io"),
                ("c", "https://c.io"),
                ("d", "https://d.io")
            ])
            .is_err()
        );
        for bad in [
            "http://example.org",
            "javascript:alert(1)",
            "data:text/html,<script>",
            "https://",
            "https://nodot",
            "https://exa mple.org",
            "//example.org",
        ] {
            assert!(ok(&[("x", bad)]).is_err(), "{bad}");
        }
        assert!(ok(&[(" ", "https://example.org")]).is_err(), "blank label");
    }

    #[test]
    fn a_department_post_is_for_that_department_and_editors() {
        let mine = vec!["Finance".to_string()];
        assert!(audience_includes(None, &[], false), "everyone");
        assert!(
            audience_includes(Some("finance"), &mine, false),
            "case-insensitive"
        );
        assert!(!audience_includes(Some("Engineering"), &mine, false));
        assert!(
            audience_includes(Some("Engineering"), &mine, true),
            "editors see all"
        );
        assert!(
            !audience_includes(Some("Engineering"), &[], false),
            "no known department"
        );
    }

    #[test]
    fn a_post_is_live_from_publish_day_through_expiry_day() {
        assert_eq!(status(d(6), None, d(5)), Status::Scheduled);
        assert_eq!(status(d(5), None, d(5)), Status::Live);
        assert_eq!(
            status(d(1), Some(d(5)), d(5)),
            Status::Live,
            "expiry day inclusive"
        );
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
