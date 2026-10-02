//! Pure rules for **skills gap against a role** (WPM-R37): grading a
//! worker's declared proficiency against a role profile's requirement.
//!
//! Like every workforce-intelligence rule it compares **declarations**,
//! and never turns a missing declaration into a zero: an undeclared skill
//! is `undeclared` — we do not know — which is not the same as `below`.

/// Grade one declared proficiency against a requirement's minimum.
///
/// - `met` — declared at or above the minimum.
/// - `below` — declared, but under the minimum.
/// - `undeclared` — no declaration (unknown).
#[must_use]
pub fn grade(declared: Option<i32>, min_proficiency: i32) -> &'static str {
    match declared {
        None => "undeclared",
        Some(level) if level >= min_proficiency => "met",
        Some(_) => "below",
    }
}

/// How far below the minimum a declared proficiency is; `None` when met
/// or undeclared (an unknown shortfall is not reported as a number).
#[must_use]
pub fn shortfall(declared: Option<i32>, min_proficiency: i32) -> Option<i32> {
    declared
        .filter(|level| *level < min_proficiency)
        .map(|level| min_proficiency - level)
}

/// Counts behind a worker's readiness for a role: requirements met out
/// of requirements, overall and for the `critical` ones.
#[derive(Debug, PartialEq, Eq)]
pub struct Readiness {
    /// Critical requirements met.
    pub critical_met: usize,
    /// Critical requirements.
    pub critical_total: usize,
    /// All requirements met.
    pub met: usize,
    /// All requirements.
    pub total: usize,
}

/// Roll `(importance, grade)` pairs up into [`Readiness`].
#[must_use]
pub fn readiness(graded: &[(&str, &str)]) -> Readiness {
    let mut out = Readiness {
        critical_met: 0,
        critical_total: 0,
        met: 0,
        total: graded.len(),
    };
    for (importance, grade) in graded {
        let met = *grade == "met";
        if met {
            out.met += 1;
        }
        if *importance == "critical" {
            out.critical_total += 1;
            if met {
                out.critical_met += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each grade is reachable; "undeclared" is not "below".
    #[test]
    fn grading() {
        assert_eq!(grade(Some(4), 3), "met");
        assert_eq!(grade(Some(3), 3), "met", "the minimum itself is met");
        assert_eq!(grade(Some(2), 3), "below");
        assert_eq!(grade(None, 3), "undeclared");
    }

    /// A shortfall is a number only when it is actually known.
    #[test]
    fn shortfalls() {
        assert_eq!(shortfall(Some(1), 4), Some(3));
        assert_eq!(shortfall(Some(4), 4), None);
        assert_eq!(shortfall(Some(5), 4), None);
        assert_eq!(shortfall(None, 4), None, "unknown, not min");
    }

    /// Readiness counts critical and overall separately.
    #[test]
    fn readiness_rollup() {
        let r = readiness(&[
            ("critical", "met"),
            ("critical", "below"),
            ("important", "met"),
            ("useful", "undeclared"),
        ]);
        assert_eq!(
            r,
            Readiness {
                critical_met: 1,
                critical_total: 2,
                met: 2,
                total: 4
            }
        );
        assert_eq!(
            readiness(&[]),
            Readiness {
                critical_met: 0,
                critical_total: 0,
                met: 0,
                total: 0
            }
        );
    }
}
