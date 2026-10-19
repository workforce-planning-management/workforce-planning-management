//! String similarity, all in `0.0..=1.0` where `1.0` is identical.

use strsim::{jaro_winkler, levenshtein};

/// Jaro-Winkler: rewards a shared prefix; good for short names.
pub fn jaro_winkler_similarity(a: &str, b: &str) -> f64 {
    jaro_winkler(a, b)
}

/// One minus edit distance over the longer length. Two empty strings are identical.
pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let longest = a.chars().count().max(b.chars().count());
    if longest == 0 {
        return 1.0;
    }
    1.0 - levenshtein(a, b) as f64 / longest as f64
}

/// `0.7 × Jaro-Winkler + 0.3 × Levenshtein`: the default for names.
pub fn combined_similarity(a: &str, b: &str) -> f64 {
    0.7 * jaro_winkler_similarity(a, b) + 0.3 * levenshtein_similarity(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_one_and_far_is_low() {
        assert_eq!(combined_similarity("smith", "smith"), 1.0);
        assert!(combined_similarity("smith", "smyth") > 0.8);
        assert!(combined_similarity("smith", "jones") < 0.5);
    }

    #[test]
    fn empty_strings() {
        assert_eq!(levenshtein_similarity("", ""), 1.0);
        assert_eq!(levenshtein_similarity("a", ""), 0.0);
    }

    #[test]
    fn symmetric_and_bounded() {
        for (a, b) in [("anna", "ann"), ("li", "lee"), ("alexander", "alex")] {
            let ab = combined_similarity(a, b);
            assert!((ab - combined_similarity(b, a)).abs() < 1e-12);
            assert!((0.0..=1.0).contains(&ab));
        }
    }
}
