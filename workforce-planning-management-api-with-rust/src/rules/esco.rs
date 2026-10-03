//! Pure rules for **ESCO** (European Skills, Competences, Qualifications and
//! Occupations; `spec/esco/index.md`): parsing the CSV download into
//! occupations, skills, and occupation–skill relations, and the stated
//! mappings to WPM's own vocabulary.
//!
//! ESCO says which skills an occupation needs and whether each is
//! *essential* or *optional* — it has **no proficiency scale**. Nothing here
//! invents a level: seeding a role profile takes the planner's chosen level
//! as an input. Columns are resolved **by header name**, so a different
//! column order across ESCO versions does not matter.

use crate::rules::csv::parse_csv;

/// An ESCO skill concept.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EscoSkill {
    /// The concept URI — the identifier.
    pub uri: String,
    /// The preferred label.
    pub label: String,
    /// `knowledge` or `skill/competence`, as ESCO states it.
    pub skill_type: Option<String>,
    /// `transversal`, `cross-sector`, `sector-specific`, or
    /// `occupation-specific`.
    pub reuse_level: Option<String>,
}

/// An ESCO occupation concept.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EscoOccupation {
    /// The concept URI.
    pub uri: String,
    /// The preferred label.
    pub label: String,
    /// The parent ISCO-08 group code, when present.
    pub isco_code: Option<String>,
    /// The description, when present.
    pub description: Option<String>,
}

/// How an occupation relates to a skill.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Relation {
    /// The skill is essential for the occupation.
    Essential,
    /// The skill is optional.
    Optional,
}

impl Relation {
    /// The token stored and reported.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Essential => "essential",
            Self::Optional => "optional",
        }
    }

    /// The role-requirement importance this relation drafts: essential →
    /// `important`, optional → `useful` (never `critical`: ESCO does not
    /// say any skill is).
    #[must_use]
    pub fn importance(self) -> &'static str {
        match self {
            Self::Essential => "important",
            Self::Optional => "useful",
        }
    }
}

/// One `occupation → skill` relation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EscoRelation {
    /// The occupation URI.
    pub occupation_uri: String,
    /// The skill URI.
    pub skill_uri: String,
    /// Essential or optional.
    pub relation: Relation,
}

fn header_index(header: &[String], name: &str) -> Option<usize> {
    header
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(name))
}

fn required(header: &[String], name: &str, file: &str) -> Result<usize, String> {
    header_index(header, name).ok_or_else(|| format!("{file}: missing column `{name}`"))
}

fn cell(row: &[String], index: Option<usize>) -> Option<String> {
    index
        .and_then(|i| row.get(i))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Parse an ESCO skills CSV (`skills_<lang>.csv`).
///
/// # Errors
/// A message when the CSV is malformed or a required column is missing.
pub fn parse_skills(text: &str) -> Result<Vec<EscoSkill>, String> {
    let rows = parse_csv(text)?;
    let header = rows.first().ok_or("skills file is empty")?;
    let uri = required(header, "conceptUri", "skills")?;
    let label = required(header, "preferredLabel", "skills")?;
    let skill_type = header_index(header, "skillType");
    let reuse = header_index(header, "reuseLevel");
    Ok(rows
        .iter()
        .skip(1)
        .filter_map(|row| {
            Some(EscoSkill {
                uri: cell(row, Some(uri))?,
                label: cell(row, Some(label))?,
                skill_type: cell(row, skill_type),
                reuse_level: cell(row, reuse),
            })
        })
        .collect())
}

/// Parse an ESCO occupations CSV (`occupations_<lang>.csv`).
///
/// # Errors
/// A message when the CSV is malformed or a required column is missing.
pub fn parse_occupations(text: &str) -> Result<Vec<EscoOccupation>, String> {
    let rows = parse_csv(text)?;
    let header = rows.first().ok_or("occupations file is empty")?;
    let uri = required(header, "conceptUri", "occupations")?;
    let label = required(header, "preferredLabel", "occupations")?;
    let isco = header_index(header, "iscoGroup");
    let description = header_index(header, "description");
    Ok(rows
        .iter()
        .skip(1)
        .filter_map(|row| {
            Some(EscoOccupation {
                uri: cell(row, Some(uri))?,
                label: cell(row, Some(label))?,
                isco_code: cell(row, isco),
                description: cell(row, description),
            })
        })
        .collect())
}

/// Parse an ESCO occupation–skill relations CSV
/// (`occupationSkillRelations_<lang>.csv`). A row whose relation type is
/// neither `essential` nor `optional` is skipped.
///
/// # Errors
/// A message when the CSV is malformed or a required column is missing.
pub fn parse_relations(text: &str) -> Result<Vec<EscoRelation>, String> {
    let rows = parse_csv(text)?;
    let header = rows.first().ok_or("relations file is empty")?;
    let occupation = required(header, "occupationUri", "relations")?;
    let relation = required(header, "relationType", "relations")?;
    let skill = required(header, "skillUri", "relations")?;
    Ok(rows
        .iter()
        .skip(1)
        .filter_map(|row| {
            let kind = match cell(row, Some(relation))?.to_ascii_lowercase().as_str() {
                "essential" => Relation::Essential,
                "optional" => Relation::Optional,
                _ => return None,
            };
            Some(EscoRelation {
                occupation_uri: cell(row, Some(occupation))?,
                skill_uri: cell(row, Some(skill))?,
                relation: kind,
            })
        })
        .collect())
}

/// A **draft** WPM skill category for an ESCO skill, from its type and reuse
/// level: knowledge → `domain`; transversal skills → `other`; any other
/// skill → `technical`. A planner corrects it in `/skills`.
#[must_use]
pub fn category_for(skill_type: Option<&str>, reuse_level: Option<&str>) -> &'static str {
    let lower = |v: Option<&str>| v.map(str::to_ascii_lowercase);
    if lower(skill_type).as_deref() == Some("knowledge") {
        "domain"
    } else if lower(reuse_level).as_deref() == Some("transversal") {
        "other"
    } else {
        "technical"
    }
}

/// A label normalised for **exact** matching: trimmed, lower-cased, with runs
/// of whitespace collapsed. Never fuzzy — a near-match is a guess about
/// what a skill means.
#[must_use]
pub fn normalise_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKILLS: &str = "\
conceptType,conceptUri,skillType,reuseLevel,preferredLabel,altLabels,description
KnowledgeSkillCompetence,http://data.europa.eu/esco/skill/s1,knowledge,sector-specific,Python (computer programming),\"python\nscripting\",A language
KnowledgeSkillCompetence,http://data.europa.eu/esco/skill/s2,skill/competence,transversal,work in teams,,Collaborate
KnowledgeSkillCompetence,http://data.europa.eu/esco/skill/s3,skill/competence,occupation-specific,test software,,\"Run, check\"
";

    const OCCUPATIONS: &str = "\
conceptType,conceptUri,iscoGroup,preferredLabel,description,code
Occupation,http://data.europa.eu/esco/occupation/o1,2512,software developer,\"Designs, builds\nsoftware\",2512.1
Occupation,http://data.europa.eu/esco/occupation/o2,,odd job,,
";

    const RELATIONS: &str = "\
occupationUri,relationType,skillType,skillUri
http://data.europa.eu/esco/occupation/o1,essential,knowledge,http://data.europa.eu/esco/skill/s1
http://data.europa.eu/esco/occupation/o1,optional,skill/competence,http://data.europa.eu/esco/skill/s2
http://data.europa.eu/esco/occupation/o1,mystery,skill/competence,http://data.europa.eu/esco/skill/s3
";

    #[test]
    fn parses_skills_by_header_name() {
        let skills = parse_skills(SKILLS).unwrap();
        assert_eq!(skills.len(), 3);
        assert_eq!(skills[0].label, "Python (computer programming)");
        assert_eq!(skills[0].skill_type.as_deref(), Some("knowledge"));
        assert_eq!(skills[1].reuse_level.as_deref(), Some("transversal"));
        // A reordered header with the optional columns absent still parses.
        let minimal = "preferredLabel,conceptUri\nTesting,http://x/1\n";
        let parsed = parse_skills(minimal).unwrap();
        assert_eq!(parsed[0].uri, "http://x/1");
        assert_eq!(parsed[0].skill_type, None);
        assert!(
            parse_skills("preferredLabel\nTesting\n")
                .unwrap_err()
                .contains("conceptUri")
        );
        assert!(parse_skills("").is_err());
    }

    #[test]
    fn parses_occupations_with_multiline_descriptions() {
        let occupations = parse_occupations(OCCUPATIONS).unwrap();
        assert_eq!(occupations.len(), 2);
        assert_eq!(occupations[0].isco_code.as_deref(), Some("2512"));
        assert_eq!(
            occupations[0].description.as_deref(),
            Some("Designs, builds\nsoftware")
        );
        assert_eq!(occupations[1].isco_code, None, "absent, not empty");
    }

    #[test]
    fn parses_relations_and_skips_unknown_types() {
        let relations = parse_relations(RELATIONS).unwrap();
        assert_eq!(relations.len(), 2, "the `mystery` relation is skipped");
        assert_eq!(relations[0].relation, Relation::Essential);
        assert_eq!(relations[1].relation, Relation::Optional);
        assert_eq!(relations[0].relation.importance(), "important");
        assert_eq!(relations[1].relation.importance(), "useful");
        assert_eq!(relations[1].relation.as_str(), "optional");
    }

    #[test]
    fn draft_category_mapping() {
        assert_eq!(
            category_for(Some("knowledge"), Some("sector-specific")),
            "domain"
        );
        assert_eq!(
            category_for(Some("skill/competence"), Some("transversal")),
            "other"
        );
        assert_eq!(
            category_for(Some("skill/competence"), Some("occupation-specific")),
            "technical"
        );
        assert_eq!(category_for(None, None), "technical");
        assert_eq!(
            category_for(Some("Knowledge"), None),
            "domain",
            "case-insensitive"
        );
    }

    #[test]
    fn labels_match_exactly_after_normalising() {
        assert_eq!(normalise_label("  Work   in  Teams "), "work in teams");
        assert_ne!(
            normalise_label("work in team"),
            normalise_label("work in teams"),
            "never fuzzy"
        );
    }
}
