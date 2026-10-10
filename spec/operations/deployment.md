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

The compose file starts PostgreSQL and the API. **The UI is not containerized**: it has
only `adapter-auto`, so a deployer picks an adapter for their platform (for example
`adapter-node`) and runs it with `WPM_API_URL` pointing at the API. The image build and
the compose file are written but were not built where this was written; try them first.

## Required settings (production)

| Variable | Required | Default | What it does |
| --- | --- | --- | --- |
| `DATABASE_URL` | yes | none | PostgreSQL 18 connection string. **In production it must require TLS across a network**: add `?sslmode=verify-full` (best: the server's certificate is checked) or `require` or `verify-ca`; boot refuses a plaintext connection to another host. A loopback address or a Unix socket needs no TLS |
| `WPM_ALLOW_PLAINTEXT_DATABASE` | no | off | Set to `1` only if the network path to the database is protected another way (a private link, a tunnel) and you accept the risk; boot then logs a warning every start |
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
| `WPM_HSTS` | off | Send HSTS even when the request is not seen as TLS |
| `WPM_RETENTION_DAYS` | unset | Legacy: one horizon for every record kind without its own override |
| `WPM_RETENTION_<KIND>_DAYS` | per kind | Retention per record kind; see [retention-schedule](../governance/retention-schedule.md) |
| `WPM_EVENT_TRANSPORT` | `memory` | Event transport |
| `WPM_UPSTREAM_MODE` | `stub` | Upstream identity lookups: `stub` or `http` |

The UI reads `WPM_API_URL`, `AUTH_API_URL` and `WPM_PUBLIC_URL`.

## Proxy and TLS

Terminate TLS at a proxy. It must send `X-Forwarded-Proto: https` (for HSTS), and if you
set `WPM_TRUST_FORWARDED` it must overwrite `X-Forwarded-For` with the real client
address, or callers can pick their own rate-limit bucket. Without `WPM_TRUST_FORWARDED`,
every caller behind the proxy shares one bucket, which is safe but coarse.

## Probes

`GET /_health` and `GET /_ping` (liveness), `GET /_readiness`, and `GET /_posture`
(`{"auth_enforced": true|false}`: alert if it is false in production). `GET /metrics.prom`
needs a bearer token; give the scraper a service token.

## Scheduled tasks

Nothing runs these by itself; schedule each with cron, a timer or a Kubernetes CronJob.

| Task | When | Why |
| --- | --- | --- |
| `task snapshot_headcount` | daily | Headcount history cannot be backfilled |
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
