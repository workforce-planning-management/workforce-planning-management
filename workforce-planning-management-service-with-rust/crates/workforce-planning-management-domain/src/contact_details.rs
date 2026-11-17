//! **A worker's own contact details** (WPM-R124): pure validation of the address, telephone
//! numbers and personal e-mail a worker keeps about themself.
//!
//! Validation is by *shape* only: a length, the characters a telephone number may hold, the
//! outline of an e-mail address. Nothing is looked up, nothing is sent, and no address is
//! "verified". A blank value is no value, and a worker can leave any field empty.
//!
//! **No one country's shapes (WPM-T382, WPM-R214).** A postcode is a short line of any script,
//! not a pattern; a telephone number's digits may be typed in any of the decimal digit sets in
//! [`DIGIT_ZEROS`] (Arabic-Indic, Devanagari, Bengali, full-width and others) as well as ASCII. A
//! value is stored as typed (trimmed); nothing is rewritten to ASCII or to E.164.

/// The longest any free-text line may be, in characters.
pub const MAX_LINE: usize = 120;
/// The longest e-mail address the standard allows.
pub const MAX_EMAIL: usize = 254;
/// A telephone number holds this many digits at least and at most (the international maximum).
pub const DIGITS: std::ops::RangeInclusive<usize> = 5..=15;
/// The longest postcode, in characters: a length, not a shape.
pub const MAX_POSTCODE: usize = 20;
/// The longest free-text address block, in characters (WPM-D152: the fallback for a country the UI's
/// address table lacks).
pub const MAX_ADDRESS_TEXT: usize = 500;
/// The most lines a free-text address block may have.
pub const MAX_ADDRESS_LINES: usize = 10;

/// The zero of each Unicode decimal digit set a telephone number may be typed in; the nine digits
/// after it are 1 to 9. ASCII, and full-width (folded by [`fold_width`]), are handled before this.
/// Arabic-Indic, Extended Arabic-Indic (Persian, Urdu), N'Ko, Devanagari, Bengali, Gurmukhi,
/// Gujarati, Oriya, Tamil, Telugu, Kannada, Malayalam, Sinhala Lith, Thai, Lao, Tibetan, Myanmar,
/// Myanmar Shan, Khmer, Mongolian, Limbu, New Tai Lue, Balinese, Sundanese, Lepcha, Ol Chiki, Vai,
/// Javanese, Myanmar Tai Laing, Cham, Meetei Mayek.
pub const DIGIT_ZEROS: [u32; 31] = [
    0x0660, 0x06f0, 0x07c0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66, 0x0ce6, 0x0d66,
    0x0de6, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0, 0x1b50, 0x1bb0,
    0x1c40, 0x1c50, 0xa620, 0xa9d0, 0xa9f0, 0xaa50, 0xabf0,
];

/// A full-width form (`＋`, `８`, `（`) as its ASCII one, and the ideographic space as a space, as an
/// East Asian keyboard types them; any other character unchanged.
#[must_use]
pub fn fold_width(c: char) -> char {
    match u32::from(c) {
        0x3000 => ' ',
        code @ 0xff01..=0xff5e => char::from_u32(code - 0xfee0).unwrap_or(c),
        _ => c,
    }
}

/// The value 0 to 9 of a decimal digit in ASCII, full-width or any set in [`DIGIT_ZEROS`]; `None`
/// otherwise (a superscript or a Roman numeral is not a digit).
#[must_use]
pub fn digit_value(c: char) -> Option<u32> {
    if let Some(d) = fold_width(c).to_digit(10) {
        return Some(d);
    }
    let code = u32::from(c);
    DIGIT_ZEROS
        .iter()
        .find(|zero| (**zero..**zero + 10).contains(&code))
        .map(|zero| code - zero)
}

/// How many digits a telephone number holds, or `None` when it holds a character a number does
/// not: digits in any set above, spaces, hyphens, dots and brackets, and one leading `+`. Full-width
/// forms (`＋８１`) count as their ASCII ones.
#[must_use]
pub fn phone_digits(value: &str) -> Option<usize> {
    let folded: String = value.chars().map(fold_width).collect();
    let body = folded.strip_prefix('+').unwrap_or(&folded);
    let mut digits = 0;
    for c in body.chars() {
        if digit_value(c).is_some() {
            digits += 1;
        } else if !matches!(c, ' ' | '-' | '.' | '(' | ')') {
            return None;
        }
    }
    Some(digits)
}

/// Trim, and treat blank as absent.
#[must_use]
pub fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

/// A free-text address block as typed, in the person's own order: line endings become `\n`, each
/// line loses its trailing spaces, and blank lines at the start and the end are dropped. Blank is
/// absent. Nothing inside a line is rewritten.
#[must_use]
pub fn clean_block(value: Option<&str>) -> Option<String> {
    let value = value?.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = value.split('\n').map(str::trim_end).collect();
    let first = lines.iter().position(|l| !l.trim().is_empty())?;
    let last = lines.iter().rposition(|l| !l.trim().is_empty())?;
    Some(lines[first..=last].join("\n"))
}

/// A free-text address block: up to [`MAX_ADDRESS_TEXT`] characters on up to [`MAX_ADDRESS_LINES`]
/// lines, with no control character other than the line break. No country's order is assumed.
///
/// # Errors
///
/// Too long, too many lines, or holding a control character.
pub fn validate_address_text(value: &str) -> Result<(), String> {
    if value.chars().count() > MAX_ADDRESS_TEXT {
        return Err(format!(
            "address_text is longer than {MAX_ADDRESS_TEXT} characters"
        ));
    }
    if value.split('\n').count() > MAX_ADDRESS_LINES {
        return Err(format!(
            "address_text has more than {MAX_ADDRESS_LINES} lines"
        ));
    }
    if value.chars().any(|c| c.is_control() && c != '\n') {
        return Err(
            "address_text must not contain control characters other than a line break".to_string(),
        );
    }
    Ok(())
}

/// A line of an address, a city or a region: not too long, no control characters.
///
/// # Errors
///
/// Too long, or holding a control character.
pub fn validate_line(field: &str, value: &str) -> Result<(), String> {
    if value.chars().count() > MAX_LINE {
        return Err(format!("{field} is longer than {MAX_LINE} characters"));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("{field} must not contain control characters"));
    }
    Ok(())
}

/// A postcode: any text of up to [`MAX_POSTCODE`] characters with no control characters. No
/// country's pattern is assumed: letters and digits of any script, and any punctuation a country's
/// codes use (a hyphen, a dot, a slash, the Japanese postal mark), are all accepted.
///
/// # Errors
///
/// Too long, or holding a control character.
pub fn validate_postcode(value: &str) -> Result<(), String> {
    if value.chars().count() > MAX_POSTCODE {
        return Err(format!("postcode is longer than {MAX_POSTCODE} characters"));
    }
    if value.chars().any(char::is_control) {
        return Err("postcode must not contain control characters".to_string());
    }
    Ok(())
}

/// A country as a two-letter ISO 3166 code in capitals.
///
/// # Errors
///
/// Anything but two capital letters.
pub fn validate_country(value: &str) -> Result<(), String> {
    if value.len() == 2 && value.bytes().all(|b| b.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err("country is a two-letter ISO 3166-1 alpha-2 code in capital letters".to_string())
    }
}

/// The ISO 3166-1 alpha-2 codes, one per line, with `#` comment lines (a code list kept as data).
const ISO_3166_1: &str = include_str!("data/iso-3166-1-alpha-2.txt");

/// Every officially assigned ISO 3166-1 alpha-2 code, in order.
pub fn iso_countries() -> impl Iterator<Item = &'static str> {
    ISO_3166_1
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

/// Whether `code` is an officially assigned ISO 3166-1 alpha-2 code.
#[must_use]
pub fn is_iso_country(code: &str) -> bool {
    iso_countries().any(|c| c == code)
}

/// A country that may be stored (WPM-D152 question 3): an ISO 3166-1 alpha-2 code, or one of `deployer_codes`, the codes
/// ISO 3166-1 lacks that the worker's organization has added (its national jurisdiction codes).
///
/// # Errors
///
/// Not two capital letters, or a code that is neither ISO 3166-1 nor the deployer's.
pub fn validate_country_known(value: &str, deployer_codes: &[String]) -> Result<(), String> {
    validate_country(value)?;
    if is_iso_country(value) || deployer_codes.iter().any(|c| c == value) {
        Ok(())
    } else {
        Err("country is neither an ISO 3166-1 code nor one the organization has added".to_string())
    }
}

/// A telephone number: digits (in any set [`phone_digits`] reads), spaces, hyphens, dots and
/// brackets, an optional leading `+`, and between five and fifteen digits. It is stored as typed.
///
/// # Errors
///
/// A character a number does not hold, a misplaced `+`, or too few or too many digits.
pub fn validate_phone(field: &str, value: &str) -> Result<(), String> {
    let Some(digits) = phone_digits(value) else {
        return Err(format!(
            "{field} holds digits, spaces, hyphens, dots and brackets, with an optional leading +"
        ));
    };
    if !DIGITS.contains(&digits) {
        return Err(format!(
            "{field} needs {} to {} digits",
            DIGITS.start(),
            DIGITS.end()
        ));
    }
    Ok(())
}

/// An e-mail address by outline: one `@`, something either side, a dot in the domain, no spaces.
/// It does not prove the address exists.
///
/// # Errors
///
/// Anything that does not have that outline, or is too long.
pub fn validate_email(value: &str) -> Result<(), String> {
    let shaped = value.len() <= MAX_EMAIL
        && !value.chars().any(|c| c.is_whitespace() || c.is_control())
        && value.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && !domain.contains('@')
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !domain.contains("..")
        });
    if shaped {
        Ok(())
    } else {
        Err("personal_email is not shaped like an e-mail address".to_string())
    }
}

/// The fields a worker keeps about themself, after cleaning.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Details {
    /// First address line.
    pub address_line1: Option<String>,
    /// Second address line.
    pub address_line2: Option<String>,
    /// Town or city.
    pub city: Option<String>,
    /// County, state or region.
    pub region: Option<String>,
    /// Postcode.
    pub postcode: Option<String>,
    /// The whole address as one free-text block, in the person's own order (WPM-D152), for a
    /// country the UI's address table lacks.
    pub address_text: Option<String>,
    /// Country, two capital letters.
    pub country: Option<String>,
    /// Mobile telephone number.
    pub phone_mobile: Option<String>,
    /// Home telephone number.
    pub phone_home: Option<String>,
    /// Personal e-mail address.
    pub personal_email: Option<String>,
}

impl Details {
    /// Whether nothing is recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Check every field that is present.
///
/// # Errors
///
/// The first field that fails, named.
pub fn validate(details: &Details) -> Result<(), String> {
    for (field, value) in [
        ("address_line1", &details.address_line1),
        ("address_line2", &details.address_line2),
        ("city", &details.city),
        ("region", &details.region),
    ] {
        if let Some(v) = value {
            validate_line(field, v)?;
        }
    }
    if let Some(v) = &details.postcode {
        validate_postcode(v)?;
    }
    if let Some(v) = &details.address_text {
        validate_address_text(v)?;
    }
    if let Some(v) = &details.country {
        validate_country(v)?;
    }
    if let Some(v) = &details.phone_mobile {
        validate_phone("phone_mobile", v)?;
    }
    if let Some(v) = &details.phone_home {
        validate_phone("phone_home", v)?;
    }
    if let Some(v) = &details.personal_email {
        validate_email(v)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_is_absent() {
        assert_eq!(clean(None), None);
        assert_eq!(clean(Some("")), None);
        assert_eq!(clean(Some("   ")), None);
        assert_eq!(clean(Some("  12 High St ")), Some("12 High St".to_string()));
    }

    #[test]
    fn telephone_numbers_are_judged_by_shape() {
        for ok in [
            "01632 960001",
            "+44 20 7946 0958",
            "(020) 7946-0958",
            "+1.202.555.0123",
            "12345",
        ] {
            assert!(validate_phone("phone", ok).is_ok(), "{ok}");
        }
        for bad in [
            "1234",
            "1234567890123456",
            "abc12345",
            "07700 9000x1",
            "44+2079460958",
            "++4420794609",
        ] {
            assert!(validate_phone("phone", bad).is_err(), "{bad}");
        }
        // The digit count is what counts, not the punctuation.
        assert!(validate_phone("phone", "+1-2-3-4-5").is_ok());
        assert!(validate_phone("phone", "+1-2-3-4").is_err());
    }

    #[test]
    fn telephone_digits_may_be_typed_in_any_script() {
        for ok in [
            "+٩٦٦ ٥٠ ١٢٣ ٤٥٦٧",              // Arabic-Indic
            "+۹۲ ۳۰۰ ۱۲۳۴۵۶۷",               // Extended Arabic-Indic (Urdu, Persian)
            "+९१ ९८७६५ ४३२१०",               // Devanagari
            "+৮৮০ ১৭১১ ২৩৪৫৬৭",              // Bengali
            "＋８１ ９０-１２３４-５６７８", // full-width, as a Japanese keyboard types it
            "+66 ๘๑ ๒๓๔ ๕๖๗๘",               // Thai digits after an ASCII country code
        ] {
            assert!(validate_phone("phone", ok).is_ok(), "{ok}");
        }
        assert_eq!(phone_digits("+٩٦٦ ٥٠"), Some(5));
        assert_eq!(digit_value('৭'), Some(7));
        assert_eq!(digit_value('²'), None, "a superscript is not a digit");
        assert_eq!(digit_value('Ⅻ'), None);
        assert_eq!(digit_value('８'), Some(8));
        assert!(validate_phone("phone", "²³⁴⁵⁶").is_err());
        assert!(
            validate_phone("phone", "١٢٣٤").is_err(),
            "four digits are too few in any script"
        );
        assert!(
            validate_phone("phone", "٥٠+١٢٣٤٥").is_err(),
            "a + only leads"
        );
        assert!(validate_phone("phone", "٥٠ ١٢٣ x").is_err());
        // every set in the table is ten consecutive code points that Unicode calls decimal digits
        for zero in DIGIT_ZEROS {
            for d in 0..10 {
                let c = char::from_u32(zero + d).expect("a char");
                assert!(c.is_numeric(), "{zero:#x} + {d}");
                assert_eq!(digit_value(c), Some(d));
            }
        }
    }

    #[test]
    fn email_is_judged_by_outline_only() {
        for ok in ["a@b.co", "first.last@example.org", "x+tag@sub.example.com"] {
            assert!(validate_email(ok).is_ok(), "{ok}");
        }
        for bad in [
            "", "a", "a@", "@b.co", "a@b", "a@@b.co", "a b@c.co", "a@b..co", "a@.co", "a@b.",
            "a@b.co\n",
        ] {
            assert!(validate_email(bad).is_err(), "{bad:?}");
        }
        assert!(validate_email(&format!("{}@b.co", "a".repeat(MAX_EMAIL))).is_err());
    }

    #[test]
    fn address_parts_are_bounded() {
        assert!(validate_line("city", "Leeds").is_ok());
        assert!(validate_line("city", &"x".repeat(MAX_LINE)).is_ok());
        assert!(validate_line("city", &"x".repeat(MAX_LINE + 1)).is_err());
        assert!(validate_line("city", "a\nb").is_err());
        // no country's pattern: any script, any punctuation, a bound on length only
        for ok in [
            "LS1 4AP",
            "10115",
            "12345-6789",
            "〒100-0001",
            "١١٥١١",
            "SE-123 45",
            "1234567890123",
            "Box 1234/5",
        ] {
            assert!(validate_postcode(ok).is_ok(), "{ok}");
        }
        assert!(validate_postcode(&"9".repeat(MAX_POSTCODE)).is_ok());
        assert!(validate_postcode(&"9".repeat(MAX_POSTCODE + 1)).is_err());
        assert!(validate_postcode("100\n0001").is_err());
        assert!(validate_country("GB").is_ok());
        assert!(validate_country("IN").is_ok());
        for bad in ["gb", "GBR", "G", "", "G1"] {
            assert!(validate_country(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_free_text_block_keeps_its_lines_and_order() {
        assert_eq!(clean_block(None), None);
        assert_eq!(clean_block(Some(" \n  \r\n")), None);
        assert_eq!(
            clean_block(Some("\r\n〒100-0001  \r\n東京都千代田区\r\n  1-1 \n\n")),
            Some("〒100-0001\n東京都千代田区\n  1-1".to_string()),
            "line endings unified, trailing spaces and outer blank lines gone, the rest as typed"
        );
        assert_eq!(clean_block(Some("a\rb")), Some("a\nb".to_string()));
        assert!(validate_address_text("Flat 2\n10 Rue X\n75001 Ville").is_ok());
        assert!(validate_address_text(&"x".repeat(MAX_ADDRESS_TEXT)).is_ok());
        assert!(validate_address_text(&"x".repeat(MAX_ADDRESS_TEXT + 1)).is_err());
        assert!(validate_address_text(&["a"; MAX_ADDRESS_LINES].join("\n")).is_ok());
        assert!(validate_address_text(&["a"; MAX_ADDRESS_LINES + 1].join("\n")).is_err());
        assert!(validate_address_text("a\tb").is_err());
        assert!(validate_address_text("a\u{0}b").is_err());
        let bad = Details {
            address_text: Some("a\u{7}".into()),
            ..Details::default()
        };
        assert!(validate(&bad).unwrap_err().contains("address_text"));
    }

    #[test]
    fn a_country_is_iso_3166_1_or_the_deployer_s_own() {
        let all: Vec<&str> = iso_countries().collect();
        assert_eq!(all.len(), 249, "the officially assigned codes");
        let mut sorted = all.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted, all, "sorted, no repeats");
        assert!(all.iter().all(|c| validate_country(c).is_ok()));
        for code in ["KR", "NP", "AX", "SS", "BQ"] {
            assert!(validate_country_known(code, &[]).is_ok(), "{code}");
        }
        // User-assigned and reserved codes are not ISO 3166-1's: a deployer adds one, or it is refused.
        for code in ["XK", "QZ", "ZZ", "AA", "EU", "UK"] {
            assert!(!is_iso_country(code), "{code}");
            assert!(
                validate_country_known(code, &[])
                    .unwrap_err()
                    .contains("neither"),
                "{code}"
            );
        }
        let added = vec!["XK".to_string()];
        assert!(validate_country_known("XK", &added).is_ok());
        assert!(validate_country_known("QZ", &added).is_err());
        assert!(
            validate_country_known("xk", &added)
                .unwrap_err()
                .contains("capital")
        );
    }

    #[test]
    fn an_empty_record_is_valid_and_a_bad_field_is_named() {
        assert!(Details::default().is_empty());
        assert!(validate(&Details::default()).is_ok());
        let bad = Details {
            phone_home: Some("nope".into()),
            ..Details::default()
        };
        assert!(!bad.is_empty());
        assert!(validate(&bad).unwrap_err().contains("phone_home"));
        let good = Details {
            address_line1: Some("1 High Street".into()),
            postcode: Some("LS1 4AP".into()),
            country: Some("GB".into()),
            phone_mobile: Some("+44 7700 900123".into()),
            personal_email: Some("me@example.org".into()),
            ..Details::default()
        };
        assert!(validate(&good).is_ok());
    }
}
