# Install

How to build and run both WPM subprojects from source.

## Prerequisites

- **Rust** — MSRV is declared as `rust-version = "1.96"` in
  `workforce-planning-management-api-with-rust/Cargo.toml`.
- **Podman** (not Docker) — for the service's test database.
- **PostgreSQL 18** — provided by
  `workforce-planning-management-api-with-rust/compose.test.yaml`;
  no host install needed for development.
- **Node.js 26** and **pnpm** — for the SvelteKit front-end
  (`"engines": { "node": "=26" }` in its `package.json`).

## Build and run the service

```sh
cd workforce-planning-management-api-with-rust
cargo run -- db migrate    # apply migrations
cargo run -- task seed     # synthetic org, ~40 employees
cargo run -- start         # JSON API, default port 5150
```

```sh
cargo test                 # DB-free unit tests
cargo test -- --ignored    # request tests (needs Postgres — see below)
```

## Run the service's DB-gated tests

```sh
podman compose -f compose.test.yaml up -d
cargo test -- --ignored
podman compose -f compose.test.yaml down
```

## Build and run the front-end

```sh
cd workforce-planning-management-ui-with-svelte
cp .env.example .env       # WPM_API_URL / AUTH_API_URL — both default
                            # to http://localhost:5150
pnpm install
pnpm dev                   # expects the service running (stub mode is fine)
```

```sh
pnpm check                 # svelte-kit sync && svelte-check
pnpm test                  # vitest
pnpm exec playwright test  # page.route-stubbed — no running service needed
```

## Auth

Both subprojects run with authentication enforcement **off** by
default. See [spec/auth.md](spec/auth.md) to activate
`WPM_REQUIRE_AUTH` and mount a real ABAC policy, and
[SECURITY.md](SECURITY.md) before exposing either subproject to
untrusted callers.
