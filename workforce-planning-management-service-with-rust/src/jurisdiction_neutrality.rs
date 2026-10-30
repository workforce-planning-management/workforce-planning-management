//! **The pure core names no country** (WPM-R214, WPM-D143, WPM-T369): a source-text check that fails when an
//! identifier or a string literal in `src/rules/` names a country, a region, or a law or body of one jurisdiction.
//!
//! A rule takes a jurisdiction's values as parameters; the values themselves are a deployer's profile. The few
//! legitimate names (a worked example profile beside a second one, a code list, a framework's own identifier, a
//! test's sample input) are listed in [`ALLOWED`], each with its reason, and a stale entry fails too.
//!
//! Comments are not checked: D143 asks the prose to *name* the regime it describes. Only code is.
//!
//! The check is lexical, so it is coarse on purpose. It splits identifiers and literal text into words (on
//! `_`, `-`, punctuation and camelCase) and compares them with [`WORDS`], [`PHRASES`] and [`CODES`]. It does not
//! know every country: the lists hold the names this codebase has used or is likely to reach for, and a name not
//! on them is not caught. `IN` (India) is left out of [`CODES`] because it is an SQL keyword and an English word.

use std::path::{Path, PathBuf};

/// Single words, compared without regard to case.
const WORDS: &[&str] = &[
    // countries, nations and regions
    "uk",
    "gb",
    "gbr",
    "britain",
    "british",
    "england",
    "wales",
    "welsh",
    "scotland",
    "scottish",
    "usa",
    "america",
    "american",
    "eu",
    "europe",
    "european",
    "germany",
    "france",
    "india",
    "indian",
    "japan",
    "ireland",
    "irish",
    "canada",
    "canadian",
    "australia",
    "australian",
    "mexico",
    "brazil",
    "china",
    "nigeria",
    "kenya",
    "uae",
    "emirates",
    "singapore",
    "pakistan",
    "bangladesh",
    "indonesia",
    // bodies and laws of one jurisdiction
    "nhs",
    "hmrc",
    "gdpr",
    "acas",
    "tupe",
    "tulrca",
    "wdes",
    "wres",
    "mhra",
    "cqc",
    "paye",
    "ssn",
    "irs",
    "flsa",
];

/// Phrases (consecutive words), compared without regard to case.
const PHRASES: &[&[&str]] = &[
    &["united", "kingdom"],
    &["united", "states"],
    &["great", "britain"],
    &["northern", "ireland"],
    &["new", "zealand"],
    &["south", "africa"],
    &["saudi", "arabia"],
    &["working", "time", "regulations"],
    &["equality", "act"],
    &["agenda", "for", "change"],
    &["national", "insurance"],
    &["social", "security"],
    &["employment", "rights", "act"],
    &["data", "protection", "act"],
    &["trade", "union", "and", "labour", "relations"],
];

/// Upper-case country codes, compared exactly (so the pronoun "us" is not one).
const CODES: &[&str] = &[
    "US", "DE", "FR", "JP", "IE", "AU", "CA", "NZ", "CN", "BR", "ZA", "AE", "SA", "MX", "NG", "KE",
    "SG", "PK", "BD",
];

/// `(file under src/rules, the word or phrase as reported, why it is there)`. Every hit in that file for that word
/// is allowed; an entry with no hit fails the check, so the list cannot go stale.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "consultation.rs",
        "britain",
        "great_britain() is a worked example profile beside eu_directive_98_59(); the rule takes a Profile",
    ),
    (
        "consultation.rs",
        "great britain",
        "the same example profile, in its id and its tests",
    ),
    (
        "consultation.rs",
        "gb",
        "a test binding of the great_britain() example profile",
    ),
    (
        "consultation.rs",
        "uk",
        "the example profiles' source URLs (gov.uk, legislation.gov.uk)",
    ),
    (
        "consultation.rs",
        "eu",
        "eu_directive_98_59() is the second, differently shaped example profile",
    ),
    (
        "consultation.rs",
        "trade union and labour relations",
        "the source citation of the great_britain() example profile",
    ),
    (
        "consultation_register.rs",
        "britain",
        "tests use the great_britain() example profile from consultation.rs",
    ),
    (
        "consultation_register.rs",
        "great britain",
        "the same example profile, imported by its tests",
    ),
    (
        "consultation_register.rs",
        "gb",
        "a test helper over the great_britain() example profile",
    ),
    (
        "consultation_register.rs",
        "eu",
        "a test binding over the eu_directive_98_59() example profile",
    ),
    (
        "contact_details.rs",
        "gbr",
        "a test input: a three-letter code is not an ISO 3166 alpha-2 country",
    ),
    (
        "holiday.rs",
        "gb",
        "the GB-ENG rule pack, a worked example marked unverified; generate() takes any pack's rules",
    ),
    ("holiday.rs", "england", "the GB-ENG example pack's name"),
    ("holiday.rs", "wales", "the GB-ENG example pack's name"),
    (
        "holiday.rs",
        "US",
        "the US federal example pack and a region-code example in a validation message",
    ),
    (
        "holiday.rs",
        "united states",
        "the US federal example pack's name",
    ),
    ("holiday.rs", "DE", "the DE example pack"),
    ("holiday.rs", "germany", "the DE example pack's name"),
    (
        "holiday.rs",
        "FR",
        "a test that a region with no shipped pack has none",
    ),
    (
        "esco.rs",
        "eu",
        "ESCO concept URIs (data.europa.eu) in the importer's test fixtures; ESCO is an optional import",
    ),
    (
        "credential.rs",
        "eu",
        "an ESCO concept URI (data.europa.eu) as a test skill code",
    ),
    (
        "holiday.rs",
        "AU",
        "a well-formed subdivision code (AU-NSW) in the region-code test",
    ),
    (
        "framework_roles.rs",
        "uk",
        "uk-gdad-pcf is that framework's own identifier in the list of importable frameworks",
    ),
    (
        "contact_details.rs",
        "gb",
        "test inputs for the ISO 3166 shape (the message names no country since WPM-T382)",
    ),
    (
        "cost.rs",
        "gb",
        "a test input: a two-letter string is not an ISO 4217 code",
    ),
    (
        "expenses.rs",
        "gb",
        "the example in the currency message, and a test input that is not an ISO 4217 code",
    ),
    (
        "engagement.rs",
        "gb",
        "the example in the currency message, and a test input that is not an ISO 4217 code",
    ),
];

/// One hit: the file (relative to `src/rules`), the line, and the word or phrase found.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    file: String,
    line: usize,
    term: String,
}

/// The code of a Rust source, with each comment replaced by spaces (line breaks kept, so line numbers hold).
/// String, raw-string and char literals are kept as they are; a lifetime is not mistaken for a char literal.
fn without_comments(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    let blank = |c: char| if c == '\n' { '\n' } else { ' ' };
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(chars[i]));
                    i += 1;
                }
            }
        } else if c == 'r'
            && matches!(next, Some('#' | '"'))
            && !chars
                .get(i.wrapping_sub(1))
                .is_some_and(|p| p.is_alphanumeric() || *p == '_')
        {
            // a raw string: r"…" or r#"…"#
            let mut hashes = 0;
            let mut j = i + 1;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                let end: String = std::iter::once('"')
                    .chain(std::iter::repeat_n('#', hashes))
                    .collect();
                let rest: String = chars[j + 1..].iter().collect();
                let close = rest.find(&end).map_or(chars.len(), |at| {
                    j + 1 + rest[..at].chars().count() + end.chars().count()
                });
                out.extend(&chars[i..close]);
                i = close;
            } else {
                out.push(c);
                i += 1;
            }
        } else if c == '"' {
            out.push(c);
            i += 1;
            while i < chars.len() {
                out.push(chars[i]);
                if chars[i] == '\\' {
                    if let Some(&escaped) = chars.get(i + 1) {
                        out.push(escaped);
                    }
                    i += 2;
                    continue;
                }
                i += 1;
                if chars[i - 1] == '"' {
                    break;
                }
            }
        } else if c == '\'' {
            // a char literal ('x', '\n', '\'') is copied whole; a lifetime ('a) is just a quote
            let len = if next == Some('\\') {
                chars[i + 2..]
                    .iter()
                    .position(|&q| q == '\'')
                    .map(|p| p + 3)
            } else if chars.get(i + 2) == Some(&'\'') {
                Some(3)
            } else {
                None
            };
            let len = len.unwrap_or(1).min(chars.len() - i);
            out.extend(&chars[i..i + len]);
            i += len;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// The words of a line of code: runs of letters and digits, split again at a lower-to-upper case change.
fn words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for c in line.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() && previous_lower && !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            previous_lower = c.is_lowercase() || c.is_ascii_digit();
            current.push(c);
        } else {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            previous_lower = false;
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Every country, region or single-jurisdiction law named in the code of `source` (comments excluded).
fn scan(file: &str, source: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for (index, line) in without_comments(source).lines().enumerate() {
        let found = words(line);
        let lower: Vec<String> = found.iter().map(|w| w.to_lowercase()).collect();
        let mut push = |term: String| {
            hits.push(Hit {
                file: file.to_string(),
                line: index + 1,
                term,
            });
        };
        for (word, low) in found.iter().zip(&lower) {
            if WORDS.contains(&low.as_str()) {
                push(low.clone());
            } else if CODES.contains(&word.as_str()) {
                push(word.clone());
            }
        }
        for phrase in PHRASES {
            for window in lower.windows(phrase.len()) {
                if window.iter().zip(phrase.iter()).all(|(w, p)| w == p) {
                    push(phrase.join(" "));
                }
            }
        }
    }
    hits
}

fn rule_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src/rules is readable") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules"),
        &mut out,
    );
    out.sort();
    out
}

/// The hits not covered by `allowed`, and the allow-list entries that cover nothing.
fn judge<'a>(
    hits: &[Hit],
    allowed: &'a [(&'a str, &'a str, &'a str)],
) -> (Vec<Hit>, Vec<&'a (&'a str, &'a str, &'a str)>) {
    let refused = hits
        .iter()
        .filter(|h| {
            !allowed
                .iter()
                .any(|(file, term, _)| *file == h.file && *term == h.term)
        })
        .cloned()
        .collect();
    let unused = allowed
        .iter()
        .filter(|(file, term, _)| !hits.iter().any(|h| h.file == *file && h.term == *term))
        .collect();
    (refused, unused)
}

#[test]
fn the_pure_core_names_no_country_outside_the_allow_list() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
    let mut hits = Vec::new();
    for path in rule_files() {
        let name = path
            .strip_prefix(&root)
            .expect("under src/rules")
            .to_string_lossy()
            .replace('\\', "/");
        let source = std::fs::read_to_string(&path).expect("readable source");
        hits.extend(scan(&name, &source));
    }
    let (refused, unused) = judge(&hits, ALLOWED);
    assert!(
        refused.is_empty(),
        "src/rules names a country or one jurisdiction's law in code (take it as a parameter, or add an ALLOWED \
         entry with its reason): {refused:#?}"
    );
    assert!(
        unused.is_empty(),
        "ALLOWED entries that no longer match anything (remove them): {unused:#?}"
    );
}

#[test]
fn a_planted_country_name_is_caught_and_a_comment_is_not() {
    let planted = "/// The rest gap in the United Kingdom.\n\
                   pub const UK_MAX_WEEKLY_MINUTES: i64 = 48 * 60;\n\
                   fn japan_weekend() -> &'static str { \"Saturday and Sunday\" }\n\
                   const REGIME: &str = \"Working Time Regulations 1998\";\n\
                   const CODE: &str = \"DE\";\n\
                   fn indiaProfile<'a>(x: &'a str) -> char { 'u' } // India, in a comment\n\
                   const S: &str = r#\"under the GDPR\"#;\n\
                   /* the NHS, in a block comment */ const K: i64 = 1;\n";
    let terms: Vec<(usize, String)> = scan("planted.rs", planted)
        .into_iter()
        .map(|h| (h.line, h.term))
        .collect();
    assert_eq!(
        terms,
        [
            (2, "uk".to_string()),
            (3, "japan".to_string()),
            (4, "working time regulations".to_string()),
            (5, "DE".to_string()),
            (6, "india".to_string()),
            (7, "gdpr".to_string()),
        ]
    );
    let (refused, _) = judge(&scan("planted.rs", planted), ALLOWED);
    assert_eq!(
        refused.len(),
        6,
        "an allow-list entry for another file does not cover a planted name"
    );
}

#[test]
fn ordinary_words_and_the_pronoun_are_not_country_names() {
    let clean = "let us = in_range(x); // nothing\nconst SQL: &str = \"WHERE id IN (1)\";\nfn usable() {}\n";
    assert_eq!(scan("clean.rs", clean), Vec::<Hit>::new());
}

#[test]
fn an_allow_list_entry_covers_only_its_own_file_and_term_and_a_stale_one_is_reported() {
    let hits = scan("a.rs", "const X: &str = \"GB\";\nconst Y: &str = \"EU\";\n");
    let allowed = [("a.rs", "gb", "test"), ("b.rs", "eu", "elsewhere")];
    let (refused, unused) = judge(&hits, &allowed);
    assert_eq!(
        refused.iter().map(|h| h.term.as_str()).collect::<Vec<_>>(),
        ["eu"]
    );
    assert_eq!(unused.len(), 1);
}
