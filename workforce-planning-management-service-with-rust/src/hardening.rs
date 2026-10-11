//! **Production refuses insecure data paths** (WPM-R116): pure checks of how the service reaches
//! its database, run at boot.
//!
//! The service holds personal data. A connection to the database that crosses a network without
//! TLS exposes every query and every row to anyone on the path, so in the `production`
//! environment the service refuses to start on one, unless the operator says in so many words
//! that they accept it (`WPM_ALLOW_PLAINTEXT_DATABASE`). A connection that never leaves the host
//! (a loopback address or a Unix socket) is not on a network and needs no override.
//!
//! Nothing here connects to anything. It reads the connection URL.

/// How the database connection is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// TLS is required, and the server's certificate is also verified (`verify-full`).
    EncryptedAndVerified,
    /// TLS is required (`require` or `verify-ca`): encrypted, but the server's identity is only
    /// partly checked or not at all, so a person on the path could impersonate it.
    Encrypted,
    /// No TLS, but the server is on this host (a loopback address or a Unix socket).
    LocalPlain,
    /// No TLS across a network: `disable`, `allow`, `prefer` (which falls back to plaintext) or
    /// no `sslmode` at all.
    Plain,
}

/// Split a connection URL into its host and its query parameters.
fn parts(url: &str) -> (&str, Vec<(String, String)>) {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let (authority_and_path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let authority = authority_and_path.split('/').next().unwrap_or("");
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        stripped.split(']').next().unwrap_or("")
    } else {
        host_port.split(':').next().unwrap_or("")
    };
    let params = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (k.to_ascii_lowercase(), v.to_string())
        })
        .collect();
    (host, params)
}

/// Whether `host` is this machine: `localhost` or a loopback address. A name that merely starts like
/// one (`127.0.0.1.example.com`) is a remote host.
fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

fn is_local(host: &str, params: &[(String, String)]) -> bool {
    // An empty host, or a `host=/path` parameter, is a Unix socket.
    let socket = params
        .iter()
        .any(|(k, v)| k == "host" && v.starts_with('/'));
    host.is_empty() || socket || is_loopback_host(host)
}

/// How the connection `url` is protected.
#[must_use]
pub fn transport(url: &str) -> Transport {
    let (host, params) = parts(url);
    let mode = params
        .iter()
        .find(|(k, _)| k == "sslmode" || k == "ssl-mode")
        .map(|(_, v)| v.to_ascii_lowercase());
    match mode.as_deref() {
        Some("verify-full") => Transport::EncryptedAndVerified,
        Some("require" | "verify-ca") => Transport::Encrypted,
        _ if is_local(host, &params) => Transport::LocalPlain,
        _ => Transport::Plain,
    }
}

/// The boot decision. `Ok(Some(text))` is a warning to log; `Err(text)` stops the service.
///
/// # Errors
///
/// In production, a plaintext connection across a network without the explicit override.
pub fn database_check(
    production: bool,
    url: &str,
    allow_plaintext: bool,
) -> Result<Option<String>, String> {
    match transport(url) {
        Transport::EncryptedAndVerified | Transport::LocalPlain => Ok(None),
        Transport::Encrypted => Ok(Some(
            "the database connection is encrypted but the server's certificate is not fully verified; \
             use sslmode=verify-full"
                .to_string(),
        )),
        Transport::Plain if !production => Ok(None),
        Transport::Plain if allow_plaintext => Ok(Some(
            "the database connection is NOT encrypted and WPM_ALLOW_PLAINTEXT_DATABASE accepts that; \
             anyone on the network path can read personal data"
                .to_string(),
        )),
        Transport::Plain => Err(
            "refusing to start: the database connection is not encrypted. Add `?sslmode=verify-full` \
             (or `require`) to DATABASE_URL, or set WPM_ALLOW_PLAINTEXT_DATABASE=1 if the network path is \
             protected another way and you accept the risk"
                .to_string(),
        ),
    }
}

/// The host of a `redis://` or `rediss://` URL, without credentials or port.
fn redis_host(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?']).next().unwrap_or("");
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if let Some(inner) = host_port.strip_prefix('[') {
        return inner.split(']').next().unwrap_or("");
    }
    host_port.split(':').next().unwrap_or("")
}

/// The boot decision for the cache's Redis connection (WPM-D78). It holds only non-personal lookups,
/// but a plaintext link to a remote Redis exposes its password and lets anyone on the path put a
/// wrong name or catalogue answer into it. `Ok(Some(text))` is a warning to log; `Err(text)` stops
/// the service.
///
/// # Errors
///
/// In production, a plaintext `redis://` connection to a host that is not this machine, without the
/// explicit override.
pub fn redis_check(
    production: bool,
    url: &str,
    allow_plaintext: bool,
) -> Result<Option<String>, String> {
    if url.starts_with("rediss://") {
        return Ok(None);
    }
    let host = redis_host(url);
    let local = host.is_empty() || is_loopback_host(host);
    if local || !production {
        return Ok(None);
    }
    if allow_plaintext {
        return Ok(Some(
            "the Redis connection is NOT encrypted and WPM_ALLOW_PLAINTEXT_REDIS accepts that"
                .to_string(),
        ));
    }
    Err("refusing to start: the Redis connection is not encrypted. Use a `rediss://` REDIS_URL, or set \
         WPM_ALLOW_PLAINTEXT_REDIS=1 if the network path is protected another way and you accept the risk"
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_connection_is_classified_from_its_url() {
        use Transport::{Encrypted, EncryptedAndVerified, LocalPlain, Plain};
        for (url, expected) in [
            (
                "postgres://u:p@db.internal:5432/wpm?sslmode=verify-full",
                EncryptedAndVerified,
            ),
            (
                "postgres://u:p@db.internal/wpm?sslmode=VERIFY-FULL",
                EncryptedAndVerified,
            ),
            ("postgres://u:p@db.internal/wpm?sslmode=require", Encrypted),
            (
                "postgres://u:p@db.internal/wpm?sslmode=verify-ca",
                Encrypted,
            ),
            (
                "postgres://u:p@db.internal/wpm?x=1&ssl-mode=require",
                Encrypted,
            ),
            ("postgres://u:p@db.internal/wpm", Plain),
            ("postgres://u:p@db.internal/wpm?sslmode=disable", Plain),
            ("postgres://u:p@db.internal/wpm?sslmode=allow", Plain),
            ("postgres://u:p@db.internal/wpm?sslmode=prefer", Plain),
            ("postgres://u:p@db.internal/wpm?sslmode=", Plain),
            ("postgres://u:p@localhost:5432/wpm", LocalPlain),
            ("postgres://u:p@127.0.0.1/wpm", LocalPlain),
            ("postgres://u:p@[::1]:5432/wpm", LocalPlain),
            ("postgres:///wpm?host=/var/run/postgresql", LocalPlain),
            ("postgres://u:p@/wpm?host=/var/run/postgresql", LocalPlain),
            // Encryption is judged by sslmode even for a local host.
            (
                "postgres://u:p@localhost/wpm?sslmode=verify-full",
                EncryptedAndVerified,
            ),
            // A password that looks like a host or an option does not fool the parser.
            ("postgres://u:localhost@db.internal/wpm", Plain),
            (
                "postgres://u:p%40s@db.internal/wpm?sslmode=require",
                Encrypted,
            ),
            ("postgres://u@db.internal.example/wpm?host=/tmp", LocalPlain),
        ] {
            assert_eq!(transport(url), expected, "{url}");
        }
    }

    #[test]
    fn production_refuses_plaintext_across_a_network_unless_told_otherwise() {
        let plain = "postgres://u:p@db.internal/wpm";
        let err = database_check(true, plain, false).unwrap_err();
        assert!(err.contains("not encrypted") && err.contains("sslmode"));
        // The explicit override starts, with a warning that says so.
        let warn = database_check(true, plain, true).unwrap().unwrap();
        assert!(warn.contains("NOT encrypted"));
        // Outside production nothing is said.
        assert_eq!(database_check(false, plain, false), Ok(None));
        // Encrypted and verified starts quietly; encrypted-only starts with advice.
        let verified = "postgres://u:p@db.internal/wpm?sslmode=verify-full";
        assert_eq!(database_check(true, verified, false), Ok(None));
        let partly = "postgres://u:p@db.internal/wpm?sslmode=require";
        assert!(
            database_check(true, partly, false)
                .unwrap()
                .unwrap()
                .contains("verify-full")
        );
        // A local connection needs no override.
        assert_eq!(
            database_check(true, "postgres://u:p@localhost/wpm", false),
            Ok(None)
        );
        // The override never makes a refusal message appear when it is not needed.
        assert_eq!(database_check(true, verified, true), Ok(None));
    }

    #[test]
    fn a_database_host_that_only_looks_local_is_remote() {
        for url in [
            "postgres://u:p@127.0.0.1.evil.example/db",
            "postgres://u:p@localhost.evil.example/db",
            "postgres://u:p@127.evil.example/db",
        ] {
            assert_eq!(transport(url), Transport::Plain, "{url}");
        }
        assert_eq!(
            transport("postgres://u:p@127.0.0.1/db"),
            Transport::LocalPlain
        );
        assert_eq!(transport("postgres://u:p@[::1]/db"), Transport::LocalPlain);
    }

    #[test]
    fn redis_hosts_are_read_from_the_url() {
        for (url, host) in [
            ("redis://cache.internal:6379", "cache.internal"),
            ("redis://:secret@cache.internal:6379/0", "cache.internal"),
            (
                "rediss://user:p%40ss@cache.internal/1?x=y",
                "cache.internal",
            ),
            ("redis://[::1]:6379", "::1"),
            ("redis://localhost", "localhost"),
            ("redis://", ""),
        ] {
            assert_eq!(redis_host(url), host, "{url}");
        }
    }

    #[test]
    fn production_refuses_a_plaintext_remote_redis_unless_told_otherwise() {
        let refused = |url, allow| redis_check(true, url, allow);
        assert!(refused("redis://cache.internal:6379", false).is_err());
        assert!(refused("redis://:secret@10.0.0.5:6379", false).is_err());
        assert!(
            refused("redis://cache.internal:6379", true)
                .unwrap()
                .is_some()
        );
        // Encrypted, or on this machine, is fine.
        assert_eq!(refused("rediss://cache.internal:6380", false), Ok(None));
        assert_eq!(refused("redis://localhost:6379", false), Ok(None));
        assert_eq!(refused("redis://127.0.0.1:6379", false), Ok(None));
        assert_eq!(refused("redis://[::1]:6379", false), Ok(None));
        // A look-alike host is not local.
        assert!(refused("redis://localhost.evil.example:6379", false).is_err());
        assert!(
            refused("redis://127.0.0.1.evil.example:6379", false).is_err(),
            "starts with 127. but is not local"
        );
        assert_eq!(
            refused("redis://127.1.2.3:6379", false),
            Ok(None),
            "all of 127/8 is loopback"
        );
        // Outside production nothing is refused.
        assert_eq!(
            redis_check(false, "redis://cache.internal", false),
            Ok(None)
        );
        // The warning never carries the URL (which may hold a password).
        let warning = refused("redis://:topsecret@cache.internal", true)
            .unwrap()
            .unwrap();
        assert!(!warning.contains("topsecret"));
    }
}
