# Production readiness (WPM-R72–R74, WPM-R77, WPM-R78, WPM-R87, WPM-R88)

What a deployment needs beyond the demo, as built. The backup plan (WPM-R75)
and the data protection impact assessment (WPM-R76) are still proposed in
[plan.md](plan.md). Delivered as WPM-T123–T128, WPM-T144 and WPM-T145 (see
[tasks.md](tasks.md)); the production gates WPM-G1 and WPM-G2 are in the same
file.

> ⚠️ **Demo software.** Everything here makes the demo safer by default; none of
> it makes it a production HR system. See [regulatory.md](regulatory.md).

## WPM-R72 — Sign-in is enforced by default

- `WPM_REQUIRE_AUTH` is **on** unless it is an explicit `0`, `false`, `no` or
  `off` (case-insensitive). Unset, empty and unrecognized values leave it on, so
  a typo can only make the service stricter.
- With it off the service logs a warning at boot and every ten minutes, and
  `GET /_posture` (public) answers `{"auth_enforced": false}`.
- **Boot refuses** in the `production` environment when enforcement is off, and
  when no token key source is set (`WPM_PASETO_KEYS`, `WPM_PASETO_KEYS_URL` or
  `WPM_KEYCLOAK_JWKS_URL`). Outside production, no key source is a warning: every
  request is then a `401`.
- Public paths are `/_health`, `/_ping`, `/_posture`, `/api-docs/openapi.json`
  and `/swagger-ui*`. **`/metrics.prom` is not public**: a scraper presents a
  bearer token the ABAC policy allows (the reference policy allows `svc`). Any
  other path, including an unknown one, is a `401`.

## WPM-R73 — Security headers and an explicit CORS list

- The API sends, on every response including `401` and `429`:
  `Content-Security-Policy` (`default-src 'none'; frame-ancestors 'none'`, and a
  looser policy for the Swagger UI), `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy`,
  `Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Resource-Policy:
  same-origin`, `Strict-Transport-Security` when the request is over TLS (a
  proxy's `X-Forwarded-Proto: https`, or `WPM_HSTS`), and `Cache-Control:
  no-store` on every `/api` response.
- The UI sends the same set from `hooks.server.ts` (pure helper
  `src/lib/security-headers.ts`, plus `X-Frame-Options: DENY` for older
  browsers) and a content security policy from SvelteKit's `csp` setting in
  `vite.config.ts` (nonces and hashes for its own inline scripts;
  `frame-ancestors 'none'`; `style-src` allows inline styles because Svelte sets
  them).
- The one exception is the content manager's path (`/admin`): its sign-in uses a popup, so it gets
  `Cross-Origin-Opener-Policy: same-origin-allow-popups`. It also loads a script from a public CDN
  without a pinned version or an integrity hash, which is a known gap.
- **CORS** in production is an allow-list from `WPM_CORS_ORIGIN`, required: boot
  fails when it is unset rather than falling back to `*`. Methods and headers are
  listed, credentials are off.

## WPM-R74 — Rate limiting

- Per caller and per minute: `WPM_RATE_LIMIT_PER_MINUTE` (default 600) for any
  route, and `WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE` (default 20) for erase,
  sweep, import, merge, deduplicate and token routes and for exports. A refusal
  is `429` with `Retry-After`. `0` disables a class. Health, ping and posture
  are never limited.
- The caller is its **network address**: the peer address, or the first
  `X-Forwarded-For` hop only when `WPM_TRUST_FORWARDED` is set, which a
  deployment behind a reverse proxy that *overwrites* that header must do
  (otherwise every caller shares the proxy's address and one bucket).
- Limits are per instance (in memory); behind several instances each counts
  alone.

## WPM-R77 — Builds from its own repository, and CI checks it

- The two shared crates are **vendored** in
  `workforce-planning-management-service-with-rust/crates/` (`entity-ref`,
  `authentication-verifier`); no `../../` path remains. The test database's init
  script is in `postgres-init/`.
- `.github/workflows/ci.yml` runs on every push and pull request: the documents
  checks (`scripts/check-links.py`, `check-days.py`, `check-licenses.py`,
  `status.py --check`), `cargo fmt --check`, `clippy -D warnings`, unit tests,
  the database suites (`mod`, `enforcement`, `enforcement_expenses`, `security`)
  against a Postgres service, `cargo deny check`, the UI (`svelte-check`,
  vitest, build, prettier, Playwright, an advisory `pnpm audit`), plus two jobs
  not yet required: the Keycloak suite and an OWASP ZAP baseline against the
  defaults. `.github/dependabot.yml` covers cargo, npm and the workflow actions.
- Branch protection (requiring those jobs) is a repository setting, not a file.

## WPM-R78 — License files

A root `LICENSE.md` states each subproject's SPDX expression and links the texts.
Every license text is in `LICENSE/` (`LICENSE-MIT`, `LICENSE-APACHE`,
`LICENSE-BSD-3-CLAUSE`, `LICENSE-GPL-2.0`, `LICENSE-GPL-3.0`), with the reason for
the options in `LICENSE/index.md`. `scripts/check-licenses.py` checks that the
`license` fields in `Cargo.toml` and `package.json` match both `LICENSE.md` and
`LICENSE/index.md`.

## WPM-R87 — A release people can pin

`.github/workflows/release.yml`, on a `v*` tag that equals the crate version:
build the API container image from `Containerfile`, push it to GHCR by version
and by commit, generate a CycloneDX bill of materials with `cargo cyclonedx`, and
create a GitHub release with generated notes and the bill of materials attached.
`compose.yaml` runs the API and PostgreSQL from a fresh clone as a **demo with
sign-in enforcement explicitly off** (see its header).

## WPM-R88 — Documents match the code

- `spec/implementation-status.md` is **generated** by `scripts/status.py` from
  the files (counts of migrations, controllers, declared tests, locales, tasks);
  CI fails when the committed copy is stale. The counts are of what is declared,
  not what passes.
- `scripts/check-links.py` fails on any dead relative link. A reference to a
  sibling repository is plain text, never a link.

## WPM-R116 — Production refuses insecure data paths (built 2026-10-11)

- **Database TLS.** In the `production` environment, boot refuses a `DATABASE_URL` that reaches another host
  without TLS (`sslmode` of `require`, `verify-ca` or `verify-full`). `disable`, `allow`, `prefer` (which falls
  back to plaintext) and no `sslmode` all count as plaintext. A loopback address or a Unix socket is not on a
  network and needs nothing. `WPM_ALLOW_PLAINTEXT_DATABASE=1` is the explicit, logged override. `require` and
  `verify-ca` start with a warning to use `verify-full`, because they do not check the server's identity.
- **Request timeout.** The production configuration answers a request that runs past
  `WPM_REQUEST_TIMEOUT_MS` (default 30000 milliseconds) with `408`.
- Pure checks in `src/hardening.rs`, unit-tested over nineteen URLs; verified by booting the real binary in
  production against a non-loopback address (refused; started with the override and the warning; a loopback
  address started without either). The request identifier is on every response (`x-request-id`).

## WPM-R132 — Idempotency keys (built 2026-10-11)

- Every request the bundled UI makes that can change something (any method but GET, HEAD and OPTIONS) carries an
  `Idempotency-Key` header: a random UUID, or 128 random bits in hexadecimal where `crypto.randomUUID` is missing.
  A read never carries one. The key names one user action, so a retry passes the original key back; two different
  actions never share one. The same-origin proxy forwards the header only when it is a plain token.
- The API acts on it (`src/idempotency.rs`, rules in `src/rules/idempotency.rs`, table `idempotency_keys`). A key is
  16 to 64 letters, digits, hyphens or underscores, else `422`. The first request under a caller's key runs and, if it
  succeeds, its response (at most 256 KiB) is kept for **one calendar day**. A repeat of the same method, path, query and
  body from the same caller gets that response again with `Idempotent-Replayed: true` and runs nothing. The same key
  with a different request is `422`. A repeat that arrives while the first is still running is `409` with
  `Retry-After`; a claim never finished for 300 seconds is taken over. **A failure is not kept**, so the request can be
  corrected and sent again under the same key. A request with no key, and every read, is untouched.
- **A key belongs to the caller.** It is stored with the caller's identity (`anonymous` only while sign-in is off), and the
  middleware runs after sign-in and need-to-know, so one caller can never be answered with another's stored response.
- Every operation that changes something is documented with the key, the replay header and the `409` and `422`
  answers in the OpenAPI document, added in one place so none can be missed.
- Stored responses may repeat personal data the first response held, which is why they are kept for one calendar day and
  no longer; see the [retention schedule](governance/retention-schedule.md).
- Not done: a response over 256 KiB is returned but not kept (a repeat runs again); a request body over 1 MiB with a key
  is refused (`413`); a key is not shared between instances except through the database, which is shared.

## Decisions

- **WPM-D81 The demo's mail server is a sink that cannot deliver outside, set up by its own container.** A demo that sends
  email needs somewhere safe for it to go. `compose.yaml` runs Stalwart (`deploy/stalwart`): one domain, `wpm.test`, with a
  **catch-all** account, so every message lands in `inbox@wpm.test` and can be read, and **no relaying**, so a message to any
  other address is refused (`550 Relay not allowed`) and nothing leaves the machine. It is not a mail server for use: its
  administrator, account passwords and plain SMTP without a login are demo constants, and the ports are bound to the loopback
  address. The image adds Stalwart's command-line tool and an entrypoint that, on the first start, completes the server's own
  setup, restarts it and applies a plan of upserts, so `podman compose up` needs no clicks and a restart converges to the same
  state. The production SMTP settings (`SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD`) are unchanged; the demo
  configuration reads `SMTP_HOST` and `SMTP_PORT` too, with the old values as defaults.
- **WPM-D82 An early warning goes to the organization's HR administrators, and its date is the date of the first sweep that saw it.**
  WPM-R68 says "the owner", but a skill pool and an oversight measure have no owner, and inventing a field for one would be a
  decision without a requirement behind it. Until an owner exists the warning goes to the workers who hold an `hr_admin`
  membership in the organization, as the engagement reminders do. The warning names a pool or a measure, never a person, and the
  notification carries no figure (WPM-D23). The date a condition was "first true" is the date of the first sweep that saw it (a
  sweep run less often than daily dates it late, and the list says so); a condition that stops and returns is a new warning. Time
  to report is the longest calendar days to acknowledgement, and unknown while any warning is unacknowledged.
- **WPM-D80 The client names the action; the server will decide what a repeat means.** The key is made where the user acts
  (one per action, not per HTTP attempt), so a retry after a lost response or a double click can be told from a new
  action. The proxy checks the key's shape only; it keeps no state, because deduplication belongs next to the data it
  protects. Deduplication sits behind the sign-in and need-to-know checks, so a stored response is only ever returned to the
  caller it was made for, and it holds a success only, so a correctable failure can be retried.
- **WPM-D78 The shared cache holds non-personal lookups only, and fails open.** Loco's cache provider is Redis in
  production and the demo (so several instances share one) and in memory in development and tests. What may be cached is
  a closed list in `src/shared_cache.rs` (`Namespace`): the names of organizations and courses from the services that
  own them, and searches of the public ESCO catalogue. There is no variant for a person, a worker, a score or any
  report, so a cached answer can never be one that a permission check should have decided; a unit test pins the list
  and another that a person's or worker's name never takes the shared route. Every call is bounded to 250 milliseconds and a
  failure is a miss, so a Redis that is down slows nothing and breaks nothing. Keys are versioned and generational
  (`wpm:v1:{namespace}:{generation}:{key}`); the ESCO import bumps its namespace's generation. `REDIS_URL` is optional: unset, the cache is in
  memory per instance and boot logs a warning. In production, a plaintext `redis://` link to another host refuses to
  start unless `WPM_ALLOW_PLAINTEXT_REDIS=1`; `rediss://` works (rustls, `ring`).

- **WPM-D52 Secure by default.** A fresh install enforces sign-in. Turning it
  off is explicit, logged, visible at `/_posture`, and refused in production; a
  missing key source stops a production service instead of opening it.
- **WPM-D54 Rate limits keep no history, and trust no header the caller
  controls.** Counters are in memory per window, keyed by a hash of the network
  address; no address, subject or token is stored or logged. A bearer token is
  never the key, because varying it would buy a fresh bucket on every request.
