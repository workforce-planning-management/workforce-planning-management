//! **The shared cache** (WPM-D78): in memory by default, Redis in production so several instances share
//! one, used for **non-personal lookups only**.
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use redis::aio::MultiplexedConnection;
use serde::{Serialize, de::DeserializeOwned};

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

/// The most an in-memory cache holds, in bytes of keys and values.
pub const MEMORY_CAPACITY: usize = 32 * 1024 * 1024;

/// What an in-memory cache holds.
#[derive(Debug, Default)]
pub struct Memory {
    entries: HashMap<String, (String, Option<Instant>)>,
    bytes: usize,
}

impl Memory {
    fn get(&mut self, key: &str) -> Option<String> {
        let (value, expires) = self.entries.get(key)?;
        if expires.is_some_and(|at| at <= Instant::now()) {
            self.remove(key);
            return None;
        }
        Some(value.clone())
    }

    fn remove(&mut self, key: &str) {
        if let Some((value, _)) = self.entries.remove(key) {
            self.bytes -= key.len() + value.len();
        }
    }

    /// Hold `value`. Over `capacity`, expired entries go first and then everything, which only makes later lookups misses.
    fn put(&mut self, key: &str, value: String, ttl: Option<Duration>, capacity: usize) {
        self.remove(key);
        let size = key.len() + value.len();
        if self.bytes + size > capacity {
            let now = Instant::now();
            self.entries
                .retain(|_, (_, expires)| expires.is_none_or(|at| at > now));
            self.bytes = self
                .entries
                .iter()
                .map(|(k, (v, _))| k.len() + v.len())
                .sum();
        }
        if self.bytes + size > capacity {
            self.entries.clear();
            self.bytes = 0;
        }
        if size <= capacity {
            self.bytes += size;
            self.entries.insert(
                key.to_string(),
                (value, ttl.map(|ttl| Instant::now() + ttl)),
            );
        }
    }
}

/// A cache: this process's memory, or a Redis several instances share.
#[derive(Debug)]
pub enum Cache {
    /// One per instance: correct, only less shared.
    Memory {
        /// What it holds.
        store: Mutex<Memory>,
        /// How much it may hold.
        capacity: usize,
    },
    /// Shared by every instance.
    Redis {
        /// How to reach it.
        client: Box<redis::Client>,
        /// The connection in use, made on first need and dropped when it fails, so a Redis that comes back is used again.
        connection: tokio::sync::Mutex<Option<MultiplexedConnection>>,
    },
}

impl Cache {
    /// A cache in this process's memory holding at most `capacity` bytes.
    #[must_use]
    pub fn memory(capacity: usize) -> Self {
        Self::Memory {
            store: Mutex::new(Memory::default()),
            capacity,
        }
    }

    /// A cache in the Redis at `uri`. Nothing is connected yet, so a Redis that is down is a miss and not a failure to start.
    ///
    /// # Errors
    /// The address cannot be read.
    pub fn redis(uri: &str) -> Result<Self, String> {
        Ok(Self::Redis {
            client: Box::new(redis::Client::open(uri).map_err(|cause| cause.to_string())?),
            connection: tokio::sync::Mutex::new(None),
        })
    }

    /// The cache `REDIS_URL` asks for, else one in memory.
    ///
    /// # Errors
    /// `REDIS_URL` cannot be read.
    pub fn from_env() -> Result<Self, String> {
        match crate::compat::env_var("REDIS_URL").filter(|uri| !uri.is_empty()) {
            Some(uri) => Self::redis(&uri),
            None => Ok(Self::memory(MEMORY_CAPACITY)),
        }
    }

    async fn connection(
        client: &redis::Client,
        slot: &tokio::sync::Mutex<Option<MultiplexedConnection>>,
    ) -> Result<MultiplexedConnection, String> {
        let mut slot = slot.lock().await;
        if let Some(connection) = slot.as_ref() {
            return Ok(connection.clone());
        }
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|cause| cause.to_string())?;
        *slot = Some(connection.clone());
        Ok(connection)
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        let text = match self {
            Self::Memory { store, .. } => store.lock().map_err(|cause| cause.to_string())?.get(key),
            Self::Redis { client, connection } => {
                let mut live = Self::connection(client, connection).await?;
                let answer: Result<Option<String>, _> =
                    redis::cmd("GET").arg(key).query_async(&mut live).await;
                match answer {
                    Ok(text) => text,
                    Err(cause) => {
                        *connection.lock().await = None;
                        return Err(cause.to_string());
                    }
                }
            }
        };
        text.map(|text| serde_json::from_str(&text).map_err(|cause| cause.to_string()))
            .transpose()
    }

    async fn put<T: Serialize + Sync + ?Sized>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), String> {
        let text = serde_json::to_string(value).map_err(|cause| cause.to_string())?;
        match self {
            Self::Memory { store, capacity } => {
                store
                    .lock()
                    .map_err(|cause| cause.to_string())?
                    .put(key, text, ttl, *capacity);
                Ok(())
            }
            Self::Redis { client, connection } => {
                let mut live = Self::connection(client, connection).await?;
                let mut command = redis::cmd("SET");
                command.arg(key).arg(text);
                if let Some(ttl) = ttl {
                    command
                        .arg("PX")
                        .arg(u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX));
                }
                let answer: Result<(), _> = command.query_async(&mut live).await;
                if answer.is_err() {
                    *connection.lock().await = None;
                }
                answer.map_err(|cause| cause.to_string())
            }
        }
    }
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
    future: impl std::future::Future<Output = Result<T, String>>,
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
        cache.put(
            &make_key(namespace, generation, key),
            value,
            Some(namespace.ttl()),
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
        cache.put(&generation_key(namespace), &next, None),
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
    async fn memory_answers_expires_and_stays_under_its_capacity() {
        let cache = Cache::memory(64);
        cache.put("a", "one", None).await.unwrap();
        assert_eq!(
            cache.get::<String>("a").await.unwrap().as_deref(),
            Some("one")
        );
        cache.put("b", "two", Some(Duration::ZERO)).await.unwrap();
        assert_eq!(cache.get::<String>("b").await.unwrap(), None, "expired");
        cache.put("big", &"x".repeat(100), None).await.unwrap();
        assert_eq!(
            cache.get::<String>("big").await.unwrap(),
            None,
            "over capacity"
        );
        cache.put("c", &"y".repeat(40), None).await.unwrap();
        cache.put("d", &"z".repeat(40), None).await.unwrap();
        let Cache::Memory { store, .. } = &cache else {
            panic!("memory")
        };
        assert!(store.lock().unwrap().bytes <= 64);
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
