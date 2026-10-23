//! Pure rules for reasonable adjustments (WPM-R33 / WPM-D25): the
//! suggestion categories and the lifecycle. The request shape itself
//! is the load-bearing rule — barrier, impact, change; the vocabulary
//! has no place for a diagnosis.

/// The closed suggestion categories (navigation, not gatekeeping —
/// `other` is always available).
pub const ADJUSTMENT_CATEGORIES: &[&str] = &[
    "written_instructions",
    "agendas_in_advance",
    "quieter_workspace",
    "flexible_breaks",
    "clear_priorities",
    "equipment",
    "schedule",
    "other",
];

/// Adjustment statuses.
pub const ADJUSTMENT_STATUSES: &[&str] =
    &["requested", "agreed", "declined", "in_place", "withdrawn"];

/// The lifecycle: `requested → agreed | declined | withdrawn`, then
/// `agreed → in_place | withdrawn`. `declined`, `in_place`, and
/// `withdrawn` are terminal — a declined request is asked again as a
/// new request (each ask stays on the record, in writing).
///
/// # Errors
///
/// A human-readable refusal naming the legal moves.
pub fn transition(current: &str, to: &str) -> Result<(), String> {
    if !ADJUSTMENT_STATUSES.contains(&to) {
        return Err(format!(
            "unknown status `{to}` (statuses: {ADJUSTMENT_STATUSES:?})"
        ));
    }
    let ok = matches!(
        (current, to),
        ("requested", "agreed" | "declined" | "withdrawn") | ("agreed", "in_place" | "withdrawn")
    );
    if ok {
        Ok(())
    } else {
        Err(format!(
            "illegal transition `{current}` → `{to}` \
             (requested → agreed|declined|withdrawn; agreed → in_place|withdrawn)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The suggestion list carries the practical asks — and no entry
    /// names a condition (WPM-D25: navigation, never a label).
    #[test]
    fn categories_are_practical_not_clinical() {
        assert!(ADJUSTMENT_CATEGORIES.contains(&"quieter_workspace"));
        assert!(ADJUSTMENT_CATEGORIES.contains(&"written_instructions"));
        assert!(
            ADJUSTMENT_CATEGORIES.contains(&"other"),
            "never a closed gate"
        );
        for category in ADJUSTMENT_CATEGORIES {
            for clinical in ["adhd", "autis", "dyslex", "diagnos", "condition", "medical"] {
                assert!(
                    !category.contains(clinical),
                    "{category:?} must stay practical"
                );
            }
        }
    }

    #[test]
    fn lifecycle_matrix() {
        assert!(transition("requested", "agreed").is_ok());
        assert!(transition("requested", "declined").is_ok());
        assert!(transition("requested", "withdrawn").is_ok());
        assert!(transition("agreed", "in_place").is_ok());
        assert!(transition("agreed", "withdrawn").is_ok());
        assert!(transition("requested", "in_place").is_err(), "agree first");
        assert!(
            transition("declined", "agreed").is_err(),
            "declined is terminal; ask anew"
        );
        assert!(transition("in_place", "withdrawn").is_err(), "terminal");
        assert!(transition("agreed", "sideways").is_err(), "unknown");
    }

    /// Every pair of statuses, checked against the written lifecycle: exactly
    /// five moves are legal and the three terminal states have none out.
    #[test]
    fn exactly_the_documented_moves_are_legal() {
        let mut legal = Vec::new();
        for from in ADJUSTMENT_STATUSES {
            for to in ADJUSTMENT_STATUSES {
                if transition(from, to).is_ok() {
                    legal.push((*from, *to));
                }
            }
        }
        assert_eq!(
            legal,
            [
                ("requested", "agreed"),
                ("requested", "declined"),
                ("requested", "withdrawn"),
                ("agreed", "in_place"),
                ("agreed", "withdrawn"),
            ]
        );
    }

    #[test]
    fn refusals_name_the_problem() {
        assert!(
            transition("requested", "sideways")
                .unwrap_err()
                .contains("unknown status")
        );
        assert!(
            transition("declined", "agreed")
                .unwrap_err()
                .contains("illegal transition")
        );
        assert!(
            transition("nonsense", "agreed").is_err(),
            "an unknown current status moves nowhere"
        );
        assert!(transition("agreed", "agreed").is_err(), "no self-loop");
    }

    #[test]
    fn categories_are_unique_snake_case() {
        let mut seen = std::collections::BTreeSet::new();
        for c in ADJUSTMENT_CATEGORIES {
            assert!(seen.insert(*c), "{c} repeats");
            assert!(c.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_'));
        }
    }
}
