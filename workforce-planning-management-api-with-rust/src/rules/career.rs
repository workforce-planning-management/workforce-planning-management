//! Pure rules for **career history and aspirations**: roles and skill levels
//! as intervals over time (past), and aspirations / learning goals / growth
//! ideas (future).
//!
//! DB-free and clock-free (instants are always supplied). Intervals are
//! half-open `[start, end)`: a role that ends at noon and the next that
//! starts at noon do not overlap, and a level read "at noon" is the new one.

use chrono::{DateTime, Utc};

/// A span of time; `end = None` means still current.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    /// When it began.
    pub start: DateTime<Utc>,
    /// When it ended; `None` while current.
    pub end: Option<DateTime<Utc>>,
}

impl Interval {
    /// Whether `at` falls inside: `start <= at < end`.
    #[must_use]
    pub fn contains(&self, at: DateTime<Utc>) -> bool {
        self.start <= at && self.end.is_none_or(|end| at < end)
    }

    /// Whether two intervals share any time.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        let before = |a: &Self, b: &Self| a.end.is_some_and(|end| end <= b.start);
        !(before(self, other) || before(other, self))
    }
}

/// Validate a **past** entry added retrospectively: it must have ended, start
/// before it ended, and not end in the future.
///
/// # Errors
/// A message naming the problem.
pub fn validate_past(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Interval, String> {
    if end <= start {
        return Err("it must end after it starts".to_string());
    }
    if end > now {
        return Err("a past entry cannot end in the future".to_string());
    }
    Ok(Interval {
        start,
        end: Some(end),
    })
}

/// Index of the first existing interval that overlaps `new`, if any.
#[must_use]
pub fn first_overlap(new: &Interval, existing: &[Interval]) -> Option<usize> {
    existing.iter().position(|e| e.overlaps(new))
}

/// The level held at `at` from `(interval, level)` history: the entry whose
/// interval contains it, or `None`.
#[must_use]
pub fn level_at(history: &[(Interval, i32)], at: DateTime<Utc>) -> Option<i32> {
    history
        .iter()
        .find(|(interval, _)| interval.contains(at))
        .map(|(_, level)| *level)
}

/// Whether the caller **is** the person (`sub` is the UUID inside a
/// `person:<uuid>` reference). Used to tell a person acting for themselves
/// from someone acting on their behalf.
#[must_use]
pub fn is_self(sub: &str, person_ref: &str) -> bool {
    person_ref
        .strip_prefix("person:")
        .is_some_and(|id| id.eq_ignore_ascii_case(sub.trim()))
}

/// What an aspiration is about.
pub const KINDS: &[&str] = &["role", "skill"];

/// Where an aspiration stands.
pub const STATUSES: &[&str] = &["idea", "planned", "in_progress", "achieved", "dropped"];

/// How far ahead it looks.
pub const HORIZONS: &[&str] = &["within_1y", "one_to_three_years", "beyond", "someday"];

/// The longest accepted growth note.
pub const MAX_NOTE_LEN: usize = 1000;

/// Validate an aspiration: known tokens, a 1–5 target for a skill, a bounded
/// note.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_aspiration(
    kind: &str,
    target_level: Option<i32>,
    horizon: &str,
    status: &str,
    note: Option<&str>,
) -> Result<(), String> {
    if !KINDS.contains(&kind) {
        return Err(format!("kind must be one of {}", KINDS.join(", ")));
    }
    if !HORIZONS.contains(&horizon) {
        return Err(format!("horizon must be one of {}", HORIZONS.join(", ")));
    }
    if !STATUSES.contains(&status) {
        return Err(format!("status must be one of {}", STATUSES.join(", ")));
    }
    match (kind, target_level) {
        ("skill", None) => return Err("a skill aspiration needs a target_level".to_string()),
        ("skill", Some(level)) if !crate::rules::learning::valid_proficiency(level) => {
            return Err("target_level must be between 1 and 5".to_string());
        }
        ("role", Some(_)) => return Err("a role aspiration has no target_level".to_string()),
        _ => {}
    }
    if note.is_some_and(|n| n.chars().count() > MAX_NOTE_LEN) {
        return Err(format!("note is longer than {MAX_NOTE_LEN} characters"));
    }
    Ok(())
}

/// Validate a visibility token.
///
/// # Errors
/// A message listing the choices.
pub fn validate_visibility(visibility: &str) -> Result<(), String> {
    if VISIBILITIES.contains(&visibility) {
        Ok(())
    } else {
        Err(format!(
            "visibility must be one of {}",
            VISIBILITIES.join(", ")
        ))
    }
}

/// Who may see an aspiration: just the person, their management chain, or
/// anyone who can view the record.
pub const VISIBILITIES: &[&str] = &["private", "manager", "everyone"];

/// How a viewer relates to the person whose aspirations they are reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewer {
    /// The person themselves.
    Person,
    /// Someone above the person in their management chain.
    Manager,
    /// Anyone else who can view the record.
    Other,
}

/// Whether `viewer` may see an aspiration with this `visibility`: the person
/// sees all of theirs; a manager sees those shared with managers (or
/// everyone); anyone else only those shared with everyone. An unknown token is
/// treated as private.
#[must_use]
pub fn can_view(viewer: Viewer, visibility: &str) -> bool {
    matches!(
        (viewer, visibility),
        (Viewer::Person, _)
            | (Viewer::Manager, "manager" | "everyone")
            | (Viewer::Other, "everyone")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap()
    }

    #[test]
    fn intervals_are_half_open() {
        let a = Interval {
            start: at(2024, 1, 1),
            end: Some(at(2025, 1, 1)),
        };
        let b = Interval {
            start: at(2025, 1, 1),
            end: None,
        };
        assert!(
            !a.overlaps(&b) && !b.overlaps(&a),
            "back-to-back is not an overlap"
        );
        assert!(a.contains(at(2024, 6, 1)) && !a.contains(at(2025, 1, 1)));
        assert!(
            b.contains(at(2025, 1, 1)) && b.contains(at(2030, 1, 1)),
            "open-ended"
        );
        let c = Interval {
            start: at(2024, 12, 1),
            end: Some(at(2025, 3, 1)),
        };
        assert!(a.overlaps(&c) && b.overlaps(&c));
    }

    #[test]
    fn past_entries_are_validated() {
        let now = at(2026, 10, 3);
        assert!(validate_past(at(2020, 1, 1), at(2022, 1, 1), now).is_ok());
        assert!(
            validate_past(at(2022, 1, 1), at(2022, 1, 1), now).is_err(),
            "zero length"
        );
        assert!(
            validate_past(at(2023, 1, 1), at(2022, 1, 1), now).is_err(),
            "backwards"
        );
        assert!(
            validate_past(at(2025, 1, 1), at(2027, 1, 1), now)
                .unwrap_err()
                .contains("future")
        );
    }

    #[test]
    fn overlap_and_level_at() {
        let existing = [
            Interval {
                start: at(2020, 1, 1),
                end: Some(at(2022, 1, 1)),
            },
            Interval {
                start: at(2022, 1, 1),
                end: None,
            },
        ];
        assert_eq!(
            first_overlap(
                &Interval {
                    start: at(2019, 1, 1),
                    end: Some(at(2020, 1, 1))
                },
                &existing
            ),
            None
        );
        assert_eq!(
            first_overlap(
                &Interval {
                    start: at(2021, 1, 1),
                    end: Some(at(2021, 6, 1))
                },
                &existing
            ),
            Some(0)
        );
        let history = [(existing[0], 2), (existing[1], 4)];
        assert_eq!(level_at(&history, at(2021, 1, 1)), Some(2));
        assert_eq!(
            level_at(&history, at(2022, 1, 1)),
            Some(4),
            "the boundary belongs to the new one"
        );
        assert_eq!(level_at(&history, at(2019, 1, 1)), None);
    }

    #[test]
    fn who_is_the_person() {
        assert!(is_self(
            "11111111-1111-1111-1111-111111111111",
            "person:11111111-1111-1111-1111-111111111111"
        ));
        assert!(is_self(" ABC ", "person:abc"));
        assert!(!is_self("abc", "person:def"));
        assert!(!is_self("abc", "worker:abc"), "only a person reference");
    }

    #[test]
    fn aspirations_are_validated() {
        assert!(
            validate_aspiration("role", None, "within_1y", "idea", Some("lead a team")).is_ok()
        );
        assert!(validate_aspiration("skill", Some(4), "someday", "planned", None).is_ok());
        assert!(validate_aspiration("skill", None, "within_1y", "idea", None).is_err());
        assert!(validate_aspiration("skill", Some(6), "within_1y", "idea", None).is_err());
        assert!(validate_aspiration("role", Some(3), "within_1y", "idea", None).is_err());
        assert!(validate_aspiration("dream", None, "within_1y", "idea", None).is_err());
        assert!(validate_aspiration("role", None, "tomorrow", "idea", None).is_err());
        assert!(validate_aspiration("role", None, "within_1y", "wishing", None).is_err());
        assert!(
            validate_aspiration("role", None, "within_1y", "idea", Some(&"x".repeat(1001)))
                .is_err()
        );
    }

    #[test]
    fn privacy_of_aspirations() {
        for v in VISIBILITIES {
            assert!(
                can_view(Viewer::Person, v),
                "the person sees all of theirs ({v})"
            );
        }
        assert!(
            !can_view(Viewer::Manager, "private"),
            "a manager never sees private"
        );
        assert!(can_view(Viewer::Manager, "manager") && can_view(Viewer::Manager, "everyone"));
        assert!(!can_view(Viewer::Other, "private") && !can_view(Viewer::Other, "manager"));
        assert!(can_view(Viewer::Other, "everyone"));
        assert!(
            !can_view(Viewer::Manager, "???") && !can_view(Viewer::Other, "???"),
            "unknown is private"
        );
    }

    #[test]
    fn visibility_is_validated() {
        assert!(validate_visibility("manager").is_ok());
        assert!(validate_visibility("friends").is_err());
    }
}
