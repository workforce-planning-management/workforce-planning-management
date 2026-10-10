//! **Equality and diversity monitoring** (WPM-R125, WPM-D74): pure rules for voluntary,
//! self-declared protected characteristics held to monitor the equality of a workforce.
//!
//! This is the one deliberate exception to "WPM holds no demographics", and it is bounded:
//!
//! - **Off until a deployer records the lawful basis they rely on.** Nothing here runs otherwise.
//! - **The classification is the deployer's.** WPM ships no list of ethnic groups, religions or
//!   orientations: they differ by country and change over time, and choosing them is a judgment for
//!   the deployer's equality lead. Every category always offers `prefer_not_to_say`.
//! - **Readable by the worker only.** The output is an aggregate with small groups withheld, and a
//!   completeness figure that names no value.
//! - **Never an input to a person-level decision**; a test fails if any other code reads it.

use std::collections::{BTreeMap, BTreeSet};

/// The answer every category offers.
pub const PREFER_NOT_TO_SAY: &str = "prefer_not_to_say";
/// The most categories a deployment may configure.
pub const MAX_CATEGORIES: usize = 12;
/// The most values one category may offer (not counting `prefer_not_to_say`).
pub const MAX_VALUES: usize = 40;
/// The smallest floor a deployment may set: below this a cell can point at a person.
pub const MIN_FLOOR: usize = 5;
/// The floor when none is configured.
pub const DEFAULT_FLOOR: usize = 10;

/// The categories and, for each, the values a worker may declare. `prefer_not_to_say` is added to
/// every category and cannot be configured away.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    categories: BTreeMap<String, Vec<String>>,
}

fn token_ok(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 60
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

impl Config {
    /// Parse the deployer's configuration: a JSON object of category to list of values.
    ///
    /// # Errors
    ///
    /// Not an object of lists, too many categories or values, a name that is not a lowercase
    /// token, an empty category, a repeated value, or an attempt to configure
    /// `prefer_not_to_say` (it is always offered).
    pub fn parse(json: &str) -> Result<Self, String> {
        let raw: BTreeMap<String, Vec<String>> = serde_json::from_str(json).map_err(|e| {
            format!("the categories must be a JSON object of category to list of values: {e}")
        })?;
        if raw.is_empty() {
            return Err("at least one category is needed".to_string());
        }
        if raw.len() > MAX_CATEGORIES {
            return Err(format!("at most {MAX_CATEGORIES} categories"));
        }
        let mut categories = BTreeMap::new();
        for (category, values) in raw {
            if !token_ok(&category) {
                return Err(format!("`{category}` is not a lowercase token"));
            }
            if values.is_empty() || values.len() > MAX_VALUES {
                return Err(format!("`{category}` needs 1 to {MAX_VALUES} values"));
            }
            let mut seen = BTreeSet::new();
            for value in &values {
                if !token_ok(value) {
                    return Err(format!(
                        "`{value}` in `{category}` is not a lowercase token"
                    ));
                }
                if value == PREFER_NOT_TO_SAY {
                    return Err(format!(
                        "`{PREFER_NOT_TO_SAY}` is always offered; do not list it in `{category}`"
                    ));
                }
                if !seen.insert(value.clone()) {
                    return Err(format!("`{value}` is listed twice in `{category}`"));
                }
            }
            categories.insert(category, values);
        }
        Ok(Self { categories })
    }

    /// The categories and their allowed values, each with `prefer_not_to_say` last.
    #[must_use]
    pub fn offered(&self) -> BTreeMap<String, Vec<String>> {
        self.categories
            .iter()
            .map(|(c, v)| {
                let mut all = v.clone();
                all.push(PREFER_NOT_TO_SAY.to_string());
                (c.clone(), all)
            })
            .collect()
    }

    /// Whether the category is configured.
    #[must_use]
    pub fn has_category(&self, category: &str) -> bool {
        self.categories.contains_key(category)
    }

    /// Check one declaration.
    ///
    /// # Errors
    ///
    /// An unknown category, or a value the category does not offer.
    pub fn validate(&self, category: &str, value: &str) -> Result<(), String> {
        let values = self
            .categories
            .get(category)
            .ok_or_else(|| format!("`{category}` is not a category this deployment monitors"))?;
        if value == PREFER_NOT_TO_SAY || values.iter().any(|v| v == value) {
            Ok(())
        } else {
            Err(format!("`{value}` is not offered for `{category}`"))
        }
    }
}

/// Whether monitoring is on, and with what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    /// No lawful basis is recorded: nothing is collected, stored or shown.
    Off,
    /// A deployer has recorded a basis and configured the categories.
    On {
        /// The lawful basis, in the deployer's words (for example a reference to their assessment).
        basis: String,
        /// The categories and values.
        config: Config,
        /// The small-group floor.
        floor: usize,
    },
}

/// Decide the gate from the three settings. Monitoring is on **only** when a lawful basis is
/// recorded; with a basis, the categories must be valid and the floor, if given, at least
/// [`MIN_FLOOR`], or this is an error so a half-configured deployment does not start.
///
/// # Errors
///
/// A basis without valid categories, or an invalid floor.
pub fn gate(
    basis: Option<&str>,
    categories: Option<&str>,
    floor: Option<&str>,
) -> Result<Gate, String> {
    let Some(basis) = basis.map(str::trim).filter(|b| !b.is_empty()) else {
        return Ok(Gate::Off);
    };
    let categories = categories
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .ok_or_else(|| {
            "WPM_EQUALITY_MONITORING_BASIS is set but WPM_EQUALITY_CATEGORIES is not: the categories are yours to define (WPM ships none)".to_string()
        })?;
    let config = Config::parse(categories)?;
    let floor = match floor.map(str::trim).filter(|f| !f.is_empty()) {
        None => DEFAULT_FLOOR,
        Some(text) => {
            let n: usize = text
                .parse()
                .map_err(|_| "WPM_EQUALITY_FLOOR is not a whole number".to_string())?;
            validate_floor(n)?;
            n
        }
    };
    Ok(Gate::On {
        basis: basis.to_string(),
        config,
        floor,
    })
}

/// A floor from configuration: at least [`MIN_FLOOR`].
///
/// # Errors
///
/// A value below the minimum.
pub fn validate_floor(floor: usize) -> Result<(), String> {
    if floor >= MIN_FLOOR {
        Ok(())
    } else {
        Err(format!("the small-group floor cannot be below {MIN_FLOOR}"))
    }
}

/// One category's aggregate. The figures are counts of people; they name no one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// Workers counted (employed on the day).
    pub headcount: usize,
    /// Workers who have answered this category, with any answer, `prefer_not_to_say` included.
    pub responded: usize,
    /// Responses as a percentage of headcount, rounded down; `None` with nobody employed. Names no value.
    pub completeness_percent: Option<usize>,
    /// Counts per value across the organization; each cell shows only when it reaches the floor,
    /// else `None`. `None` altogether when the responses are too few to show anything.
    pub organization: Option<BTreeMap<String, Option<usize>>>,
    /// Counts per department then value, **only when every department and every cell reaches the
    /// floor**, so a withheld cell cannot be worked out from a total and the others.
    pub by_department: Option<BTreeMap<String, BTreeMap<String, usize>>>,
}

/// Aggregate one category. `rows` is `(department, value)` for each response; `departments` is the
/// headcount per department of everyone employed (responded or not).
#[must_use]
pub fn summarise(
    rows: &[(String, String)],
    departments: &BTreeMap<String, usize>,
    floor: usize,
) -> Summary {
    let headcount: usize = departments.values().sum();
    let responded = rows.len();
    let mut org: BTreeMap<String, usize> = BTreeMap::new();
    let mut dept: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for (department, value) in rows {
        *org.entry(value.clone()).or_insert(0) += 1;
        *dept
            .entry(department.clone())
            .or_default()
            .entry(value.clone())
            .or_insert(0) += 1;
    }
    let organization = (responded >= floor).then(|| {
        org.iter()
            .map(|(v, n)| (v.clone(), (*n >= floor).then_some(*n)))
            .collect()
    });
    let departments_ok = !dept.is_empty()
        && dept.len() == departments.len()
        && dept
            .values()
            .all(|cells| cells.values().all(|n| *n >= floor));
    Summary {
        headcount,
        responded,
        completeness_percent: (headcount > 0).then(|| responded * 100 / headcount),
        organization,
        by_department: departments_ok.then_some(dept),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"religion_or_belief":["none","other"],"sex":["female","male"]}"#;

    #[test]
    fn the_configuration_is_the_deployers_and_always_offers_prefer_not_to_say() {
        let config = Config::parse(GOOD).unwrap();
        let offered = config.offered();
        assert_eq!(offered["sex"], vec!["female", "male", "prefer_not_to_say"]);
        assert!(config.validate("sex", "female").is_ok());
        assert!(config.validate("sex", "prefer_not_to_say").is_ok());
        assert!(config.validate("sex", "robot").is_err());
        assert!(config.validate("age", "30").is_err(), "an unknown category");
        assert!(config.has_category("sex") && !config.has_category("age"));
    }

    #[test]
    fn a_bad_configuration_is_refused_with_the_reason() {
        for (json, needle) in [
            ("{}", "at least one"),
            ("[]", "JSON object"),
            ("not json", "JSON object"),
            (r#"{"Sex":["a"]}"#, "lowercase"),
            (r#"{"sex":[]}"#, "needs 1 to"),
            (r#"{"sex":["Female"]}"#, "lowercase"),
            (r#"{"sex":["a","a"]}"#, "twice"),
            (r#"{"sex":["prefer_not_to_say"]}"#, "always offered"),
        ] {
            let err = Config::parse(json).unwrap_err();
            assert!(err.contains(needle), "{json}: {err}");
        }
        let many: BTreeMap<String, Vec<String>> = (0..=MAX_CATEGORIES)
            .map(|i| (format!("c{i}"), vec!["a".to_string()]))
            .collect();
        assert!(Config::parse(&serde_json::to_string(&many).unwrap()).is_err());
        let wide = serde_json::json!({ "c": (0..=MAX_VALUES).map(|i| format!("v{i}")).collect::<Vec<_>>() });
        assert!(Config::parse(&wide.to_string()).is_err());
    }

    #[test]
    fn it_is_off_unless_a_lawful_basis_is_recorded() {
        assert_eq!(
            gate(None, Some(GOOD), None),
            Ok(Gate::Off),
            "categories alone switch nothing on"
        );
        assert_eq!(gate(Some("  "), Some(GOOD), None), Ok(Gate::Off));
        assert_eq!(gate(None, None, None), Ok(Gate::Off));
        let on = gate(Some("Our assessment ref 12"), Some(GOOD), None).unwrap();
        match on {
            Gate::On {
                basis,
                floor,
                config,
            } => {
                assert_eq!(basis, "Our assessment ref 12");
                assert_eq!(floor, DEFAULT_FLOOR);
                assert!(config.has_category("sex"));
            }
            Gate::Off => panic!("expected on"),
        }
        assert!(gate(Some("b"), Some(GOOD), Some("25")).is_ok());
        // A basis with no categories, bad categories, or a floor below the minimum does not start.
        assert!(
            gate(Some("b"), None, None)
                .unwrap_err()
                .contains("yours to define")
        );
        assert!(gate(Some("b"), Some("{}"), None).is_err());
        assert!(gate(Some("b"), Some(GOOD), Some("4")).is_err());
        assert!(gate(Some("b"), Some(GOOD), Some("ten")).is_err());
    }

    /// Nothing else may read these declarations: no score, ranking or decision may take them as
    /// an input. Only the module, the entity, the subject-access export and the wiring may name them.
    #[test]
    fn no_other_code_reads_the_declarations() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let allowed = [
            "controllers/equality_monitoring.rs",
            "controllers/privacy.rs",
            "models/_entities/equality_declarations.rs",
            "models/_entities/mod.rs",
            "models/_entities/prelude.rs",
            "app.rs",
            "rules/equality_monitoring.rs",
        ];
        let mut stack = vec![root.clone()];
        let mut offenders = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    let rel = path
                        .strip_prefix(&root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    let text = std::fs::read_to_string(&path).unwrap();
                    if (text.contains("equality_declarations")
                        || text.contains("EqualityDeclarations"))
                        && !allowed.contains(&rel.as_str())
                    {
                        offenders.push(rel);
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "code that reads equality declarations: {offenders:?}"
        );
    }

    #[test]
    fn the_floor_has_a_minimum() {
        assert!(validate_floor(MIN_FLOOR).is_ok() && validate_floor(DEFAULT_FLOOR).is_ok());
        assert!(validate_floor(MIN_FLOOR - 1).is_err());
        assert!(validate_floor(0).is_err());
    }

    fn rows(items: &[(&str, &str, usize)]) -> Vec<(String, String)> {
        items
            .iter()
            .flat_map(|(d, v, n)| (0..*n).map(move |_| ((*d).to_string(), (*v).to_string())))
            .collect()
    }

    fn heads(items: &[(&str, usize)]) -> BTreeMap<String, usize> {
        items.iter().map(|(d, n)| ((*d).to_string(), *n)).collect()
    }

    #[test]
    fn completeness_names_no_value() {
        let s = summarise(
            &rows(&[("eng", "a", 3), ("eng", "prefer_not_to_say", 1)]),
            &heads(&[("eng", 8)]),
            10,
        );
        assert_eq!(
            (s.headcount, s.responded, s.completeness_percent),
            (8, 4, Some(50))
        );
        assert!(s.organization.is_none(), "four answers are below the floor");
        assert!(s.by_department.is_none());
        let none = summarise(&[], &heads(&[]), 10);
        assert_eq!(none.completeness_percent, None, "no one employed: not zero");
    }

    #[test]
    fn a_cell_below_the_floor_is_withheld_and_departments_only_show_when_all_pass() {
        // 24 answers across two departments; one value has only 2.
        let data = rows(&[("eng", "a", 10), ("eng", "b", 2), ("ops", "a", 12)]);
        let s = summarise(&data, &heads(&[("eng", 15), ("ops", 15)]), 10);
        let org = s.organization.unwrap();
        assert_eq!(org["a"], Some(22));
        assert_eq!(org["b"], None, "two is below the floor");
        assert!(
            s.by_department.is_none(),
            "eng has a cell of 2, so no department breakdown at all"
        );
        // When every cell of every department reaches the floor, the breakdown shows.
        let all = rows(&[
            ("eng", "a", 10),
            ("eng", "b", 10),
            ("ops", "a", 11),
            ("ops", "b", 10),
        ]);
        let ok = summarise(&all, &heads(&[("eng", 20), ("ops", 21)]), 10);
        let by = ok.by_department.unwrap();
        assert_eq!(by["eng"]["b"], 10);
        assert_eq!(by["ops"]["a"], 11);
        // A department with no answers at all also blocks the breakdown.
        let silent = summarise(&all, &heads(&[("eng", 20), ("ops", 21), ("legal", 6)]), 10);
        assert!(silent.by_department.is_none());
    }

    #[test]
    fn the_floor_is_inclusive() {
        let data = rows(&[("eng", "a", 10)]);
        let s = summarise(&data, &heads(&[("eng", 10)]), 10);
        assert_eq!(s.organization.unwrap()["a"], Some(10));
        let below = summarise(&rows(&[("eng", "a", 9)]), &heads(&[("eng", 10)]), 10);
        assert!(below.organization.is_none());
    }
}
