//! Single-use, hashed link tokens (a link in an email or a message that lets one person do one thing once). DB-free and clock-free.
//!
//! A token is 32 random bytes shown **once**, in the link. What is stored is only its SHA-256 hash, so a copy of the database holds
//! nothing that can be used as a link. A token has a purpose, so a link made to confirm one thing cannot be used for another; it
//! expires; and it works exactly once. A caller who presents a token that fails is told nothing about why (unknown, expired, used and
//! wrong purpose look the same); the reason goes to the audit trail, never the response.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

/// The random bytes in a token (256 bits).
pub const TOKEN_BYTES: usize = 32;
/// The characters of a token: 32 bytes in URL-safe base64 without padding.
pub const TOKEN_CHARS: usize = 43;
/// The shortest life a token may be given, in minutes.
pub const TTL_MIN_MINUTES: i64 = 1;
/// The longest life a token may be given, in minutes (seven days).
pub const TTL_MAX_MINUTES: i64 = 7 * 24 * 60;

/// The token text for `bytes`. The caller supplies random bytes from the operating system's generator; this function is pure.
#[must_use]
pub fn encode(bytes: [u8; TOKEN_BYTES]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Whether a presented string could be a token: exactly [`TOKEN_CHARS`] URL-safe base64 characters. Anything else is refused before
/// it is hashed or looked up.
#[must_use]
pub fn well_formed(token: &str) -> bool {
    token.len() == TOKEN_CHARS
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        && URL_SAFE_NO_PAD
            .decode(token)
            .is_ok_and(|v| v.len() == TOKEN_BYTES)
}

/// The SHA-256 of a token, as 64 lower-case hex characters. This is what is stored and what is looked up.
#[must_use]
pub fn hash(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// Problems with a purpose: 1 to 60 lower-case letters, digits and underscores.
#[must_use]
pub fn purpose_problems(purpose: &str) -> Vec<String> {
    let ok = (1..=60).contains(&purpose.len())
        && purpose
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if ok {
        Vec::new()
    } else {
        vec!["purpose must be 1 to 60 lower-case letters, digits or underscores".to_string()]
    }
}

/// The moment a token issued at `now` stops working, for a life of `ttl_minutes`.
///
/// # Errors
/// A sentence when the life is outside [`TTL_MIN_MINUTES`] to [`TTL_MAX_MINUTES`].
pub fn expires_at(now: DateTime<Utc>, ttl_minutes: i64) -> Result<DateTime<Utc>, String> {
    if !(TTL_MIN_MINUTES..=TTL_MAX_MINUTES).contains(&ttl_minutes) {
        return Err(format!(
            "a link lives {TTL_MIN_MINUTES} to {TTL_MAX_MINUTES} minutes"
        ));
    }
    Ok(now + Duration::minutes(ttl_minutes))
}

/// What is stored about a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    /// What the link was made for.
    pub purpose: String,
    /// When it stops working.
    pub expires_at: DateTime<Utc>,
    /// When it was used, if it has been.
    pub used_at: Option<DateTime<Utc>>,
}

/// Why a token was refused. **For the audit trail only; never sent to the caller.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// No such token.
    Unknown,
    /// Made for something else.
    WrongPurpose,
    /// Already used.
    Used,
    /// Past its time.
    Expired,
}

impl Refusal {
    /// The word the audit trail records.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Refusal::Unknown => "unknown",
            Refusal::WrongPurpose => "wrong_purpose",
            Refusal::Used => "used",
            Refusal::Expired => "expired",
        }
    }
}

/// Whether `stored` (or no row at all) may be used for `purpose` at `now`. A token is good until `expires_at`, exclusive.
///
/// # Errors
/// The reason, for the audit trail.
pub fn check(stored: Option<&Stored>, purpose: &str, now: DateTime<Utc>) -> Result<(), Refusal> {
    let Some(s) = stored else {
        return Err(Refusal::Unknown);
    };
    if s.purpose != purpose {
        Err(Refusal::WrongPurpose)
    } else if s.used_at.is_some() {
        Err(Refusal::Used)
    } else if now >= s.expires_at {
        Err(Refusal::Expired)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 28, h, m, 0).unwrap()
    }

    fn stored(used: bool) -> Stored {
        Stored {
            purpose: "confirm".to_string(),
            expires_at: at(12, 0),
            used_at: used.then(|| at(11, 0)),
        }
    }

    #[test]
    fn a_token_is_forty_three_url_safe_characters_for_thirty_two_bytes() {
        let t = encode([0xFF; TOKEN_BYTES]);
        assert_eq!(t.len(), TOKEN_CHARS);
        assert!(
            t.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        );
        assert!(well_formed(&t));
        assert_ne!(encode([1; TOKEN_BYTES]), encode([2; TOKEN_BYTES]));
    }

    #[test]
    fn only_a_well_formed_string_is_looked_up() {
        let good = encode([7; TOKEN_BYTES]);
        assert!(well_formed(&good));
        for bad in [
            "",
            "abc",
            &good[..42],
            &format!("{good}A"),
            &good.replace(&good[..1], "+"),
            &format!("{} ", &good[..42]),
            "é".repeat(22).as_str(),
        ] {
            assert!(!well_formed(bad), "{bad:?}");
        }
        // Standard-alphabet characters and padding are not accepted.
        assert!(!well_formed(&format!("{}=", &good[..42])));
        assert!(!well_formed(&format!("{}+", &good[..42])));
    }

    #[test]
    fn the_hash_is_sha256_in_hex_and_is_not_the_token() {
        // SHA-256 of the empty string and of "abc" are standard test vectors.
        assert_eq!(
            hash(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let t = encode([9; TOKEN_BYTES]);
        let h = hash(&t);
        assert_eq!(h.len(), 64);
        assert!(
            h.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
        assert!(!h.contains(&t) && !t.contains(&h));
        assert_ne!(
            hash(&encode([9; TOKEN_BYTES])),
            hash(&encode([8; TOKEN_BYTES]))
        );
    }

    #[test]
    fn a_life_is_bounded() {
        assert_eq!(expires_at(at(10, 0), 30), Ok(at(10, 30)));
        assert!(expires_at(at(10, 0), 0).is_err());
        assert!(expires_at(at(10, 0), -5).is_err());
        assert!(expires_at(at(10, 0), TTL_MAX_MINUTES + 1).is_err());
        assert!(expires_at(at(10, 0), TTL_MAX_MINUTES).is_ok());
        assert!(expires_at(at(10, 0), TTL_MIN_MINUTES).is_ok());
    }

    #[test]
    fn a_token_works_once_for_its_purpose_before_it_expires() {
        let s = stored(false);
        assert_eq!(check(Some(&s), "confirm", at(11, 59)), Ok(()));
        assert_eq!(
            check(Some(&s), "confirm", at(12, 0)),
            Err(Refusal::Expired),
            "expiry is exclusive"
        );
        assert_eq!(check(Some(&s), "confirm", at(13, 0)), Err(Refusal::Expired));
        assert_eq!(
            check(Some(&s), "reset", at(11, 0)),
            Err(Refusal::WrongPurpose)
        );
        assert_eq!(
            check(Some(&stored(true)), "confirm", at(11, 30)),
            Err(Refusal::Used)
        );
        assert_eq!(check(None, "confirm", at(11, 0)), Err(Refusal::Unknown));
    }

    #[test]
    fn a_used_token_that_has_also_expired_is_reported_as_used_first_only_when_purpose_matches() {
        let mut s = stored(true);
        s.expires_at = at(10, 0);
        assert_eq!(check(Some(&s), "confirm", at(11, 30)), Err(Refusal::Used));
        assert_eq!(
            check(Some(&s), "other", at(11, 30)),
            Err(Refusal::WrongPurpose),
            "the wrong purpose is reported before anything else"
        );
    }

    #[test]
    fn a_purpose_is_a_short_snake_case_word() {
        for good in ["confirm", "pay_information_response", "a1"] {
            assert_eq!(purpose_problems(good), Vec::<String>::new(), "{good}");
        }
        for bad in ["", "Confirm", "has space", "dash-ed", &"x".repeat(61)] {
            assert_eq!(purpose_problems(bad).len(), 1, "{bad:?}");
        }
    }

    #[test]
    fn refusal_words_are_distinct() {
        let all = [
            Refusal::Unknown,
            Refusal::WrongPurpose,
            Refusal::Used,
            Refusal::Expired,
        ];
        let words: std::collections::BTreeSet<_> = all.iter().map(|r| r.token()).collect();
        assert_eq!(words.len(), 4);
    }
}
