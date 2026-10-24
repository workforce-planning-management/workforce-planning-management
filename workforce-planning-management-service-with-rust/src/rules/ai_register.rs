//! The AI and scoring deployer register (WPM-R148, WPM-D103), DB-free and clock-free. Spec: `spec/regulatory.md`.
//!
//! Where a deployer uses a feature that scores or ranks people as a high-risk AI system in employment, Article 26 of the
//! AI Act puts duties on it. The register **records** them per feature; it does not classify a system, certify
//! conformity or count as evidence of compliance. Nothing recorded is `unknown`, never "overseen", "informed" or "kept".

use chrono::NaiveDate;
use serde::Serialize;

/// A feature that derives something about people, from the scoring register of the equality impact assessments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Feature {
    /// The register id (`S1`).
    pub id: &'static str,
    /// What it is.
    pub name: &'static str,
    /// Who or what works it out.
    pub computed_by: &'static str,
    /// The decision it informs.
    pub informs: &'static str,
    /// What it is made of, in a sentence a worker can read.
    pub made_of: &'static str,
}

/// The features, from `spec/equality-impact/register.md` (S1 to S8). S9, a risk-of-loss score, is deliberately not built.
pub const FEATURES: &[Feature] = &[
    Feature {
        id: "S1",
        name: "Succession readiness",
        computed_by: "a person",
        informs: "a succession shortlist",
        made_of: "A manager or HR rates a candidate ready now, in one year or in two. No input is computed.",
    },
    Feature {
        id: "S2",
        name: "Talent pipeline stage",
        computed_by: "a person",
        informs: "development offers and placement",
        made_of: "A person moves a candidate through the stages from identified to placed.",
    },
    Feature {
        id: "S3",
        name: "Assessment score band",
        computed_by: "rules (norm-referenced)",
        informs: "selection and development",
        made_of: "A raw test score turned into a percentile band by fixed cut points against the deployer's norm group.",
    },
    Feature {
        id: "S4",
        name: "Review rating and competency mean",
        computed_by: "a person; rules for the mean",
        informs: "pay review and development",
        made_of: "A reviewer's rating, and the mean of the competency ratings given.",
    },
    Feature {
        id: "S5",
        name: "Internal-mobility fit",
        computed_by: "rules",
        informs: "the worker's own ordering of roles",
        made_of: "How many of a role's critical skill requirements the worker's declared skills meet.",
    },
    Feature {
        id: "S6",
        name: "Skill-gap priority",
        computed_by: "rules",
        informs: "training spend and hiring",
        made_of: "Skill gaps ordered by how far demand exceeds declared supply, as aggregates.",
    },
    Feature {
        id: "S7",
        name: "Change readiness",
        computed_by: "rules",
        informs: "reskilling plans",
        made_of: "How many skills meet a bar, as aggregates.",
    },
    Feature {
        id: "S8",
        name: "Workforce insight flags",
        computed_by: "rules",
        informs: "leadership attention",
        made_of: "Turnover, span of control and time to fill against thresholds, as aggregates.",
    },
];

/// Every person-facing statement a feature shares: it advises, and a person decides.
pub const ADVISORY: &str =
    "Advisory. A person decides; nothing here is an automatic decision about you.";

/// The Annex III point 4 category a deployer chose. `not_high_risk` is the deployer's statement, not WPM's.
pub const CATEGORIES: &[&str] = &[
    "recruitment_selection",
    "work_relationship_decisions",
    "task_allocation",
    "monitoring_evaluation",
    "not_high_risk",
];

/// The shortest log retention Article 26(6) names, unless other law says otherwise.
pub const MINIMUM_LOG_MONTHS: i32 = 6;

/// When the Annex III duties apply, unless configured otherwise (as the amended Article 113 shows it).
///
/// # Panics
///
/// Never: the date is a constant.
#[must_use]
pub fn default_applies_from() -> NaiveDate {
    NaiveDate::from_ymd_opt(2027, 12, 2).expect("a valid date")
}

/// The feature with this id.
#[must_use]
pub fn feature(id: &str) -> Option<&'static Feature> {
    FEATURES.iter().find(|f| f.id == id)
}

/// What a deployer records for one feature. Every field may be absent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    /// An Annex III category from [`CATEGORIES`].
    pub annex_category: Option<String>,
    /// The role responsible for human oversight; a role, not a person.
    pub overseer_role: Option<String>,
    /// When workers' representatives were informed.
    pub representatives_informed_on: Option<NaiveDate>,
    /// When the affected workers were informed.
    pub workers_informed_on: Option<NaiveDate>,
    /// How long the system's logs are kept, in months.
    pub log_retention_months: Option<i32>,
    /// Why retention is shorter than the minimum, where other law says so.
    pub retention_note: Option<String>,
}

/// Check an entry made `today`. Returns the problems found.
#[must_use]
pub fn validate(e: &Entry, today: NaiveDate) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(c) = &e.annex_category
        && !CATEGORIES.contains(&c.as_str())
    {
        out.push(format!(
            "annex_category must be one of {}",
            CATEGORIES.join(", ")
        ));
    }
    if let Some(role) = &e.overseer_role {
        let n = role.trim().chars().count();
        if !(1..=120).contains(&n) {
            out.push("overseer_role is 1 to 120 characters".to_string());
        }
        if role.contains('@') {
            out.push("overseer_role is a role, not a person or an address".to_string());
        }
    }
    for (name, date) in [
        ("representatives_informed_on", e.representatives_informed_on),
        ("workers_informed_on", e.workers_informed_on),
    ] {
        if date.is_some_and(|d| d > today) {
            out.push(format!(
                "{name} is a date that has happened, not one to come"
            ));
        }
    }
    match e.log_retention_months {
        Some(m) if !(1..=240).contains(&m) => out.push("log_retention_months is 1 to 240".to_string()),
        Some(m) if m < MINIMUM_LOG_MONTHS && e.retention_note.as_deref().is_none_or(|n| n.trim().is_empty()) => out.push(format!(
            "a retention under {MINIMUM_LOG_MONTHS} months needs retention_note saying what other law requires it"
        )),
        _ => {}
    }
    if e.retention_note
        .as_ref()
        .is_some_and(|n| n.chars().count() > 500)
    {
        out.push("retention_note is at most 500 characters".to_string());
    }
    out
}

/// A recorded value, or the plain fact that nothing is recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Field<T> {
    /// Nothing recorded: not "none", not "yes".
    Unknown,
    /// Recorded.
    Recorded {
        /// The value.
        value: T,
    },
}

fn field<T>(v: Option<T>) -> Field<T> {
    v.map_or(Field::Unknown, |value| Field::Recorded { value })
}

/// One feature with what is recorded for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Row {
    /// The feature.
    pub feature: Feature,
    /// What the feature says to the people it is used on.
    pub advisory: &'static str,
    /// The Annex III category.
    pub annex_category: Field<String>,
    /// The oversight role.
    pub overseer_role: Field<String>,
    /// When representatives were informed.
    pub representatives_informed_on: Field<NaiveDate>,
    /// When workers were informed.
    pub workers_informed_on: Field<NaiveDate>,
    /// Log retention in months.
    pub log_retention_months: Field<i32>,
    /// Why retention is short.
    pub retention_note: Field<String>,
    /// Whether the recorded retention is under the six months, which other law must then explain.
    pub retention_below_minimum: bool,
}

/// The row for a feature, whether or not anything is recorded.
#[must_use]
pub fn row(f: &Feature, e: Option<&Entry>) -> Row {
    let none = Entry::default();
    let e = e.unwrap_or(&none);
    Row {
        feature: *f,
        advisory: ADVISORY,
        annex_category: field(e.annex_category.clone()),
        overseer_role: field(e.overseer_role.clone()),
        representatives_informed_on: field(e.representatives_informed_on),
        workers_informed_on: field(e.workers_informed_on),
        log_retention_months: field(e.log_retention_months),
        retention_note: field(e.retention_note.clone()),
        retention_below_minimum: e
            .log_retention_months
            .is_some_and(|m| m < MINIMUM_LOG_MONTHS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn nothing_recorded_is_unknown_in_every_field() {
        let r = row(feature("S3").unwrap(), None);
        assert_eq!(r.annex_category, Field::Unknown);
        assert_eq!(r.overseer_role, Field::Unknown);
        assert_eq!(r.representatives_informed_on, Field::Unknown);
        assert_eq!(r.workers_informed_on, Field::Unknown);
        assert_eq!(r.log_retention_months, Field::Unknown);
        assert!(!r.retention_below_minimum, "unknown is not short");
        let text = serde_json::to_string(&r).unwrap();
        assert!(
            !text.contains("overseen") && !text.contains("compliant"),
            "{text}"
        );
    }

    #[test]
    fn a_recorded_entry_is_shown_as_recorded() {
        let e = Entry {
            annex_category: Some("recruitment_selection".into()),
            overseer_role: Some("Head of Resourcing".into()),
            workers_informed_on: Some(d(2027, 6, 1)),
            log_retention_months: Some(12),
            ..Entry::default()
        };
        let r = row(feature("S3").unwrap(), Some(&e));
        assert_eq!(
            r.overseer_role,
            Field::Recorded {
                value: "Head of Resourcing".into()
            }
        );
        assert_eq!(
            r.representatives_informed_on,
            Field::Unknown,
            "only what was recorded"
        );
        assert!(!r.retention_below_minimum);
    }

    #[test]
    fn a_valid_entry_has_no_problems_and_each_fault_is_named() {
        let today = d(2026, 10, 24);
        let ok = Entry {
            annex_category: Some("monitoring_evaluation".into()),
            overseer_role: Some("HR Director".into()),
            representatives_informed_on: Some(today),
            log_retention_months: Some(6),
            ..Entry::default()
        };
        assert_eq!(validate(&ok, today), Vec::<String>::new());
        assert_eq!(
            validate(&Entry::default(), today),
            Vec::<String>::new(),
            "an empty entry clears a record"
        );
        let bad = |e: Entry| validate(&e, today).join("; ");
        assert!(
            bad(Entry {
                annex_category: Some("high_risk".into()),
                ..Entry::default()
            })
            .contains("annex_category")
        );
        assert!(
            bad(Entry {
                overseer_role: Some("   ".into()),
                ..Entry::default()
            })
            .contains("overseer_role")
        );
        assert!(
            bad(Entry {
                overseer_role: Some("a.person@example.org".into()),
                ..Entry::default()
            })
            .contains("not a person")
        );
        assert!(
            bad(Entry {
                workers_informed_on: Some(d(2026, 10, 25)),
                ..Entry::default()
            })
            .contains("workers_informed_on")
        );
        assert!(
            bad(Entry {
                log_retention_months: Some(0),
                ..Entry::default()
            })
            .contains("1 to 240")
        );
        assert!(
            bad(Entry {
                log_retention_months: Some(241),
                ..Entry::default()
            })
            .contains("1 to 240")
        );
    }

    #[test]
    fn a_retention_under_six_months_needs_the_other_law_named() {
        let today = d(2026, 10, 24);
        let short = Entry {
            log_retention_months: Some(3),
            ..Entry::default()
        };
        assert!(validate(&short, today).join(" ").contains("retention_note"));
        let explained = Entry {
            retention_note: Some("Data protection law requires deletion at three months.".into()),
            ..short
        };
        assert_eq!(validate(&explained, today), Vec::<String>::new());
        assert!(row(feature("S1").unwrap(), Some(&explained)).retention_below_minimum);
        let exactly = Entry {
            log_retention_months: Some(6),
            ..Entry::default()
        };
        assert!(!row(feature("S1").unwrap(), Some(&exactly)).retention_below_minimum);
    }

    #[test]
    fn the_features_are_the_register_s1_to_s8_with_no_risk_of_loss() {
        let ids: Vec<&str> = FEATURES.iter().map(|f| f.id).collect();
        assert_eq!(ids, ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8"]);
        assert!(
            feature("S9").is_none(),
            "a risk-of-loss score is deliberately not built"
        );
        for f in FEATURES {
            assert!(!f.made_of.is_empty() && !f.informs.is_empty() && !f.name.is_empty());
        }
    }

    #[test]
    fn the_date_the_duties_apply_is_a_default_for_configuration_to_override() {
        assert_eq!(default_applies_from(), d(2027, 12, 2));
    }
}
