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

/// A named attribute of a level (SFIA's autonomy, influence, …) and what the framework says of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Attribute {
    /// The attribute's name.
    pub name: String,
    /// The framework's wording.
    pub text: String,
}

/// A level held as owned text: the shape of a ladder read from the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredLevel {
    /// The level number.
    pub number: u8,
    /// The code, e.g. `L3`.
    pub code: String,
    /// The usual title for the level.
    pub title: String,
    /// What the level is, in the source's words.
    pub summary: String,
    /// Typical experience, when stated.
    pub experience: Option<String>,
    /// The management-track equivalent, when stated.
    pub management_equivalent: Option<String>,
    /// Named attributes of the level; empty for a ladder that has none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
}

/// A whole ladder as owned text, whether coded or stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredFramework {
    /// Stable id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The organization whose ladder this is.
    pub organization: String,
    /// The track covered.
    pub track: String,
    /// Where the figures come from.
    pub source: String,
    /// Levels, most junior first.
    pub levels: Vec<StoredLevel>,
}

impl From<LevelFramework> for StoredFramework {
    fn from(f: LevelFramework) -> Self {
        Self {
            id: f.id.to_string(),
            name: f.name.to_string(),
            organization: f.organization.to_string(),
            track: f.track.to_string(),
            source: f.source.to_string(),
            levels: f
                .levels
                .into_iter()
                .map(|l| StoredLevel {
                    number: l.number,
                    code: l.code.to_string(),
                    title: l.title.to_string(),
                    summary: l.summary.to_string(),
                    experience: l.experience.map(str::to_string),
                    management_equivalent: l.management_equivalent.map(str::to_string),
                    attributes: Vec::new(),
                })
                .collect(),
        }
    }
}

impl StoredFramework {
    /// The level with this code (`l5`, `L5`) or number-only (`5`).
    #[must_use]
    pub fn level(&self, code: &str) -> Option<&StoredLevel> {
        let code = code.trim();
        let wanted = code.strip_prefix(['l', 'L']).unwrap_or(code);
        self.levels.iter().find(|l| l.number.to_string() == wanted)
    }

    /// The next level up, if any.
    #[must_use]
    pub fn next_above(&self, number: u8) -> Option<&StoredLevel> {
        let above = number.checked_add(1)?;
        self.levels.iter().find(|l| l.number == above)
    }
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
        let code = code.trim();
        let wanted = code.strip_prefix(['l', 'L']).unwrap_or(code);
        self.levels.iter().find(|l| l.number.to_string() == wanted)
    }

    /// The next level up, if any.
    #[must_use]
    pub fn next_above(&self, number: u8) -> Option<&Level> {
        let above = number.checked_add(1)?;
        self.levels.iter().find(|l| l.number == above)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_owned_form_is_the_coded_ladder_and_looks_levels_up_the_same_way() {
        let owned = StoredFramework::from(google_levels());
        assert_eq!(owned.id, "google-levels");
        assert_eq!(owned.levels.len(), 9);
        assert_eq!(owned.level("l5").map(|l| l.number), Some(5));
        assert_eq!(owned.level("5").map(|l| l.number), Some(5));
        assert!(owned.level("L2").is_none());
        assert_eq!(owned.next_above(11), None);
        assert_eq!(owned.next_above(5).map(|l| l.number), Some(6));
        let json = serde_json::to_value(&owned.levels[0]).unwrap();
        assert!(
            json.get("attributes").is_none(),
            "a coded ladder shows no attributes key"
        );
    }

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

    #[test]
    fn frameworks_have_unique_ids_and_levels_ascend_without_pay() {
        let all = all();
        let ids: std::collections::BTreeSet<&str> = all.iter().map(|f| f.id).collect();
        assert_eq!(ids.len(), all.len());
        for f in &all {
            assert!(
                f.levels.windows(2).all(|w| w[0].number < w[1].number),
                "{} ascends",
                f.id
            );
            assert!(f.next_above(f.levels.last().unwrap().number).is_none());
            let text = serde_json::to_string(f).unwrap().to_lowercase();
            assert!(
                !text.contains("salary") && !text.contains("pay_"),
                "{} states no pay",
                f.id
            );
        }
    }

    #[test]
    fn next_above_is_the_following_number_and_never_overflows() {
        let g = google_levels();
        assert_eq!(g.next_above(2).unwrap().number, 3);
        assert_eq!(g.next_above(10).unwrap().number, 11);
        assert!(g.next_above(0).is_none(), "level 1 is not on this ladder");
        assert!(g.next_above(u8::MAX).is_none());
    }

    #[test]
    fn odd_lookups_are_refused_not_guessed() {
        let g = google_levels();
        for bad in ["L", "LL5", "5.0", "-5", "L-5", "L 5", "l05x"] {
            assert!(g.level(bad).is_none(), "{bad:?}");
        }
        assert!(find("").is_none() && find("Google-Levels").is_none());
    }
}
