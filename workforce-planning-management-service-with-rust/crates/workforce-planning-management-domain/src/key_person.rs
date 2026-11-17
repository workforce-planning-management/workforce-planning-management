//! **Key-person risk** (WPM-R61, WPM-D47): for a critical skill, how many people hold it at the
//! minimum proficiency, and whether anyone could cover for them.
//!
//! Pure and clock-free. The register exists to fund backups and succession, not to judge anyone
//! (WPM-D47), so the answer is a count and a status; the named list is a separate, HR-only read.

use std::collections::BTreeSet;
use uuid::Uuid;

/// One person who holds the skill at the minimum, with the people who could cover for them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder {
    /// The holder (never leaves the pure core as a name).
    pub id: Uuid,
    /// Employed colleagues who are their named backups **and** hold the skill at the minimum.
    pub backups: BTreeSet<Uuid>,
    /// People ranked `ready_now` for a succession plan whose incumbent is this holder.
    pub ready_successors: BTreeSet<Uuid>,
}

/// Where a critical skill stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Nobody employed holds it at the minimum.
    NoHolder,
    /// One holder, no qualified backup and no ready successor.
    SinglePointOfFailure,
    /// One holder, but someone could cover.
    CoveredSingleHolder,
    /// Two or more holders.
    Shared,
}

impl Status {
    /// The token the API returns.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoHolder => "no_holder",
            Self::SinglePointOfFailure => "single_point_of_failure",
            Self::CoveredSingleHolder => "covered_single_holder",
            Self::Shared => "shared",
        }
    }
}

/// The count of holders, distinct backups and distinct ready successors, and the status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    /// Employed holders at the minimum proficiency.
    pub holders: usize,
    /// Distinct qualified backups across the holders.
    pub backups: usize,
    /// Distinct ready successors across the holders.
    pub ready_successors: usize,
    /// The status.
    pub status: Status,
}

/// Assess one skill in one pool from its holders.
#[must_use]
pub fn assess(holders: &[Holder]) -> Assessment {
    let backups: BTreeSet<Uuid> = holders
        .iter()
        .flat_map(|h| h.backups.iter().copied())
        .collect();
    let successors: BTreeSet<Uuid> = holders
        .iter()
        .flat_map(|h| h.ready_successors.iter().copied())
        .collect();
    let status = match holders.len() {
        0 => Status::NoHolder,
        1 if backups.is_empty() && successors.is_empty() => Status::SinglePointOfFailure,
        1 => Status::CoveredSingleHolder,
        _ => Status::Shared,
    };
    Assessment {
        holders: holders.len(),
        backups: backups.len(),
        ready_successors: successors.len(),
        status,
    }
}

/// The proficiency a skill needs to count as held when several critical requirements name it:
/// the highest, because a holder below it misses at least one critical need.
#[must_use]
pub fn bar(minimums: &[i32]) -> Option<i32> {
    minimums.iter().copied().max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn holder(n: u128, backups: &[u128], successors: &[u128]) -> Holder {
        Holder {
            id: id(n),
            backups: backups.iter().map(|b| id(*b)).collect(),
            ready_successors: successors.iter().map(|s| id(*s)).collect(),
        }
    }

    #[test]
    fn nobody_holding_it_is_not_a_single_point_of_failure() {
        let a = assess(&[]);
        assert_eq!(a.status, Status::NoHolder);
        assert_eq!((a.holders, a.backups, a.ready_successors), (0, 0, 0));
    }

    #[test]
    fn one_holder_with_no_backup_and_no_successor_is_a_single_point_of_failure() {
        let a = assess(&[holder(1, &[], &[])]);
        assert_eq!(a.status, Status::SinglePointOfFailure);
        assert_eq!(a.status.as_str(), "single_point_of_failure");
    }

    #[test]
    fn a_backup_or_a_ready_successor_each_cover_a_single_holder() {
        assert_eq!(
            assess(&[holder(1, &[2], &[])]).status,
            Status::CoveredSingleHolder
        );
        assert_eq!(
            assess(&[holder(1, &[], &[3])]).status,
            Status::CoveredSingleHolder
        );
    }

    #[test]
    fn two_holders_share_the_skill_even_with_no_backups() {
        let a = assess(&[holder(1, &[], &[]), holder(2, &[], &[])]);
        assert_eq!(a.status, Status::Shared);
        assert_eq!(a.holders, 2);
    }

    #[test]
    fn a_person_who_backs_up_two_holders_is_counted_once() {
        let a = assess(&[holder(1, &[9], &[7]), holder(2, &[9], &[7])]);
        assert_eq!((a.backups, a.ready_successors), (1, 1));
    }

    #[test]
    fn the_bar_is_the_highest_critical_minimum() {
        assert_eq!(bar(&[]), None);
        assert_eq!(bar(&[2, 4, 3]), Some(4));
    }
}
