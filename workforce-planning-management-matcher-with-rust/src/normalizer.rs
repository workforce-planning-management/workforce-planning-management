//! Text normalisation: names, emails, phone numbers, phonetic codes.
//!
//! Every function is pure. Input that cannot be normalised gives `None`, never
//! a made-up value, so the matcher can treat it as unknown.

use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Letters that do not decompose under NFKD but have a plain-letter spelling.
fn fold_special(c: char) -> Option<&'static str> {
    Some(match c {
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'ø' => "o",
        'đ' | 'ð' => "d",
        'ł' => "l",
        'þ' => "th",
        'ı' => "i",
        _ => return None,
    })
}

/// Lowercase, strip diacritics, turn punctuation into single spaces.
///
/// `"Siân O'Brien-Smith"` becomes `"sian o brien smith"`. Returns `None` when
/// nothing alphanumeric is left, so an empty or punctuation-only name is
/// unknown rather than a name that matches every other empty name.
pub fn fold(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut gap = true;
    for c in text
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
    {
        if let Some(plain) = fold_special(c) {
            out.push_str(plain);
            gap = false;
        } else if c.is_alphanumeric() {
            out.push(c);
            gap = false;
        } else if !gap {
            out.push(' ');
            gap = true;
        }
    }
    let out = out.trim_end().to_string();
    (!out.is_empty()).then_some(out)
}

/// Canonical email: trimmed, lowercase, exactly one `@`, both sides non-empty.
///
/// With `fold_gmail`, `j.smith+hr@gmail.com` becomes `jsmith@gmail.com`
/// (`googlemail.com` is the same mailbox).
pub fn email(text: &str, fold_gmail: bool) -> Option<String> {
    let lower = text.trim().to_lowercase();
    let (local, domain) = lower.split_once('@')?;
    if local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || lower.contains(char::is_whitespace)
    {
        return None;
    }
    if fold_gmail && (domain == "gmail.com" || domain == "googlemail.com") {
        let local = local.split('+').next().unwrap_or(local).replace('.', "");
        return (!local.is_empty()).then(|| format!("{local}@gmail.com"));
    }
    Some(lower)
}

/// Phone number in `+<digits>` form, or `None`.
///
/// `+44 20 7946 0958` and `0044 20 7946 0958` are international. A number with
/// a leading `0` is national and needs `default_country_code` (digits, no `+`);
/// without one it is unknown, because guessing a country would match strangers.
/// Anything outside 8 to 15 digits is not a phone number.
pub fn phone(text: &str, default_country_code: Option<&str>) -> Option<String> {
    let trimmed = text.trim();
    let digits: String = trimmed.chars().filter(char::is_ascii_digit).collect();
    let international = if trimmed.starts_with('+') {
        digits
    } else if let Some(rest) = digits.strip_prefix("00") {
        rest.to_string()
    } else {
        let rest = digits.strip_prefix('0')?;
        format!("{}{rest}", default_country_code?)
    };
    (8..=15)
        .contains(&international.len())
        .then(|| format!("+{international}"))
}

/// An employer worker number with case, spaces and dashes removed.
pub fn worker_number(text: &str) -> Option<String> {
    let n: String = text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    (!n.is_empty()).then_some(n)
}

/// American Soundex of a folded name, or `None` when it has no letters.
///
/// Used only as a tie-breaker for spelling variants (`Stephen` / `Steven`), never
/// as evidence by itself.
pub fn soundex(folded: &str) -> Option<String> {
    fn code(c: char) -> u8 {
        match c {
            'b' | 'f' | 'p' | 'v' => b'1',
            'c' | 'g' | 'j' | 'k' | 'q' | 's' | 'x' | 'z' => b'2',
            'd' | 't' => b'3',
            'l' => b'4',
            'm' | 'n' => b'5',
            'r' => b'6',
            _ => 0,
        }
    }
    let mut letters = folded.chars().filter(char::is_ascii_lowercase);
    let first = letters.next()?;
    let mut out = String::from(first.to_ascii_uppercase());
    let mut last = code(first);
    for c in letters {
        let k = code(c);
        if k != 0 && k != last {
            out.push(k as char);
            if out.len() == 4 {
                break;
            }
        }
        // h and w do not separate equal codes; vowels do
        if c != 'h' && c != 'w' {
            last = k;
        }
    }
    while out.len() < 4 {
        out.push('0');
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_strips_diacritics_case_and_punctuation() {
        assert_eq!(fold("Siân").as_deref(), Some("sian"));
        assert_eq!(fold("  O'Brien-Smith ").as_deref(), Some("o brien smith"));
        assert_eq!(fold("Müller").as_deref(), Some("muller"));
        assert_eq!(fold("Strauß").as_deref(), Some("strauss"));
        assert_eq!(fold("Łukasz").as_deref(), Some("lukasz"));
    }

    #[test]
    fn fold_of_nothing_is_unknown() {
        assert_eq!(fold(""), None);
        assert_eq!(fold("  -- ' "), None);
    }

    #[test]
    fn email_is_canonical_or_unknown() {
        assert_eq!(
            email(" Jo@Example.COM ", false).as_deref(),
            Some("jo@example.com")
        );
        assert_eq!(email("no-at-sign", false), None);
        assert_eq!(email("a@b@c", false), None);
        assert_eq!(email("@example.com", false), None);
        assert_eq!(email("jo @example.com", false), None);
    }

    #[test]
    fn gmail_folding_is_opt_in() {
        assert_eq!(
            email("J.Smith+hr@gmail.com", false).as_deref(),
            Some("j.smith+hr@gmail.com")
        );
        assert_eq!(
            email("J.Smith+hr@gmail.com", true).as_deref(),
            Some("jsmith@gmail.com")
        );
        assert_eq!(
            email("jsmith@googlemail.com", true).as_deref(),
            Some("jsmith@gmail.com")
        );
        assert_eq!(
            email("j.smith@example.com", true).as_deref(),
            Some("j.smith@example.com")
        );
    }

    #[test]
    fn phone_international_forms_agree() {
        let a = phone("+44 20 7946 0958", None);
        assert_eq!(a.as_deref(), Some("+442079460958"));
        assert_eq!(phone("0044 (20) 7946-0958", None), a);
        assert_eq!(phone("020 7946 0958", Some("44")), a);
    }

    #[test]
    fn phone_national_without_a_country_is_unknown() {
        assert_eq!(phone("020 7946 0958", None), None);
        assert_eq!(phone("2079460958", Some("44")), None);
        assert_eq!(phone("+1234", None), None);
        assert_eq!(phone("+1234567890123456", None), None);
    }

    #[test]
    fn worker_number_ignores_case_and_separators() {
        assert_eq!(worker_number(" ab-1042 ").as_deref(), Some("ab1042"));
        assert_eq!(worker_number("---"), None);
    }

    #[test]
    fn soundex_matches_the_reference_codes() {
        assert_eq!(soundex("robert").as_deref(), Some("R163"));
        assert_eq!(soundex("rupert").as_deref(), Some("R163"));
        assert_eq!(soundex("stephen"), soundex("steven"));
        assert_eq!(soundex("ashcraft").as_deref(), Some("A261"));
        assert_eq!(soundex("lee").as_deref(), Some("L000"));
        assert_eq!(soundex("123"), None);
    }
}
