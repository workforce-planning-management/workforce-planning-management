//! Pure rules for importing a **job-capability framework** (WPM-R34) —
//! currently the UK Government Digital and Data (GDAD) Profession
//! Capability Framework (PCF), see `spec/uk-gdad-pcf/index.md`.
//!
//! This module only *parses and maps*; reading files and writing rows is
//! the import task's job. Nothing is invented: a skill with no stated level
//! stays without one, and a level on the framework's own scale is converted
//! to WPM's 1–5 scale by a stated, visible mapping ([`map_level`]), with the
//! source level kept alongside.

use std::collections::BTreeMap;

/// WPM's own proficiency scale maximum.
pub const WPM_SCALE_MAX: i32 = 5;

/// The PCF's own proficiency scale maximum (1 Awareness … 4 Expert).
pub const PCF_SCALE_MAX: i32 = 4;

/// One skill a role level names, with the framework's statements about it.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedSkill {
    /// The skill name — exact; a qualified name is a different skill.
    pub name: String,
    /// What the framework says the skill means at this level.
    pub statements: Vec<String>,
}

/// A role level parsed from a framework role summary.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedLevel {
    /// The profession as written, e.g. `Software development`.
    pub profession_label: String,
    /// The role, e.g. `Software developer`.
    pub role: String,
    /// The one-sentence role description.
    pub role_description: Option<String>,
    /// The level, e.g. `Senior developer`; `NOT IN USE` for a retired one.
    pub level: String,
    /// The one-sentence level description.
    pub level_description: Option<String>,
    /// What the role-holder does at this level.
    pub accountabilities: Vec<String>,
    /// Whether the framework has retired this level.
    pub retired: bool,
    /// The skills, in order, with repeated names merged.
    pub skills: Vec<ParsedSkill>,
}

enum Section {
    None,
    RoleDescription,
    Duties,
    LevelDescription,
    Accountabilities,
    Skill(usize),
}

/// Parse a role summary: line-oriented plain text.
///
/// ```text
/// <Profession> role: <Role>
/// - <role description>
/// In this role, you will:
/// - <duty>
/// Role level: <Level>
/// - <level description>
/// At this role level, you will:
/// - <accountability>
/// Skill: <name>
/// - <statement>
/// ```
///
/// # Errors
/// A message when the role line or the `Role level:` line is missing.
pub fn parse_role_summary(text: &str) -> Result<ParsedLevel, String> {
    let mut out = ParsedLevel {
        profession_label: String::new(),
        role: String::new(),
        role_description: None,
        level: String::new(),
        level_description: None,
        accountabilities: Vec::new(),
        retired: false,
        skills: Vec::new(),
    };
    let mut have_role = false;
    let mut have_level = false;
    let mut section = Section::None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if !have_role {
            let (profession, role) = line
                .split_once(" role: ")
                .ok_or_else(|| "first line must be `<Profession> role: <Role>`".to_string())?;
            out.profession_label = profession.trim().to_string();
            out.role = role.trim().to_string();
            have_role = true;
            section = Section::RoleDescription;
        } else if line == "In this role, you will:" {
            section = Section::Duties;
        } else if let Some(level) = line.strip_prefix("Role level:") {
            out.level = level.trim().to_string();
            out.retired = out.level.eq_ignore_ascii_case("NOT IN USE");
            have_level = true;
            section = Section::LevelDescription;
        } else if line == "At this role level, you will:" {
            section = Section::Accountabilities;
        } else if let Some(name) = line.strip_prefix("Skill:") {
            let name = name.trim().to_string();
            let index = out
                .skills
                .iter()
                .position(|s| s.name == name)
                .unwrap_or_else(|| {
                    out.skills.push(ParsedSkill {
                        name,
                        statements: Vec::new(),
                    });
                    out.skills.len() - 1
                });
            section = Section::Skill(index);
        } else if let Some(bullet) = line.strip_prefix("- ") {
            let bullet = bullet.trim().to_string();
            match section {
                Section::RoleDescription if out.role_description.is_none() => {
                    out.role_description = Some(bullet);
                }
                Section::LevelDescription if out.level_description.is_none() => {
                    out.level_description = Some(bullet);
                }
                Section::Accountabilities => out.accountabilities.push(bullet),
                Section::Skill(index) => out.skills[index].statements.push(bullet),
                _ => {}
            }
        }
        // Any other line (a stray note) is ignored.
    }
    if !have_level {
        return Err("missing a `Role level:` line".to_string());
    }
    Ok(out)
}

/// Read each skill's **baseline level** from a competency assessment: the
/// first `Baseline: <integer>` under each `### Skill: <name>` heading, e.g.
/// `Baseline: 3 — Practitioner. …`. The baseline may begin its own line or
/// follow a bold heading on the same line (`**Baseline for this role level:**
/// Baseline: 3 — …`), as some real files write it. A skill with no baseline is
/// simply absent from the result.
#[must_use]
pub fn parse_baselines(text: &str) -> BTreeMap<String, i32> {
    let mut out = BTreeMap::new();
    let mut current: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(name) = line.strip_prefix("### Skill:") {
            current = Some(name.trim().to_string());
            continue;
        }
        let Some(name) = &current else { continue };
        if out.contains_key(name) {
            continue;
        }
        // Every `Baseline:` on the line, in order; the first followed by an integer wins.
        let level = line.match_indices("Baseline:").find_map(|(at, marker)| {
            let digits: String = line[at + marker.len()..]
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse::<i32>().ok()
        });
        if let Some(level) = level {
            out.insert(name.clone(), level);
        }
    }
    out
}

/// Where a role level sits in the framework, from its path
/// (`<profession>/<role>/<number>-<level>` or `<profession>/<role>`).
#[derive(Debug, PartialEq, Eq)]
pub struct SlugParts {
    /// Profession slug.
    pub profession: String,
    /// Role slug.
    pub role: String,
    /// Display order of the level within the role; `None` for a role with
    /// no levels.
    pub order: Option<i32>,
    /// The management track at that order (`…-management`).
    pub management: bool,
    /// A retired role file (`…-not-in-use`).
    pub not_in_use: bool,
}

/// Parse a slug (the path under `roles/`, without `.md`).
#[must_use]
pub fn parse_slug(slug: &str) -> Option<SlugParts> {
    let parts: Vec<&str> = slug.split('/').collect();
    match parts.as_slice() {
        [profession, role, level] => {
            let (number, rest) = level.split_once('-')?;
            Some(SlugParts {
                profession: (*profession).to_string(),
                role: (*role).to_string(),
                order: Some(number.parse().ok()?),
                management: rest.ends_with("-management"),
                not_in_use: rest.ends_with("-not-in-use"),
            })
        }
        [profession, role] => Some(SlugParts {
            profession: (*profession).to_string(),
            role: (*role).to_string(),
            order: None,
            management: false,
            not_in_use: role.ends_with("-not-in-use"),
        }),
        _ => None,
    }
}

/// How a framework's levels become WPM's 1–5 levels. The source level is
/// always kept beside the result, so the choice is visible and reversible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelMapping {
    /// The same number: PCF 3 (Practitioner) is WPM 3. Simplest to explain
    /// and the least manufactured; WPM 5 is simply above the framework.
    #[default]
    Identity,
    /// Linear rescaling so the ends meet: PCF 1→1, 2→2, 3→4, 4→5. WPM 3 is
    /// then unused, so a worker declaring 3 falls *below* a Practitioner
    /// requirement — stricter, and surprising.
    Linear,
}

impl LevelMapping {
    /// Parse the task argument (`identity` / `linear`).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "identity" => Some(Self::Identity),
            "linear" => Some(Self::Linear),
            _ => None,
        }
    }
}

/// Convert a level on the framework's scale (`1..=source_max`) to WPM's
/// scale (`1..=target_max`). `None` for an out-of-range level, a degenerate
/// scale, or an identity mapping onto a smaller scale.
#[must_use]
pub fn map_level(
    mapping: LevelMapping,
    source: i32,
    source_max: i32,
    target_max: i32,
) -> Option<i32> {
    if source_max < 2 || target_max < 2 || !(1..=source_max).contains(&source) {
        return None;
    }
    match mapping {
        LevelMapping::Identity => (source <= target_max).then_some(source),
        LevelMapping::Linear => {
            let numerator = (source - 1) * (target_max - 1);
            let denominator = source_max - 1;
            Some(1 + (2 * numerator + denominator) / (2 * denominator))
        }
    }
}

/// Whether a level name marks the management track (the framework writes
/// it as `Senior developer - management`). A level that merely *ends in the
/// word* — `Head of IT service management` — is not a track.
#[must_use]
pub fn is_management_track(level: &str) -> bool {
    level.trim().to_lowercase().ends_with(" - management")
}

/// The job title a level is imported under: the level name exactly as the
/// framework writes it (which already carries ` - management` for that
/// track) and, when the same name appears in more than one role, the role
/// in brackets.
#[must_use]
pub fn job_title(level: &str, role: &str, collision: bool) -> String {
    let mut title = level.trim().to_string();
    if collision {
        title.push_str(" (");
        title.push_str(role.trim());
        title.push(')');
    }
    title
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARY: &str = "\
Software development role: Software developer
- A software developer designs, runs and improves software.

In this role, you will:
- be responsible for writing clean, secure code

Role level: Senior developer
- A senior developer delivers and integrates software.

At this role level, you will:
- plan and lead development
- coach and mentor more junior colleagues
There are 2 different roles at this level - a technical role and a management role.

Skill: Information security
- design solutions with security controls included

Skill: Prototyping
- approach prototyping as a team activity

Skill: Information security
- a second statement under the same name
";

    #[test]
    fn parses_a_role_summary() {
        let level = parse_role_summary(SUMMARY).expect("parses");
        assert_eq!(level.profession_label, "Software development");
        assert_eq!(level.role, "Software developer");
        assert_eq!(
            level.role_description.as_deref(),
            Some("A software developer designs, runs and improves software.")
        );
        assert_eq!(level.level, "Senior developer");
        assert!(!level.retired);
        assert_eq!(level.accountabilities.len(), 2, "the stray note is ignored");
        assert_eq!(level.skills.len(), 2, "repeated skill names merge");
        assert_eq!(level.skills[0].name, "Information security");
        assert_eq!(level.skills[0].statements.len(), 2);
    }

    #[test]
    fn retired_and_malformed_summaries() {
        let retired =
            "Chief digital and data role: Chief data officer\n- x\n\nRole level: NOT IN USE\n- y\n";
        let parsed = parse_role_summary(retired).expect("parses");
        assert!(parsed.retired && parsed.skills.is_empty());
        assert!(parse_role_summary("not a role line").is_err());
        assert!(
            parse_role_summary("Data role: Analyst\n- x\n")
                .unwrap_err()
                .contains("Role level")
        );
    }

    #[test]
    fn reads_baselines_per_skill() {
        let text = "\
## Competency matrix

### Skill: Availability and capacity management

**Baseline for this role level**

Baseline: 3 — Practitioner. A senior developer operates the services.

- 1 — Awareness: …

### Skill: Development process optimisation

Baseline: 2 — Working. Under guidance.
Baseline: 4 — a second line is ignored.

### Skill: No baseline here

### Skill: Inline variant

**Baseline for this role level:** Baseline: 4 — Expert. Real files write it this way.
The baseline for Inline variant at this level is 2 — prose, not the marker.
";
        let baselines = parse_baselines(text);
        assert_eq!(
            baselines.get("Availability and capacity management"),
            Some(&3)
        );
        assert_eq!(baselines.get("Development process optimisation"), Some(&2));
        assert_eq!(
            baselines.get("No baseline here"),
            None,
            "absent, not defaulted"
        );
    }

    #[test]
    fn slugs() {
        let s = parse_slug("software-development/software-developer/4-senior-developer").unwrap();
        assert_eq!(
            (s.order, s.management, s.not_in_use),
            (Some(4), false, false)
        );
        let m = parse_slug("software-development/software-developer/5-senior-developer-management")
            .unwrap();
        assert!(m.management);
        let chief = parse_slug("chief-digital-and-data/chief-data-officer-not-in-use").unwrap();
        assert!(chief.order.is_none() && chief.not_in_use);
        assert!(parse_slug("a/b/c/d").is_none());
        assert!(
            parse_slug("a/b/x-level").is_none(),
            "the order must be a number"
        );
    }

    /// Identity keeps the number; linear makes the ends meet.
    #[test]
    fn level_mapping() {
        let identity: Vec<i32> = (1..=4)
            .map(|l| map_level(LevelMapping::Identity, l, 4, 5).unwrap())
            .collect();
        assert_eq!(identity, [1, 2, 3, 4]);
        let linear: Vec<i32> = (1..=4)
            .map(|l| map_level(LevelMapping::Linear, l, 4, 5).unwrap())
            .collect();
        assert_eq!(linear, [1, 2, 4, 5]);
        assert_eq!(
            map_level(LevelMapping::Linear, 3, 5, 5),
            Some(3),
            "same scale is the identity"
        );
        assert_eq!(map_level(LevelMapping::Identity, 0, 4, 5), None);
        assert_eq!(map_level(LevelMapping::Identity, 5, 4, 5), None);
        assert_eq!(
            map_level(LevelMapping::Linear, 1, 1, 5),
            None,
            "a one-point scale cannot be rescaled"
        );
        assert_eq!(
            map_level(LevelMapping::Identity, 5, 5, 4),
            None,
            "identity cannot exceed the target"
        );
        assert_eq!(LevelMapping::parse(" Linear "), Some(LevelMapping::Linear));
        assert_eq!(
            LevelMapping::parse("identity"),
            Some(LevelMapping::Identity)
        );
        assert_eq!(LevelMapping::parse("other"), None);
        assert_eq!(LevelMapping::default(), LevelMapping::Identity);
    }

    #[test]
    fn titles() {
        assert_eq!(
            job_title(" Senior developer ", "Software developer", false),
            "Senior developer"
        );
        assert_eq!(
            job_title("Senior developer - management", "Software developer", false),
            "Senior developer - management",
            "the framework's own suffix is kept, not doubled"
        );
        assert_eq!(
            job_title("Lead", "Data engineer", true),
            "Lead (Data engineer)"
        );
        assert!(is_management_track("Senior developer - management"));
        assert!(!is_management_track("Head of IT service management"));
    }
}
