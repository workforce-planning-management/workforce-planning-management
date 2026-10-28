//! Pure rules for the **SFIA CSV contract** (WPM-R109, WPM-D70).
//!
//! The deployer saves their licensed download as CSV with the documented
//! header; this module parses and validates it and nothing else. No SFIA
//! wording is stored in this repository: the test fixture is invented text,
//! labelled as not SFIA. Nothing is invented on import: a skill defined at
//! some levels only has exactly those levels, and a malformed row is refused
//! with the row and the column named.

use std::collections::BTreeMap;

use super::csv::parse_csv;

/// SFIA's own scale: seven levels of responsibility.
pub const SFIA_SCALE_MAX: i32 = 7;

/// The documented header, in order. Matching ignores case and surrounding space.
pub const COLUMNS: [&str; 8] = [
    "skill_code",
    "skill_name",
    "category",
    "subcategory",
    "description",
    "level",
    "level_description",
    "attributes",
];

/// One level at which a skill is defined.
#[derive(Debug, PartialEq, Eq)]
pub struct SkillLevel {
    /// 1 to 7.
    pub level: i32,
    /// The framework's wording for the skill at this level.
    pub description: String,
    /// The level-of-responsibility attributes named for this row, if any.
    pub attributes: Option<String>,
}

/// A skill and the levels it is defined at, ascending.
#[derive(Debug, PartialEq, Eq)]
pub struct Skill {
    /// The SFIA code, e.g. a four-letter code.
    pub code: String,
    /// The skill name.
    pub name: String,
    /// The category.
    pub category: String,
    /// The subcategory.
    pub subcategory: String,
    /// The skill's overall description.
    pub description: String,
    /// The levels it is defined at.
    pub levels: Vec<SkillLevel>,
}

/// Parse and validate the contract file.
///
/// # Errors
/// A message naming the row (1 is the header) and column when the header
/// does not match, a required cell is blank, a level is not 1 to 7, a
/// skill's own fields differ between its rows, or a level repeats.
pub fn parse_skills(text: &str) -> Result<Vec<Skill>, String> {
    let rows = parse_csv(text)?;
    let (header, body) = rows.split_first().ok_or("the file is empty")?;
    let got: Vec<String> = header.iter().map(|h| h.trim().to_lowercase()).collect();
    if got != COLUMNS {
        return Err(format!(
            "row 1: the columns must be exactly {} in that order",
            COLUMNS.join(",")
        ));
    }
    let mut skills: BTreeMap<String, Skill> = BTreeMap::new();
    for (i, row) in body.iter().enumerate() {
        let n = i + 2;
        if row.len() != COLUMNS.len() {
            return Err(format!(
                "row {n}: expected {} columns, found {}",
                COLUMNS.len(),
                row.len()
            ));
        }
        let cell = |c: usize| row[c].trim();
        let need = |c: usize| {
            let v = cell(c);
            if v.is_empty() {
                Err(format!("row {n}: column {} is blank", COLUMNS[c]))
            } else {
                Ok(v.to_string())
            }
        };
        let code = need(0)?;
        let (name, category, subcategory, description) = (need(1)?, need(2)?, need(3)?, need(4)?);
        let level: i32 = cell(5)
            .parse()
            .ok()
            .filter(|l| (1..=SFIA_SCALE_MAX).contains(l))
            .ok_or_else(|| {
                format!("row {n}: column level must be a whole number from 1 to {SFIA_SCALE_MAX}")
            })?;
        let level_description = need(6)?;
        let attributes = Some(cell(7)).filter(|a| !a.is_empty()).map(str::to_string);
        let skill = skills.entry(code.clone()).or_insert_with(|| Skill {
            code: code.clone(),
            name: name.clone(),
            category: category.clone(),
            subcategory: subcategory.clone(),
            description: description.clone(),
            levels: Vec::new(),
        });
        for (c, same) in [
            (1, skill.name == name),
            (2, skill.category == category),
            (3, skill.subcategory == subcategory),
            (4, skill.description == description),
        ] {
            if !same {
                return Err(format!(
                    "row {n}: column {} differs from the skill's earlier rows ({code})",
                    COLUMNS[c]
                ));
            }
        }
        if skill.levels.iter().any(|l| l.level == level) {
            return Err(format!("row {n}: {code} level {level} appears twice"));
        }
        skill.levels.push(SkillLevel {
            level,
            description: level_description,
            attributes,
        });
    }
    let mut out: Vec<Skill> = skills.into_values().collect();
    for s in &mut out {
        s.levels.sort_by_key(|l| l.level);
    }
    Ok(out)
}

/// The mapping from a framework's own scale to WPM's 1–5, as the interface names it (WPM-D70).
pub const MAPPING_NAME: &str = "linear";

/// A level stated on a framework's own scale, as WPM's 1–5 figure that comparisons use.
///
/// # Errors
/// A message when the level is outside `1..=scale_max` or the scale is degenerate.
pub fn to_wpm(level: i32, scale_max: i32) -> Result<i32, String> {
    use super::framework::{LevelMapping, WPM_SCALE_MAX, map_level};
    map_level(LevelMapping::Linear, level, scale_max, WPM_SCALE_MAX)
        .ok_or_else(|| format!("level must be a whole number from 1 to {scale_max}"))
}

/// The documented header of the levels-of-responsibility file, in order.
pub const LEVEL_COLUMNS: [&str; 8] = [
    "level",
    "title",
    "essence",
    "autonomy",
    "influence",
    "complexity",
    "knowledge",
    "business_skills",
];

/// One level of responsibility with its five generic attributes.
#[derive(Debug, PartialEq, Eq)]
pub struct ResponsibilityLevel {
    /// 1 to 7.
    pub level: i32,
    /// The level's name.
    pub title: String,
    /// The level's essence, in the framework's words.
    pub essence: String,
    /// The five attributes as (name, wording), in the documented order.
    pub attributes: Vec<(String, String)>,
}

/// Parse and validate the levels-of-responsibility contract: exactly the
/// levels 1 to 7, each once, no blank cell.
///
/// # Errors
/// A message naming the row and column, or the level that is missing.
pub fn parse_levels(text: &str) -> Result<Vec<ResponsibilityLevel>, String> {
    let rows = parse_csv(text)?;
    let (header, body) = rows.split_first().ok_or("the file is empty")?;
    let got: Vec<String> = header.iter().map(|h| h.trim().to_lowercase()).collect();
    if got != LEVEL_COLUMNS {
        return Err(format!(
            "row 1: the columns must be exactly {} in that order",
            LEVEL_COLUMNS.join(",")
        ));
    }
    let mut out: Vec<ResponsibilityLevel> = Vec::new();
    for (i, row) in body.iter().enumerate() {
        let n = i + 2;
        if row.len() != LEVEL_COLUMNS.len() {
            return Err(format!(
                "row {n}: expected {} columns, found {}",
                LEVEL_COLUMNS.len(),
                row.len()
            ));
        }
        for (c, cell) in row.iter().enumerate() {
            if cell.trim().is_empty() {
                return Err(format!("row {n}: column {} is blank", LEVEL_COLUMNS[c]));
            }
        }
        let level: i32 = row[0]
            .trim()
            .parse()
            .ok()
            .filter(|l| (1..=SFIA_SCALE_MAX).contains(l))
            .ok_or_else(|| {
                format!("row {n}: column level must be a whole number from 1 to {SFIA_SCALE_MAX}")
            })?;
        if out.iter().any(|l| l.level == level) {
            return Err(format!("row {n}: level {level} appears twice"));
        }
        out.push(ResponsibilityLevel {
            level,
            title: row[1].trim().to_string(),
            essence: row[2].trim().to_string(),
            attributes: (3..LEVEL_COLUMNS.len())
                .map(|c| (LEVEL_COLUMNS[c].to_string(), row[c].trim().to_string()))
                .collect(),
        });
    }
    for want in 1..=SFIA_SCALE_MAX {
        if !out.iter().any(|l| l.level == want) {
            return Err(format!(
                "level {want} is missing: all {SFIA_SCALE_MAX} levels are required"
            ));
        }
    }
    out.sort_by_key(|l| l.level);
    Ok(out)
}

/// The documented header of the `SFIAplus` resources file, in order. **Provisional**: the format is to be agreed with the
/// licence holder (WPM-R112); this is this repository's own contract, shaped like the others.
pub const RESOURCE_COLUMNS: [&str; 6] = ["skill_code", "level", "kind", "title", "provider", "url"];

/// The kinds of resource a pointer can be.
pub const RESOURCE_KINDS: [&str; 4] = ["typical_task", "training", "certification", "membership"];

/// One resource attached to a SFIA skill at a level.
#[derive(Debug, PartialEq, Eq)]
pub struct Resource {
    /// The SFIA code it attaches to.
    pub code: String,
    /// 1 to 7.
    pub level: i32,
    /// One of [`RESOURCE_KINDS`].
    pub kind: String,
    /// The resource's title, as the source gives it.
    pub title: String,
    /// Who provides or awards it, when stated.
    pub provider: Option<String>,
    /// A web address, when stated; only `http://` or `https://`.
    pub url: Option<String>,
}

/// Parse and validate the `SFIAplus` resources contract.
///
/// # Errors
/// A message naming the row and column: a wrong header, a blank code or title, a level outside 1 to 7, an unknown kind,
/// a web address that is not `http(s)://`, or the same resource twice.
pub fn parse_resources(text: &str) -> Result<Vec<Resource>, String> {
    let rows = parse_csv(text)?;
    let (header, body) = rows.split_first().ok_or("the file is empty")?;
    let got: Vec<String> = header.iter().map(|h| h.trim().to_lowercase()).collect();
    if got != RESOURCE_COLUMNS {
        return Err(format!(
            "row 1: the columns must be exactly {} in that order",
            RESOURCE_COLUMNS.join(",")
        ));
    }
    let mut out: Vec<Resource> = Vec::new();
    for (i, row) in body.iter().enumerate() {
        let n = i + 2;
        if row.len() != RESOURCE_COLUMNS.len() {
            return Err(format!(
                "row {n}: expected {} columns, found {}",
                RESOURCE_COLUMNS.len(),
                row.len()
            ));
        }
        let cell = |c: usize| row[c].trim();
        let need = |c: usize| {
            if cell(c).is_empty() {
                Err(format!("row {n}: column {} is blank", RESOURCE_COLUMNS[c]))
            } else {
                Ok(cell(c).to_string())
            }
        };
        let optional = |c: usize| Some(cell(c)).filter(|v| !v.is_empty()).map(str::to_string);
        let level: i32 = cell(1)
            .parse()
            .ok()
            .filter(|l| (1..=SFIA_SCALE_MAX).contains(l))
            .ok_or_else(|| {
                format!("row {n}: column level must be a whole number from 1 to {SFIA_SCALE_MAX}")
            })?;
        let kind = need(2)?;
        if !RESOURCE_KINDS.contains(&kind.as_str()) {
            return Err(format!(
                "row {n}: column kind must be one of {}",
                RESOURCE_KINDS.join(", ")
            ));
        }
        let url = optional(5);
        if let Some(u) = &url
            && !(u.starts_with("https://") || u.starts_with("http://"))
        {
            return Err(format!(
                "row {n}: column url must start with http:// or https://"
            ));
        }
        let resource = Resource {
            code: need(0)?,
            level,
            kind,
            title: need(3)?,
            provider: optional(4),
            url,
        };
        if out.iter().any(|r| {
            (&r.code, r.level, &r.kind, &r.title)
                == (&resource.code, level, &resource.kind, &resource.title)
        }) {
            return Err(format!("row {n}: this resource appears twice"));
        }
        out.push(resource);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::framework::{LevelMapping, WPM_SCALE_MAX, map_level};

    const FIXTURE: &str = include_str!("../../tests/fixtures/sfia-synthetic/skills.csv");
    const HEAD: &str = "skill_code,skill_name,category,subcategory,description,level,level_description,attributes\n";

    fn err(body: &str) -> String {
        parse_skills(&format!("{HEAD}{body}")).unwrap_err()
    }

    #[test]
    fn the_synthetic_fixture_parses_with_levels_as_given() {
        let skills = parse_skills(FIXTURE).unwrap();
        assert_eq!(skills.len(), 2);
        let levels: Vec<i32> = skills[0].levels.iter().map(|l| l.level).collect();
        assert_eq!(levels, [2, 3, 5]);
        assert_eq!(
            skills[0].levels[1].attributes.as_deref(),
            Some("Invented attribute")
        );
        assert_eq!(skills[0].levels[0].attributes, None);
        assert_eq!(skills[1].levels.len(), 1);
    }

    #[test]
    fn a_wrong_header_is_refused() {
        let e = parse_skills("code,name\nA,B\n").unwrap_err();
        assert!(e.starts_with("row 1:"), "{e}");
        assert!(parse_skills("").is_err());
    }

    #[test]
    fn levels_outside_one_to_seven_are_refused() {
        for bad in ["0", "8", "", "two", "2.5"] {
            let e = err(&format!("A,n,c,s,d,{bad},x,\n"));
            assert!(e.contains("column level"), "{bad}: {e}");
        }
    }

    #[test]
    fn blank_text_is_never_filled_in() {
        assert!(err("A,n,c,s,d,2,,\n").contains("column level_description is blank"));
        assert!(err("A,,c,s,d,2,x,\n").contains("column skill_name is blank"));
    }

    #[test]
    fn rows_must_agree_and_levels_must_not_repeat() {
        assert!(err("A,n,c,s,d,2,x,\nA,m,c,s,d,3,y,\n").contains("column skill_name differs"));
        assert!(err("A,n,c,s,d,2,x,\nA,n,c,s,d,2,y,\n").contains("level 2 appears twice"));
        assert!(err("A,n,c,s,d,2,x\n").contains("expected 8 columns"));
    }

    const LEVELS: &str = include_str!("../../tests/fixtures/sfia-synthetic/levels.csv");

    #[test]
    fn the_synthetic_levels_parse_in_order_with_five_attributes() {
        let levels = parse_levels(LEVELS).unwrap();
        assert_eq!(
            levels.iter().map(|l| l.level).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(levels[0].attributes.len(), 5);
        assert_eq!(levels[0].attributes[0].0, "autonomy");
    }

    #[test]
    fn the_levels_file_needs_all_seven_once_and_no_blank_cell() {
        let head = "level,title,essence,autonomy,influence,complexity,knowledge,business_skills\n";
        let without_seven: String = LEVELS
            .lines()
            .filter(|l| !l.starts_with("7,"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            parse_levels(&without_seven)
                .unwrap_err()
                .contains("level 7 is missing")
        );
        assert!(
            parse_levels(&format!("{LEVELS}\n1,a,b,c,d,e,f,g\n"))
                .unwrap_err()
                .contains("level 1 appears twice")
        );
        assert!(
            parse_levels(&format!("{head}1,a,b,,d,e,f,g\n"))
                .unwrap_err()
                .contains("column autonomy is blank")
        );
        assert!(
            parse_levels(&format!("{head}8,a,b,c,d,e,f,g\n"))
                .unwrap_err()
                .contains("column level")
        );
        assert!(parse_levels("level,title\n1,a\n").is_err());
    }

    #[test]
    fn a_native_level_becomes_the_wpm_figure_and_a_bad_one_is_refused() {
        assert_eq!((to_wpm(4, 7), to_wpm(5, 7)), (Ok(3), Ok(4)));
        assert_eq!(to_wpm(7, 7), Ok(5));
        assert_eq!(to_wpm(3, 5), Ok(3), "WPM's own scale maps to itself");
        assert_eq!(to_wpm(2, 4), Ok(2));
        assert!(to_wpm(0, 7).is_err() && to_wpm(8, 7).is_err() && to_wpm(1, 1).is_err());
    }

    const RESOURCES: &str = include_str!("../../tests/fixtures/sfia-synthetic/resources.csv");
    const RHEAD: &str = "skill_code,level,kind,title,provider,url\n";

    #[test]
    fn the_synthetic_resources_parse_with_optional_provider_and_url() {
        let r = parse_resources(RESOURCES).unwrap();
        assert_eq!(r.len(), 4);
        assert_eq!(r[0].kind, "training");
        assert_eq!(r[0].url.as_deref(), Some("https://example.invalid/course"));
        assert_eq!(r[1].provider, None);
        assert_eq!(r[1].url, None);
    }

    #[test]
    fn a_resource_needs_a_known_kind_a_safe_address_and_no_repeat() {
        let bad = |body: &str| parse_resources(&format!("{RHEAD}{body}")).unwrap_err();
        assert!(bad("A,2,course,t,,\n").contains("column kind"));
        assert!(bad("A,9,training,t,,\n").contains("column level"));
        assert!(bad("A,2,training,,,\n").contains("column title is blank"));
        assert!(bad("A,2,training,t,,javascript:alert(1)\n").contains("column url"));
        assert!(bad("A,2,training,t,,ftp://x\n").contains("column url"));
        assert!(bad("A,2,training,t,,\nA,2,training,t,p,\n").contains("appears twice"));
        assert!(parse_resources("skill_code,level\nA,2\n").is_err());
        assert!(parse_resources(&format!("{RHEAD}A,2,membership,t,,\n")).is_ok());
    }

    #[test]
    fn the_linear_mapping_is_pinned() {
        let mapped: Vec<i32> = (1..=SFIA_SCALE_MAX)
            .map(|l| map_level(LevelMapping::Linear, l, SFIA_SCALE_MAX, WPM_SCALE_MAX).unwrap())
            .collect();
        assert_eq!(mapped, [1, 2, 2, 3, 4, 4, 5]);
    }
}
