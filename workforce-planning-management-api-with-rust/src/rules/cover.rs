//! Pure rules for **backups** (cover): the colleague a worker names to
//! cover for them when they are out sick or on leave. A worker may name a
//! few, ranked (`priority` 1 is the first to ask), each optionally for a
//! dated window; [`resolve`] says who actually covers on a given day —
//! skipping a backup who has left or is on approved leave themselves.
//!
//! DB-free and clock-free: the controller supplies the facts.

use chrono::NaiveDate;
use uuid::Uuid;

/// The most backups one worker can name.
pub const MAX_BACKUPS: usize = 3;

/// Validate a backup designation.
///
/// # Errors
///
/// A message when the worker names themselves, the priority is below 1, or
/// the window ends before it starts.
pub fn validate_backup(
    worker: Uuid,
    backup: Uuid,
    priority: i32,
    starts_on: Option<NaiveDate>,
    ends_on: Option<NaiveDate>,
) -> Result<(), String> {
    if worker == backup {
        return Err("a person cannot be their own backup".to_string());
    }
    if priority < 1 {
        return Err("priority must be 1 or more (1 is the first backup to ask)".to_string());
    }
    if let (Some(start), Some(end)) = (starts_on, ends_on)
        && end < start
    {
        return Err("ends_on must not be before starts_on".to_string());
    }
    Ok(())
}

/// One designated backup, with the facts about them on the day in question.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    /// The backup's worker pid.
    pub backup: Uuid,
    /// Rank: lower is asked first.
    pub priority: i32,
    /// First day the designation applies, when bounded.
    pub starts_on: Option<NaiveDate>,
    /// Last day the designation applies, when bounded.
    pub ends_on: Option<NaiveDate>,
    /// Whether the backup is still employed on the day.
    pub employed: bool,
    /// Whether the backup is on approved leave on the day.
    pub on_leave: bool,
}

/// Whether `candidate`'s window includes `on` (an absent bound is open).
#[must_use]
pub fn in_window(candidate: &Candidate, on: NaiveDate) -> bool {
    candidate.starts_on.is_none_or(|s| s <= on) && candidate.ends_on.is_none_or(|e| on <= e)
}

/// Who covers on `on`: the best-ranked backup whose window includes the
/// day, who is employed and not on leave. Ties go to the lower pid, so the
/// answer is stable. `None` when nobody can cover — the caller shows that
/// plainly rather than guessing.
#[must_use]
pub fn resolve(candidates: &[Candidate], on: NaiveDate) -> Option<Uuid> {
    candidates
        .iter()
        .filter(|c| in_window(c, on) && c.employed && !c.on_leave)
        .min_by_key(|c| (c.priority, c.backup))
        .map(|c| c.backup)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, d).unwrap()
    }

    fn cand(n: u128, priority: i32) -> Candidate {
        Candidate {
            backup: Uuid::from_u128(n),
            priority,
            starts_on: None,
            ends_on: None,
            employed: true,
            on_leave: false,
        }
    }

    #[test]
    fn a_backup_is_valid_only_for_someone_else_with_a_sane_window() {
        let (a, b) = (Uuid::from_u128(1), Uuid::from_u128(2));
        assert!(validate_backup(a, b, 1, Some(day(1)), Some(day(5))).is_ok());
        assert!(validate_backup(a, b, 1, None, None).is_ok());
        assert!(validate_backup(a, a, 1, None, None).is_err());
        assert!(validate_backup(a, b, 0, None, None).is_err());
        assert!(validate_backup(a, b, 1, Some(day(5)), Some(day(1))).is_err());
    }

    #[test]
    fn the_first_ranked_available_backup_covers() {
        let c = [cand(2, 2), cand(1, 1)];
        assert_eq!(resolve(&c, day(10)), Some(Uuid::from_u128(1)));
    }

    #[test]
    fn a_backup_on_leave_or_gone_is_skipped() {
        let mut first = cand(1, 1);
        first.on_leave = true;
        let mut gone = cand(3, 1);
        gone.employed = false;
        let c = [first, gone, cand(2, 2)];
        assert_eq!(resolve(&c, day(10)), Some(Uuid::from_u128(2)));
        let nobody = [first, gone];
        assert_eq!(resolve(&nobody, day(10)), None);
    }

    #[test]
    fn a_dated_backup_covers_only_inside_its_window() {
        let mut dated = cand(1, 1);
        dated.starts_on = Some(day(10));
        dated.ends_on = Some(day(12));
        let standing = cand(2, 2);
        let c = [dated, standing];
        assert_eq!(resolve(&c, day(9)), Some(Uuid::from_u128(2)));
        assert_eq!(resolve(&c, day(10)), Some(Uuid::from_u128(1)));
        assert_eq!(resolve(&c, day(12)), Some(Uuid::from_u128(1)));
        assert_eq!(resolve(&c, day(13)), Some(Uuid::from_u128(2)));
    }

    #[test]
    fn equal_priority_goes_to_the_lower_pid() {
        let c = [cand(9, 1), cand(4, 1)];
        assert_eq!(resolve(&c, day(10)), Some(Uuid::from_u128(4)));
    }

    #[test]
    fn no_backups_means_no_cover() {
        assert_eq!(resolve(&[], day(10)), None);
    }
}
