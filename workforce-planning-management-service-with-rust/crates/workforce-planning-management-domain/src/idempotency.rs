//! **Idempotency keys** (WPM-R132, WPM-D80): pure, database-free and clock-free rules for recognising
//! a repeat of a request that changes something.
//!
//! A caller names one action with an `Idempotency-Key`. The first request under a key runs and its
//! response is kept for a short time; a repeat of the *same* request under the *same* key gets that
//! response back instead of running again; the same key with a *different* request is refused. A key
//! belongs to the caller who made it, so no one can replay another caller's response.

use sha2::{Digest, Sha256};

/// The shortest and longest key accepted, in characters.
pub const KEY_MIN: usize = 16;
/// See [`KEY_MIN`].
pub const KEY_MAX: usize = 64;
/// How long a stored response is kept, in calendar days.
pub const KEEP_CALENDAR_DAYS: i64 = 1;
/// The largest request body that is fingerprinted, in bytes. A larger one is refused with `413`.
pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
/// The largest response that is stored for replay, in bytes. A larger one is returned but not stored.
pub const MAX_STORED_RESPONSE_BYTES: usize = 256 * 1024;
/// How long a request may stay unfinished before a repeat is allowed to take over, in seconds.
pub const IN_PROGRESS_SECONDS: i64 = 300;

/// Whether a method changes something, so a key means anything on it.
#[must_use]
pub fn is_mutating(method: &str) -> bool {
    !matches!(
        method.to_ascii_uppercase().as_str(),
        "GET" | "HEAD" | "OPTIONS"
    )
}

/// Check a key's shape: letters, digits, hyphen and underscore, 16 to 64 characters.
///
/// # Errors
///
/// A message saying what to change.
pub fn validate_key(key: &str) -> Result<(), String> {
    let n = key.chars().count();
    if !(KEY_MIN..=KEY_MAX).contains(&n) {
        return Err(format!(
            "Idempotency-Key must be {KEY_MIN} to {KEY_MAX} characters"
        ));
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Idempotency-Key may hold only letters, digits, hyphens and underscores".into(),
        );
    }
    Ok(())
}

/// A fingerprint of a request: its method, its path and query, and its body. Two requests with the
/// same fingerprint are the same action. Each part is length-prefixed so that moving a byte from
/// one part to the next changes the fingerprint.
#[must_use]
pub fn fingerprint(method: &str, path_and_query: &str, body: &[u8]) -> String {
    let mut hash = Sha256::new();
    for part in [
        method.to_ascii_uppercase().as_bytes(),
        path_and_query.as_bytes(),
        body,
    ] {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize().iter().fold(String::new(), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Whether a response is kept for replay: a success, and not too large. A failure is not kept, so the
/// caller can fix the request and try again under the same key.
#[must_use]
pub fn is_storable(status: u16, size: Option<usize>) -> bool {
    (200..300).contains(&status) && size.is_some_and(|n| n <= MAX_STORED_RESPONSE_BYTES)
}

/// A record already held under the caller's key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Existing {
    /// The fingerprint of the request that claimed the key.
    pub fingerprint: String,
    /// Whether a response has been stored.
    pub finished: bool,
    /// Seconds since the key was claimed.
    pub age_seconds: i64,
}

/// What to do with a request that carries a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Nothing is held: claim the key and run the request.
    Run,
    /// The same request already finished: return the stored response.
    Replay,
    /// The key was used for a different request: refuse (`422`).
    Clash,
    /// The same request is still running: ask the caller to wait (`409`).
    InProgress,
    /// The same request started long ago and never finished: drop the claim and run it.
    TakeOver,
}

/// Decide, given what is held under the caller's key and the fingerprint of this request.
#[must_use]
pub fn decide(existing: Option<&Existing>, fingerprint: &str) -> Decision {
    let Some(held) = existing else {
        return Decision::Run;
    };
    if held.fingerprint != fingerprint {
        return Decision::Clash;
    }
    if held.finished {
        Decision::Replay
    } else if held.age_seconds >= IN_PROGRESS_SECONDS {
        Decision::TakeOver
    } else {
        Decision::InProgress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(finished: bool, age_seconds: i64, fp: &str) -> Existing {
        Existing {
            fingerprint: fp.into(),
            finished,
            age_seconds,
        }
    }

    #[test]
    fn only_a_change_carries_a_key() {
        for m in ["POST", "PUT", "PATCH", "DELETE", "post"] {
            assert!(is_mutating(m), "{m}");
        }
        for m in ["GET", "HEAD", "OPTIONS", "get"] {
            assert!(!is_mutating(m), "{m}");
        }
    }

    #[test]
    fn a_key_is_a_plain_token_of_sixteen_to_sixty_four_characters() {
        assert!(validate_key(&"a".repeat(KEY_MIN)).is_ok());
        assert!(validate_key(&"a".repeat(KEY_MAX)).is_ok());
        assert!(validate_key("3f2b8c1e-aaaa-4bbb-8ccc-0123456789ab").is_ok());
        assert!(validate_key("under_score-and-DIGITS-0123").is_ok());
        for bad in [
            "",
            "short",
            &"a".repeat(KEY_MIN - 1),
            &"a".repeat(KEY_MAX + 1),
            "has space in it 12345",
            "semi;colon-1234567890",
            "new\nline-1234567890ab",
            "ключ-1234567890abcdef",
        ] {
            assert!(validate_key(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_fingerprint_is_stable_and_sees_every_part() {
        let base = fingerprint("POST", "/api/workers", b"{\"a\":1}");
        assert_eq!(base, fingerprint("POST", "/api/workers", b"{\"a\":1}"));
        assert_eq!(base, fingerprint("post", "/api/workers", b"{\"a\":1}"));
        assert_eq!(base.len(), 64);
        assert_ne!(base, fingerprint("PUT", "/api/workers", b"{\"a\":1}"));
        assert_ne!(base, fingerprint("POST", "/api/workers/1", b"{\"a\":1}"));
        assert_ne!(base, fingerprint("POST", "/api/workers?x=1", b"{\"a\":1}"));
        assert_ne!(base, fingerprint("POST", "/api/workers", b"{\"a\":2}"));
        assert_ne!(base, fingerprint("POST", "/api/workers", b""));
    }

    #[test]
    fn moving_a_byte_between_parts_changes_the_fingerprint() {
        assert_ne!(
            fingerprint("POST", "/a", b"bc"),
            fingerprint("POST", "/ab", b"c")
        );
    }

    #[test]
    fn only_a_small_success_is_stored() {
        assert!(is_storable(200, Some(10)));
        assert!(is_storable(201, Some(MAX_STORED_RESPONSE_BYTES)));
        assert!(is_storable(204, Some(0)));
        assert!(!is_storable(201, Some(MAX_STORED_RESPONSE_BYTES + 1)));
        assert!(!is_storable(201, None), "an unknown size is not stored");
        for status in [199, 301, 400, 404, 409, 422, 500] {
            assert!(!is_storable(status, Some(10)), "{status}");
        }
    }

    #[test]
    fn nothing_held_means_run() {
        assert_eq!(decide(None, "x"), Decision::Run);
    }

    #[test]
    fn the_same_request_that_finished_is_replayed() {
        assert_eq!(decide(Some(&held(true, 5, "x")), "x"), Decision::Replay);
    }

    #[test]
    fn a_different_request_under_the_same_key_is_a_clash_even_before_it_finishes() {
        assert_eq!(decide(Some(&held(true, 5, "x")), "y"), Decision::Clash);
        assert_eq!(decide(Some(&held(false, 5, "x")), "y"), Decision::Clash);
        assert_eq!(decide(Some(&held(false, 9_999, "x")), "y"), Decision::Clash);
    }

    #[test]
    fn a_running_request_makes_a_repeat_wait_until_it_is_abandoned() {
        assert_eq!(
            decide(Some(&held(false, 0, "x")), "x"),
            Decision::InProgress
        );
        assert_eq!(
            decide(Some(&held(false, IN_PROGRESS_SECONDS - 1, "x")), "x"),
            Decision::InProgress
        );
        assert_eq!(
            decide(Some(&held(false, IN_PROGRESS_SECONDS, "x")), "x"),
            Decision::TakeOver
        );
    }

    #[test]
    fn a_finished_request_is_replayed_however_old_the_claim() {
        assert_eq!(
            decide(Some(&held(true, 99_999, "x")), "x"),
            Decision::Replay
        );
    }
}
