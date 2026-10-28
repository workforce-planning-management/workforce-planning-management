//! The scoring register in code (WPM-R99, WPM-T166), DB-free and clock-free. Spec: `spec/equality-impact/register.md`.
//!
//! Every score WPM derives about people is listed here with the decision it informs, where its assessment is and what
//! state that assessment is in. **Nothing is approved by anyone yet**: a status is what the assessment says, never
//! what would be convenient. [`ready_for_use`] is the gate a deployer's own approval would open; today it is closed for
//! every score, and a test pins that, so approving one is a deliberate edit.
//!
//! A lint test reads `src/rules/*.rs` and names any new `pub fn` whose name contains score, rank, rating, band,
//! readiness or risk that is neither registered here nor exempt with a reason, so a new score cannot arrive unlisted.

use chrono::NaiveDate;
use serde::Serialize;

/// What kind of figure a score is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A person's judgement, kept as entered.
    Rating,
    /// A stage a person moves someone through.
    Stage,
    /// A band worked out by fixed rules.
    Band,
    /// A ratio or count worked out by rules.
    Ratio,
    /// An ordering worked out by rules.
    Order,
    /// A flag against a threshold, as aggregates.
    Flag,
    /// A level stated on one scale and counted on another by a stated rule.
    Mapping,
}

/// Where an assessment stands. Only what the assessment says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Written, not approved by anyone.
    Draft,
    /// Approved with conditions that must be met.
    Conditional,
    /// Approved.
    Approved,
}

/// One registered score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// The register id (`S1`). `S9`, a risk-of-loss score, is deliberately not built and has no entry.
    pub id: &'static str,
    /// What it is.
    pub name: &'static str,
    /// What kind of figure.
    pub kind: Kind,
    /// What it is made of.
    pub inputs: &'static str,
    /// The decision it informs.
    pub informs: &'static str,
    /// The assessment's section in `spec/equality-impact/register.md`, or `None` when none is written.
    pub assessment: Option<&'static str>,
    /// Where the assessment stands.
    pub status: Status,
    /// When the assessment must be reviewed by; `None` until someone sets it.
    pub review_by: Option<NaiveDate>,
    /// The code that makes it (`rules::module::item`).
    pub code: &'static [&'static str],
}

/// Every registered score.
pub const REGISTER: &[Entry] = &[
    Entry {
        id: "S1",
        name: "Succession readiness",
        kind: Kind::Rating,
        inputs: "a person's judgement",
        informs: "a succession shortlist",
        assessment: Some("S1"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::talent::READINESS"],
    },
    Entry {
        id: "S2",
        name: "Talent pipeline stage",
        kind: Kind::Stage,
        inputs: "a person's judgement",
        informs: "development offers and placement",
        assessment: Some("S2"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::talent::PIPELINE_STAGES"],
    },
    Entry {
        id: "S3",
        name: "Assessment score band",
        kind: Kind::Band,
        inputs: "a raw test score and the deployer's norm group",
        informs: "selection and development",
        assessment: Some("S3"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::assessment::band_for_percentile"],
    },
    Entry {
        id: "S4",
        name: "Review rating and competency mean",
        kind: Kind::Rating,
        inputs: "a reviewer's ratings",
        informs: "pay review and development",
        assessment: Some("S4"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::appraisal::competency_mean"],
    },
    Entry {
        id: "S5",
        name: "Internal-mobility fit",
        kind: Kind::Ratio,
        inputs: "a role's critical requirements against declared skills",
        informs: "the worker's own ordering of roles",
        assessment: Some("S5"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::mobility::fit_score", "rules::gap::readiness"],
    },
    Entry {
        id: "S6",
        name: "Skill-gap priority",
        kind: Kind::Order,
        inputs: "demand against declared supply, as aggregates",
        informs: "training spend and hiring",
        assessment: Some("S6"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::skill_gap::priority", "rules::skill_gap::rank"],
    },
    Entry {
        id: "S7",
        name: "Change readiness",
        kind: Kind::Ratio,
        inputs: "skills meeting a bar, as aggregates",
        informs: "reskilling plans",
        assessment: Some("S7"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::change::skill_readiness"],
    },
    Entry {
        id: "S8",
        name: "Workforce insight flags",
        kind: Kind::Flag,
        inputs: "turnover, span of control and time to fill, as aggregates",
        informs: "leadership attention",
        assessment: Some("S8"),
        status: Status::Draft,
        review_by: None,
        code: &["rules::insights"],
    },
    Entry {
        id: "S10",
        name: "Stated skill level counted on WPM's 1–5",
        kind: Kind::Mapping,
        inputs: "a level stated on a framework's own scale, and the stated linear mapping",
        informs: "gap, plan and requirement views",
        assessment: None,
        status: Status::Draft,
        review_by: None,
        code: &["rules::sfia::to_wpm", "rules::framework::map_level"],
    },
];

/// A score may be used to inform a decision only when its assessment exists, is approved (with or without conditions) and its review
/// date is set and not passed. `today` is passed in: this module reads no clock.
#[must_use]
pub fn ready_for_use(entry: &Entry, today: NaiveDate) -> bool {
    entry.assessment.is_some()
        && matches!(entry.status, Status::Conditional | Status::Approved)
        && entry.review_by.is_some_and(|d| d >= today)
}

/// The ids of registered scores with no assessment written.
#[must_use]
pub fn unassessed() -> Vec<&'static str> {
    REGISTER
        .iter()
        .filter(|e| e.assessment.is_none())
        .map(|e| e.id)
        .collect()
}

/// The entry with this id.
#[must_use]
pub fn find(id: &str) -> Option<&'static Entry> {
    REGISTER.iter().find(|e| e.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const REGISTER_DOC: &str = include_str!("../../../spec/equality-impact/register.md");

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// The text of the register document's section for an id (a heading may name several ids, as `S6` and `S7` share one).
    fn section(id: &str) -> Option<&'static str> {
        let marker = format!("{id} ");
        let start = REGISTER_DOC
            .match_indices("\n## ")
            .map(|(i, _)| i + 1)
            .find(|&i| {
                REGISTER_DOC[i..]
                    .lines()
                    .next()
                    .is_some_and(|h| h.contains(&marker))
            })?;
        let rest = &REGISTER_DOC[start + 3..];
        Some(&rest[..rest.find("\n## ").unwrap_or(rest.len())])
    }

    #[test]
    fn ids_are_unique_and_s9_is_not_built() {
        let ids: BTreeSet<&str> = REGISTER.iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), REGISTER.len());
        assert!(
            !ids.contains("S9"),
            "a risk-of-loss score is deliberately not built (WPM-D67)"
        );
    }

    #[test]
    fn every_ai_register_feature_is_a_registered_score() {
        for f in super::super::ai_register::FEATURES {
            assert!(
                find(f.id).is_some(),
                "{} is in the AI register but not the scoring register",
                f.id
            );
        }
    }

    #[test]
    fn each_assessment_names_a_section_that_exists_and_says_what_its_status_is() {
        for e in REGISTER {
            let Some(anchor) = e.assessment else { continue };
            let text = section(anchor)
                .unwrap_or_else(|| panic!("{} has no section {anchor} in register.md", e.id));
            let says_not_approved = text.to_lowercase().contains("not approved by anyone");
            assert_eq!(
                says_not_approved,
                e.status == Status::Draft,
                "{}: the register says {:?} but the assessment text disagrees",
                e.id,
                e.status
            );
        }
        for e in REGISTER {
            assert!(
                REGISTER_DOC.contains(&format!("| {} |", e.id)),
                "{} is not in the table in register.md",
                e.id
            );
        }
    }

    #[test]
    fn a_score_without_an_assessment_is_named_and_is_never_ready() {
        assert_eq!(
            unassessed(),
            ["S10"],
            "adding or assessing a score is a deliberate edit to this list"
        );
        let mut e = *find("S10").unwrap();
        e.status = Status::Approved;
        e.review_by = Some(day(2099, 1, 1));
        assert!(
            !ready_for_use(&e, day(2026, 10, 27)),
            "approval without an assessment counts for nothing"
        );
    }

    #[test]
    fn the_gate_needs_an_approval_and_a_review_date_not_yet_passed() {
        let mut e = *find("S5").unwrap();
        let today = day(2026, 10, 27);
        assert!(!ready_for_use(&e, today), "a draft is not ready");
        e.status = Status::Conditional;
        assert!(!ready_for_use(&e, today), "no review date set");
        e.review_by = Some(day(2026, 10, 26));
        assert!(!ready_for_use(&e, today), "review date passed");
        e.review_by = Some(today);
        assert!(ready_for_use(&e, today), "the day itself is still in date");
        e.status = Status::Approved;
        assert!(ready_for_use(&e, today));
    }

    /// Nothing is approved by anyone, and the register says so (WPM-T166 asked for approved-or-conditional; that cannot be true yet).
    #[test]
    fn today_no_score_is_ready_for_use() {
        let today = day(2026, 10, 27);
        let ready: Vec<&str> = REGISTER
            .iter()
            .filter(|e| ready_for_use(e, today))
            .map(|e| e.id)
            .collect();
        assert!(
            ready.is_empty(),
            "{ready:?} became ready: check an approval was actually given"
        );
    }

    /// Functions whose names look like scores but are not, each with the reason.
    const EXEMPT: &[(&str, &str)] = &[
        (
            "check_scores",
            "validates the ratings a reviewer entered; computes nothing about a person",
        ),
        (
            "valid_raw_score",
            "validates a raw test score; the band is S3",
        ),
        (
            "valid_score",
            "validates an anonymous pulse answer; no person is named",
        ),
        (
            "valid_age_band",
            "validates a band the deployer chose for a wellbeing answer",
        ),
        (
            "resolve_band",
            "looks up a recorded pay band; a band is a fact someone recorded, not a derived score",
        ),
        ("band", "looks up a pay band on a published scale"),
        (
            "risk_deviation",
            "counts critical roles with no ready successor in a plan: roles, not people",
        ),
    ];

    fn registered_fn_names() -> BTreeSet<&'static str> {
        REGISTER
            .iter()
            .flat_map(|e| e.code.iter())
            .filter_map(|p| p.rsplit("::").next())
            .collect()
    }

    fn scorelike_pub_fns() -> BTreeSet<String> {
        const WORDS: [&str; 6] = ["score", "rank", "rating", "band", "readiness", "risk"];
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
        let mut found = BTreeSet::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|x| x != "rs")
                || path.file_name().is_some_and(|n| n == "scoring.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for line in text.lines() {
                let t = line.trim_start();
                let Some(rest) = t
                    .strip_prefix("pub fn ")
                    .or_else(|| t.strip_prefix("pub const fn "))
                else {
                    continue;
                };
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if WORDS.iter().any(|w| name.contains(w)) {
                    found.insert(name);
                }
            }
        }
        found
    }

    #[test]
    fn a_new_score_like_function_must_be_registered_or_exempt_with_a_reason() {
        let registered = registered_fn_names();
        let exempt: BTreeSet<&str> = EXEMPT.iter().map(|(n, _)| *n).collect();
        let unlisted: Vec<String> = scorelike_pub_fns()
            .into_iter()
            .filter(|n| !registered.contains(n.as_str()) && !exempt.contains(n.as_str()))
            .collect();
        assert!(
            unlisted.is_empty(),
            "{unlisted:?}: a function that derives a score, rank, rating, band, readiness or risk must be in rules::scoring::REGISTER \
             with an assessment (spec/equality-impact/template.md), or in EXEMPT with a reason"
        );
    }

    #[test]
    fn the_exemptions_are_not_stale_and_every_reason_is_given() {
        let present = scorelike_pub_fns();
        for (name, reason) in EXEMPT {
            assert!(
                present.contains(*name),
                "{name} is exempt but no longer exists: remove it"
            );
            assert!(reason.len() > 15, "{name}: say why");
        }
    }
}
