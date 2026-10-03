//! Org-chart rules (WPM-R7), DB-free: the manager chain must stay a
//! forest — assigning a manager may not create a cycle.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::hash::BuildHasher;
use uuid::Uuid;

/// Whether setting `worker`'s manager to `manager` would create a
/// cycle, given the current `manager_of` map (worker pid → manager
/// pid). Walks up from the proposed manager; hitting `worker` means
/// a cycle. Self-management is a cycle of length one. The walk is
/// bounded by the map size, so a (corrupt) pre-existing cycle
/// elsewhere terminates rather than spinning.
#[must_use]
pub fn would_create_cycle<S: BuildHasher>(
    worker: Uuid,
    manager: Uuid,
    manager_of: &HashMap<Uuid, Uuid, S>,
) -> bool {
    if worker == manager {
        return true;
    }
    let mut current = manager;
    for _ in 0..=manager_of.len() {
        match manager_of.get(&current) {
            Some(&next) if next == worker => return true,
            Some(&next) => current = next,
            None => return false,
        }
    }
    false
}

/// The chain of managers above `worker`, nearest first (`level` 1 is the direct
/// manager), from a `worker → manager` map. Bounded by the map size, so a
/// corrupt cycle ends rather than spins.
#[must_use]
pub fn upline<T: Ord + Copy>(worker: T, manager_of: &BTreeMap<T, T>) -> Vec<T> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::from([worker]);
    let mut current = worker;
    while let Some(manager) = manager_of.get(&current) {
        if !seen.insert(*manager) {
            break;
        }
        out.push(*manager);
        current = *manager;
    }
    out
}

/// Everyone below `manager` with their depth — direct reports at depth 1,
/// their reports at depth 2, and so on — from a `manager → reports` map,
/// breadth first. A cycle cannot revisit anyone.
#[must_use]
pub fn downline<T: Ord + Copy>(manager: T, reports_of: &BTreeMap<T, Vec<T>>) -> Vec<(T, usize)> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::from([manager]);
    let mut queue = VecDeque::from([(manager, 0usize)]);
    while let Some((person, depth)) = queue.pop_front() {
        for report in reports_of.get(&person).into_iter().flatten() {
            if seen.insert(*report) {
                out.push((*report, depth + 1));
                queue.push_back((*report, depth + 1));
            }
        }
    }
    out
}

/// How a report relates to a manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    /// Reports straight to the manager (depth 1).
    Direct,
    /// Reports to the manager through someone else (depth 2 or more).
    Indirect,
}

impl ReportKind {
    /// The token the API reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Indirect => "indirect",
        }
    }
}

/// Classify a downline depth: 1 is a direct report; anything deeper is
/// indirect. Depth 0 is the manager themself, who is neither.
#[must_use]
pub fn report_kind(depth: usize) -> Option<ReportKind> {
    match depth {
        0 => None,
        1 => Some(ReportKind::Direct),
        _ => Some(ReportKind::Indirect),
    }
}

/// The longest accepted work location.
pub const MAX_LOCATION_LEN: usize = 100;

/// Normalise a work-location input: trimmed, blank ⇒ `None` (cleared /
/// unknown), and no longer than [`MAX_LOCATION_LEN`] characters.
///
/// # Errors
/// A message when the trimmed value is too long.
pub fn normalize_location(input: Option<&str>) -> Result<Option<String>, String> {
    let Some(raw) = input.map(str::trim) else {
        return Ok(None);
    };
    if raw.is_empty() {
        return Ok(None);
    }
    if raw.chars().count() > MAX_LOCATION_LEN {
        return Err(format!(
            "location is longer than {MAX_LOCATION_LEN} characters"
        ));
    }
    Ok(Some(raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The management chain upward and the reporting tree downward, with the
    /// direct/indirect distinction; cycles terminate.
    #[test]
    fn upline_downline_and_report_kinds() {
        // 1 (ceo) ← 2 (vp) ← 3 (lead) ← 4, 5 (devs);  1 ← 6 (peer branch)
        let manager_of: BTreeMap<u8, u8> = BTreeMap::from([(2, 1), (3, 2), (4, 3), (5, 3), (6, 1)]);
        assert_eq!(upline(4, &manager_of), [3, 2, 1], "nearest first");
        assert_eq!(upline(1, &manager_of), Vec::<u8>::new(), "the top has none");
        let mut reports: BTreeMap<u8, Vec<u8>> = BTreeMap::new();
        for (w, m) in &manager_of {
            reports.entry(*m).or_default().push(*w);
        }
        let below = downline(2, &reports);
        assert_eq!(
            below,
            [(3, 1), (4, 2), (5, 2)],
            "indirect reports are included, with depth"
        );
        assert!(
            !below.iter().any(|(w, _)| *w == 6),
            "a peer branch is not downline"
        );
        assert_eq!(downline(4, &reports), Vec::<(u8, usize)>::new());
        assert_eq!(report_kind(0), None, "a manager is not their own report");
        assert_eq!(report_kind(1), Some(ReportKind::Direct));
        assert_eq!(report_kind(2), Some(ReportKind::Indirect));
        assert_eq!(report_kind(7), Some(ReportKind::Indirect));
        assert_eq!(ReportKind::Direct.as_str(), "direct");
        // A corrupt cycle terminates and never repeats anyone.
        let cyclic: BTreeMap<u8, u8> = BTreeMap::from([(1, 2), (2, 1)]);
        assert_eq!(upline(1, &cyclic), [2]);
        let cyc: BTreeMap<u8, Vec<u8>> = BTreeMap::from([(1, vec![2]), (2, vec![1])]);
        assert_eq!(downline(1, &cyc), [(2, 1)]);
    }

    /// Locations are trimmed, blank means "unknown", over-long is refused.
    #[test]
    fn location_is_normalised() {
        assert_eq!(normalize_location(None), Ok(None));
        assert_eq!(normalize_location(Some("   ")), Ok(None));
        assert_eq!(
            normalize_location(Some(" Cardiff ")),
            Ok(Some("Cardiff".to_string()))
        );
        assert!(normalize_location(Some(&"x".repeat(100))).is_ok());
        assert!(normalize_location(Some(&"x".repeat(101))).is_err());
    }

    fn u(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    /// Self, direct, and transitive cycles are caught; legal chains and
    /// re-parenting pass.
    #[test]
    fn cycle_detection() {
        // chain: 3 -> 2 -> 1 (1 is the root)
        let mut map = HashMap::new();
        map.insert(u(3), u(2));
        map.insert(u(2), u(1));
        assert!(would_create_cycle(u(5), u(5), &map)); // self
        assert!(would_create_cycle(u(1), u(3), &map)); // root under leaf
        assert!(would_create_cycle(u(1), u(2), &map)); // root under middle
        assert!(!would_create_cycle(u(4), u(3), &map)); // new leaf
        assert!(!would_create_cycle(u(3), u(1), &map)); // re-parent up
    }

    /// A corrupt pre-existing cycle elsewhere terminates (bounded walk)
    /// and does not implicate an unrelated assignment.
    #[test]
    fn bounded_walk_survives_corrupt_data() {
        let mut map = HashMap::new();
        map.insert(u(1), u(2));
        map.insert(u(2), u(1)); // pre-existing corruption
        assert!(!would_create_cycle(u(9), u(1), &map));
    }
}
