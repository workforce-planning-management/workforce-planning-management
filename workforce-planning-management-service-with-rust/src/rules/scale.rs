//! How many workers one request may load (WPM-R120, WPM-D90).
//!
//! The plan forecast and cost views read the workers of the departments a plan names into memory and
//! work them out there (the directory pages in SQL and the capacity view reads in batches, WPM-T230). That is fine until it is not, so the count is checked first and a
//! request over the ceiling is refused with a message, not left to run for seconds.
//!
//! The default ceiling is the largest organization the load test ran with no failed request at eight
//! concurrent callers (`BENCHMARKS.md`; 50,000 workers starved the runtime and failed requests): nothing
//! is accepted that was not measured. A deployer who measures a larger organization on their own
//! hardware raises it with `WPM_MAX_ORG_WORKERS`.

/// The largest worker count a whole-organization view accepts by default.
pub const DEFAULT_MAX_ORG_WORKERS: u64 = 10_000;

/// The ceiling from a setting: absent, blank, zero or unparseable means the default.
#[must_use]
pub fn parse_ceiling(setting: Option<&str>) -> u64 {
    setting
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_MAX_ORG_WORKERS)
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
        "this view reads {count} workers at once, over the limit of {ceiling}; narrow it to one organization or raise WPM_MAX_ORG_WORKERS after measuring"
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
    fn a_setting_that_is_not_a_positive_number_means_the_default() {
        for bad in [
            None,
            Some(""),
            Some("  "),
            Some("0"),
            Some("-1"),
            Some("many"),
        ] {
            assert_eq!(parse_ceiling(bad), DEFAULT_MAX_ORG_WORKERS, "{bad:?}");
        }
        assert_eq!(parse_ceiling(Some(" 120000 ")), 120_000);
    }
}
