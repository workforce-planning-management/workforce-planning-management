//! How many workers one request may load (WPM-R120, WPM-D90).
//!
//! The plan forecast and cost views count the workers of the departments a plan names in the database
//! (WPM-D95), so their cost no longer grows with the organization as fast as it did, but it does grow.
//! The count is checked first and a request over the ceiling is refused with a message, not left to run
//! for seconds.
//!
//! The default ceiling is the supported size, 200,000 workers (WPM-R142), the largest organization the load
//! test ran with no failed request at eight concurrent callers (`BENCHMARKS.md`): nothing is accepted that
//! was not measured. A deployer who measures a larger organization on their own hardware raises it with
//! `WORKFORCE_PLANNING_MANAGEMENT_MAX_ORG_WORKERS`.

/// The largest worker count a whole-organization view accepts by default.
pub const DEFAULT_MAX_ORG_WORKERS: u64 = 200_000;

/// The ceiling from a setting: the default when the setting is absent (RCA-IN-3).
///
/// # Errors
///
/// A setting that is supplied but is not a whole number above zero (RCA-IN-2, RCA-IN-4): it is never replaced by the default.
pub fn parse_ceiling(setting: Option<&str>) -> Result<u64, String> {
    let Some(text) = setting else {
        return Ok(DEFAULT_MAX_ORG_WORKERS); // WPM-D90: absent means the measured ceiling
    };
    match text.trim().parse::<u64>() {
        Ok(ceiling) if ceiling > 0 => Ok(ceiling),
        _ => Err(format!(
            "WORKFORCE_PLANNING_MANAGEMENT_MAX_ORG_WORKERS must be a whole number above zero, not {text:?}"
        )),
    }
}

/// Whether `count` workers may be loaded under `ceiling`; otherwise the refusal to show the caller.
///
/// # Errors
///
/// A message naming the count, the ceiling and what to do, when `count` is above `ceiling`.
pub fn check(count: u64, ceiling: u64) -> Result<(), String> {
    if count <= ceiling {
        return Ok(());
    }
    Err(format!(
        "this view reads {count} workers at once, over the limit of {ceiling}; narrow it to one organization or raise WORKFORCE_PLANNING_MANAGEMENT_MAX_ORG_WORKERS after measuring"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ceiling_itself_is_allowed() {
        assert_eq!(check(10_000, 10_000), Ok(()));
        assert_eq!(check(0, 10_000), Ok(()));
    }

    #[test]
    fn one_over_the_ceiling_is_refused_with_both_numbers() {
        let message = check(10_001, 10_000).unwrap_err();
        assert!(message.contains("10001") && message.contains("10000"));
    }

    #[test]
    fn a_setting_that_is_not_a_positive_number_is_refused_and_an_absent_one_is_the_default() {
        assert_eq!(parse_ceiling(None), Ok(DEFAULT_MAX_ORG_WORKERS));
        for bad in [Some(""), Some("  "), Some("0"), Some("-1"), Some("many")] {
            let error = parse_ceiling(bad).unwrap_err();
            assert!(error.contains("MAX_ORG_WORKERS"), "{bad:?}: {error}");
        }
        assert_eq!(parse_ceiling(Some(" 120000 ")), Ok(120_000));
    }
}
