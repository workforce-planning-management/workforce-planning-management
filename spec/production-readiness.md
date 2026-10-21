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
  (otherwise every caller shares the proxy's address and one bucket). The UI's
  same-origin proxy is such a proxy: it overwrites `X-Forwarded-For` with the
  address it sees (`getClientAddress()`; behind a further proxy, set the Node
  adapter's `ADDRESS_HEADER`), and forwards only `accept`, `content-type`,
  `if-none-match`, a plain idempotency key and its own bearer (WPM-T248).
- Limits are per instance (in memory); behind several instances each counts
  alone, so the caller's real limit is the number of instances times the setting.
  What a deployer adds (a limit at the proxy, divided limits, or sticky routing) is in
  [deployment.md](operations/deployment.md#running-more-than-one-instance-wpm-t198) (WPM-T198).

## WPM-R77 — Builds from its own repository, and CI checks it

- `entity-ref` is **vendored** in
  `workforce-planning-management-service-with-rust/crates/` and `authentication-verifier`
  comes from crates.io (0.9, the same source the vendored copy was); no `../../` path remains. The test database's init
  script is in `postgres-init/`.
- `.github/workflows/ci.yml` runs on every push and pull request: the documents
  checks (`scripts/check-links.py`, `check-days.py`, `check-licenses.py`,
  `status.py --check`), `cargo fmt --check`, `clippy -D warnings`, unit tests,
  the database suites (`mod`, `enforcement`, `enforcement_expenses`, `security`)
  against a Postgres service, `cargo deny check`, the UI (`svelte-check`,
  vitest, build, prettier, Playwright, an advisory `pnpm audit`), the Keycloak suite and an OWASP ZAP baseline
  against the defaults (both blocking since they stayed green over six runs). `.github/dependabot.yml` covers cargo, npm and the workflow actions.
- Branch protection (requiring those 12 jobs; force-push and deletion blocked; admins may bypass) is a repository setting, not a file. Set 2026-10-19.

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
**Signed and attested (WPM-T195):** the image is signed by its digest with cosign (keyless: the signature names the release workflow run, and
there is no key to keep or lose), carries SLSA build provenance (a GitHub attestation pushed to the registry), and the workflow verifies
both before it creates the release. The `Containerfile`'s two base images are pinned by digest.
To check an image yourself:

```sh
IMAGE=ghcr.io/workforce-planning-management/workforce-planning-management-api:v0.1.2   # use the tag you run
cosign verify "$IMAGE" --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp '^https://github.com/workforce-planning-management/workforce-planning-management/\.github/workflows/release\.yml@refs/tags/v'
gh attestation verify "oci://$IMAGE" --repo workforce-planning-management/workforce-planning-management
```
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
- **WPM-D83 A partner reads the monthly pack through a role that can read nothing else, and the pack holds no person.** The pack
  (WPM-R69) is one stored payload, written once (a trigger refuses any update or delete), that the board, the oversight body and
  partner organizations all read, so no audience is shown different figures. It holds aggregates only: pools by name, months,
  counts and verdicts; no worker, worker number or list of people, so nothing is removed for a partner and nothing can be
  restored by mistake. The `partner` membership role is enforced in the need-to-know middleware: a caller whose live memberships
  are all `partner` may `GET` the packs of the organizations in their scope and no other route, and may write nothing. A partner
  who also holds another role is not restricted from reading, but is still refused the freeze unless they are an administrator, because that role is theirs for another reason. The check costs one membership
  query per signed-in request.
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

## WPM-R137 — The service fails cleanly when its database misbehaves, and heals

- **Bounded and plain.** With the database refusing, silent or slow, a request ends within seconds as a `5xx` whose body names no
  connection string, driver, SQL or address. No request hangs, and none panics.
- **All or nothing.** A write that fails leaves nothing behind. Of the writes that were cut off at random, every worker that exists has
  exactly one `created` audit row. There may be more workers than success replies, because a cut can lose the reply to a write that
  committed; never fewer, and never a write applied twice.
- **It heals.** Once the database answers again, the service answers `200` within seconds, without a restart.
- **Reproducible.** The fault schedule comes from a seed that is printed, and `WPM_CHAOS_SEED` replays it.

## WPM-D88 — Chaos by a proxy in the test, not by killing containers

A proxy in the test process injects the faults, so the same test runs on a laptop under Podman and on a CI runner, and each fault is exact
and repeatable. Stopping the real container would be closer to an outage but is slow, differs between Podman and the runner, and cannot
produce a half-open or slow connection. The proxy cuts every open connection when its mode changes, as a real network does when it
partitions and when it heals.

**A known limit, found by this test.** While a database accepts a connection and then says nothing, a connection attempt holds its pool
slot until that socket dies, and the test configuration's pool has one slot, so the service stays failing until the socket is reset. A
deployment's pool is larger, and a healed partition resets the half-open flows, but a database that stays silent for a long time keeps
slots occupied. The remedy is a connection-level timeout (TCP keepalive or a proxy that closes idle handshakes) in the deployment, not in
this repository.

## WPM-R122 — Every request leaves a line with an id and nothing personal; the audit trail can be exported

- **A request id on every response.** `X-Request-Id` is the caller's when it is plain (letters, digits, `-`, `_`, `@`, up to 128), else a
  new UUID. The outermost layer sets it, so a `401` or `429` carries it too.
- **One log line per request, and what is in it.** `request_id`, `method`, a `route`, then on completion `status` and `latency_ms`. The route is
  the path with every segment that is not a plain lowercase word replaced by `{}` (`/api/workers/{}/status`), so no identifier, reference or
  name in a path reaches the log. **Not in it:** the query string, any header (so no token, address or user agent), the body, the caller.
  Loco's own request logger writes the whole URI and is switched off in every profile. A test captures the real log output for a request
  carrying a name in the query, an identifier in the path and a token, and fails if any appears.
- **The audit trail for a security monitoring system.** `GET /api/audits/export?after=<chain_seq>&limit=<n>` returns JSON Lines, oldest
  first, at most 1000 (default 500). Each line has `chain_seq`, `at`, `entity`, `entity_pid`, `action`, `actor`, `prev_hash` and
  `entry_hash`, and **never the snapshot**, which carries record content. `X-Next-After` names the last `chain_seq`; the monitor polls
  with it. Privileged callers only (WPM-R114), and in the sensitive rate-limit class (it ends in `/export`).

## WPM-D89 — Replace the framework's request log rather than filter it

Filtering a URI after the fact trusts every future route and every future query parameter not to carry a name. A line that is built from
a short list of fields (and a path that is reduced to its words) cannot leak what it never reads. The cost is one small middleware in
place of a configuration switch. **A limit:** a path segment that is a lowercase word is kept, so a route whose path carries a lowercase
word that is itself personal would be logged; none does today, and the test will not know about a new one. Keep personal values out of paths.

## WPM-D90 — Read an organization in pages and batches, and refuse what is still read whole over a measured ceiling

The load test (`BENCHMARKS.md`) showed the views that loaded an organization grew with it and, at 50,000 workers, failed requests,
including unrelated ones. Three changes, each with the largest effect for its view:

- **The directory** searches, orders and pages in the database (`strpos(lower(…), term)`, an `EXISTS` for the readable manager's name, the
  on-call match as a pid list, ordering by lower-cased name in the `C` collation then pid so pages are stable and equal to the in-memory rule's
  order). The leave, backups and managers are read for the page only. `rules::directory::search` stays as the reference the unit tests state.
- **The capacity view** reads workers in batches of 2,000 by `id` and adds up `pool_weighted_days`; the one rounding happens on the total
  (`supply_from_weighted`), so the answer is identical (a unit test says so) and memory no longer depends on the organization.
  *(Superseded by WPM-D95: the sum is now made by the database, one query per pool; the rule stays the reference and a test walks every day.)*
- **The plan forecast and cost views** read only the departments the plan names, and only the skills its roles ask for.

What is **still read whole** is the plan views' planned-department workers. For them, one `COUNT` before the load and a `422` over
`WPM_MAX_ORG_WORKERS` (default 10,000 then; at 50,000 they failed; **200,000 since WPM-D95**). A refusal that names the number is better than a request that holds
a connection for a second and a runtime for the others. **The cost is real**: a plan over 10,000 workers loses its forecast and cost until
the setting is raised or the arithmetic is moved into SQL aggregates.

## WPM-D95 — The supported size is 200,000 workers, and a view that cannot hold it is a defect, not a limit

The requirement changed from "bounded and refused above a measured ceiling" (WPM-D90) to **200,000 workers in one organization**.
The rule for every view that loads an organization: its work is done by the database (aggregates, pages, joins) or in batches whose
memory does not depend on the organization, and it answers **within 2 seconds at the 95th percentile with 8 concurrent connections and
no failed request** on `scripts/load-test.sh 200000`. The `WPM_MAX_ORG_WORKERS` refusal stays as a deployer's guard; its default **is 200,000** since the plan forecast and cost views
met the bar (WPM-T239). The views that still do not are listed in `BENCHMARKS.md`. Seeding 200,000 workers is the evidence; a claim without it is not
made. **The cost:** the arithmetic of the plan views (forecast, cost, capacity) has to move into SQL aggregates, which is the largest
part of the work and is done view by view (WPM-T239).

## WPM-D96 — Working time reads a weekly rollup, kept by triggers

The 17-week average of recorded minutes summed every time entry of the window: at 200,000 workers (4,000,000 entries) one request took 515 ms
and eight at once took 1.9 s at the median, bound by the database's CPU. The view now reads **`time_entry_weeks`**, one row per worker per
calendar week (Monday start) holding the live (not deleted) recorded minutes, kept by a trigger on `time_entries` for every insert, update and
delete, so no writer can forget it. Migration 78 builds it from the entries.

The answer is unchanged: the rollup only *names who can be over*. The window of 119 calendar days touches 18 calendar weeks, so a total over the
ceiling (48 hours x 17 weeks) must contain a week of at least 2,720 minutes (`working_time::HEAVY_WEEK_MINUTES`); a partial index holds those weeks, the
view finds the workers with one, sums their weeks as an upper bound, and then sums their entries over the exact window. Workers who rest-flag but
are not over still read their own total. A test pins the window edges (the day the window starts is out, the day of `as_of` is in) and that the rollup
equals the entries after an insert, an update, a soft delete, a restore and a hard delete; a second test pins the floor to the index.

**Not claimed:** the rollup is derived from entries already exported and erased with the worker; it holds only a worker reference, a week and a
total, so it is not listed as a separate record in subject access. A deployer whose workers all record more than 45 hours in a week gets no pruning
(the index holds everything) and the view costs what it did before the index, about 1 s at the median for 200,000 workers.

## WPM-D97 — A view reads what it needs: counts in the database, batches for lists, a cap on what it names

Surveying every parameterless `GET` route at 200,000 workers (`LOAD_SURVEY`) found what the bar had not: four views **failed outright**, because
they passed every worker id to one `IN (…)` list and a statement carries at most 65,535 bind parameters (a size-dependent `500`); two returned
22 and 36 MB; and eight more took 0.5 to 1.1 s alone and 2 to 5 s at eight connections. The rules that follow:

- **Count in the database** what a view only counts (training analytics, the skills matrix, capability, capability analysis, the contingent
  share, the workforce skill gaps). The rule that grades one person (`skill_gap::merge`) stays the reference; `gap_sql` grades everyone at once and
  `rollup_counted` and `contingent_share_counted` give the answer that one row per person gives (a test pins each).
- **Read a list in batches** (`controllers::find_in`, 5,000 ids a time), never as one `IN` list. A quadratic lookup (`iter().find()` per row) is a
  defect at this size.
- **Read whole only the people who can change the answer**: training demand reads only those below a need; the rest are counted.
- **A view that names people names a bounded number and says how many there are in all.** The mentorship overview names the newest 200 unmatched
  people (at most 1,000) and always gives the total; the org chart draws at most 5,000 people (at most 20,000), roots first and then each level,
  and `/api/org-chart/size` gives the total so the page says when it shows only part.
- **Index what a periodic report scans**: every soft-deleting table has a partial index on `deleted_at` (migration 79; a test pins it), so the
  retention report and sweep do not scan millions of time entries.

**Not claimed:** routes that take an id (a worker, a plan) were not surveyed; the case-insensitive department filter now uses `lower()`, which also
folds non-ASCII letters (the old code folded ASCII only).

## WPM-D98 — The API version is a header, not a path

Every route is `/api/<noun>`. A client selects a representation with `Accepts-version: 1.0`; a request without the header gets the current
version, a bare major (`1`) is read as its current minor, and an unsupported version is a `406`. The response carries the version it served.
A new version is added to `SUPPORTED_API_VERSIONS` in `src/version.rs`, **never** to the URL: **do not** add `/api/v1`, `/api/v2` or any
version segment, and do not mount a second copy of a route under one. One URL then names one resource for its whole life, so links, the
OpenAPI document, the UI's `/api/proxy/*` calls and bookmarks do not change when a version does.

**Why:** a version in the path forks every URL and every client at once, and invites two copies of each handler to be kept in step. A header
changes only the representation. **Not claimed:** only version `1.0` exists, so no request has yet been served a different representation;
the header is checked on `/api/*` only (not `/metrics.prom` or `/api-docs`).
