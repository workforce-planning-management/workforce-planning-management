//! Pure rules for **emergency contacts**: the people a worker names to be
//! reached if something happens to them. Validation only — who may see or
//! edit a contact is the controller's authorization (the worker and HR).
//!
//! Deliberately lenient about *format* (phone numbers and names are
//! worldwide) and strict about *usefulness*: a contact must have a name, a
//! relationship, and a phone number someone could dial.

/// The most contacts one worker can hold.
pub const MAX_CONTACTS: usize = 5;

/// Longest free-text field accepted.
const MAX_TEXT: usize = 200;

/// Whether `phone` could be dialled: only digits and `+ - ( ) . space`,
/// with 5 to 15 digits (E.164 allows 15).
#[must_use]
pub fn phone_ok(phone: &str) -> bool {
    let allowed = phone
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '(' | ')' | '.' | ' '));
    let digits = phone.chars().filter(char::is_ascii_digit).count();
    allowed && (5..=15).contains(&digits)
}

/// A minimal email shape check: one `@`, text either side, a dot in the
/// domain, no spaces. Delivery is not verified.
#[must_use]
pub fn email_ok(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace)
}

/// Validate one contact.
///
/// # Errors
///
/// A message naming the first problem: a blank or over-long name or
/// relationship, an undialable phone (or alternate phone), a malformed
/// email, or a priority below 1.
pub fn validate_contact(
    name: &str,
    relationship: &str,
    phone: &str,
    alt_phone: Option<&str>,
    email: Option<&str>,
    priority: i32,
) -> Result<(), String> {
    if name.trim().is_empty() || name.len() > MAX_TEXT {
        return Err("name is required (up to 200 characters)".to_string());
    }
    if relationship.trim().is_empty() || relationship.len() > MAX_TEXT {
        return Err("relationship is required (up to 200 characters)".to_string());
    }
    if !phone_ok(phone) {
        return Err("phone must be a dialable number (5 to 15 digits)".to_string());
    }
    if alt_phone.is_some_and(|p| !p.trim().is_empty() && !phone_ok(p)) {
        return Err("alt_phone must be a dialable number (5 to 15 digits)".to_string());
    }
    if email.is_some_and(|e| !e.trim().is_empty() && !email_ok(e)) {
        return Err("email is not a valid address".to_string());
    }
    if priority < 1 {
        return Err("priority must be 1 or more (1 is the first person to call)".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phones_need_5_to_15_digits_and_no_letters() {
        assert!(phone_ok("+44 7700 900123"));
        assert!(phone_ok("(020) 7946-0018"));
        assert!(phone_ok("12345"));
        assert!(!phone_ok("1234"));
        assert!(!phone_ok("+44 7700 900123 4567 89"));
        assert!(!phone_ok("call me"));
        assert!(!phone_ok(""));
    }

    #[test]
    fn emails_have_the_basic_shape() {
        assert!(email_ok("sam@example.org"));
        assert!(!email_ok("sam"));
        assert!(!email_ok("@example.org"));
        assert!(!email_ok("sam@example"));
        assert!(!email_ok("sam@@example.org"));
        assert!(!email_ok("sam lee@example.org"));
        assert!(!email_ok("sam@.org"));
    }

    #[test]
    fn a_contact_needs_name_relationship_and_phone() {
        assert!(validate_contact("Sam Lee", "Partner", "+44 7700 900123", None, None, 1).is_ok());
        assert!(validate_contact(" ", "Partner", "+44 7700 900123", None, None, 1).is_err());
        assert!(validate_contact("Sam", "", "+44 7700 900123", None, None, 1).is_err());
        assert!(validate_contact("Sam", "Partner", "n/a", None, None, 1).is_err());
        assert!(validate_contact("Sam", "Partner", "+44 7700 900123", None, None, 0).is_err());
    }

    #[test]
    fn optional_fields_are_checked_only_when_given() {
        let ok = |alt, email| validate_contact("Sam", "Partner", "12345", alt, email, 1).is_ok();
        assert!(ok(None, None));
        assert!(ok(Some(""), Some("")));
        assert!(ok(Some("98765"), Some("sam@example.org")));
        assert!(!ok(Some("x"), None));
        assert!(!ok(None, Some("nope")));
    }
}
