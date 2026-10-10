# Install

How to build and run both WPM subprojects from source.

## Prerequisites

- **Rust** — MSRV is declared as `rust-version = "1.96"` in
  `workforce-planning-management-service-with-rust/Cargo.toml`.
- **Podman** (not Docker) — for the service's test database.
- **PostgreSQL 18** — provided by
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
cargo test                 # DB-free unit tests (379)
cargo test -- --ignored    # request tests (73; needs Postgres — see below)
cargo test --test enforcement -- --ignored   # auth persona matrix
```

### Schedule the tasks

Two loco tasks do nothing unless something runs them — schedule each **daily**
(cron, a systemd timer, a Kubernetes CronJob):

```sh
cargo run -- task snapshot_headcount     # records aggregate headcount (cannot be backfilled)
cargo run -- task rota_reminders         # tells whoever's on-call turn starts tomorrow (idempotent)
cargo run -- task pay_progression_reminders  # tells who becomes eligible for a pay step within 30 calendar days (idempotent)
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
pnpm test                  # vitest (80)
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

## Run the demo in containers

From a fresh clone, with no sibling repositories:

```sh
podman compose up --build      # or: docker compose up --build
curl http://localhost:5150/_health
```

This starts PostgreSQL and the API only. **It turns sign-in enforcement off**
(`WPM_REQUIRE_AUTH: "0"` in `compose.yaml`, with a warning in the log) because
the demo has no identity service; use synthetic data only. The UI is not
containerized. Not yet verified: the image build, because the maintainer's
machine could not pull base images when this was written.

## Running the service tests

The two shared crates (`entity-ref`, `authentication-verifier`) are vendored in
`workforce-planning-management-service-with-rust/crates/`, so a fresh clone builds
and tests on its own. Start a throwaway database and run the suites (the request
suite runs serially, and each of the other test binaries in its own process):

```sh
cd workforce-planning-management-service-with-rust
podman compose -f compose.test.yaml up -d --wait
cargo test                                   # unit tests, no database
cargo test --test mod -- --ignored --test-threads=1
cargo test --test enforcement -- --ignored --test-threads=1
cargo test --test enforcement_expenses -- --ignored --test-threads=1
cargo test --test security -- --ignored --test-threads=1
podman compose -f compose.test.yaml down
```

CI (`.github/workflows/ci.yml`) runs the same commands, plus `cargo fmt`,
`clippy -D warnings`, `cargo deny`, the front-end checks and Playwright.

## Auth

The service enforces sign-in **by default** (`WPM_REQUIRE_AUTH`; only an
explicit `0` disables it, and `production` refuses to start that way). See
[spec/auth.md](spec/auth.md) to point it at your token keys and mount a real
ABAC policy, and
[SECURITY.md](SECURITY.md) before exposing either subproject to
untrusted callers.
