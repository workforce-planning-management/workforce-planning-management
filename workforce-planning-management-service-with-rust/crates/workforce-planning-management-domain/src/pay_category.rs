//! **Categories of workers** for pay reporting (WPM-R144, WPM-D99): pure rules for what a category must say and
//! how a worker's job title finds it. A category is the employer's own grouping of roles that do the same work or
//! work of equal value; the **basis** says why, in words, and is required. Nothing is inferred from a title: a
//! title in no category is simply in none, and the report counts those workers.

use std::collections::{BTreeMap, BTreeSet};

/// The most job titles one category may list.
pub const MAX_TITLES: usize = 50;
/// The longest category name or job title.
pub const MAX_NAME: usize = 120;
/// The shortest basis. A word or two is not a statement of why roles count as equal value.
pub const MIN_BASIS: usize = 10;
/// The longest basis.
pub const MAX_BASIS: usize = 2_000;

/// A title as it is compared: trimmed, inner whitespace collapsed, lower-cased.
#[must_use]
pub fn title_key(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Check a category and return its job titles tidied (trimmed, inner whitespace collapsed), in the order given.
///
/// # Errors
///
/// A message naming the first thing wrong: an empty or long name, a basis that is too short or long, no titles,
/// too many, a blank or long title, or one listed twice.
pub fn validate(name: &str, basis: &str, titles: &[String]) -> Result<Vec<String>, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(format!(
            "a category needs a name of 1 to {MAX_NAME} characters"
        ));
    }
    let basis_len = basis.trim().chars().count();
    if !(MIN_BASIS..=MAX_BASIS).contains(&basis_len) {
        return Err(format!(
            "say the basis on which these roles count as equal value, in {MIN_BASIS} to {MAX_BASIS} characters"
        ));
    }
    if titles.is_empty() || titles.len() > MAX_TITLES {
        return Err(format!("a category lists 1 to {MAX_TITLES} job titles"));
    }
    let mut seen = BTreeSet::new();
    let mut tidy = Vec::with_capacity(titles.len());
    for title in titles {
        let words = title.split_whitespace().collect::<Vec<_>>().join(" ");
        if words.is_empty() || words.chars().count() > MAX_NAME {
            return Err(format!("a job title is 1 to {MAX_NAME} characters"));
        }
        if !seen.insert(title_key(&words)) {
            return Err(format!("the job title \"{words}\" is listed twice"));
        }
        tidy.push(words);
    }
    Ok(tidy)
}

/// The first job title in `titles` that another category already holds, with that category's name. A title is in at
/// most one category, so a worker is in at most one. `others` is `(name, titles)` of every other category.
#[must_use]
pub fn clash(others: &[(String, Vec<String>)], titles: &[String]) -> Option<(String, String)> {
    let wanted: BTreeMap<String, &String> = titles.iter().map(|t| (title_key(t), t)).collect();
    others.iter().find_map(|(name, held)| {
        held.iter().find_map(|t| {
            wanted
                .get(&title_key(t))
                .map(|w| ((*w).clone(), name.clone()))
        })
    })
}

/// A lookup from a job title (as [`title_key`] compares it) to its category's name.
#[must_use]
pub fn index(categories: &[(String, Vec<String>)]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (name, titles) in categories {
        for title in titles {
            out.entry(title_key(title)).or_insert_with(|| name.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    const BASIS: &str = "Same skills, effort, responsibility and conditions.";

    #[test]
    fn a_good_category_is_returned_tidy() {
        let tidy = validate("  Nursing ", BASIS, &v(&["Staff  Nurse", " Senior Nurse "])).unwrap();
        assert_eq!(tidy, v(&["Staff Nurse", "Senior Nurse"]));
    }

    #[test]
    fn a_category_must_say_its_basis_and_name_and_list_titles() {
        for (name, basis, titles, needle) in [
            ("", BASIS, v(&["A"]), "name"),
            ("   ", BASIS, v(&["A"]), "name"),
            ("N", "short", v(&["A"]), "basis"),
            ("N", "          ", v(&["A"]), "basis"),
            ("N", BASIS, vec![], "titles"),
            ("N", BASIS, v(&["  "]), "job title"),
            ("N", BASIS, v(&["Nurse", "nurse"]), "twice"),
            ("N", BASIS, v(&["Nurse", "  NURSE  "]), "twice"),
        ] {
            let err = validate(name, basis, &titles).unwrap_err();
            assert!(err.contains(needle), "{name:?}/{basis:?}/{titles:?}: {err}");
        }
        let long = "x".repeat(MAX_NAME + 1);
        assert!(validate(&long, BASIS, &v(&["A"])).is_err());
        assert!(validate("N", BASIS, &[long]).is_err());
        assert!(validate("N", &"b".repeat(MAX_BASIS + 1), &v(&["A"])).is_err());
        let many: Vec<String> = (0..=MAX_TITLES).map(|i| format!("t{i}")).collect();
        assert!(validate("N", BASIS, &many).is_err());
        let exactly: Vec<String> = (0..MAX_TITLES).map(|i| format!("t{i}")).collect();
        assert!(validate("N", BASIS, &exactly).is_ok());
    }

    #[test]
    fn a_title_clash_names_the_title_and_the_other_category() {
        let others = vec![("Nursing".to_string(), v(&["Staff Nurse", "Matron"]))];
        assert_eq!(
            clash(&others, &v(&["porter", "STAFF  nurse"])),
            Some(("STAFF  nurse".to_string(), "Nursing".to_string()))
        );
        assert_eq!(clash(&others, &v(&["porter"])), None);
        assert_eq!(clash(&[], &v(&["porter"])), None);
    }

    #[test]
    fn the_index_finds_a_title_however_it_is_spaced_or_cased() {
        let idx = index(&[
            ("Nursing".to_string(), v(&["Staff Nurse"])),
            ("Facilities".to_string(), v(&["Porter"])),
        ]);
        assert_eq!(
            idx.get(&title_key("  staff   NURSE ")).map(String::as_str),
            Some("Nursing")
        );
        assert_eq!(
            idx.get(&title_key("Porter")).map(String::as_str),
            Some("Facilities")
        );
        assert_eq!(
            idx.get(&title_key("Surgeon")),
            None,
            "a title in no category is in none"
        );
    }
}
