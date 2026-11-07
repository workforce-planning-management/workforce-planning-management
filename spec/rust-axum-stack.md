### Technology stack for Rust and Axum

- **WPM-D189 Lists page with `axa::cursor_pagination`, signed with one key per deployment.** The owner asked on 2026-11-07 (WPM-R241, WPM-T478). `controllers::Page` is an extractor that keeps the request's URI as the client sent it (`OriginalUri`), and `resolve` builds `axa`'s `CursorPage` from it with the list's default limit and `MAX_LIMIT` 500, so every list keeps the call it had: `window(&page)?`, `window.bounds()` for `LIMIT` (one more than the page) and `OFFSET`, `respond(rows)`, or `split` and `with_next` for a body that is not a bare array or a list that must drop the extra row before it acts on it. WPM refuses `?offset=` itself, because `axa` takes it as a filter, and an old client would re-read the first page without being told. The key is `WPM_CURSOR_KEY`, base64url of at least 32 bytes, read at start: a value that is set but unusable stops the start, and none is a random key for the process, with a warning in production. **Rejected:** a key derived from another secret, which would tie list cursors to a key with another purpose; failing to start without a key, which would stop every one-instance deployment and CI job over a feature that fails safely; keeping `/api/workers` on offsets for CRM's import, which the owner declined on 2026-11-07, so CRM follows `Link` ([cross-app-bridges.md](cross-app-bridges.md)). ([production-readiness.md](production-readiness.md))
- **WPM-D188 `axa::ratelimit` counts the rate limit; WPM keeps the classes and the layer's place.** The owner asked on 2026-11-07. `RateLimits` holds two `axa::ratelimit::Limiter`s, ordinary and sensitive, so the classes are counted apart by construction, and `0` leaves a class with no limiter. The caller is `axa::ratelimit::client_key` of the address `rate_limit_mw` resolves (WPM-D187), and the limiter holds it only as a keyed hash, for at most two windows, with at most 100,000 callers before new ones share one count. The refusal and the `RateLimit-Limit`, `RateLimit-Remaining` and `RateLimit-Reset` headers on what is let through are `axa`'s. The layer is still WPM's, because it runs outside `axa::middleware::apply`, before sign-in, where `axa`'s `by_client` would find no address; `is_sensitive` is WPM's, as `axa` knows no routes. The limits are made with the router, not once per process. **Not adopted:** `lock_out_failures`, a lockout after a client's `401`s, which would be a new requirement. ([production-readiness.md](production-readiness.md), WPM-R74, WPM-D54)
- **WPM-D187 The rate limit names a caller with `axa`'s resolved address, and `WPM_TRUST_FORWARDED` is retired.** This supersedes the note in WPM-T469 that `X-Forwarded-For` stays WPM's own. `axa::client::resolve` believes the header only from a peer in `TRUSTED_PROXIES` and walks it from the right; taking the first entry trusted the caller unless the proxy overwrote it. `rate_limit_mw` calls `resolve` itself, with `config.http.trusted_proxies`, and does not read the address `axa::middleware::apply` records: WPM's layers are outside `apply`, so that address is not on the request yet, and the limit cannot move inside `apply`, because it must run before sign-in refuses a guesser. The retired variable is a warning and not a refusal, because a running deployment should not stop over a setting that now does nothing. The key stayed an unkeyed 64-bit hash, as it is held in memory and never logged; WPM-D188 now holds it as `axa::ratelimit`'s keyed hash ([production-readiness.md](production-readiness.md), WPM-R239).
- **WPM-D186 Errors become RFC 9457 problems in one layer, not in every handler.** Handlers return `axa::Error`, and `src/problem.rs` rewrites what is not a problem yet, with `axa::problem::Problem`. `error` becomes the extension `code`, which code can match on; `description` becomes `detail`. The layer runs twice: inside `axa`'s compression for what handlers refuse, because a compressed body cannot be read, and outside WPM's layers for sign-in, need-to-know, version, rate limit, timeout and panic. A `5xx` in plain text loses its text. A JSON-RPC error on `/mcp` stays one. The owner asked for this on 2026-11-06. **From `axa` 0.6 (WPM-T472)** `axa::Error` is a problem already and passes through untouched. The layer stays, because sign-in, need-to-know, idempotency, version and the rate limit refuse outside `axa::middleware::apply`, where `axa` converts nothing, and several of them answer in plain text or as `{"error"}`; inside `apply`, `axa` keeps a plain-text error's status and drops its words. It is deleted when each of those layers answers with a `Problem` itself.
- **WPM-D185 The refusal of destructive commands lives in `axa`.** `cli::parse` refuses `db down`, `db reset` and `db truncate` outside development and test, and asks for `AXA_DESTROY_DATA=true` in both (`axa` 0.4.1), because a guard in the shared parser cannot be left out by a service. WPM's own check is gone; a test holds the binary to `cli::parse`, since `Cli::parse()` skips it.
- **WPM-D184 Queries move from SeaORM to `sqlx`, one area at a time.** Each query is written out with `sqlx::query!`, so a reviewer reads what runs and a wrong column or type fails the build. `sqlx` and SeaORM are the same `sqlx` 0.9 underneath, so `db::pool` lends the one pool to both while the work goes on. A handler moves whole, because a transaction belongs to one library. A build reads `.sqlx/` with `SQLX_OFFLINE` set in `.cargo/config.toml`, so it needs no database, and CI fails if that cache has drifted from the schema. Migrations move last, because `seaql_migrations` is what tells a database what has run. The owner asked for this on 2026-11-05. A query whose conditions vary with its input stays one fixed statement, so it stays checked: the values go in as an array, read with `unnest`, and "every term matches" is written `NOT EXISTS (... WHERE (...) IS NOT TRUE)`, because with a plain `NOT` a NULL column would pass the term. `QueryBuilder` gives up the check, and is used only where a fixed statement cannot work; the owner chose this on 2026-11-06, for `directory`.

- **WPM-D183 The service runs on Axum and axa, with the schedule and the cache in its own code.** `axa` gives the routes, the middleware, the configuration from the environment, the command line and tasks. It has no scheduler and no cache, so `src/scheduler.rs` keeps the schedule (`workforce-planning-management scheduler --tag wpm`) and `src/shared_cache.rs` talks to memory or Redis directly, reading what the old provider wrote. Settings are environment variables; `config/` holds only the reference policy. `APP_ENV` names the environment and the old `LOCO_ENV` still works. **Changed on purpose:** no automatic `ETag` or `304`; `JWT_SECRET` and `SMTP_*` are no longer read; the request timeout rounds up to whole seconds; CORS mirrors the request's headers where it listed four. The owner asked for this on 2026-11-05. SeaORM stays until a separate change.


| Component            | Technology                                        | Purpose                                     |
| -------------------- | ------------------------------------------------- | ------------------------------------------- |
| **Language**         | Rust 1.95+ 2024 Edition                           | Systems programming, performance, safety    |
| **Async Runtime**    | Tokio                                             | Asynchronous I/O and concurrency            |
| **HTTP**             | tower-http                                        | HTTP layer, CORS, compression               |
| **Web Framework**    | Axum, axa                                         | HTTP server and routing (backend-only)      |
| **Database**         | PostgreSQL 18+                                    | Data persistence                            |
| **ORM**              | SeaORM                                            | Async database object-relational mapper     |
| **Search Engine**    | Tantivy                                           | Full-text search indexing                   |
| **Data Streaming**   | Fluvio                                            | Event publishing and durable data streaming |
| **API Docs**         | Utoipa                                            | OpenAPI 3.0 specification                   |
| **Swagger Docs **    | utoipa-swagger-ui                                 | OpenAPI 3.0 specification                   |
| **Serialization**    | Serde                                             | JSON serialization/deserialization          |
| **Logging**          | Tracing                                           | Structured logging                          |
| **Observability**    | OpenTelemetry, opentelemetry-semantic-conventions | Distributed tracing, metrics, spans, logs   |
| **String Matching**  | strsim, fuzzy-matcher                             | Jaro-Winkler, Levenshtein                   |
| **Geo**              | geo, haversine                                    | Coordinate distance calculations            |
| **Containerization** | Podman                                            | Deployment packaging                        |
| **gRPC**             | Tonic                                             | High-performance RPC framework              |
| **Protocols**        | Prost                                             | Protocol buffers                            |
| **Timestamps**       | jiff                                              | Dates, times, durations                     |
| **Environment**      | dotenvy                                           | Config env var                              |
| **Error Handling**   | thiserror, anyhow                                 | Typed and contextual error handling         |
| **Security**         | argon2                                            | Password hashing                            |
| **Authentication**   | jsonwebtoken                                      | JWT authentication                          |
| **Testing**          | assertables, tokio-test, mockall, tempfile        | Unit, integration, and mock testing         |
| **Benchmarking**     | Criterion                                         | Statistical performance benchmarking        |
| **Memory Allocator** | MiMalloc                                          | High-performance MUSL static builds         |
| **Numbers**          | bigdecimal                                        |                                             |
| **Identifiers**      | uuid                                              | UUID generation                             |
| **Validation**       | validator                                         | Declarative validation                      |

[Configure main.rs for MUSL MiMalloc:

```rust
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

](Constraints:

- Podman NOT Docker (because Podman is more open source and more free)
- Tokio NOT async_std (because we harmonize Tower, Axum)
- MiMalloc NOT jemalloc (because MiMalloc handles multiple page sizes)
- PostgreSQL NOT SQLite (because we want testing on the same database)
- jiff NOT chrono (because jiff is better-designed and more-powerful)
- sea-orm feature "with-jiff" NOT "with-chrono" (jiff support is coming)
- rustls NOT OpenSSL (because rustls is easier for static compile)

## Configurations

Add to files `lib.rs` `main.rs` immediately after top-level doc comment.

```rust
// Always start with high quality coding conventions.
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::clippy::pedantic)]

// When we build for MUSL static, use faster memory allocator.
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```
