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
cargo test                 # DB-free unit tests (297)
cargo test -- --ignored    # request tests (54; needs Postgres — see below)
cargo test --test enforcement -- --ignored   # auth persona matrix
```

### Schedule the tasks

Two loco tasks do nothing unless something runs them — schedule each **daily**
(cron, a systemd timer, a Kubernetes CronJob):

```sh
cargo run -- task snapshot_headcount     # records aggregate headcount (cannot be backfilled)
cargo run -- task rota_reminders         # tells whoever's on-call turn starts tomorrow (idempotent)
cargo run -- task pay_progression_reminders  # tells who becomes eligible for a pay step within 30 days (idempotent)
```

Optional imports: `cargo run -- task import_framework` (UK GDAD PCF) and
`import_esco` (ESCO occupations and skills) — see the task entries in
[spec/tasks.md](spec/tasks.md) for the data files they read.

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
pnpm test                  # vitest (77)
pnpm exec playwright test  # 45 specs, page.route-stubbed — no running service needed
pnpm build
```

Open `http://localhost:5173/` — it redirects to your locale, e.g.
`/en-001/`. Every page is served under its locale (`/cy-001/workers`); `/en/…`
redirects to `/en-001/…`.

### Edit the translations

The UI strings are content: `content/locales/<locale>/ui.json`. Edit them in a
file, or in the browser with Sveltia CMS at `/<locale>/admin/` (it signs in with
GitHub and commits to the repository). After adding a key to `en-001`, add it to
every other `-001` locale and run `pnpm cms-config` to regenerate
`static/admin/config.yml`; the tests fail if either is missed.

### The CEO dashboard

`/ceo` is built for one screen: an iPad (9th generation), 2160 × 1620 pixels =
1080 × 810 CSS pixels at 2×. In Chrome DevTools use a custom device of 1080 × 810
with a device pixel ratio of 2 to see it as intended.

## Running the service tests without the sibling crates

The service depends on two crates that live outside this repository
(`entity-ref`, `authentication-verifier`). See
[spec/testing.md](spec/testing.md) for building a scratch workspace that
provides them and running the database-backed tests against a throwaway
Postgres under Podman.

## Auth

Both subprojects run with authentication enforcement **off** by
default. See [spec/auth.md](spec/auth.md) to activate
`WPM_REQUIRE_AUTH` and mount a real ABAC policy, and
[SECURITY.md](SECURITY.md) before exposing either subproject to
untrusted callers.
