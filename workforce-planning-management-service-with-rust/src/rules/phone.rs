//! **A telephone number kept three ways** (WPM-D152 question 2, WPM-T382): as typed (unchanged), as ASCII digits (for
//! search and export), and in E.164 where it can be derived. Pure: the country is supplied.
//!
//! The E.164 form is derived only when the number was written with `+` (its digits are then the country calling code
//! and the national number), or when its country is known and is in the calling-code table (`data/calling-codes.tsv`,
//! a code list kept as data, each row the deployer's to correct). For a known country the number is read as a
//! national one: the plan's trunk prefix, where it has one, is dropped and the calling code put in front. The North
//! American plan (`1`) takes a ten-digit national number, or eleven digits starting `1`. A number starting `00` (an
//! international access prefix whose meaning depends on where it is dialled from), a country not in the table, or a
//! result outside E.164's 15 digits has **no** E.164 form: it is absent, never guessed.

use super::contact_details::{digit_value, fold_width, phone_digits};

/// The table: `country<TAB>calling code<TAB>trunk prefix or ->`, with `#` comment lines.
const CALLING_CODES: &str = include_str!("data/calling-codes.tsv");

/// The most digits an E.164 number holds, the country calling code included.
pub const E164_MAX: usize = 15;
/// The fewest digits a national number keeps after its trunk prefix, for a derived form.
pub const NATIONAL_MIN: usize = 4;

/// A country's numbering plan, as far as deriving E.164 needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    /// ISO 3166-1 alpha-2.
    pub country: &'static str,
    /// The E.164 country calling code, digits only.
    pub calling_code: &'static str,
    /// The national trunk prefix, where the plan has one.
    pub trunk: Option<&'static str>,
}

/// Every plan in the table, in its order.
pub fn plans() -> impl Iterator<Item = Plan> {
    CALLING_CODES
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let (country, code, trunk) = (parts.next()?, parts.next()?, parts.next()?);
            Some(Plan {
                country,
                calling_code: code,
                trunk: (trunk != "-").then_some(trunk),
            })
        })
}

/// The plan for `country` (two capital letters), if the table holds it.
#[must_use]
pub fn plan(country: &str) -> Option<Plan> {
    plans().find(|p| p.country == country)
}

/// The country part of a jurisdiction code (`IN-MH` → `IN`, `JP` → `JP`); `None` for a code whose first part is not
/// two letters (a supranational code such as `EU` is two letters, but is in no plan, so it derives nothing).
#[must_use]
pub fn country_of_place(code: &str) -> Option<&str> {
    let head = code.split('-').next()?;
    (head.len() == 2 && head.bytes().all(|b| b.is_ascii_uppercase())).then_some(head)
}

/// The number's digits in ASCII, in order, whatever digit set it was typed in; `None` when it is not shaped like a
/// telephone number ([`phone_digits`] refuses it).
#[must_use]
pub fn ascii_digits(value: &str) -> Option<String> {
    phone_digits(value)?;
    Some(
        value
            .chars()
            .filter_map(digit_value)
            .filter_map(|d| char::from_digit(d, 10))
            .collect(),
    )
}

/// Whether the number was written with a leading `+` (full width counts).
#[must_use]
pub fn written_international(value: &str) -> bool {
    value.chars().map(fold_width).find(|c| !c.is_whitespace()) == Some('+')
}

/// The E.164 form (`+` and up to 15 digits), where it can be derived without guessing; see the module notes.
#[must_use]
pub fn e164(value: &str, country: Option<&str>) -> Option<String> {
    let digits = ascii_digits(value)?;
    if written_international(value) {
        return (!digits.starts_with('0') && digits.len() <= E164_MAX)
            .then(|| format!("+{digits}"));
    }
    let plan = plan(country?)?;
    if digits.starts_with("00") {
        return None;
    }
    if plan.calling_code == "1" {
        let first_ok = |d: &str| d.chars().next().is_some_and(|c| ('2'..='9').contains(&c));
        return match digits.len() {
            10 if first_ok(&digits) => Some(format!("+1{digits}")),
            11 if digits.starts_with('1') && first_ok(&digits[1..]) => Some(format!("+{digits}")),
            _ => None,
        };
    }
    let national = plan
        .trunk
        .and_then(|trunk| digits.strip_prefix(trunk))
        .unwrap_or(&digits);
    (national.len() >= NATIONAL_MIN && plan.calling_code.len() + national.len() <= E164_MAX)
        .then(|| format!("+{}{national}", plan.calling_code))
}

/// The two derived forms of one number.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Forms {
    /// ASCII digits, for search and export.
    pub digits: Option<String>,
    /// E.164, where derivable.
    pub e164: Option<String>,
}

/// Both derived forms of `value` (absent for an absent number).
#[must_use]
pub fn forms(value: Option<&str>, country: Option<&str>) -> Forms {
    value.map_or_else(Forms::default, |v| Forms {
        digits: ascii_digits(v),
        e164: e164(v, country),
    })
}

/// The country a worker's own number is read in: the address's country, else the country of the place the work is
/// done (`work_jurisdiction`). The first that is present is used, even when it has no plan: a number is never read in
/// a second country because the first had none.
#[must_use]
pub fn country_for<'a>(
    address_country: Option<&'a str>,
    work_jurisdiction: Option<&'a str>,
) -> Option<&'a str> {
    address_country.or_else(|| work_jurisdiction.and_then(country_of_place))
}

/// The ASCII digits a search for `query` compares, with the leading zeros a trunk prefix may add removed; `None` when
/// the query is not shaped like a telephone number.
#[must_use]
pub fn search_digits(query: &str) -> Option<String> {
    ascii_digits(query).map(|d| d.trim_start_matches('0').to_string())
}

/// The fewest digits a search compares, once leading zeros are gone.
pub const SEARCH_MIN: usize = 5;
/// The fewest digits a search compares against the end of a stored E.164 form (a national number found under its
/// international form).
pub const SUFFIX_MIN: usize = 7;

/// Whether a stored number (its ASCII digits and its E.164 form) answers a search for `query`: the digits are equal once
/// leading zeros are gone; or the query's own E.164 form (written with `+`) is the stored one; or the query, at least
/// [`SUFFIX_MIN`] digits, ends the stored E.164 form. Nothing is compared as typed.
#[must_use]
pub fn matches(stored_digits: Option<&str>, stored_e164: Option<&str>, query: &str) -> bool {
    let Some(key) = search_digits(query).filter(|k| k.len() >= SEARCH_MIN) else {
        return false;
    };
    if stored_digits.is_some_and(|d| d.trim_start_matches('0') == key) {
        return true;
    }
    stored_e164.is_some_and(|e| {
        e164(query, None).as_deref() == Some(e) || (key.len() >= SUFFIX_MIN && e.ends_with(&key))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_come_out_in_ascii_whatever_script_they_were_typed_in() {
        assert_eq!(
            ascii_digits("+٩٦٦ ٥٠ ١٢٣ ٤٥٦٧").as_deref(),
            Some("966501234567")
        );
        assert_eq!(
            ascii_digits("+৮৮০ ১৭১১ ২৩৪৫৬৭").as_deref(),
            Some("8801711234567")
        );
        assert_eq!(
            ascii_digits("＋８１ ９０-１２３４-５６７８").as_deref(),
            Some("819012345678")
        );
        assert_eq!(
            ascii_digits("(020) 7946-0958").as_deref(),
            Some("02079460958")
        );
        assert_eq!(
            ascii_digits("+66 ๘๑ ๒๓๔ ๕๖๗๘").as_deref(),
            Some("66812345678")
        );
        assert_eq!(ascii_digits("call me"), None);
        assert_eq!(ascii_digits("12+34"), None);
    }

    #[test]
    fn a_number_written_with_plus_is_its_own_e164_whatever_the_country() {
        assert_eq!(
            e164("+44 7700 900123", None).as_deref(),
            Some("+447700900123")
        );
        assert_eq!(
            e164("+44 7700 900123", Some("KR")).as_deref(),
            Some("+447700900123")
        );
        assert_eq!(
            e164("＋８１ ９０-１２３４-５６７８", None).as_deref(),
            Some("+819012345678")
        );
        assert_eq!(
            e164("+٩٦٦ ٥٠ ١٢٣ ٤٥٦٧", None).as_deref(),
            Some("+966501234567")
        );
        assert_eq!(
            e164("+0 123 4567", None),
            None,
            "no calling code starts with 0"
        );
        // A + number is taken from any country, even one with no plan in the table.
        assert_eq!(
            e164("+977 1 4123456", Some("NP")).as_deref(),
            Some("+97714123456")
        );
    }

    #[test]
    fn a_national_number_needs_a_known_country_and_loses_its_trunk_prefix() {
        assert_eq!(e164("07700 900123", None), None, "no country, no E.164");
        assert_eq!(
            e164("07700 900123", Some("NP")),
            None,
            "a country not in the table"
        );
        assert_eq!(
            e164("03-1234-5678", Some("KR")).as_deref(),
            Some("+82312345678")
        );
        assert_eq!(
            e164("०९८७६ ५४३२१०", Some("IN")).as_deref(),
            Some("+919876543210")
        );
        // No trunk prefix: the leading 0 is part of the number.
        assert_eq!(
            e164("06 1234 5678", Some("IT")).as_deref(),
            Some("+390612345678")
        );
        // A two-digit trunk prefix, and one that is not 0.
        assert_eq!(
            e164("06 1 234 5678", Some("HU")).as_deref(),
            Some("+3612345678")
        );
        assert_eq!(
            e164("8 495 123-45-67", Some("RU")).as_deref(),
            Some("+74951234567")
        );
        // An international access prefix is not read.
        assert_eq!(e164("0044 7700 900123", Some("IT")), None);
        // Too short once the prefix is gone, or too long for E.164.
        assert_eq!(e164("0 123", Some("KR")), None);
        assert_eq!(e164("0123456789012345", Some("IT")), None);
    }

    #[test]
    fn the_plan_with_calling_code_one_takes_ten_digits_or_eleven_starting_with_one() {
        let nanp = plans()
            .find(|p| p.calling_code == "1")
            .expect("a plan with calling code 1");
        let c = Some(nanp.country);
        assert_eq!(e164("(202) 555-0123", c).as_deref(), Some("+12025550123"));
        assert_eq!(e164("1 202 555 0123", c).as_deref(), Some("+12025550123"));
        assert_eq!(
            e164("555-0123", c),
            None,
            "seven digits: the area code is unknown"
        );
        assert_eq!(
            e164("(102) 555-0123", c),
            None,
            "an area code does not start with 1"
        );
        assert_eq!(
            e164("011 44 7700 900123", c),
            None,
            "an access prefix is not read"
        );
    }

    #[test]
    fn the_country_is_the_address_s_else_the_place_of_work_s_and_never_a_second_guess() {
        assert_eq!(country_for(Some("KR"), Some("IN-MH")), Some("KR"));
        assert_eq!(country_for(None, Some("IN-MH")), Some("IN"));
        assert_eq!(country_for(None, Some("EU")), Some("EU"));
        assert_eq!(
            e164("0 98765 43210", country_for(None, Some("EU"))),
            None,
            "a bloc has no plan"
        );
        assert_eq!(country_for(None, Some("QZ1")), None);
        assert_eq!(country_for(None, None), None);
        // An address country with no plan is not replaced by the place of work's.
        assert_eq!(
            e164("01 4123456", country_for(Some("NP"), Some("IN"))),
            None
        );
        assert_eq!(country_of_place("IN-MH"), Some("IN"));
        assert_eq!(country_of_place("ABC"), None);
    }

    #[test]
    fn both_forms_and_the_search_key() {
        assert_eq!(forms(None, Some("KR")), Forms::default());
        assert_eq!(
            forms(Some("010-1234-5678"), Some("KR")),
            Forms {
                digits: Some("01012345678".into()),
                e164: Some("+821012345678".into())
            }
        );
        assert_eq!(
            forms(Some("010-1234-5678"), None),
            Forms {
                digits: Some("01012345678".into()),
                e164: None
            }
        );
        assert_eq!(search_digits("07700 900123").as_deref(), Some("7700900123"));
        assert_eq!(search_digits("nope"), None);
    }

    #[test]
    fn a_search_compares_digits_never_the_typed_text() {
        let stored = forms(Some("٠١٠-١٢٣٤-٥٦٧٨"), Some("KR"));
        let (d, e) = (stored.digits.as_deref(), stored.e164.as_deref());
        assert!(
            matches(d, e, "010 1234 5678"),
            "the same digits typed in ASCII"
        );
        assert!(matches(d, e, "+82 10-1234-5678"), "its international form");
        assert!(matches(d, e, "10 1234 5678"), "without the trunk prefix");
        assert!(!matches(d, e, "010 1234 5679"));
        assert!(!matches(d, e, "5678"), "too few digits");
        assert!(!matches(d, e, "call me"));
        // A national number with no E.164 form is still found by its digits, not by an international query.
        let bare = forms(Some("07700 900123"), None);
        assert!(matches(bare.digits.as_deref(), None, "7700900123"));
        assert!(!matches(bare.digits.as_deref(), None, "+44 7700 900123"));
        assert!(!matches(None, None, "07700 900123"));
    }

    #[test]
    fn the_migration_s_frozen_copies_are_the_rule_s_tables() {
        use migration::m20261031_000114_phone_forms as frozen;
        assert_eq!(
            frozen::DIGIT_ZEROS,
            super::super::contact_details::DIGIT_ZEROS
        );
        let rule: Vec<(&str, &str, Option<&str>)> = plans()
            .map(|p| (p.country, p.calling_code, p.trunk))
            .collect();
        assert_eq!(
            frozen::CALLING_CODES,
            rule.as_slice(),
            "the backfill read the table the rule reads"
        );
    }

    #[test]
    fn the_table_is_well_formed() {
        let all: Vec<Plan> = plans().collect();
        assert!(all.len() >= 90);
        let mut countries: Vec<&str> = all.iter().map(|p| p.country).collect();
        countries.sort_unstable();
        countries.dedup();
        assert_eq!(countries.len(), all.len(), "one row per country");
        for p in &all {
            assert!(
                p.country.len() == 2 && p.country.bytes().all(|b| b.is_ascii_uppercase()),
                "{p:?}"
            );
            assert!(
                !p.calling_code.is_empty() && p.calling_code.len() <= 3,
                "{p:?}"
            );
            assert!(
                p.calling_code.bytes().all(|b| b.is_ascii_digit())
                    && !p.calling_code.starts_with('0'),
                "{p:?}"
            );
            assert!(
                p.trunk
                    .is_none_or(|t| t.bytes().all(|b| b.is_ascii_digit())),
                "{p:?}"
            );
        }
    }
}
