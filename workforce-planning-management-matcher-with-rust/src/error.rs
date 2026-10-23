//! The one error type: a configuration that cannot be used.
//!
//! Matching itself never fails. Missing or unparseable fields are `None` in
//! the breakdown, not errors, so a record with a bad phone number still matches
//! on its name.

use thiserror::Error;

/// Why a [`crate::MatchConfig`] was refused by [`crate::MatchConfig::validate`].
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MatchError {
    /// A field weight is negative, infinite or NaN.
    #[error("weight for {field} must be finite and not negative, got {value}")]
    BadWeight {
        /// The weight's field name.
        field: &'static str,
        /// The refused value.
        value: f64,
    },
    /// Every weight is zero, so there is nothing to score.
    #[error("at least one field weight must be above zero")]
    NoWeights,
    /// A threshold is outside `0.0..=1.0`, or review is above match.
    #[error("thresholds must satisfy 0 <= review ({review}) <= match ({matched}) <= 1")]
    BadThresholds {
        /// The review threshold given.
        review: f64,
        /// The match threshold given.
        matched: f64,
    },
    /// `min_fields` is zero: a score from no evidence is not a score.
    #[error("min_fields must be at least 1")]
    NoMinimum,
    /// The default phone country code is not 1 to 3 digits.
    #[error("default phone country code must be 1 to 3 digits, got {0:?}")]
    BadCountryCode(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_name_the_problem() {
        let bad = MatchError::BadWeight {
            field: "email",
            value: -1.0,
        };
        assert_eq!(
            bad.to_string(),
            "weight for email must be finite and not negative, got -1"
        );
        assert_eq!(
            MatchError::NoWeights.to_string(),
            "at least one field weight must be above zero"
        );
        let t = MatchError::BadThresholds {
            review: 0.9,
            matched: 0.5,
        };
        assert!(t.to_string().contains("review (0.9)"));
        assert!(t.to_string().contains("match (0.5)"));
        assert_eq!(
            MatchError::NoMinimum.to_string(),
            "min_fields must be at least 1"
        );
        assert!(
            MatchError::BadCountryCode("abcd".into())
                .to_string()
                .contains("\"abcd\"")
        );
    }
}
