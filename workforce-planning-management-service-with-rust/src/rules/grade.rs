//! Grades (WPM-R53), DB-free: validating that a job level and a pay band named
//! by a client exist on the reference ladders, and that a level held "since" a
//! date is not held since the future.
//!
//! A level and a pay band are **recorded facts about a person or a role**, set by
//! someone entitled to say so. Nothing here derives one from the other: no source
//! says which job level corresponds to which pay band, so the service does not
//! pretend to know.

use chrono::NaiveDate;

use super::job_levels as levels;
use super::pay_scale as scales;

/// A job level resolved against its ladder: the canonical framework id and the
/// level number.
///
/// # Errors
/// A message when the framework or the level is not on the ladder.
pub fn resolve_level(framework: &str, level: &str) -> Result<(&'static str, u8), String> {
    let found = levels::find(framework.trim())
        .ok_or_else(|| format!("unknown job-level framework {:?}", framework.trim()))?;
    let wanted = found
        .level(level)
        .ok_or_else(|| format!("{} has no level {:?}", found.id, level.trim()))?;
    Ok((found.id, wanted.number))
}

/// A pay band resolved against its scale: the canonical scale id and band code.
///
/// # Errors
/// A message when the scale or the band does not exist.
pub fn resolve_band(scale: &str, band: &str) -> Result<(&'static str, &'static str), String> {
    let found = scales::find(scale.trim())
        .ok_or_else(|| format!("unknown pay scale {:?}", scale.trim()))?;
    let wanted = found
        .band(band.trim())
        .ok_or_else(|| format!("{} has no band {:?}", found.id, band.trim()))?;
    Ok((found.id, wanted.code))
}

/// A level is held *since* a date, so that date cannot be after `today`.
///
/// # Errors
/// A message when `effective_on` is in the future.
pub fn validate_effective_on(effective_on: NaiveDate, today: NaiveDate) -> Result<(), String> {
    if effective_on > today {
        Err("effective_on cannot be in the future".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    /// A level resolves from any accepted spelling to the canonical id and number;
    /// an unknown framework or level is named in the message.
    #[test]
    fn levels_resolve_or_say_why_not() {
        assert_eq!(
            resolve_level("google-levels", "L5"),
            Ok(("google-levels", 5))
        );
        assert_eq!(
            resolve_level(" google-levels ", "l11"),
            Ok(("google-levels", 11))
        );
        assert_eq!(
            resolve_level("google-levels", "7"),
            Ok(("google-levels", 7))
        );
        assert!(
            resolve_level("meta-levels", "L5")
                .unwrap_err()
                .contains("meta-levels")
        );
        assert!(
            resolve_level("google-levels", "L2")
                .unwrap_err()
                .contains("L2")
        );
        assert!(resolve_level("google-levels", "").is_err());
    }

    #[test]
    fn bands_resolve_or_say_why_not() {
        assert_eq!(
            resolve_band("national-2026-27", "8A"),
            Ok(("national-2026-27", "8a"))
        );
        assert!(resolve_band("national-2025-26", "5").is_err());
        assert!(
            resolve_band("national-2026-27", "10")
                .unwrap_err()
                .contains("10")
        );
    }

    /// Today is allowed; the future is not.
    #[test]
    fn a_level_is_not_held_since_the_future() {
        assert!(validate_effective_on(d(2026, 10, 6), d(2026, 10, 6)).is_ok());
        assert!(validate_effective_on(d(2020, 1, 1), d(2026, 10, 6)).is_ok());
        assert!(validate_effective_on(d(2026, 10, 7), d(2026, 10, 6)).is_err());
    }
}
