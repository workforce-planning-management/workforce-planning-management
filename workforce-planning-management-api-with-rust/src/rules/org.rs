//! Org-chart rules (WPM-R7), DB-free: the manager chain must stay a
//! forest — assigning a manager may not create a cycle.

use std::collections::HashMap;
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
        return Err(format!("location is longer than {MAX_LOCATION_LEN} characters"));
    }
    Ok(Some(raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Locations are trimmed, blank means "unknown", over-long is refused.
    #[test]
    fn location_is_normalised() {
        assert_eq!(normalize_location(None), Ok(None));
        assert_eq!(normalize_location(Some("   ")), Ok(None));
        assert_eq!(normalize_location(Some(" Cardiff ")), Ok(Some("Cardiff".to_string())));
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
