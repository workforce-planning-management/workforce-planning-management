//! Possible duplicate workers (WPM-R136, WPM-D87): given a name, and optionally a worker number,
//! which existing workers might be the same person?
//!
//! WPM holds a single `display_name` and no date of birth, so the evidence is a name and an
//! employer worker number and nothing else. Email and phone are **not** compared: the contact
//! details sit behind a narrower access check than the directory, and "this email matches worker
//! X" would hand a caller the very fact that check withholds.
//!
//! **The name is compared whole (WPM-T381).** It is never split into a first and a last word,
//! because that is wrong for a family name written first, for a name of one word, and for names
//! written without spaces. The name is folded ([`name_tokens`]) into tokens and the two token lists
//! are compared in any order ([`name_similarity`]). The worker number is still compared by
//! `workforce-planning-management-matcher`, which owns that rule.
//!
//! A name alone never says "same person" (two real people share one), so a name is at most
//! `review`. Only a shared worker number, from the same organization, is `match`. Nothing here
//! writes or merges anything.

use std::cmp::Ordering;

use serde::Serialize;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use uuid::Uuid;
use workforce_planning_management_matcher::matcher::{Basis, Decision, MatchConfig};
use workforce_planning_management_matcher::scorer::combined_similarity;
use workforce_planning_management_matcher::{MatchingEngine, WorkerNumber, WorkerRecord};

/// The most candidates one answer lists.
pub const MAX_RESULTS: usize = 10;

/// An existing worker, as much of them as the comparison may see.
#[derive(Debug, Clone)]
pub struct Known {
    /// The worker's public id.
    pub pid: Uuid,
    /// The name as held (one string).
    pub display_name: String,
    /// The organization that issued the number.
    pub organization_ref: String,
    /// The employer's number for them.
    pub worker_number: String,
}

/// Who is being checked.
#[derive(Debug, Clone)]
pub struct Query {
    /// The name being checked.
    pub display_name: String,
    /// The organization the new worker would join.
    pub organization_ref: String,
    /// The number they would be given, when known.
    pub worker_number: Option<String>,
}

/// What each compared piece of evidence scored. Only a name and a worker number are compared.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Breakdown {
    /// The whole-name similarity, `0.0..=1.0`; `null` when either name is too little to compare.
    pub name: Option<f64>,
    /// Whether the worker numbers agree; `null` unless both come from the same organization.
    pub worker_number: Option<bool>,
}

/// One possible duplicate, with its working.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Possible {
    /// The existing worker's public id.
    pub worker_pid: Uuid,
    /// Their name, as the directory shows it.
    pub display_name: String,
    /// `match` (a shared worker number) or `review`.
    pub decision: Decision,
    /// What the decision rests on: `worker_number` or `fields` (the name).
    pub basis: Basis,
    /// The score the decision rests on.
    pub score: Option<f64>,
    /// Each compared field's score.
    pub breakdown: Breakdown,
}

/// Scripts whose combining marks are diacritics a name is often typed without (Latin, Greek,
/// Cyrillic): "Zoë" and "Zoe", "Ἀθηνᾶ" and "Αθηνα", "Ёлкин" and "Елкин" fold together.
fn folds_its_marks(c: char) -> bool {
    matches!(u32::from(c),
        0x0041..=0x024f | 0x1e00..=0x1eff | 0x2c60..=0x2c7f | 0xa720..=0xa7ff | 0xab30..=0xab6f
        | 0x0370..=0x03ff | 0x1f00..=0x1fff
        | 0x0400..=0x052f | 0x1c80..=0x1c8f | 0x2de0..=0x2dff | 0xa640..=0xa69f)
}

/// Marks that are optional in writing and dropped in every script: Arabic short vowels and
/// shadda, Hebrew points, the Arabic tatweel, and the zero-width joiner and non-joiner.
fn is_optional_mark(c: char) -> bool {
    matches!(u32::from(c),
        0x064b..=0x065f | 0x0670 | 0x0640
        | 0x0591..=0x05bd | 0x05bf | 0x05c1..=0x05c2 | 0x05c4..=0x05c5 | 0x05c7
        | 0x200c..=0x200d)
}

/// Letters that do not decompose but have a plain spelling, and the Greek final sigma.
fn push_folded(c: char, token: &mut String) {
    for lower in c.to_lowercase() {
        match lower {
            'ß' => token.push_str("ss"),
            'æ' => token.push_str("ae"),
            'œ' => token.push_str("oe"),
            'ø' => token.push('o'),
            'đ' | 'ð' => token.push('d'),
            'ł' => token.push('l'),
            'þ' => token.push_str("th"),
            'ı' => token.push('i'),
            'ς' => token.push('σ'),
            other => token.push(other),
        }
    }
}

/// A name as the tokens it is compared by: NFKC (so full-width letters are plain), lower case,
/// punctuation and spaces as boundaries, diacritics dropped in Latin, Greek and Cyrillic only.
///
/// In other scripts a combining mark is part of the letter (a Devanagari vowel sign, a Bengali
/// virama) and is kept, so "राम" and "रम" stay different; Arabic and Hebrew optional vowel marks are
/// dropped. A name written without spaces (Chinese, Japanese, Thai) is one token. Nothing is
/// assumed about which token is a given or a family name.
#[must_use]
pub fn name_tokens(display_name: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut base_folds = false;
    for c in display_name.nfkc() {
        if is_optional_mark(c) {
            continue;
        }
        if is_combining_mark(c) {
            if !base_folds && !token.is_empty() {
                token.push(c);
            }
            continue;
        }
        if c.is_alphanumeric() {
            base_folds = folds_its_marks(c);
            if base_folds {
                for d in c.nfd().filter(|d| !is_combining_mark(*d)) {
                    push_folded(d, &mut token);
                }
            } else {
                push_folded(c, &mut token);
            }
        } else {
            base_folds = false;
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

/// How alike two folded names are, `0.0..=1.0`, in any token order; `None` when there is too little.
///
/// - Either name empty: `None`.
/// - The same letters once spaces are ignored ("山田 太郎", "山田太郎"): `1.0`.
/// - One token against several ("Ada" against "Ada Lovelace"): `None`. One word is a part of the
///   other name, not evidence of it; two one-word names are compared.
/// - Otherwise each token is paired with its most alike token on the other side, best pairs first,
///   each used once; a token left unpaired scores 0. The score is the pairs' similarity weighted by
///   the letters in them, over all the letters on both sides. Order is ignored, so "Lovelace Ada"
///   and "Ada Lovelace" are `1.0`.
///
/// Symmetric: `name_similarity(a, b) == name_similarity(b, a)`.
#[must_use]
pub fn name_similarity(a: &[String], b: &[String]) -> Option<f64> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    if a.concat() == b.concat() {
        return Some(1.0);
    }
    if (a.len() == 1) != (b.len() == 1) {
        return None;
    }
    let mut pairs: Vec<(f64, usize, usize)> = Vec::with_capacity(a.len() * b.len());
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            pairs.push((combined_similarity(x, y), i, j));
        }
    }
    // best first; a tie is broken by the pair's own text, not by which side it came from
    let key = |i: usize, j: usize| {
        let (x, y) = (a[i].as_str(), b[j].as_str());
        if x <= y { (x, y) } else { (y, x) }
    };
    pairs.sort_by(|p, q| {
        q.0.partial_cmp(&p.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| key(p.1, p.2).cmp(&key(q.1, q.2)))
    });
    let letters = |s: &String| s.chars().count();
    let total: usize = a.iter().map(letters).sum::<usize>() + b.iter().map(letters).sum::<usize>();
    let (mut used_a, mut used_b) = (vec![false; a.len()], vec![false; b.len()]);
    let mut weighted = 0.0;
    for (similarity, i, j) in pairs {
        if used_a[i] || used_b[j] {
            continue;
        }
        used_a[i] = true;
        used_b[j] = true;
        #[allow(clippy::cast_precision_loss)] // name lengths are far below 2^52
        let weight = (letters(&a[i]) + letters(&b[j])) as f64;
        weighted += similarity * weight;
    }
    #[allow(clippy::cast_precision_loss)] // name lengths are far below 2^52
    let score = weighted / total as f64;
    Some(score)
}

fn numbers_only(organization: &str, number: Option<&str>) -> WorkerRecord {
    let mut r = WorkerRecord::new();
    if let Some(n) = number.filter(|n| !n.trim().is_empty()) {
        r.worker_numbers.push(WorkerNumber::new(organization, n));
    }
    r
}

/// The candidates that are a `match` or a `review`, best first, at most [`MAX_RESULTS`].
///
/// Best first is by descending score, a `match` before a `review` at the same score, then the
/// order given.
#[must_use]
pub fn possible_duplicates(query: &Query, known: &[Known]) -> Vec<Possible> {
    let engine = MatchingEngine::default();
    let review_at = MatchConfig::default().review_threshold;
    let asked_name = name_tokens(&query.display_name);
    let asked_number = numbers_only(&query.organization_ref, query.worker_number.as_deref());
    let mut found: Vec<(usize, Possible)> = known
        .iter()
        .enumerate()
        .filter_map(|(index, k)| {
            let numbers = engine.match_workers(
                &asked_number,
                &numbers_only(&k.organization_ref, Some(&k.worker_number)),
            );
            let name = name_similarity(&asked_name, &name_tokens(&k.display_name));
            let breakdown = Breakdown {
                name,
                worker_number: numbers.breakdown.worker_number,
            };
            let (decision, basis, score) =
                if numbers.basis == Basis::WorkerNumber && numbers.decision == Decision::Match {
                    (Decision::Match, Basis::WorkerNumber, Some(1.0))
                } else {
                    // a name alone is not evidence of identity
                    match name {
                        Some(s) if s >= review_at => (Decision::Review, Basis::Fields, Some(s)),
                        _ => return None,
                    }
                };
            Some((
                index,
                Possible {
                    worker_pid: k.pid,
                    display_name: k.display_name.clone(),
                    decision,
                    basis,
                    score,
                    breakdown,
                },
            ))
        })
        .collect();
    found.sort_by(|(i, x), (j, y)| {
        y.score
            .partial_cmp(&x.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| (y.decision == Decision::Match).cmp(&(x.decision == Decision::Match)))
            .then(i.cmp(j))
    });
    found
        .into_iter()
        .map(|(_, p)| p)
        .take(MAX_RESULTS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(name: &str, org: &str, number: &str) -> Known {
        Known {
            pid: Uuid::new_v4(),
            display_name: name.into(),
            organization_ref: org.into(),
            worker_number: number.into(),
        }
    }

    fn query(name: &str, number: Option<&str>) -> Query {
        Query {
            display_name: name.into(),
            organization_ref: "organization:a".into(),
            worker_number: number.map(Into::into),
        }
    }

    fn similarity(a: &str, b: &str) -> Option<f64> {
        name_similarity(&name_tokens(a), &name_tokens(b))
    }

    /// What the check says of `b` when `a` is asked: `None` when it is not listed.
    fn listed(a: &str, b: &str) -> Option<Decision> {
        possible_duplicates(&query(a, None), &[known(b, "organization:a", "1001")])
            .first()
            .map(|p| p.decision)
    }

    #[test]
    fn a_name_folds_into_tokens_without_first_or_last() {
        let t = |s: &str| name_tokens(s);
        assert_eq!(t("Siân O'Brien-Smith"), ["sian", "o", "brien", "smith"]);
        assert_eq!(t("  Ada   Byron King "), ["ada", "byron", "king"]);
        assert_eq!(
            t("ＡＤＡ　ＫＩＮＧ"),
            ["ada", "king"],
            "full-width letters and space"
        );
        assert_eq!(t("Jürgen Weiß"), ["jurgen", "weiss"]);
        assert_eq!(t("İlkay Gündoğan"), ["ilkay", "gundogan"]);
        assert_eq!(
            t("ΣΩΚΡΆΤΗΣ"),
            t("σωκρατης"),
            "Greek case, accent and final sigma"
        );
        assert_eq!(t("Пётр ИВАНОВ"), ["петр", "иванов"]);
        assert_eq!(
            t("山田太郎"),
            ["山田太郎"],
            "written without spaces: one token"
        );
        assert_eq!(
            t("アン・シャーリー"),
            ["アン", "シャーリー"],
            "the katakana middle dot"
        );
        assert_eq!(t("مُحَمَّد"), t("محمد"), "Arabic short vowels are optional");
        assert_ne!(
            t("राम"),
            t("रम"),
            "a Devanagari vowel sign is part of the letter"
        );
        assert_ne!(t("কমল"), t("কমলা"));
        assert_eq!(t(" - . ' "), Vec::<String>::new());
    }

    #[test]
    fn the_same_name_alone_is_review_never_match() {
        let found = possible_duplicates(
            &query("Ada Lovelace", None),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].decision, Decision::Review);
        assert_eq!(found[0].basis, Basis::Fields);
        assert_eq!(found[0].score, Some(1.0));
    }

    #[test]
    fn the_same_worker_number_in_one_organization_is_a_match() {
        let found = possible_duplicates(
            &query("Ada Lovelace", Some("1001")),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found[0].decision, Decision::Match);
        assert_eq!(found[0].basis, Basis::WorkerNumber);
        assert_eq!(found[0].breakdown.worker_number, Some(true));
        // whatever the name: a number decides
        let found = possible_duplicates(
            &query("李华", Some("10-01")),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found[0].decision, Decision::Match);
    }

    #[test]
    fn the_same_number_in_another_organization_is_no_evidence() {
        let found = possible_duplicates(
            &query("Grace Hopper", Some("1001")),
            &[known("Ada Lovelace", "organization:b", "1001")],
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn a_different_number_in_the_same_organization_leaves_a_name_at_review() {
        let found = possible_duplicates(
            &query("Ada Lovelace", Some("2002")),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found[0].decision, Decision::Review);
        assert_eq!(found[0].breakdown.worker_number, Some(false));
    }

    #[test]
    fn a_different_name_is_not_listed() {
        assert_eq!(listed("Ada Lovelace", "Grace Hopper"), None);
        assert_eq!(listed("山田太郎", "田中花子"), None);
        assert_eq!(listed("محمد علي", "فاطمة حسن"), None);
        assert_eq!(listed("राम शर्मा", "सीता वर्मा"), None);
        assert_eq!(listed("张伟", "李娜"), None);
    }

    #[test]
    fn one_word_against_a_longer_name_is_too_little_to_say_anything() {
        let found = possible_duplicates(
            &query("Ada", None),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found, Vec::new());
        assert_eq!(similarity("Suharto", "Suharto Wijaya"), None);
        assert_eq!(similarity("Wijaya", "Suharto Wijaya"), None);
        assert_eq!(similarity("", "Ada Lovelace"), None);
        assert_eq!(
            listed("—", "—"),
            None,
            "a name with no letters lists no one"
        );
    }

    #[test]
    fn a_one_word_name_is_a_whole_name() {
        assert_eq!(listed("Suharto", "Suharto"), Some(Decision::Review));
        assert_eq!(listed("SUHARTO", "suharto"), Some(Decision::Review));
        assert_eq!(listed("Suharto", "Megawati"), None);
    }

    #[test]
    fn the_family_name_may_come_first() {
        assert_eq!(similarity("Lovelace Ada", "Ada Lovelace"), Some(1.0));
        assert_eq!(similarity("Nguyễn Văn An", "An Nguyen Van"), Some(1.0));
        assert_eq!(similarity("Kim Min-jun", "Min-jun Kim"), Some(1.0));
        assert_eq!(similarity("Петров Иван", "Иван Петров"), Some(1.0));
        assert_eq!(similarity("علي محمد", "محمد علي"), Some(1.0));
        assert_eq!(similarity("शर्मा राम", "राम शर्मा"), Some(1.0));
        assert_eq!(listed("YAMADA Taro", "Taro Yamada"), Some(Decision::Review));
    }

    #[test]
    fn a_name_written_with_or_without_spaces_is_the_same() {
        assert_eq!(listed("山田 太郎", "山田太郎"), Some(Decision::Review));
        assert_eq!(listed("毛泽东", "毛 泽东"), Some(Decision::Review));
        assert_eq!(listed("김민준", "김 민준"), Some(Decision::Review));
    }

    #[test]
    fn particles_and_middle_names_are_tokens_like_any_other() {
        assert_eq!(
            similarity("Ludwig van Beethoven", "Ludwig Van Beethoven"),
            Some(1.0)
        );
        assert_eq!(similarity("de la Cruz, Juan", "Juan de la Cruz"), Some(1.0));
        assert_eq!(
            similarity("Abu Bakr al-Siddiq", "al-Siddiq Abu Bakr"),
            Some(1.0)
        );
        assert_eq!(
            listed("Ludwig van Beethoven", "Ludwig Beethoven"),
            Some(Decision::Review)
        );
        assert_eq!(
            listed("Vincent Willem van Gogh", "Vincent van Gogh"),
            Some(Decision::Review)
        );
        assert_eq!(listed("Ada Byron King", "Ada King"), Some(Decision::Review));
    }

    #[test]
    fn a_spelling_variant_is_review_and_another_script_is_not_transliterated() {
        assert_eq!(
            listed("Ada Lovelase", "Ada Lovelace"),
            Some(Decision::Review)
        );
        assert_eq!(listed("Zoë Smith", "Zoe Smith"), Some(Decision::Review));
        // no transliteration: the same person written in two scripts is not found
        assert_eq!(listed("Ivan Petrov", "Иван Петров"), None);
        assert_eq!(listed("Yamada Taro", "山田太郎"), None);
    }

    #[test]
    fn the_comparison_is_symmetric_and_bounded() {
        let names = [
            "Ada Lovelace",
            "Lovelace Ada",
            "Ada Byron King",
            "Ludwig van Beethoven",
            "山田太郎",
            "山田 次郎",
            "محمد علي",
            "Иван Петров",
            "Пётр Иванов",
            "Suharto",
            "Ana Ana Ana",
            "Ana Anna",
        ];
        for a in names {
            for b in names {
                let ab = similarity(a, b);
                assert_eq!(ab, similarity(b, a), "{a} / {b}");
                if let Some(s) = ab {
                    assert!((0.0..=1.0).contains(&s), "{a} / {b}: {s}");
                }
            }
        }
    }

    #[test]
    fn best_first_with_a_number_ahead_of_an_equal_name() {
        let by_name = known("Ada Lovelace", "organization:a", "1");
        let by_number = known("Someone Else", "organization:a", "7");
        let near = known("Ada Lovelase", "organization:a", "2");
        let found = possible_duplicates(
            &query("Ada Lovelace", Some("7")),
            &[near.clone(), by_name.clone(), by_number.clone()],
        );
        let order: Vec<Uuid> = found.iter().map(|p| p.worker_pid).collect();
        assert_eq!(order, [by_number.pid, by_name.pid, near.pid]);
    }

    #[test]
    fn at_most_ten_are_listed() {
        let many: Vec<Known> = (0..15)
            .map(|i| known("Ada Lovelace", "organization:a", &format!("{i}")))
            .collect();
        assert_eq!(
            possible_duplicates(&query("Ada Lovelace", None), &many).len(),
            MAX_RESULTS
        );
    }

    #[test]
    fn the_answer_carries_no_email_phone_or_birth_score() {
        let found = possible_duplicates(
            &query("Ada Lovelace", None),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        let json = serde_json::to_value(&found[0].breakdown).expect("json");
        let keys: Vec<&String> = json.as_object().expect("object").keys().collect();
        assert_eq!(keys, ["name", "worker_number"]);
    }
}
