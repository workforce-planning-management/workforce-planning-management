//! **The shared cache** (WPM-D78): Loco's cache provider (in memory by default, Redis in
//! production so several instances share one) used for **non-personal lookups only**.
//!
//! - **What may be cached is a closed list.** [`Namespace`] names it. There is no variant for a person,
//!   a worker, a role, a score or any report, so a cached answer can never be one that a permission
//!   check should have decided. A unit test pins the list, so adding to it is a visible decision.
//! - **Fail open.** A cache that is down, slow or full is a miss, never an error: every call is bounded
//!   by [`TIMEOUT`] and its failure is logged at debug without the key's value.
//! - **Keys are versioned and generational.** `wpm:v1:{namespace}:{generation}:{key}`. Bumping a
//!   namespace's generation ([`invalidate`]) orphans every entry in it at once; they expire by their
//!   time to live.
//! - **Values are text under a size cap**, so one entry cannot fill a shared cache.

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use loco_rs::cache::Cache;

/// The longest any one cache call may take before it counts as a miss.
pub const TIMEOUT: Duration = Duration::from_millis(250);

/// The largest value stored, in bytes.
pub const MAX_VALUE_BYTES: usize = 512 * 1024;

/// What may be cached. **Non-personal data only** (WPM-D78).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    /// The name of an organization, from the service that owns it.
    OrganizationName,
    /// The name of a course, from the service that owns it.
    CourseName,
    /// A search of the ESCO occupation catalogue (public reference data; changes only on import).
    EscoOccupations,
}

impl Namespace {
    /// Every namespace, for the test that pins the list.
    pub const ALL: [Self; 3] = [
        Self::OrganizationName,
        Self::CourseName,
        Self::EscoOccupations,
    ];

    /// The tag used in keys.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::OrganizationName => "organization_name",
            Self::CourseName => "course_name",
            Self::EscoOccupations => "esco_occupations",
        }
    }

    /// How long an entry lives.
    #[must_use]
    pub const fn ttl(self) -> Duration {
        match self {
            Self::OrganizationName | Self::CourseName => Duration::from_mins(15),
            Self::EscoOccupations => Duration::from_hours(1),
        }
    }
}

/// Choose rustls's crypto provider for the process. Two are in the build, so rustls cannot pick one
/// itself and a TLS connection (a `rediss://` cache) would panic. Safe to call more than once.
pub fn install_tls_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

static PROVIDER: OnceLock<Arc<Cache>> = OnceLock::new();

/// Hand over the application's cache provider at boot. Until then, and in a process that never calls
/// this, every lookup is a miss.
pub fn install(cache: Arc<Cache>) {
    let _ = PROVIDER.set(cache);
}

fn provider() -> Option<&'static Arc<Cache>> {
    PROVIDER.get()
}

/// The key for `key` in `namespace` at `generation`.
#[must_use]
pub fn make_key(namespace: Namespace, generation: u64, key: &str) -> String {
    format!("wpm:v1:{}:{generation}:{key}", namespace.tag())
}

fn generation_key(namespace: Namespace) -> String {
    format!("wpm:v1:generation:{}", namespace.tag())
}

async fn bounded<T>(
    what: &'static str,
    future: impl std::future::Future<Output = Result<T, loco_rs::cache::CacheError>>,
) -> Option<T> {
    match tokio::time::timeout(TIMEOUT, future).await {
        Ok(Ok(value)) => Some(value),
        Ok(Err(error)) => {
            tracing::debug!(what, %error, "shared cache call failed; treated as a miss");
            None
        }
        Err(_) => {
            tracing::debug!(what, "shared cache call timed out; treated as a miss");
            None
        }
    }
}

async fn generation(cache: &Cache, namespace: Namespace) -> u64 {
    bounded("generation", cache.get::<u64>(&generation_key(namespace)))
        .await
        .flatten()
        .unwrap_or(0)
}

/// The cached text for `key` in `cache`, or `None` on a miss or a failure.
pub async fn get_in(cache: &Cache, namespace: Namespace, key: &str) -> Option<String> {
    let generation = generation(cache, namespace).await;
    bounded(
        "get",
        cache.get::<String>(&make_key(namespace, generation, key)),
    )
    .await
    .flatten()
}

/// Remember `value` for `key` in `cache`. A value over [`MAX_VALUE_BYTES`] is not stored.
pub async fn put_in(cache: &Cache, namespace: Namespace, key: &str, value: &str) {
    if value.len() > MAX_VALUE_BYTES {
        return;
    }
    let generation = generation(cache, namespace).await;
    let _ = bounded(
        "put",
        cache.insert_with_expiry(
            &make_key(namespace, generation, key),
            value,
            namespace.ttl(),
        ),
    )
    .await;
}

/// Orphan every entry in `namespace` now, by moving its generation on. Entries expire by their time to
/// live. Use after the data behind a namespace changes.
pub async fn invalidate_in(cache: &Cache, namespace: Namespace) {
    let next = generation(cache, namespace).await + 1;
    let _ = bounded(
        "invalidate",
        cache.insert(&generation_key(namespace), &next),
    )
    .await;
}

/// [`get_in`] on the application's provider; a miss when none is installed.
pub async fn get(namespace: Namespace, key: &str) -> Option<String> {
    get_in(provider()?, namespace, key).await
}

/// [`put_in`] on the application's provider; nothing when none is installed.
pub async fn put(namespace: Namespace, key: &str, value: &str) {
    if let Some(cache) = provider() {
        put_in(cache, namespace, key, value).await;
    }
}

/// [`invalidate_in`] on the application's provider; nothing when none is installed.
pub async fn invalidate(namespace: Namespace) {
    if let Some(cache) = provider() {
        invalidate_in(cache, namespace).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cacheable_list_is_closed_and_has_nothing_personal() {
        let tags: Vec<&str> = Namespace::ALL.iter().map(|n| n.tag()).collect();
        assert_eq!(
            tags,
            ["organization_name", "course_name", "esco_occupations"],
            "adding a namespace is a decision: update WPM-D78 and this test"
        );
        for tag in tags {
            for personal in ["person", "worker", "salary", "score", "report", "health"] {
                assert!(!tag.contains(personal), "{tag} must not be {personal}");
            }
        }
    }

    #[test]
    fn keys_are_versioned_namespaced_and_generational() {
        assert_eq!(
            make_key(Namespace::CourseName, 3, "course:abc"),
            "wpm:v1:course_name:3:course:abc"
        );
        assert_ne!(
            make_key(Namespace::CourseName, 1, "k"),
            make_key(Namespace::CourseName, 2, "k"),
            "a new generation orphans the old entries"
        );
        assert_ne!(
            make_key(Namespace::CourseName, 0, "k"),
            make_key(Namespace::OrganizationName, 0, "k"),
            "namespaces do not collide"
        );
    }

    #[test]
    fn lifetimes_are_bounded() {
        for n in Namespace::ALL {
            assert!(n.ttl() >= Duration::from_mins(1) && n.ttl() <= Duration::from_hours(24));
        }
    }

    #[tokio::test]
    async fn without_a_provider_everything_is_a_miss_and_nothing_fails() {
        // No provider is installed in this unit-test process.
        assert_eq!(get(Namespace::CourseName, "k").await, None);
        put(Namespace::CourseName, "k", "v").await;
        invalidate(Namespace::CourseName).await;
        assert_eq!(get(Namespace::CourseName, "k").await, None);
    }
}
