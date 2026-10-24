# Deployment guide (WPM-R90)

> ⚠️ **Demo software.** Not a production HR or payroll system; synthetic data only.
> [regulatory.md](../regulatory.md) lists what production would additionally need.

## Two ways to run it

| | Container demo | A real deployment |
| --- | --- | --- |
| How | `podman compose up --build` at the repository root | Build the image or the binary yourself; run behind a TLS-terminating proxy |
| Sign-in | **Off**, explicitly, so the demo needs no identity service | **On** (the default). Token keys required |
| `LOCO_ENV` | `demo` | `production` (refuses to start with sign-in off or no key source) |
| Data | Synthetic | Whatever the deployer's own impact assessment permits |
| Exposure | Bound to `127.0.0.1` only | Never expose the demo |

The compose file starts PostgreSQL and the API. **The UI is not containerized**: it
builds for Node (`adapter-node`, since WPM-T265): `pnpm build`, then `node build` with `WPM_API_URL` pointing at the API. A
deployer on another platform swaps the adapter. The image build and
the compose file are written but were not built where this was written; try them first.

## Required settings (production)

| Variable | Required | Default | What it does |
| --- | --- | --- | --- |
| `DATABASE_URL` | yes | none | PostgreSQL 18 connection string. **In production it must require TLS across a network**: add `?sslmode=verify-full` (best: the server's certificate is checked) or `require` or `verify-ca`; boot refuses a plaintext connection to another host. A loopback address or a Unix socket needs no TLS |
| `REDIS_URL` | no | none: an in-memory cache per instance, with a boot warning (demo: `redis://127.0.0.1:6379`) | Redis for Loco's cache, shared by every instance (WPM-D78). It holds only non-personal lookups (organization and course names, ESCO searches) and a Redis that is down is a cache miss, never an error. Use `rediss://` across a network; boot refuses a plaintext `redis://` to another host |
| `WPM_ALLOW_PLAINTEXT_REDIS` | no | off | Set to `1` only if the network path to Redis is protected another way and you accept the risk; boot then logs a warning |
| `WPM_ALLOW_PLAINTEXT_DATABASE` | no | off | Set to `1` only if the network path to the database is protected another way (a private link, a tunnel) and you accept the risk; boot then logs a warning every start |
| `WPM_RESIGNATION_NOTICE_CALENDAR_DAYS` | no | `28` | The notice a resignation must allow (0 to 366 calendar days); a worker's proposed last day cannot be earlier unless HR agrees one |
| `WPM_FLEXIBLE_DECISION_MONTHS`, `WPM_FLEXIBLE_REQUESTS_PER_YEAR`, `WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS` | no | `2`, `2`, `14` | Flexible working: calendar months a decision may take (1 to 12), requests a worker may make in 12 calendar months (`0` for no limit), calendar days to appeal a refusal. A jurisdiction may fix these; **confirm them** |
| `WPM_FLEXIBLE_REFUSAL_REASONS` | no | a built-in list of nine | Comma-separated lowercase tokens a refusal must choose from; replace to match the grounds your jurisdiction allows |
| `WPM_EQUALITY_MONITORING_BASIS` | no | off | **Equality monitoring is off unless this is set** to the lawful basis you rely on, in your words (for example a reference to your assessment). It is special-category data: do your own impact assessment first. Boot logs a warning that it is enabled |
| `WPM_EQUALITY_CATEGORIES` | with the basis | none | A JSON object of category to list of values, for example `{"sex":["female","male"]}`. **WPM ships no classification**: the categories and their values are yours to define, as lowercase tokens; `prefer_not_to_say` is always offered. Boot refuses a basis with no valid categories |
| `WPM_EQUALITY_FLOOR` | no | `10` | The smallest group an aggregate may show (at least 5) |
| `WPM_PAY_INFORMATION_MONTHS` | no | `2` | Calendar months HR has to answer a worker's pay information request, 1 to 12 (the Directive's period is two months, WPM-R145) |
| `WPM_REQUIRE_PAY_RANGE` | no | on | A vacancy must state its pay range (a minimum and a maximum) before it is opened. **On by default** (Article 5 of the Pay Transparency Directive); turn it off (`0`, `false`, `no`, `off`) only where the Directive does not apply |
| `WPM_AI_ACT_APPLIES_FROM` | no | `2027-12-02` | The date the AI Act's Annex III duties apply to a deployer (`YYYY-MM-DD`); shown on the AI register. The date moved once already, so it is configuration, not code |
| `WPM_MCP_ALLOWED_ORIGINS` | no | none | Comma-separated `Origin` values that may call `POST /mcp` from a browser. An agent sends no `Origin` and is unaffected; any other `Origin` is refused (DNS-rebinding protection) |
| `WPM_PAY_GAP_SEX_CATEGORY` | no | `sex` | The equality category the pay gap report reads as female or male (values `female` and `male`; any other value counts as left out). It must be one of `WPM_EQUALITY_CATEGORIES`, and the report needs equality monitoring on (WPM-R144) |
| `WPM_HEALTH_REQUIREMENTS_BASIS` | no | off | **Workplace health requirements are off unless this is set** to the lawful basis you rely on, in your words. It is health data: do your own impact assessment and agree the role with your occupational health provider first. Only a token with the attribute `occupational_health=true` can record a status |
| `WPM_HEALTH_FLOOR` | no | `10` | The smallest group the compliance counts may show (at least 5) |
| `WPM_REQUEST_TIMEOUT_MS` | no | `30000` | A request running longer is answered `408` |
| `WPM_CORS_ORIGIN` | yes | none | The one browser origin allowed to call the API directly. Boot fails if unset |
| `JWT_SECRET` | yes | none | Required by the framework's config; unused by WPM's own sign-in |
| `WPM_PASETO_KEYS_URL` or `WPM_PASETO_KEYS` | one key source | none | The token keys (PASETO backend). Or `WPM_KEYCLOAK_JWKS_URL` (OIDC backend, see [entra-sign-in.md](entra-sign-in.md)) |
| `WPM_TOKEN_ISSUER`, `WPM_TOKEN_AUDIENCE` | no | `authentication-service`, `main-x-service` | Expected `iss` and `aud` (PASETO) |
| `WPM_KEYCLOAK_ISSUER`, `WPM_KEYCLOAK_AUDIENCE` | with the OIDC backend | none | Expected `iss` and `aud` |
| `WPM_ABAC_POLICY_FILE` or `WPM_ABAC_POLICY` | recommended | built-in default | The authorization policy; start from `config/abac-policy.reference.json` (hot-reloads) |
| `PORT`, `HOST` | no | `5150`, `http://localhost` | Listen port; the host used in links |
| `DB_MIN_CONNECTIONS`, `DB_MAX_CONNECTIONS`, `DB_CONNECT_TIMEOUT`, `DB_IDLE_TIMEOUT` | no | `2`, `20`, `500`, `500` | Pool sizes and timeouts |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` | no | `localhost`, `587`, empty | Mail (the service sends in-app notifications; mail is the framework's) |

## Behaviour switches

| Variable | Default | Effect |
| --- | --- | --- |
| `WPM_REQUIRE_AUTH` | on | Only an explicit `0`/`false`/`no`/`off` disables sign-in; production refuses that |
| `WPM_RATE_LIMIT_PER_MINUTE` | 600 | Requests per caller per minute (`0` disables) |
| `WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE` | 20 | Erase, sweep, import, merge, token and export routes |
| `WPM_TRUST_FORWARDED` | off | Name callers by the first `X-Forwarded-For` hop. Set only behind a proxy that **overwrites** that header |
| `WPM_MAX_TOKEN_LIFETIME_SECONDS` | `86400` | The longest token lifetime accepted (`exp` minus `iat`); a longer one, or one with no issue time, is `401`. `0` or `off` removes the limit. Tighten it to what your identity provider issues, see [Token age and revocation](../auth.md#token-age-and-revocation-wpm-r72-wpm-t200) |
| `WPM_MAX_ORG_WORKERS` | `200000` | The most employed workers the plan forecast and cost views will read for one request (the workers of the departments the plan names); over it they answer `422`. The default is the largest size the load test ran with no failure; raise it only after running `scripts/load-test.sh` on your hardware (see `BENCHMARKS.md`) |
| `WPM_LMS_COURSE_URL` | unset | A link template to a course in your learning management system, e.g. `https://lms.example.org/course/{course}`. `{course}` is replaced by the id of the course URN. `https` only, one placeholder, a real host with no user-info; anything else is ignored with a warning in the log. Unset means no links. See [skills-and-training](../skills-and-training.md) (WPM-R138, WPM-D91) |
| `WPM_HSTS` | off | Send HSTS even when the request is not seen as TLS |
| `WPM_RETENTION_DAYS` | unset | Legacy: one horizon for every record kind without its own override |
| `WPM_RETENTION_<KIND>_DAYS` | per kind | Retention per record kind; see [retention-schedule](../governance/retention-schedule.md) |
| `WPM_EVENT_TRANSPORT` | `memory` | Event transport |
| `WPM_UPSTREAM_MODE` | `stub` | Upstream identity lookups: `stub` or `http` |

The UI reads `WPM_API_URL`, `AUTH_API_URL` and `WPM_PUBLIC_URL`.

## Mail in the demo

`compose.yaml` runs a Stalwart mail server (`deploy/stalwart`, WPM-D81) and points the API at it with `SMTP_HOST=stalwart` and
`SMTP_PORT=25`. It serves one domain, `wpm.test`; every message to it lands in `inbox@wpm.test`; a message to any other domain is
refused, so nothing leaves the machine. From the host: SMTP on `127.0.0.1:1025` (no login), the account page, management and JMAP
on `http://127.0.0.1:8025` (`inbox@wpm.test` and `wpm-demo-mail`; the administrator is `admin` and `wpm-demo-admin`).
`python3 scripts/demo-mailbox.py --send` sends a test message and lists the inbox. **These are demo constants: do not reuse them,
and do not publish the ports.** For a real deployment set `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER` and `SMTP_PASSWORD` to your own mail
service (the table above).

## Proxy and TLS

Terminate TLS at a proxy. It must send `X-Forwarded-Proto: https` (for HSTS), and if you
set `WPM_TRUST_FORWARDED` it must overwrite `X-Forwarded-For` with the real client
address, or callers can pick their own rate-limit bucket. Without `WPM_TRUST_FORWARDED`,
every caller behind the proxy shares one bucket, which is safe but coarse.

## Running more than one instance (WPM-T198)

Every instance counts rate limits **on its own**, in memory. Behind N instances a caller can make up to N times the
configured limit, and the sensitive limit (erase, sweep, import, merge, token and export routes) is N times as loose, because
the load balancer spreads one caller's requests over all of them. WPM has no shared counter, and adding one (Redis) would make
the rate limiter a dependency whose outage blocks every request, so that is left to the deployer. Do one of these:

1. **Limit at the proxy or gateway** (recommended): enforce the per-client limit where all traffic meets, for example nginx `limit_req`,
   an ingress controller's rate-limit annotations, or your cloud's WAF. Keep WPM's own limits as a second, looser layer.
2. **Divide the limits by the number of instances**: with 3 instances set `WPM_RATE_LIMIT_PER_MINUTE=200` and
   `WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE=7`. This is right only when the balancer spreads one caller's requests evenly, and it is
   wrong while instances are being added or removed.
3. **Pin a caller to an instance** (sticky routing by client address): then each caller meets one counter again. It costs even load
   and does not survive an instance restart.

Whatever you choose, name callers correctly: set `WPM_TRUST_FORWARDED` only when the proxy overwrites `X-Forwarded-For` (above), or
every caller shares the proxy's bucket. The sensitive limit is the one that matters most, since it protects irreversible routes.
What does **not** need sharing: the cache (use `REDIS_URL`, WPM-D78), tokens (stateless), and the policy (each instance reloads its file).

## Database roles (WPM-R123, WPM-T199)

The service should not connect as a role that can change the schema. [`postgres-roles/roles.sql`](../../workforce-planning-management-service-with-rust/postgres-roles/roles.sql)
makes two:

| Role | Used by | May |
| --- | --- | --- |
| `wpm_migrator` | `db migrate`, as a deliberate step | create, alter and drop what WPM owns |
| `wpm_app` | the running service (`DATABASE_URL`) | select, insert, update and delete rows, and use sequences. **Not** create, alter, drop or truncate a table, create a role, or update, delete or truncate `audit_logs` |

```sh
psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 \
  -v migrator_password="$WPM_MIGRATOR_PASSWORD" -v runtime_password="$WPM_APP_PASSWORD" \
  -f postgres-roles/roles.sql                      # once, before the first migration
DATABASE_URL=postgres://wpm_migrator:...@db/wpm workforce-planning-management db migrate
psql ... -f postgres-roles/roles.sql               # again after every migration: it grants on what was just created
# start the service with DATABASE_URL=postgres://wpm_app:...@db/wpm (production has auto_migrate: false)
```

The audit trail is already append-only by trigger for every role; removing the privilege as well means a stolen runtime credential is
refused by the privilege check before the trigger, and cannot switch the trigger off (that needs ownership). Create the extensions
(`citext`, `unaccent`, `pg_trgm`, `pgcrypto`) as an administrator before the first migration. `scripts/verify-db-roles.sh` proves the
set: it migrates as `wpm_migrator`, shows `wpm_app` is refused each schema and audit change, and runs the service's request tests as
`wpm_app` (CI runs it as the `api-roles` job). It does not cover your backup and restore role, which needs its own grants.

## Probes

`GET /_health` and `GET /_ping` (liveness), `GET /_readiness`, and `GET /_posture`
(`{"auth_enforced": true|false}`: alert if it is false in production). `GET /metrics.prom`
needs a bearer token; give the scraper a service token.

## Logs and the audit feed (WPM-R122, WPM-T197)

Each request writes one line (JSON in production): `request_id`, `method`, `route` (`/api/workers/{}`), `status`, `latency_ms`. It holds
no query string, header, body or caller, so the log can go to a shared system without a personal-data review of its own. The response's
`X-Request-Id` is the same id: ask a user for it when they report a problem, and search for it. Send your proxy's request id in
`X-Request-Id` (plain characters only) to carry one id through both.

For a security monitoring system, poll `GET /api/audits/export?after=<last chain_seq>` with a privileged service token. It returns JSON
Lines, oldest first, resumable, with the chain hashes and no snapshots. Keep the last `X-Next-After` you were given. The export is rate
limited as a sensitive route; poll every minute or so, not every second.

## Scheduled tasks

Nothing runs these by itself; schedule each with cron, a timer or a Kubernetes CronJob.

| Task | When | Why |
| --- | --- | --- |
| `task snapshot_headcount` | daily | Headcount history cannot be backfilled |
| `task engagement_end_reminders` | daily | Tells the manager and HR about an engagement ending within 60 calendar days (`days_ahead:N`), or ended with no decision; once per end date |
| `task rota_reminders` | daily | Tells whoever's on-call turn starts tomorrow |
| `task pay_progression_reminders` | daily | Tells who becomes eligible for a pay step within 30 calendar days |
| `POST /api/retention/sweep` | daily or weekly | Hard-deletes rows past their kind's retention horizon (an admin action) |
| `scripts/backup.sh` | nightly or more often | [backup-and-restore.md](backup-and-restore.md) |
| `scripts/export-erasure-ledger.sh` | every few minutes | Keeps erasures safe across a restore |

## First start

```sh
cargo run -- db migrate     # or let the demo config migrate on start
cargo run -- task seed      # demo only: a synthetic organization
```

Production does not auto-migrate (`auto_migrate: false`): run `db migrate` as a deliberate
step, after a backup ([upgrade-and-migration.md](upgrade-and-migration.md)).
