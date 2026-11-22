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

/// Choose rustls's crypto provider for the process: `ring`, unless one was chosen already. reqwest is built with
/// `rustls-no-provider`, so it takes no provider of its own and a TLS connection would panic without one. Every place that builds a reqwest client, and `App::context`, calls it
/// first, so no caller depends on what ran before it. Safe to call more than once.
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
            // `bytes` is the sum of the held entries' sizes, so this cannot go below zero; were the books ever wrong, the
            // true figure is counted again from the entries rather than guessed.
            self.bytes = self
                .bytes
                .checked_sub(entry_size(key, &value))
                .unwrap_or_else(|| held_bytes(&self.entries));
        }
    }

    /// Hold `value`. Over `capacity`, expired entries go first and then everything, which only makes later lookups misses.
    fn put(&mut self, key: &str, value: String, ttl: Option<Duration>, capacity: usize) {
        self.remove(key);
        // An expiry that cannot be represented is a value that is not held: a miss, never one that lives forever.
        let expires = match ttl.map(|ttl| Instant::now().checked_add(ttl)) {
            None => None,
            Some(Some(at)) => Some(at),
            Some(None) => return,
        };
        let size = entry_size(key, &value);
        if !self.fits(size, capacity) {
            let now = Instant::now();
            self.entries
                .retain(|_, (_, expires)| expires.is_none_or(|at| at > now));
            self.bytes = held_bytes(&self.entries);
        }
        if !self.fits(size, capacity) {
            self.entries.clear();
            self.bytes = 0;
        }
        if size <= capacity {
            // `fits` holds here (the cache is empty or was checked), so the sum is within `capacity`.
            self.bytes = self.bytes.checked_add(size).unwrap_or(OVER_CAPACITY);
            self.entries.insert(key.to_string(), (value, expires));
        }
    }

    /// Whether `size` more bytes stay within `capacity`; a sum that does not fit `usize` is over any capacity.
    fn fits(&self, size: usize, capacity: usize) -> bool {
        self.bytes
            .checked_add(size)
            .is_some_and(|total| total <= capacity)
    }
}

/// A size no capacity can hold: what a byte count too large for `usize` is, so such an entry is never held (declared:
/// spec/figure-declarations.md, `cache_entry_bytes`).
const OVER_CAPACITY: usize = usize::MAX;

/// The bytes one entry takes: its key and its value.
fn entry_size(key: &str, value: &str) -> usize {
    key.len().checked_add(value.len()).unwrap_or(OVER_CAPACITY)
}

/// The bytes of all the held entries, counted from the entries themselves.
fn held_bytes(entries: &HashMap<String, (String, Option<Instant>)>) -> usize {
    entries
        .iter()
        .try_fold(0_usize, |total, (k, (v, _))| {
            total.checked_add(entry_size(k, v))
        })
        .unwrap_or(OVER_CAPACITY)
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
                // A lifetime past `u64` milliseconds (some 584 million years) is no expiry: the entry is set without one.
                if let Some(millis) = ttl.and_then(|ttl| u64::try_from(ttl.as_millis()).ok()) {
                    command.arg("PX").arg(millis);
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

/// The generation after `current`, or `None` when the counter is spent: it is never wrapped, which would revive old entries.
fn next_generation(current: u64) -> Option<u64> {
    current.checked_add(1)
}

/// Orphan every entry in `namespace` now, by moving its generation on. Entries expire by their time to
/// live. Use after the data behind a namespace changes.
pub async fn invalidate_in(cache: &Cache, namespace: Namespace) {
    let Some(next) = next_generation(generation(cache, namespace).await) else {
        // The counter is spent, so there is no newer generation to move to; entries still expire by their time to live.
        return;
    };
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
    fn the_byte_count_is_recounted_when_the_books_are_wrong() {
        let mut memory = Memory::default();
        memory.put("k", "vv".to_string(), None, 100);
        memory.put("j", "w".to_string(), None, 100);
        assert_eq!(memory.bytes, 5);
        memory.bytes = 1; // less than the entry's 3 bytes: the books are wrong
        memory.remove("k");
        assert_eq!(memory.bytes, 2, "recounted from the entries left");
    }

    #[test]
    fn a_size_past_usize_is_over_any_capacity() {
        let memory = Memory {
            bytes: usize::MAX,
            ..Memory::default()
        };
        assert!(!memory.fits(1, usize::MAX));
        assert!(Memory::default().fits(3, 3));
        assert!(!Memory::default().fits(4, 3));
    }

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
    fn a_spent_generation_counter_does_not_wrap() {
        assert_eq!(next_generation(3), Some(4));
        assert_eq!(next_generation(u64::MAX), None);
    }

    #[test]
    fn a_ttl_beyond_the_clock_is_not_held() {
        let mut memory = Memory::default();
        memory.put("k", "v".to_string(), Some(Duration::MAX), 64);
        assert!(memory.entries.is_empty() && memory.bytes == 0);
        memory.put("k", "v".to_string(), Some(Duration::from_mins(1)), 64);
        assert_eq!(memory.bytes, 2);
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
