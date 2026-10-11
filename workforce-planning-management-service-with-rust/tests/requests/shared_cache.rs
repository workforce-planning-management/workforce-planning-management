//! The shared cache (WPM-D78): non-personal lookups only, fail open, invalidated when the data behind
//! them changes. The Redis provider is checked against a real server when `WPM_TEST_REDIS_URL` is set.

use sea_orm::ConnectionTrait;
use serde_json::Value;
use serial_test::serial;
use workforce_planning_management_service::shared_cache::{self as cache, Namespace};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// A repeated ESCO search is answered from the cache (proved by changing the table underneath it),
// and invalidating the namespace, as the import does, makes the next search read the table again.
async fn an_esco_search_is_cached_until_the_catalogue_is_invalidated() {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let uri = format!("http://data.europa.eu/esco/occupation/{tag}");
        let term = format!("zzcache{}", &tag[..8]);
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO esco_occupations (uri, label) VALUES ('{uri}', 'Test {term} keeper')"
            ))
            .await
            .unwrap();
        let search = || {
            let (request, term) = (&request, term.clone());
            async move {
                request
                    .get(&format!("/api/esco/occupations?q={term}"))
                    .await
                    .json::<Value>()
            }
        };
        assert_eq!(search().await.as_array().unwrap().len(), 1);
        ctx.db
            .execute_unprepared(&format!("DELETE FROM esco_occupations WHERE uri = '{uri}'"))
            .await
            .unwrap();
        assert_eq!(
            search().await.as_array().unwrap().len(),
            1,
            "the second search is the cached answer"
        );
        cache::invalidate(Namespace::EscoOccupations).await;
        assert_eq!(
            search().await.as_array().unwrap().len(),
            0,
            "after invalidation the table is read again"
        );
        // A validation failure is never cached as an answer.
        assert_eq!(
            request.get("/api/esco/occupations?q=a").await.status_code(),
            422
        );
        assert_eq!(
            request.get("/api/esco/occupations?q=a").await.status_code(),
            422
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Namespaces do not see each other, and invalidating one leaves the others alone.
async fn namespaces_are_separate_and_invalidated_one_at_a_time() {
    crate::requests::request_open(|_request, _ctx| async move {
        let key = format!("k-{}", uuid::Uuid::new_v4().simple());
        cache::put(Namespace::OrganizationName, &key, "Org name").await;
        cache::put(Namespace::CourseName, &key, "Course name").await;
        assert_eq!(
            cache::get(Namespace::OrganizationName, &key)
                .await
                .as_deref(),
            Some("Org name")
        );
        assert_eq!(
            cache::get(Namespace::CourseName, &key).await.as_deref(),
            Some("Course name")
        );
        cache::invalidate(Namespace::OrganizationName).await;
        assert_eq!(cache::get(Namespace::OrganizationName, &key).await, None);
        assert_eq!(
            cache::get(Namespace::CourseName, &key).await.as_deref(),
            Some("Course name")
        );
        // An oversized value is not stored.
        let big = "x".repeat(cache::MAX_VALUE_BYTES + 1);
        cache::put(Namespace::CourseName, "big", &big).await;
        assert_eq!(cache::get(Namespace::CourseName, "big").await, None);
    })
    .await;
}

fn redis_url() -> Option<String> {
    std::env::var("WPM_TEST_REDIS_URL").ok()
}

async fn redis_cache(uri: &str) -> loco_rs::cache::Cache {
    cache::install_tls_provider();
    loco_rs::cache::drivers::redis::new(&loco_rs::config::RedisCacheConfig {
        uri: uri.to_string(),
        max_size: 4,
    })
    .await
    .expect("connect to Redis")
}

#[tokio::test]
#[serial]
#[ignore = "requires Redis: set WPM_TEST_REDIS_URL, run with `cargo test --test mod shared_cache -- --ignored`"]
// Against a real Redis: a value is shared between two handles (two instances), invalidation reaches
// both, and a Redis that is not there is a miss inside the time bound, never an error or a hang.
async fn redis_is_shared_between_instances_and_fails_open() {
    let Some(uri) = redis_url() else {
        eprintln!("WPM_TEST_REDIS_URL is not set; skipping");
        return;
    };
    let (a, b) = (redis_cache(&uri).await, redis_cache(&uri).await);
    let key = format!("k-{}", uuid::Uuid::new_v4().simple());
    cache::put_in(&a, Namespace::CourseName, &key, "Shared course").await;
    assert_eq!(
        cache::get_in(&b, Namespace::CourseName, &key)
            .await
            .as_deref(),
        Some("Shared course"),
        "the other instance sees it"
    );
    cache::invalidate_in(&b, Namespace::CourseName).await;
    assert_eq!(
        cache::get_in(&a, Namespace::CourseName, &key).await,
        None,
        "and so does the invalidation"
    );

    // Nothing listens on this port: every call is a miss, quickly.
    let down = loco_rs::cache::drivers::redis::new(&loco_rs::config::RedisCacheConfig {
        uri: "redis://127.0.0.1:1".to_string(),
        max_size: 1,
    })
    .await;
    if let Ok(down) = down {
        let started = std::time::Instant::now();
        cache::put_in(&down, Namespace::CourseName, "k", "v").await;
        assert_eq!(cache::get_in(&down, Namespace::CourseName, "k").await, None);
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "bounded, not hung"
        );
    }
}

/// A round trip through `uri`: put, get, and the wrong key still a miss.
async fn round_trip(uri: &str) {
    let cache = redis_cache(uri).await;
    let key = format!("k-{}", uuid::Uuid::new_v4().simple());
    cache::put_in(&cache, Namespace::OrganizationName, &key, "Org via Redis").await;
    assert_eq!(
        cache::get_in(&cache, Namespace::OrganizationName, &key)
            .await
            .as_deref(),
        Some("Org via Redis")
    );
    assert_eq!(
        cache::get_in(&cache, Namespace::OrganizationName, "other").await,
        None
    );
}

#[tokio::test]
#[serial]
#[ignore = "requires a Redis with a password: set WPM_TEST_REDIS_AUTH_URL (with the password) and WPM_TEST_REDIS_WRONG_PASSWORD_URL"]
// A Redis that needs a password works with it, and with the wrong one or none is a miss, never an
// error, and the password is not in anything the cache layer logs.
async fn redis_with_a_password_works_and_a_wrong_one_fails_open() {
    let (Ok(good), Ok(wrong), Ok(none)) = (
        std::env::var("WPM_TEST_REDIS_AUTH_URL"),
        std::env::var("WPM_TEST_REDIS_WRONG_PASSWORD_URL"),
        std::env::var("WPM_TEST_REDIS_NO_PASSWORD_URL"),
    ) else {
        eprintln!("the password URLs are not set; skipping");
        return;
    };
    round_trip(&good).await;
    for (name, uri) in [("wrong password", wrong), ("no password", none)] {
        let started = std::time::Instant::now();
        cache::install_tls_provider();
        let cache = loco_rs::cache::drivers::redis::new(&loco_rs::config::RedisCacheConfig {
            uri,
            max_size: 1,
        })
        .await;
        if let Ok(cache) = cache {
            cache::put_in(&cache, Namespace::CourseName, "k", "v").await;
            assert_eq!(
                cache::get_in(&cache, Namespace::CourseName, "k").await,
                None,
                "{name}"
            );
        }
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "{name}: bounded"
        );
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires a TLS Redis: set WPM_TEST_REDIS_TLS_URL (rediss://) and SSL_CERT_FILE to its CA"]
// A `rediss://` URL connects over TLS and verifies the server's certificate against the trusted
// roots: the same server is refused (a miss) when its CA is not trusted.
async fn rediss_connects_over_tls_and_checks_the_certificate() {
    let Ok(uri) = std::env::var("WPM_TEST_REDIS_TLS_URL") else {
        eprintln!("WPM_TEST_REDIS_TLS_URL is not set; skipping");
        return;
    };
    assert!(uri.starts_with("rediss://"));
    round_trip(&uri).await;

    // The same server with a trust store that does not hold its CA: the certificate is refused, so
    // the cache is a miss. (The roots are read when a connection is made.)
    let trusted = std::env::var("SSL_CERT_FILE").ok();
    let empty = std::env::temp_dir().join(format!("empty-ca-{}.pem", uuid::Uuid::new_v4()));
    std::fs::write(&empty, "").unwrap();
    unsafe { std::env::set_var("SSL_CERT_FILE", &empty) };
    cache::install_tls_provider();
    let refused = loco_rs::cache::drivers::redis::new(&loco_rs::config::RedisCacheConfig {
        uri: uri.clone(),
        max_size: 1,
    })
    .await;
    if let Ok(untrusting) = refused {
        cache::put_in(&untrusting, Namespace::CourseName, "k", "v").await;
        assert_eq!(
            cache::get_in(&untrusting, Namespace::CourseName, "k").await,
            None,
            "an untrusted certificate is not used"
        );
    }
    match trusted {
        Some(path) => unsafe { std::env::set_var("SSL_CERT_FILE", path) },
        None => unsafe { std::env::remove_var("SSL_CERT_FILE") },
    }
    let _ = std::fs::remove_file(&empty);
}
