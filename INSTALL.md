# Install

How to build and run both WPM subprojects from source.

## Try the demo (a guided tour with fictitious data)

The demo needs a **separate** service instance that holds nothing real, because it runs with authentication
off. In the service directory: create an empty database, `cargo run -- db migrate`, `cargo run -- task seed`,
start it with `WPM_REQUIRE_AUTH=0` (and, since a tour clicks quickly, `WPM_RATE_LIMIT_PER_MINUTE=6000`), then
fill it from the repository root:

```sh
python3 scripts/demo-data.py http://localhost:5150
```

Start the UI with `WPM_DEMO_API_URL=http://localhost:5150`. On the sign-in page, sign in as
**example@example.com** (no email is sent): you land on `/demo`, a tour of every page, and a banner on each
page says the data is fictitious and offers Previous and Next. With `WPM_DEMO_API_URL` unset there is no
demo and that address signs in like any other. Never point `WPM_DEMO_API_URL` at a service that holds real
data: a demo session reaches only that service, with no authentication.

## Prerequisites

- **Rust** — stable. The service declares `rust-version = "1.99"` in
  `workforce-planning-management-service-with-rust/Cargo.toml`; the matcher, the migration
  crate and `entity-ref` declare 1.97, the project's N-2 policy (see WPM-T390 for why the
  service is not there yet).
- **Podman** (not Docker) — for the service's test database and Redis.
- **PostgreSQL 18 and Redis 8** — provided by
  `workforce-planning-management-service-with-rust/compose.test.yaml`;
  no host install needed for development.
- **Node.js 26** and **pnpm** — for the SvelteKit front-end
  (`"engines": { "node": "=26" }` in its `package.json`).

## Build and run the service

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate    # apply migrations
cargo run -- task seed     # synthetic org, ~40 employees
WPM_REQUIRE_AUTH=0 cargo run -- start   # JSON API, port 5150; sign-in enforcement is on by default,
                                         # so local development without an identity service opts out
```

```sh
cargo test --lib           # DB-free unit tests (1,034 on 2026-10-30)
cargo test --test mod -- --ignored --test-threads=1   # request tests (184; needs Postgres — see below)
cargo test --test enforcement -- --ignored --test-threads=1   # auth persona matrix
```

### Schedule the tasks

Several loco tasks do nothing unless something runs them. The job list is
[`config/scheduler.yaml`](workforce-planning-management-service-with-rust/config/scheduler.yaml), for Loco's own scheduler;
start **one** next to the server:

```sh
cargo loco scheduler --config config/scheduler.yaml --tag wpm     # webhooks, headcount snapshot, warnings, reminders, audit-chain check
cargo loco scheduler --config config/scheduler.yaml --tag scim    # only where WPM_SCIM_BASE_URL and WPM_SCIM_TOKEN are set
cargo loco scheduler --config config/scheduler.yaml --list        # what is scheduled, and when
```

Or schedule the same tasks yourself (cron, a systemd timer, a Kubernetes CronJob) with `cargo run -- task <name>`; the table in
[spec/operations/deployment.md](spec/operations/deployment.md) says how often each one wants to run.

Optional imports: `cargo run -- task import_framework` (UK GDAD PCF) and
`import_esco` (ESCO occupations and skills) — see the task entries in
[spec/tasks.md](spec/tasks.md) for the data files they read.

## Run the service's DB-gated tests

```sh
podman compose -f compose.test.yaml up -d --wait
export WPM_TEST_REDIS_URL=redis://localhost:6379   # the Redis-backed tests skip without it
cargo test --test mod -- --ignored --test-threads=1   # serial: the tests share one database
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
pnpm lint                  # eslint, then prettier --check
pnpm check                 # svelte-kit sync && svelte-check
pnpm test                  # vitest (291 on 2026-10-30)
pnpm exec playwright test  # 335 pass, 19 skip; page.route-stubbed, no running service needed
                           # (PW_PORT=<free port> if 4173 is taken)
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

## Run the demo in containers

From a fresh clone, with no sibling repositories:

```sh
podman compose up --build
curl http://localhost:5150/_health
```

This starts PostgreSQL, Redis, a demo mail server (Stalwart: a catch-all for `wpm.test` that delivers nothing
outside; read it with `python3 scripts/demo-mailbox.py`) and the API. **It turns sign-in enforcement off**
(`WPM_REQUIRE_AUTH: "0"` in `compose.yaml`, with a warning in the log) because
the demo has no identity service; use synthetic data only. The UI is not
containerized. Not yet verified: the image build, because the maintainer's
machine could not pull base images when this was written.

## Running the service tests

`entity-ref` is vendored in `workforce-planning-management-service-with-rust/crates/`
and `authentication-verifier` comes from crates.io, so a fresh clone builds and tests
on its own. Start a throwaway Postgres and Redis and run the suites (the request
suite runs serially, and each of the other test binaries in its own process):

```sh
cd workforce-planning-management-service-with-rust
podman compose -f compose.test.yaml up -d --wait
export WPM_TEST_REDIS_URL=redis://localhost:6379   # Postgres on 5432, Redis on 6379; both have port overrides
cargo test                                   # unit tests, no database
cargo test --test mod -- --ignored --test-threads=1
cargo test --test enforcement -- --ignored --test-threads=1
cargo test --test enforcement_expenses -- --ignored --test-threads=1
cargo test --test security -- --ignored --test-threads=1
cargo test --test chaos -- --ignored --nocapture   # database faults; WPM_CHAOS_SEED=<n> replays a run
scripts/verify-db-roles.sh                         # the request tests as the least-privilege role (postgres-roles/roles.sql); a few minutes
podman compose -f compose.test.yaml down
```

CI (`.github/workflows/ci.yml`) runs the same commands and every other enforcement
binary, plus `cargo fmt`, `clippy -D warnings`, `cargo deny`, the MSRV check, the
Keycloak and Entra suites, the front-end checks and Playwright.

## Try the family bridges

[docs/tutorials/family-bridges.md](docs/tutorials/family-bridges.md) walks through every bridge to
the sibling apps with `curl` against a throwaway service.

## Auth

The service enforces sign-in **by default** (`WPM_REQUIRE_AUTH`; only an
explicit `0` disables it, and `production` refuses to start that way). See
[spec/auth.md](spec/auth.md) to point it at your token keys and mount a real
ABAC policy, and
[SECURITY.md](SECURITY.md) before exposing either subproject to
untrusted callers.
