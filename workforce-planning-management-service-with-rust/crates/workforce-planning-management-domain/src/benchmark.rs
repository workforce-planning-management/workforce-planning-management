//! Salary benchmarking flags (WPM-R14), DB-free.

/// The comparison verdict for one worker against a benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkFlag {
    /// Below the recorded market minimum.
    BelowMin,
    /// Within the recorded band.
    Within,
    /// Above the recorded market maximum.
    AboveMax,
}

/// Compare a salary to a benchmark band. `None` when the currencies
/// differ — mixed currencies never silently compare (WPM-D4 family
/// posture).
#[must_use]
pub fn compare(
    salary_minor: i64,
    salary_currency: &str,
    band_min: i64,
    band_max: i64,
    band_currency: &str,
) -> Option<BenchmarkFlag> {
    if !salary_currency.eq_ignore_ascii_case(band_currency) {
        return None;
    }
    Some(if salary_minor < band_min {
        BenchmarkFlag::BelowMin
    } else if salary_minor > band_max {
        BenchmarkFlag::AboveMax
    } else {
        BenchmarkFlag::Within
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Below / within / above / boundary values flag correctly, and a
    /// currency mismatch refuses to compare.
    #[test]
    fn flags_and_currency_guard() {
        assert_eq!(
            compare(90, "GBP", 100, 200, "GBP"),
            Some(BenchmarkFlag::BelowMin)
        );
        assert_eq!(
            compare(100, "GBP", 100, 200, "GBP"),
            Some(BenchmarkFlag::Within)
        );
        assert_eq!(
            compare(200, "GBP", 100, 200, "gbp"),
            Some(BenchmarkFlag::Within)
        );
        assert_eq!(
            compare(201, "GBP", 100, 200, "GBP"),
            Some(BenchmarkFlag::AboveMax)
        );
        assert_eq!(compare(150, "USD", 100, 200, "GBP"), None);
    }

    #[test]
    fn the_band_edges_belong_to_within() {
        assert_eq!(
            compare(100, "GBP", 100, 100, "GBP"),
            Some(BenchmarkFlag::Within)
        );
        assert_eq!(
            compare(99, "GBP", 100, 100, "GBP"),
            Some(BenchmarkFlag::BelowMin)
        );
        assert_eq!(
            compare(101, "GBP", 100, 100, "GBP"),
            Some(BenchmarkFlag::AboveMax)
        );
        assert_eq!(compare(0, "GBP", 0, 10, "GBP"), Some(BenchmarkFlag::Within));
        assert_eq!(
            compare(i64::MAX, "GBP", 0, i64::MAX, "GBP"),
            Some(BenchmarkFlag::Within)
        );
    }

    #[test]
    fn currency_codes_compare_without_regard_to_case_but_not_to_padding() {
        assert!(compare(1, "gbp", 0, 2, "GBP").is_some());
        assert!(
            compare(1, "GBP ", 0, 2, "GBP").is_none(),
            "a padded code is a different code"
        );
        assert!(
            compare(1, "", 0, 2, "").is_some(),
            "two unknowns are not a mismatch, and are not made up"
        );
    }

    #[test]
    fn flags_serialise_as_snake_case_words() {
        let words: Vec<String> = [
            BenchmarkFlag::BelowMin,
            BenchmarkFlag::Within,
            BenchmarkFlag::AboveMax,
        ]
        .iter()
        .map(|f| serde_json::to_string(f).unwrap())
        .collect();
        assert_eq!(words, ["\"below_min\"", "\"within\"", "\"above_max\""]);
    }
}
