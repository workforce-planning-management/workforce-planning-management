//! Job-level frameworks (WPM-R52), DB-free: a published ladder of levels with
//! what each level is, a typical experience and its management equivalent. The
//! first is Google's technical / individual-contributor ladder, L3–L11.
//!
//! Reference data, **no pay**: the source gives none, and a level is not a
//! salary. An experience or management equivalent the source does not state is
//! `None` — unknown, never a guess. Levels are ordered; a higher number is a more
//! senior level.

use serde::Serialize;

/// One level on a ladder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Level {
    /// The level number (`3` for L3).
    pub number: u8,
    /// The code people use, e.g. `L3`.
    pub code: &'static str,
    /// The usual title for the level.
    pub title: &'static str,
    /// What the level is, in the source's words.
    pub summary: &'static str,
    /// Typical experience as the source states it; `None` when it does not say.
    pub experience: Option<&'static str>,
    /// The management-track role at the same level, when the source names one.
    pub management_equivalent: Option<&'static str>,
}

/// A whole ladder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LevelFramework {
    /// Stable id.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// The organization whose ladder this is.
    pub organization: &'static str,
    /// The track covered (`technical`).
    pub track: &'static str,
    /// Where the figures come from and how far to trust them.
    pub source: &'static str,
    /// Levels, most junior first.
    pub levels: Vec<Level>,
}

fn level(
    number: u8,
    title: &'static str,
    summary: &'static str,
    experience: Option<&'static str>,
    management_equivalent: Option<&'static str>,
) -> Level {
    // The code is derived from the number, so the two cannot disagree.
    let code: &'static str = match number {
        3 => "L3",
        4 => "L4",
        5 => "L5",
        6 => "L6",
        7 => "L7",
        8 => "L8",
        9 => "L9",
        10 => "L10",
        _ => "L11",
    };
    Level {
        number,
        code,
        title,
        summary,
        experience,
        management_equivalent,
    }
}

/// Google's technical / individual-contributor levels, L3–L11.
#[must_use]
pub fn google_levels() -> LevelFramework {
    LevelFramework {
        id: "google-levels",
        name: "Google technical levels (L3–L11)",
        organization: "Google",
        track: "technical",
        source: "A published summary of Google's job levels supplied by the maintainer on 2026-10-06. Not an official Google publication: Google does not publish its ladder, titles differ by function, and the `T` technical-track prefix is not modelled.",
        levels: vec![
            level(
                3,
                "Software Engineer I",
                "Entry level for new university graduates.",
                Some("0–1 year"),
                None,
            ),
            level(
                4,
                "Software Engineer II",
                "Intermediate engineer.",
                Some("roughly 2–4 years, or a completed PhD"),
                None,
            ),
            level(
                5,
                "Senior Software Engineer",
                "Senior level: owns complex features and mentors junior engineers.",
                Some("5–10+ years"),
                Some("Engineering Manager (a team of roughly 5 to 40+ people)"),
            ),
            level(
                6,
                "Staff Software Engineer",
                "Recognized technical leader guiding multiple teams or large domains.",
                Some("roughly 10+ years"),
                Some("Engineering Manager (a team of roughly 5 to 40+ people)"),
            ),
            level(
                7,
                "Senior Staff / Principal Software Engineer",
                "Cross-organization technical strategist and leader.",
                None,
                Some("Engineering Manager (a team of roughly 5 to 40+ people)"),
            ),
            level(
                8,
                "Distinguished / Principal Engineer",
                "Director-equivalent executive and technical authority.",
                Some("15+ years"),
                Some("Director"),
            ),
            level(
                9,
                "Senior Distinguished Engineer",
                "Senior director-level technical leadership.",
                None,
                Some("Senior Director"),
            ),
            level(
                10,
                "Google Fellow",
                "Vice-president-level individual contributor.",
                None,
                Some("Vice President (VP)"),
            ),
            level(
                11,
                "Senior Google Fellow",
                "Senior-vice-president-level individual contributor.",
                None,
                Some("Vice President II (VP II)"),
            ),
        ],
    }
}

/// Every framework the service knows.
#[must_use]
pub fn all() -> Vec<LevelFramework> {
    vec![google_levels()]
}

/// The framework with this id, if any.
#[must_use]
pub fn find(id: &str) -> Option<LevelFramework> {
    all().into_iter().find(|f| f.id == id)
}

impl LevelFramework {
    /// The level with this code (`l5` and `L5` both work) or number-only (`5`).
    #[must_use]
    pub fn level(&self, code: &str) -> Option<&Level> {
        let wanted = code.trim().trim_start_matches(['l', 'L']);
        self.levels.iter().find(|l| l.number.to_string() == wanted)
    }

    /// The next level up, if any.
    #[must_use]
    pub fn next_above(&self, number: u8) -> Option<&Level> {
        self.levels.iter().find(|l| l.number == number + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L3–L11 with nothing missing or repeated, codes matching numbers, and every
    /// field the source did not state left `None` rather than invented.
    #[test]
    fn google_ladder_is_complete_and_honest() {
        let g = google_levels();
        let numbers: Vec<u8> = g.levels.iter().map(|l| l.number).collect();
        assert_eq!(numbers, (3..=11).collect::<Vec<u8>>());
        for l in &g.levels {
            assert_eq!(l.code, format!("L{}", l.number));
            assert!(!l.title.is_empty() && !l.summary.is_empty(), "{}", l.code);
        }
        let by = |c: &str| g.level(c).unwrap();
        assert_eq!(by("L3").experience, Some("0–1 year"));
        assert_eq!(by("L7").experience, None, "the source gives none for L7");
        assert_eq!(by("L11").experience, None);
        // Management equivalents start at L5 and follow the source.
        assert_eq!(by("L4").management_equivalent, None);
        assert!(
            by("L5")
                .management_equivalent
                .unwrap()
                .starts_with("Engineering Manager")
        );
        assert_eq!(
            by("L7").management_equivalent,
            by("L5").management_equivalent
        );
        assert_eq!(by("L8").management_equivalent, Some("Director"));
        assert_eq!(by("L9").management_equivalent, Some("Senior Director"));
        assert_eq!(by("L10").management_equivalent, Some("Vice President (VP)"));
        assert_eq!(
            by("L11").management_equivalent,
            Some("Vice President II (VP II)")
        );
        assert!(g.source.contains("Not an official"));
    }

    #[test]
    fn lookup_forms_and_unknowns() {
        let g = google_levels();
        assert_eq!(g.level("l6").unwrap().number, 6);
        assert_eq!(g.level(" L10 ").unwrap().number, 10);
        assert_eq!(g.level("5").unwrap().number, 5);
        assert!(g.level("L2").is_none());
        assert!(g.level("L12").is_none());
        assert!(g.level("").is_none());
        assert_eq!(g.next_above(3).unwrap().number, 4);
        assert!(g.next_above(11).is_none(), "L11 is the top");
        assert!(find("google-levels").is_some());
        assert!(find("meta-levels").is_none());
    }
}
