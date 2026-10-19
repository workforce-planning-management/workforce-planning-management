//! The matching engine: configuration, scoring and the explainable result.
//!
//! Rules the engine keeps, and the tests pin:
//!
//! 1. **An unknown is never a zero.** A field missing on either side, or that
//!    cannot be normalised, scores `None` and is left out of the average; it
//!    neither helps nor hurts.
//! 2. **Too little evidence is not a verdict.** Fewer than `min_fields`
//!    comparable fields gives [`Decision::Insufficient`] and no score.
//! 3. **Say what the answer rests on.** [`Basis`] says whether a match came
//!    from a shared employer worker number or from the weighted fields, and the
//!    [`Breakdown`] shows every field's score.
//! 4. **Disagreement on a stable fact cannot be averaged away.** A date of birth
//!    that differs, or a different worker number inside the same organization,
//!    caps the decision at [`Decision::Review`]: a person decides, not a threshold.
//! 5. **Symmetric and stateless.** `match_workers(a, b)` equals `match_workers(b, a)`;
//!    the engine holds configuration only and is `Send + Sync`.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::error::MatchError;
use crate::nicknames::NicknameTable;
use crate::normalizer;
use crate::record::WorkerRecord;
use crate::scorer::combined_similarity;

/// How much each field counts toward the weighted score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Weights {
    pub given_name: f64,
    pub family_name: f64,
    pub date_of_birth: f64,
    pub email: f64,
    pub phone: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            given_name: 2.0,
            family_name: 3.0,
            date_of_birth: 3.0,
            email: 3.0,
            phone: 1.5,
        }
    }
}

/// Tuning for [`MatchingEngine`]. Serialises, so it can be loaded from a file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MatchConfig {
    pub weights: Weights,
    /// Score at or above which two records are a [`Decision::Match`].
    pub match_threshold: f64,
    /// Score at or above which a person should look: [`Decision::Review`].
    pub review_threshold: f64,
    /// Comparable fields needed before a score means anything.
    pub min_fields: usize,
    /// Count nicknames (Mike / Michael) as a near-exact given name.
    pub use_nicknames: bool,
    /// Treat `j.smith+x@gmail.com` and `jsmith@gmail.com` as one mailbox.
    pub fold_gmail: bool,
    /// Country calling code (digits, no `+`) for numbers written with a leading `0`.
    pub default_phone_country_code: Option<String>,
    /// A shared employer worker number settles the match without the fields.
    pub worker_number_is_deterministic: bool,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            weights: Weights::default(),
            match_threshold: 0.88,
            review_threshold: 0.70,
            min_fields: 2,
            use_nicknames: true,
            fold_gmail: false,
            default_phone_country_code: None,
            worker_number_is_deterministic: true,
        }
    }
}

impl MatchConfig {
    /// For a decision nobody will review: higher bar, no nickname leniency.
    pub fn strict() -> Self {
        Self {
            match_threshold: 0.95,
            review_threshold: 0.85,
            min_fields: 3,
            use_nicknames: false,
            ..Self::default()
        }
    }

    /// For triage of a large candidate list, where a missed duplicate costs more than a look.
    pub fn lenient() -> Self {
        Self {
            match_threshold: 0.82,
            review_threshold: 0.60,
            ..Self::default()
        }
    }

    /// Refuse a configuration that cannot give a meaningful answer.
    pub fn validate(&self) -> Result<(), MatchError> {
        let w = &self.weights;
        let all = [
            ("given_name", w.given_name),
            ("family_name", w.family_name),
            ("date_of_birth", w.date_of_birth),
            ("email", w.email),
            ("phone", w.phone),
        ];
        for (field, value) in all {
            if !value.is_finite() || value < 0.0 {
                return Err(MatchError::BadWeight { field, value });
            }
        }
        if all.iter().all(|(_, v)| *v == 0.0) {
            return Err(MatchError::NoWeights);
        }
        let (r, m) = (self.review_threshold, self.match_threshold);
        if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&m) || r > m {
            return Err(MatchError::BadThresholds {
                review: r,
                matched: m,
            });
        }
        if self.min_fields == 0 {
            return Err(MatchError::NoMinimum);
        }
        if let Some(cc) = &self.default_phone_country_code
            && (!(1..=3).contains(&cc.len()) || !cc.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(MatchError::BadCountryCode(cc.clone()));
        }
        Ok(())
    }
}

/// What the engine concludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// The same worker.
    Match,
    /// Possibly the same worker; a person should decide.
    Review,
    /// Not the same worker, on the evidence given.
    NoMatch,
    /// Too little to compare; neither a match nor a non-match.
    Insufficient,
}

/// What a decision rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    /// The same worker number from the same organization.
    WorkerNumber,
    /// The weighted fields in the [`Breakdown`].
    Fields,
}

/// Every field's score, `None` where the field was unknown on either side.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Breakdown {
    pub given_name: Option<f64>,
    pub family_name: Option<f64>,
    pub date_of_birth: Option<f64>,
    pub email: Option<f64>,
    pub phone: Option<f64>,
    /// `Some(true)`: a shared number. `Some(false)`: both have numbers from a
    /// common organization and none agree. `None`: no organization in common.
    pub worker_number: Option<bool>,
}

/// The explainable result of comparing two records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchResult {
    pub decision: Decision,
    pub basis: Basis,
    /// Weighted average of the comparable fields; `None` for [`Decision::Insufficient`].
    pub score: Option<f64>,
    pub breakdown: Breakdown,
    /// How many of the five scored fields could be compared.
    pub fields_compared: usize,
}

/// One candidate's result, with its position in the input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ranked {
    /// Index into the candidate slice that was ranked.
    pub index: usize,
    pub result: MatchResult,
}

/// Compares worker records. Holds configuration only.
#[derive(Debug, Clone)]
pub struct MatchingEngine {
    config: MatchConfig,
    nicknames: NicknameTable,
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new(MatchConfig::default()).expect("the default configuration is valid")
    }
}

impl MatchingEngine {
    /// Validates the configuration; nicknames are the built-in English table when enabled.
    pub fn new(config: MatchConfig) -> Result<Self, MatchError> {
        config.validate()?;
        let nicknames = if config.use_nicknames {
            NicknameTable::english()
        } else {
            NicknameTable::empty()
        };
        Ok(Self { config, nicknames })
    }

    /// Replace the nickname table (used only when `use_nicknames` is on).
    pub fn with_nicknames(mut self, table: NicknameTable) -> Self {
        if self.config.use_nicknames {
            self.nicknames = table;
        }
        self
    }

    pub fn config(&self) -> &MatchConfig {
        &self.config
    }

    /// Compare two records.
    pub fn match_workers(&self, a: &WorkerRecord, b: &WorkerRecord) -> MatchResult {
        let numbers = self.compare_worker_numbers(a, b);
        let breakdown = Breakdown {
            given_name: self.given_name(a, b),
            family_name: family_name(a, b),
            date_of_birth: match (a.date_of_birth, b.date_of_birth) {
                (Some(x), Some(y)) => Some(date_of_birth(x, y)),
                _ => None,
            },
            email: self.email(a, b),
            phone: self.phone(a, b),
            worker_number: numbers,
        };
        let w = &self.config.weights;
        let parts = [
            (breakdown.given_name, w.given_name),
            (breakdown.family_name, w.family_name),
            (breakdown.date_of_birth, w.date_of_birth),
            (breakdown.email, w.email),
            (breakdown.phone, w.phone),
        ];
        // a field with weight zero is not counted as compared
        let compared: Vec<(f64, f64)> = parts
            .iter()
            .filter_map(|(s, wt)| s.filter(|_| *wt > 0.0).map(|s| (s, *wt)))
            .collect();
        let fields_compared = compared.len();

        if numbers == Some(true) && self.config.worker_number_is_deterministic {
            return MatchResult {
                decision: Decision::Match,
                basis: Basis::WorkerNumber,
                score: Some(1.0),
                breakdown,
                fields_compared,
            };
        }
        if fields_compared < self.config.min_fields {
            return MatchResult {
                decision: Decision::Insufficient,
                basis: Basis::Fields,
                score: None,
                breakdown,
                fields_compared,
            };
        }
        let total: f64 = compared.iter().map(|(_, wt)| wt).sum();
        let score = compared.iter().map(|(s, wt)| s * wt).sum::<f64>() / total;
        let mut decision = if score >= self.config.match_threshold {
            Decision::Match
        } else if score >= self.config.review_threshold {
            Decision::Review
        } else {
            Decision::NoMatch
        };
        let stable_fact_disagrees = breakdown.date_of_birth == Some(0.0) || numbers == Some(false);
        if decision == Decision::Match && stable_fact_disagrees {
            decision = Decision::Review;
        }
        MatchResult {
            decision,
            basis: Basis::Fields,
            score: Some(score),
            breakdown,
            fields_compared,
        }
    }

    /// Score one query against many candidates; the output is parallel to the input.
    pub fn match_one_to_many(
        &self,
        query: &WorkerRecord,
        candidates: &[WorkerRecord],
    ) -> Vec<MatchResult> {
        candidates
            .iter()
            .map(|c| self.match_workers(query, c))
            .collect()
    }

    /// The same results, best first: by descending score, unscored last, ties by input order.
    pub fn rank_one_to_many(
        &self,
        query: &WorkerRecord,
        candidates: &[WorkerRecord],
    ) -> Vec<Ranked> {
        let mut ranked: Vec<Ranked> = self
            .match_one_to_many(query, candidates)
            .into_iter()
            .enumerate()
            .map(|(index, result)| Ranked { index, result })
            .collect();
        ranked.sort_by(|x, y| {
            match (x.result.score, y.result.score) {
                (Some(a), Some(b)) => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            }
            .then(x.index.cmp(&y.index))
        });
        ranked
    }

    fn given_name(&self, a: &WorkerRecord, b: &WorkerRecord) -> Option<f64> {
        let x = normalizer::fold(a.given_name.as_deref()?)?;
        let y = normalizer::fold(b.given_name.as_deref()?)?;
        if x == y {
            return Some(1.0);
        }
        if self.nicknames.related(&x, &y) {
            return Some(0.95);
        }
        let base = combined_similarity(&x, &y);
        let same_sound =
            normalizer::soundex(&x).is_some_and(|s| Some(s) == normalizer::soundex(&y));
        Some(if same_sound { base.max(0.85) } else { base })
    }

    fn email(&self, a: &WorkerRecord, b: &WorkerRecord) -> Option<f64> {
        let set = |r: &WorkerRecord| -> BTreeSet<String> {
            r.emails
                .iter()
                .filter_map(|e| normalizer::email(e, self.config.fold_gmail))
                .collect()
        };
        let (x, y) = (set(a), set(b));
        if x.is_empty() || y.is_empty() {
            return None;
        }
        if x.intersection(&y).next().is_some() {
            return Some(1.0);
        }
        let local = |s: &BTreeSet<String>| -> BTreeSet<String> {
            s.iter()
                .filter_map(|e| e.split('@').next().map(str::to_string))
                .collect()
        };
        Some(if local(&x).intersection(&local(&y)).next().is_some() {
            0.5
        } else {
            0.0
        })
    }

    fn phone(&self, a: &WorkerRecord, b: &WorkerRecord) -> Option<f64> {
        let cc = self.config.default_phone_country_code.as_deref();
        let set = |r: &WorkerRecord| -> BTreeSet<String> {
            r.phones
                .iter()
                .filter_map(|p| normalizer::phone(p, cc))
                .collect()
        };
        let (x, y) = (set(a), set(b));
        if x.is_empty() || y.is_empty() {
            return None;
        }
        Some(if x.intersection(&y).next().is_some() {
            1.0
        } else {
            0.0
        })
    }

    fn compare_worker_numbers(&self, a: &WorkerRecord, b: &WorkerRecord) -> Option<bool> {
        let key = |r: &WorkerRecord| -> BTreeSet<(String, String)> {
            r.worker_numbers
                .iter()
                .filter_map(|n| {
                    Some((
                        n.organization.trim().to_lowercase(),
                        normalizer::worker_number(&n.number)?,
                    ))
                })
                .filter(|(org, _)| !org.is_empty())
                .collect()
        };
        let (x, y) = (key(a), key(b));
        if x.intersection(&y).next().is_some() {
            return Some(true);
        }
        let orgs = |s: &BTreeSet<(String, String)>| -> BTreeSet<String> {
            s.iter().map(|(o, _)| o.clone()).collect()
        };
        if orgs(&x).intersection(&orgs(&y)).next().is_some() {
            return Some(false);
        }
        None
    }
}

fn family_name(a: &WorkerRecord, b: &WorkerRecord) -> Option<f64> {
    let x = normalizer::fold(a.family_name.as_deref()?)?;
    let y = normalizer::fold(b.family_name.as_deref()?)?;
    if x == y {
        return Some(1.0);
    }
    let (tx, ty): (BTreeSet<&str>, BTreeSet<&str>) =
        (x.split(' ').collect(), y.split(' ').collect());
    // "van der Berg" against "Berg", or "Smith-Jones" against "Jones"
    if tx.is_subset(&ty) || ty.is_subset(&tx) {
        return Some(0.9);
    }
    Some(combined_similarity(&x, &y))
}

/// 1 for the same day; 0.6 for day and month swapped; 0.5 for a single typo
/// in year, month or day; otherwise 0.
fn date_of_birth(a: NaiveDate, b: NaiveDate) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.year() == b.year()
        && a.day() == b.month()
        && a.month() == b.day()
        && a.day() <= 12
        && a.month() <= 12
    {
        return 0.6;
    }
    let shared = [
        a.year() == b.year(),
        a.month() == b.month(),
        a.day() == b.day(),
    ];
    if shared.iter().filter(|s| **s).count() == 2 {
        0.5
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dob(r: WorkerRecord, y: i32, m: u32, d: u32) -> WorkerRecord {
        r.with_date_of_birth(y, m, d).expect("a real date")
    }

    fn alice() -> WorkerRecord {
        dob(
            WorkerRecord::new()
                .with_given_name("Alice")
                .with_family_name("Williams"),
            1980,
            5,
            15,
        )
    }

    #[test]
    fn identical_records_match_on_fields() {
        let r = MatchingEngine::default().match_workers(&alice(), &alice());
        assert_eq!(r.decision, Decision::Match);
        assert_eq!(r.basis, Basis::Fields);
        assert_eq!(r.score, Some(1.0));
        assert_eq!(r.fields_compared, 3);
    }

    #[test]
    fn unknown_fields_are_none_not_zero() {
        let r = MatchingEngine::default().match_workers(&alice(), &alice());
        assert_eq!(r.breakdown.email, None);
        assert_eq!(r.breakdown.phone, None);
        assert_eq!(r.breakdown.worker_number, None);
    }

    #[test]
    fn spelling_variant_with_same_birth_date_matches() {
        let b = dob(
            WorkerRecord::new()
                .with_given_name("Alyce")
                .with_family_name("Williams"),
            1980,
            5,
            15,
        );
        let r = MatchingEngine::default().match_workers(&alice(), &b);
        assert_eq!(r.decision, Decision::Match, "{r:?}");
    }

    #[test]
    fn diacritics_do_not_matter() {
        let a = WorkerRecord::new()
            .with_given_name("José")
            .with_family_name("Müller")
            .with_email("j@x.org");
        let b = WorkerRecord::new()
            .with_given_name("Jose")
            .with_family_name("Muller")
            .with_email("J@X.org");
        assert_eq!(
            MatchingEngine::default().match_workers(&a, &b).score,
            Some(1.0)
        );
    }

    #[test]
    fn nickname_scores_high_only_when_enabled() {
        let a = WorkerRecord::new()
            .with_given_name("Michael")
            .with_family_name("Brown")
            .with_email("m@x.org");
        let b = WorkerRecord::new()
            .with_given_name("Mike")
            .with_family_name("Brown")
            .with_email("m@x.org");
        let on = MatchingEngine::default().match_workers(&a, &b);
        assert_eq!(on.breakdown.given_name, Some(0.95));
        let off = MatchingEngine::new(MatchConfig {
            use_nicknames: false,
            ..MatchConfig::default()
        })
        .unwrap()
        .match_workers(&a, &b);
        assert!(off.breakdown.given_name.unwrap() < 0.95);
    }

    #[test]
    fn one_comparable_field_is_insufficient() {
        let a = WorkerRecord::new().with_given_name("Alice");
        let b = WorkerRecord::new().with_given_name("Alice");
        let r = MatchingEngine::default().match_workers(&a, &b);
        assert_eq!(r.decision, Decision::Insufficient);
        assert_eq!(r.score, None);
    }

    #[test]
    fn empty_records_are_insufficient_not_a_match() {
        let r = MatchingEngine::default().match_workers(&WorkerRecord::new(), &WorkerRecord::new());
        assert_eq!(r.decision, Decision::Insufficient);
    }

    #[test]
    fn same_worker_number_in_same_organization_is_deterministic() {
        let a = WorkerRecord::new().with_worker_number("org:a", "AB-1042");
        let b = WorkerRecord::new()
            .with_worker_number("ORG:a", "ab 1042")
            .with_given_name("Whoever");
        let r = MatchingEngine::default().match_workers(&a, &b);
        assert_eq!(
            (r.decision, r.basis, r.score),
            (Decision::Match, Basis::WorkerNumber, Some(1.0))
        );
    }

    #[test]
    fn same_number_in_another_organization_is_not_evidence() {
        let a = WorkerRecord::new().with_worker_number("org:a", "1042");
        let b = WorkerRecord::new().with_worker_number("org:b", "1042");
        let r = MatchingEngine::default().match_workers(&a, &b);
        assert_eq!(r.breakdown.worker_number, None);
        assert_eq!(r.decision, Decision::Insufficient);
    }

    #[test]
    fn deterministic_worker_number_can_be_turned_off() {
        let cfg = MatchConfig {
            worker_number_is_deterministic: false,
            ..MatchConfig::default()
        };
        let a = WorkerRecord::new().with_worker_number("org:a", "1042");
        let r = MatchingEngine::new(cfg).unwrap().match_workers(&a, &a);
        assert_eq!(r.decision, Decision::Insufficient);
    }

    #[test]
    fn different_birth_date_caps_a_match_at_review() {
        let b = dob(
            WorkerRecord::new()
                .with_given_name("Alice")
                .with_family_name("Williams"),
            1991,
            11,
            2,
        );
        let a = alice().with_email("a@x.org");
        let b = b.with_email("a@x.org");
        // with the default weights the average falls below the match threshold anyway;
        // a low date-of-birth weight is the case where only the cap protects
        let mut cfg = MatchConfig::default();
        cfg.weights.date_of_birth = 0.5;
        let r = MatchingEngine::new(cfg).unwrap().match_workers(&a, &b);
        assert_eq!(r.breakdown.date_of_birth, Some(0.0));
        assert!(
            r.score.unwrap() >= 0.88,
            "the average alone would be a match: {r:?}"
        );
        assert_eq!(r.decision, Decision::Review);
    }

    #[test]
    fn a_different_number_in_the_same_organization_caps_a_match_at_review() {
        let a = alice().with_worker_number("org:a", "1");
        let b = alice().with_worker_number("org:a", "2");
        let r = MatchingEngine::default().match_workers(&a, &b);
        assert_eq!(r.breakdown.worker_number, Some(false));
        assert_eq!(r.decision, Decision::Review);
    }

    #[test]
    fn day_and_month_swapped_scores_between() {
        let d = |m, dd| NaiveDate::from_ymd_opt(1980, m, dd).unwrap();
        assert_eq!(date_of_birth(d(5, 3), d(3, 5)), 0.6);
        assert_eq!(date_of_birth(d(5, 3), d(5, 4)), 0.5);
        assert_eq!(date_of_birth(d(5, 3), d(6, 4)), 0.0);
        assert_eq!(date_of_birth(d(5, 13), d(1, 5)), 0.0);
    }

    #[test]
    fn compound_family_names_match_a_part() {
        let a = WorkerRecord::new()
            .with_family_name("Smith-Jones")
            .with_given_name("Ann");
        let b = WorkerRecord::new()
            .with_family_name("Jones")
            .with_given_name("Ann");
        assert_eq!(
            MatchingEngine::default()
                .match_workers(&a, &b)
                .breakdown
                .family_name,
            Some(0.9)
        );
    }

    #[test]
    fn phone_forms_agree_with_a_default_country() {
        let cfg = MatchConfig {
            default_phone_country_code: Some("44".into()),
            ..MatchConfig::default()
        };
        let a = WorkerRecord::new()
            .with_phone("020 7946 0958")
            .with_given_name("A");
        let b = WorkerRecord::new()
            .with_phone("+44 20 7946 0958")
            .with_given_name("A");
        assert_eq!(
            MatchingEngine::new(cfg)
                .unwrap()
                .match_workers(&a, &b)
                .breakdown
                .phone,
            Some(1.0)
        );
    }

    #[test]
    fn shared_local_part_on_another_domain_is_weak_evidence() {
        let a = WorkerRecord::new().with_email("sam@a.org");
        let b = WorkerRecord::new().with_email("sam@b.org");
        assert_eq!(
            MatchingEngine::default()
                .match_workers(&a, &b)
                .breakdown
                .email,
            Some(0.5)
        );
    }

    #[test]
    fn match_is_symmetric() {
        let a = alice().with_email("a@x.org").with_phone("+44 20 7946 0958");
        let b = dob(
            WorkerRecord::new()
                .with_given_name("Alicia")
                .with_family_name("Williamson"),
            1980,
            15 % 12,
            5,
        )
        .with_email("a@y.org");
        let e = MatchingEngine::default();
        assert_eq!(e.match_workers(&a, &b), e.match_workers(&b, &a));
    }

    #[test]
    fn ranking_is_best_first_with_a_stable_tiebreak() {
        let q = alice();
        let far = dob(
            WorkerRecord::new()
                .with_given_name("Zed")
                .with_family_name("Quinn"),
            1960,
            1,
            1,
        );
        let close = alice();
        let none = WorkerRecord::new();
        let c = [far.clone(), none, close.clone(), close];
        let order: Vec<usize> = MatchingEngine::default()
            .rank_one_to_many(&q, &c)
            .iter()
            .map(|r| r.index)
            .collect();
        assert_eq!(order, vec![2, 3, 0, 1]);
    }

    #[test]
    fn one_to_many_is_parallel_to_the_input() {
        let c = [alice(), WorkerRecord::new()];
        let out = MatchingEngine::default().match_one_to_many(&alice(), &c);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].decision, Decision::Match);
        assert_eq!(out[1].decision, Decision::Insufficient);
    }

    #[test]
    fn invalid_configurations_are_refused() {
        let bad = |f: fn(&mut MatchConfig)| {
            let mut c = MatchConfig::default();
            f(&mut c);
            c.validate().unwrap_err()
        };
        assert!(matches!(
            bad(|c| c.weights.email = -1.0),
            MatchError::BadWeight { .. }
        ));
        assert!(matches!(
            bad(|c| c.weights.phone = f64::NAN),
            MatchError::BadWeight { .. }
        ));
        assert!(matches!(
            bad(|c| c.weights = Weights {
                given_name: 0.0,
                family_name: 0.0,
                date_of_birth: 0.0,
                email: 0.0,
                phone: 0.0
            }),
            MatchError::NoWeights
        ));
        assert!(matches!(
            bad(|c| c.review_threshold = 0.99),
            MatchError::BadThresholds { .. }
        ));
        assert!(matches!(
            bad(|c| c.match_threshold = 1.5),
            MatchError::BadThresholds { .. }
        ));
        assert_eq!(bad(|c| c.min_fields = 0), MatchError::NoMinimum);
        assert!(matches!(
            bad(|c| c.default_phone_country_code = Some("+44".into())),
            MatchError::BadCountryCode(_)
        ));
    }

    #[test]
    fn presets_are_valid_and_ordered() {
        for c in [
            MatchConfig::strict(),
            MatchConfig::default(),
            MatchConfig::lenient(),
        ] {
            c.validate().unwrap();
        }
        assert!(MatchConfig::strict().match_threshold > MatchConfig::default().match_threshold);
        assert!(MatchConfig::lenient().match_threshold < MatchConfig::default().match_threshold);
    }

    #[test]
    fn debug_never_shows_personal_data() {
        let r = alice()
            .with_email("secret@x.org")
            .with_worker_number("org:a", "9999");
        let shown = format!("{r:?}");
        for leak in ["Alice", "Williams", "secret", "9999", "1980"] {
            assert!(!shown.contains(leak), "{shown}");
        }
    }
}
