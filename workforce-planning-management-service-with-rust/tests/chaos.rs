//! Chaos testing (WPM-R137, WPM-D88): what the service does when its database misbehaves.
//!
//! A small TCP proxy sits between the app and PostgreSQL and, on command, resets connections,
//! swallows them, or slows them. The test then asks the questions that matter in an outage:
//!
//! * does every request **finish, and soon** (no hang, no panic);
//! * is the failure a plain `5xx` that **leaks nothing** (no connection string, driver or SQL text);
//! * is a write **all or nothing** (every worker has its audit row; none half-written);
//! * does the service **heal** by itself once the database is back.
//!
//! Then a seeded "monkey" flips the proxy's mode at random while writes are in flight. The seed is
//! printed (and `WPM_CHAOS_SEED` replays it), so a failure can be reproduced.
//!
//! Its own test binary: it points `DATABASE_URL` at the proxy for the whole process.
//! `#[ignore]`d: needs PostgreSQL. Run with `cargo test --test chaos -- --ignored`.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

use loco_rs::testing::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::{Value, json};
use serial_test::serial;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use workforce_planning_management::app::App;
use workforce_planning_management::models::_entities::{audit_logs, workers};

const PASS: u8 = 0;
const RESET: u8 = 1;
const BLACKHOLE: u8 = 2;
const SLOW: u8 = 3;

/// A proxy to PostgreSQL whose behaviour changes on command.
struct Chaos {
    mode: AtomicU8,
    sever: broadcast::Sender<()>,
}

impl Chaos {
    /// Set the mode and cut every connection already open, as a real network does when it
    /// partitions or heals (a healed partition resets the half-open flows left behind).
    fn set(&self, mode: u8) {
        self.mode.store(mode, Ordering::SeqCst);
        let _ = self.sever.send(());
    }
}

async fn start_proxy(upstream: String) -> (Arc<Chaos>, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let (sever, _) = broadcast::channel(16);
    let chaos = Arc::new(Chaos {
        mode: AtomicU8::new(PASS),
        sever,
    });
    let state = Arc::clone(&chaos);
    tokio::spawn(async move {
        loop {
            let Ok((mut client, _)) = listener.accept().await else {
                break;
            };
            let state = Arc::clone(&state);
            let upstream = upstream.clone();
            tokio::spawn(async move {
                let mut cut = state.sever.subscribe();
                match state.mode.load(Ordering::SeqCst) {
                    RESET => {} // dropped on return: the client sees the connection die
                    BLACKHOLE => {
                        // accepted and never answered, until the next change of mode
                        let _ = cut.recv().await;
                    }
                    mode => {
                        if mode == SLOW {
                            tokio::time::sleep(Duration::from_millis(300)).await;
                        }
                        if let Ok(mut server) = TcpStream::connect(&upstream).await {
                            tokio::select! {
                                _ = copy_bidirectional(&mut client, &mut server) => {}
                                _ = cut.recv() => {}
                            }
                        }
                    }
                }
            });
        }
    });
    (chaos, port)
}

/// `host:port` of the real database, from `DATABASE_URL` or the test default.
fn upstream_address() -> String {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://loco:loco@localhost:5432/x".to_string());
    let after_at = url.rsplit('@').next().unwrap_or("localhost:5432/x");
    after_at
        .split('/')
        .next()
        .unwrap_or("localhost:5432")
        .to_string()
}

/// The database URL with its host and port replaced by the proxy's.
fn through_proxy(port: u16) -> String {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://loco:loco@localhost:5432/workforce_planning_management_test".to_string()
    });
    let (credentials, rest) = url.rsplit_once('@').expect("a URL with credentials");
    let database = rest.split_once('/').map_or("", |(_, d)| d);
    format!("{credentials}@127.0.0.1:{port}/{database}")
}

/// A tiny seeded generator, so a run can be replayed.
struct Seeded(u64);
impl Seeded {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

/// No connection detail, driver name or SQL in what a caller is shown.
fn assert_no_leak(body: &str) {
    let lower = body.to_lowercase();
    for word in [
        "postgres",
        "sqlx",
        "sea_orm",
        "seaorm",
        "loco:loco",
        "password",
        "127.0.0.1",
        "select ",
        "insert ",
        "panicked",
    ] {
        assert!(!lower.contains(word), "the response leaks `{word}`: {body}");
    }
}

fn worker_body(org: &str, number: &str) -> Value {
    json!({
        "person_ref": format!("person:{}", uuid::Uuid::new_v4()),
        "organization_ref": org, "worker_number": number,
        "display_name": format!("Chaos Worker {number}"), "employment_type": "permanent",
        "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
    })
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test chaos -- --ignored`"]
#[allow(clippy::too_many_lines)] // one story: baseline, three faults, recovery, then the monkey
async fn the_service_fails_cleanly_and_heals() {
    let (chaos, port) = start_proxy(upstream_address()).await;
    // `set_var` is `unsafe` in edition 2024; set before the app boots and reads it.
    unsafe {
        std::env::set_var("DATABASE_URL", through_proxy(port));
        std::env::set_var("WPM_REQUIRE_AUTH", "0");
        std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", "0");
        std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "0");
    }
    let seed = std::env::var("WPM_CHAOS_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.as_secs())
        });
    println!("chaos seed: {seed} (replay with WPM_CHAOS_SEED={seed})");

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());

        // Baseline: healthy through the proxy.
        let made = request
            .post("/api/workers")
            .json(&worker_body(&org, "base-1"))
            .await;
        assert_eq!(made.status_code(), 200, "baseline write");
        assert_eq!(request.get("/api/workers").await.status_code(), 200);

        // Three faults: each request ends soon, as a plain 5xx, leaking nothing, writing nothing.
        for (name, mode) in [("reset", RESET), ("blackhole", BLACKHOLE)] {
            chaos.set(mode);
            let number = format!("fault-{name}");
            for attempt in 0..3 {
                let started = Instant::now();
                let write = if attempt == 0 {
                    request
                        .post("/api/workers")
                        .json(&worker_body(&org, &number))
                        .await
                } else if attempt == 1 {
                    request.get("/api/workers").await
                } else {
                    request.get("/api/directory").await
                };
                let took = started.elapsed();
                assert!(took < Duration::from_secs(8), "{name}: took {took:?}");
                let status = write.status_code().as_u16();
                assert!((500..600).contains(&status), "{name}: status {status}");
                assert_no_leak(&write.text());
            }
            chaos.set(PASS);
            heal(&request, &ctx.db).await;
            let leftover = workers::Entity::find()
                .filter(workers::Column::WorkerNumber.eq(number.as_str()))
                .count(&ctx.db)
                .await
                .expect("count");
            assert_eq!(leftover, 0, "{name}: a refused write left a worker behind");
        }

        // Slow: requests still end, either way, and nothing leaks.
        chaos.set(SLOW);
        let started = Instant::now();
        let slow = request.get("/api/workers").await;
        assert!(started.elapsed() < Duration::from_secs(8), "slow database");
        if slow.status_code().as_u16() >= 500 {
            assert_no_leak(&slow.text());
        }
        chaos.set(PASS);
        heal(&request, &ctx.db).await;

        // The monkey: random faults between and during writes (one after another: the test server is not shared across tasks).
        let mut rng = Seeded(seed);
        let mut created = 0usize;
        let mut attempted = 0usize;
        for round in 0..30 {
            let mode = match rng.next() % 4 {
                0 => PASS,
                1 => RESET,
                2 => BLACKHOLE,
                _ => SLOW,
            };
            chaos.set(mode);
            for i in 0..3 {
                let number = format!("monkey-{round}-{i}");
                attempted += 1;
                let response = request
                    .post("/api/workers")
                    .json(&worker_body(&org, &number))
                    .await;
                let status = response.status_code().as_u16();
                assert!(
                    status == 200 || (500..600).contains(&status),
                    "round {round}, mode {mode}: unexpected {status} (seed {seed})"
                );
                if status == 200 {
                    created += 1;
                } else {
                    assert_no_leak(&response.text());
                }
            }
        }
        chaos.set(PASS);
        heal(&request, &ctx.db).await;

        // All or nothing: a cut can lose the reply to a write that committed, so there may be more
        // workers than `201`s, never fewer, never more than were tried, and none without its audit row.
        let monkeys = workers::Entity::find()
            .filter(workers::Column::OrganizationRef.eq(org.as_str()))
            .filter(workers::Column::WorkerNumber.starts_with("monkey-"))
            .all(&ctx.db)
            .await
            .expect("workers");
        assert!(monkeys.len() >= created, "a 200 was lost (seed {seed})");
        assert!(
            monkeys.len() <= attempted,
            "a write happened twice (seed {seed})"
        );
        for worker in &monkeys {
            let audits = audit_logs::Entity::find()
                .filter(audit_logs::Column::EntityPid.eq(worker.pid))
                .filter(audit_logs::Column::Action.eq("created"))
                .count(&ctx.db)
                .await
                .expect("audits");
            assert_eq!(
                audits, 1,
                "worker {} has {audits} audit rows: half-written (seed {seed})",
                worker.worker_number
            );
        }
        println!(
            "monkey: {created} created, {} present, {attempted} tried",
            monkeys.len()
        );
    })
    .await;
}

/// The service must answer `200` again within a few seconds of the database returning.
async fn heal(request: &loco_rs::TestServer, db: &sea_orm::DatabaseConnection) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response = request.get("/api/workers").await;
        if response.status_code() == 200 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the service did not recover: {} {} (ping: {:?})",
            response.status_code(),
            response.text(),
            db.ping().await
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
