//! **A worker's own contact details** (WPM-R124): pure validation of the address, telephone
//! numbers and personal e-mail a worker keeps about themself.
//!
//! Validation is by *shape* only: a length, the characters a telephone number may hold, the
//! outline of an e-mail address. Nothing is looked up, nothing is sent, and no address is
//! "verified". A blank value is no value, and a worker can leave any field empty.

/// The longest any free-text line may be, in characters.
pub const MAX_LINE: usize = 120;
/// The longest e-mail address the standard allows.
pub const MAX_EMAIL: usize = 254;
/// A telephone number holds this many digits at least and at most (the international maximum).
pub const DIGITS: std::ops::RangeInclusive<usize> = 5..=15;

/// Trim, and treat blank as absent.
#[must_use]
pub fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
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

/// A postcode: letters, digits, spaces and hyphens, up to twelve characters.
///
/// # Errors
///
/// Any other character, or too long.
pub fn validate_postcode(value: &str) -> Result<(), String> {
    if value.chars().count() > 12
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-')
    {
        return Err("postcode is up to 12 letters, digits, spaces or hyphens".to_string());
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
        Err("country is a two-letter ISO 3166 code such as GB".to_string())
    }
}

/// A telephone number: digits, spaces, hyphens, dots and brackets, an optional leading `+`, and
/// between five and fifteen digits.
///
/// # Errors
///
/// A character a number does not hold, a misplaced `+`, or too few or too many digits.
pub fn validate_phone(field: &str, value: &str) -> Result<(), String> {
    let body = value.strip_prefix('+').unwrap_or(value);
    if !body
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '-' | '.' | '(' | ')'))
    {
        return Err(format!(
            "{field} holds digits, spaces, hyphens, dots and brackets, with an optional leading +"
        ));
    }
    let digits = body.chars().filter(char::is_ascii_digit).count();
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
        assert!(validate_postcode("LS1 4AP").is_ok());
        assert!(validate_postcode("10115").is_ok());
        assert!(validate_postcode("LS1 4AP; DROP").is_err());
        assert!(validate_postcode("1234567890123").is_err());
        assert!(validate_country("GB").is_ok());
        for bad in ["gb", "GBR", "G", "", "G1"] {
            assert!(validate_country(bad).is_err(), "{bad}");
        }
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
