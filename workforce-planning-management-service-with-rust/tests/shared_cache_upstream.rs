//! Name lookups through the shared cache, against a stub sibling service over real HTTP (WPM-D78).
//!
//! Its own test binary: the upstream mode is read once per process. It needs no database.
//! Organization and course names may enter the cache every instance shares; a person's name must
//! not, and a sibling that is down is a plain miss.

use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Json, Router, extract::Path, routing::get};
use entity_ref::EntityRef;
use serde_json::json;
use workforce_planning_management_service::clients;
use workforce_planning_management_service::shared_cache::{self as cache, Namespace};

#[tokio::test]
async fn organization_names_are_shared_and_person_names_are_not() {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let app = Router::new()
        .route(
            "/api/organizations/{id}",
            get(move |Path(_): Path<String>| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Json(json!({ "name": "Acme Health" }))
                }
            }),
        )
        .route(
            "/api/persons/{id}",
            get(|Path(_): Path<String>| async { Json(json!({ "name": "Ada Lovelace" })) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    unsafe {
        std::env::set_var("WPM_UPSTREAM_MODE", "http");
        std::env::set_var("WPM_ORGANIZATION_SERVICE_URL", &base);
        std::env::set_var("WPM_PERSON_SERVICE_URL", &base);
        std::env::set_var("WPM_COURSE_SERVICE_URL", "http://127.0.0.1:1");
    }
    let provider = loco_rs::cache::drivers::inmem::new(&loco_rs::config::InMemCacheConfig {
        max_capacity: 1024 * 1024,
    });
    cache::install(Arc::new(provider));

    // An organization: fetched once, and its name is now in the shared cache.
    let org = format!("organization:{}", uuid::Uuid::new_v4());
    let org_ref = EntityRef::from_str(&org).unwrap();
    assert_eq!(
        clients::display_name(&org_ref).await.as_deref(),
        Some("Acme Health")
    );
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert_eq!(
        cache::get(Namespace::OrganizationName, &org)
            .await
            .as_deref(),
        Some("Acme Health"),
        "an organization's name is shared"
    );

    // An organization another instance already cached is answered without calling the sibling.
    let other = format!("organization:{}", uuid::Uuid::new_v4());
    cache::put(Namespace::OrganizationName, &other, "Cached Elsewhere").await;
    let before = hits.load(Ordering::SeqCst);
    assert_eq!(
        clients::display_name(&EntityRef::from_str(&other).unwrap())
            .await
            .as_deref(),
        Some("Cached Elsewhere")
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        before,
        "no call to the sibling"
    );

    // A person: fetched, kept in this process, and found in no shared namespace.
    let person = format!("person:{}", uuid::Uuid::new_v4());
    assert_eq!(
        clients::display_name(&EntityRef::from_str(&person).unwrap())
            .await
            .as_deref(),
        Some("Ada Lovelace")
    );
    for namespace in Namespace::ALL {
        assert_eq!(cache::get(namespace, &person).await, None, "{namespace:?}");
    }

    // A sibling that is down is a plain miss, and nothing is cached for it.
    let course = format!("course:{}", uuid::Uuid::new_v4());
    assert_eq!(
        clients::display_name(&EntityRef::from_str(&course).unwrap()).await,
        None
    );
    assert_eq!(cache::get(Namespace::CourseName, &course).await, None);
}
