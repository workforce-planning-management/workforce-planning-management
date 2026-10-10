# Authentication & authorization

The family stack unchanged
(authentication-sessions,
authorization-attributes):
cookie sessions + BFF for humans, offline PASETO v4.public for
services, blanket guard `WPM_REQUIRE_AUTH` (default **on** since WPM-D52:
only an explicit `0`/`false`/`no`/`off` disables it, which is logged and
refused in the `production` environment, as is starting with no token key
source), shared ABAC engine.

## Personas as policy (not code)

One API; four typical policy personas expressed over `attrs`:

| Persona | Typical rules |
|---|---|
| **employee** | read own record/payslips/reviews (`resource.person = $sub` ownership template); write own leave requests |
| **manager** | read team records (salary masked), approve team leave, run team reviews — scoped by `resource.manager` / department |
| **hr** | full employee lifecycle in their remit (`resource.department`), reviews, succession |
| **payroll** | payroll runs + payslips + salary fields |

## Record-level attributes

Handlers derive per-record attrs for the second ABAC pass:
`resource.department`, `resource.person` (the employee's person URN,
enabling `$sub`-style self rules), `resource.status`. The `mask`
obligation redacts **salary, payslip amounts, and review content**
while leaving employment facts visible — the corridor-screen posture
applied to HR.

## Sensitivity map

| Data | Tier |
|---|---|
| salary / payslips / benchmarks-vs-salary | highest — payroll persona + self |
| review content, succession, background-check outcomes | high — HR + involved parties |
| leave kinds (esp. sick/parental) | high — self + manager + HR |
| employment facts (title, department, dates) | medium |
| requisitions, shift plans | low |
| an expense claim (where someone has been and what they spent) | high — the claimant, their manager and HR see it; others get a 404; **the claimant never decides their own claim, even as HR**; no amount or text in the audit entry or notification (WPM-D42) |
| a worker's pay band and step (is a salary) | high — the person + HR only; the audit entry and the reminder name no band, step or amount (WPM-D41) |
| a worker's job level (seniority; tracks pay) | high — the person + HR only; the audit entry names no level (WPM-D40) |
| emergency contacts (third-party personal data) | high — the person + HR only; a manager cannot read them |
| skill aspirations | private unless shared (`manager` / `everyone`); never in workforce roll-ups |
| directory, backups, on-call, announcements (live) | low — anyone who can read the organization; the directory shows *away*, never why |
| scheduled / expired announcements; posting, editing | `hr_admin` / `org_admin` of that organization (when auth is on) |
| announcement read receipts | the reader's own; editors see a count only |
| joiner / leaver records, handover | HR and whoever may write the worker's record; scoped to organizations the caller can read |

Every read of a highest/high-tier record is **audited** (see
[audit.md](audit.md)).

## Need-to-know on reads (WPM-R114, WPM-D71)

The policy engine decides by *action*, and the reference policy lets every signed-in caller read (masking
only money). So a caller holding no attributes could read a colleague's sick-leave request with its
reason, their time-entry notes, every applicant's e-mail address and the audit trail. A guard now sits inside
the sign-in guard and decides each `GET` under `/api` from a table in code
([`rules/access.rs`](../workforce-planning-management-service-with-rust/src/rules/access.rs)):

| Audience | Who may read |
| --- | --- |
| **Open** | Any signed-in caller: reference data and aggregates (pay scales, job levels, role profiles, the directory, capacity views, workforce plans, …) |
| **Controller decides** | The controller loads the record and applies its own rule (expense claims, appraisals, emergency contacts, pay position, subject access, …) |
| **Privileged** | HR, payroll, service and administrator callers only: applicants, requisitions, payroll runs, the audit trail and event feed, succession and pipelines, the retention report, … |
| **Worker, in scope** | The worker, their line managers, privileged callers, and anyone in an organization the worker is in (reports, cover, on-call, groups, the worker's employment record) |
| **Worker, with managers** | The worker, their line managers **up the chain**, and privileged callers (leave, time, skills, training, development plans, reviews, onboarding, …) |
| **Worker, self only** | The worker and privileged callers; not the manager (adjustment requests, benefit enrolments, payslips, wellbeing prompts, notifications, assessment profile, mobility interests, ergonomic assessments, contractor details) |

**Privileged** means an attribute (`hr`, `payroll` or `svc` is `true`, or `access` is `admin`) **or** an
`hr_admin` / `payroll_admin` membership: in the worker's organization (or one that contains it) for a route
about a worker; in any organization for a route that is not about one (applicants have no organization). A
test fails when a `GET` route is added without a classification, and when a table entry matches no route.

Writes are unchanged: they need `hr`, `svc` or `admin` at the policy. The guard is a no-op while sign-in is
off, which production refuses. **Limits:** the line-manager rule follows `manager_pid`, so it is only as good as
that data; the guard does not mask fields inside an allowed response (a manager reading `leave-requests` sees
the kind of leave; sick leave no longer carries a reason, WPM-T190); and it covers reads only.

## Self-service writes under `/api/me` (WPM-R130, WPM-D76)

The policy lets only HR, service and administrator callers write, and it cannot say "a worker may change *their own*
record" because it sees an action and not the record. So a short list, `rules::self_service::WRITES`, names the
routes under `/api/me` that skip the policy's action check (the caller must still hold a valid token), and each
handler resolves the person from that token alone: there is no worker id in the path to change. Today the list is
`PUT` and `DELETE /api/me/contact-details`, `POST`, `PUT`, `DELETE` on `/api/me/emergency-contacts`, `POST /api/me/leave-requests` with its `…/{pid}/cancel`, `POST /api/me/resignation` with its `…/withdraw`, and `POST /api/me/flexible-working` with its `…/withdraw`, `…/respond` and `…/appeal`. Deciding a flexible working request is an ordinary write: under the reference policy only HR can; a deployer gives line managers the power to decide with a policy rule, and the service then limits them to their own reports. A worker still cannot approve their own leave. A unit test
scans the `/api/me` controller so a write route not on the list, or a list entry with no route, fails the build.
A second short list, `rules::self_service::ATTRIBUTE_WRITES`, names routes whose write passes the policy's action check for
a token carrying a stated attribute, where the policy cannot name the role; today only `PUT` and `DELETE
/api/workers/{pid}/health-requirements/{req_pid}` for `occupational_health=true`. The handler checks the attribute again,
so HR, the service and an administrator cannot record a health status (WPM-D75).

Everything else a plain worker tries to write is still `403`. A person with several worker records gets the earliest
one still employed.

## Activation runbook (WPM-G1)

Before WPM-D52 the shipped default was wide open (the family posture).
Enforcement is now **on by default**, so activation is giving the service
what it enforces *with*, not flipping a flag:

1. **Mount a policy.** Start from the shipped reference,
   [`config/abac-policy.reference.json`](../workforce-planning-management-service-with-rust/config/abac-policy.reference.json)
   — svc/admin do everything, `payroll=true` reads unmasked,
   `hr=true` writes and reads **masked** (salary stays payroll+self),
   `resource.person = $sub` reads the own record unmasked, and every
   other authenticated caller gets the masked-read fallback. Point
   `WPM_ABAC_POLICY_FILE` at your copy (it hot-reloads on change) or
   inline it via `WPM_ABAC_POLICY`.
2. **Point at the keys.** `WPM_PASETO_KEYS_URL` (boot-fetched +
   refreshed) or `WPM_PASETO_KEYS`; set `WPM_TOKEN_ISSUER` /
   `WPM_TOKEN_AUDIENCE` if they differ from the defaults.
3. **Leave the flag alone.** Enforcement is on by default; the flag
   `WPM_REQUIRE_AUTH` is read once at boot (restart to change). Do not set it
   to `0` outside local development: the service logs a warning every ten
   minutes while it is off, `GET /_posture` reports `auth_enforced: false`, and
   `LOCO_ENV=production` refuses to start.
4. **Verify.** `cargo test --test enforcement -- --ignored` runs the
   persona matrix against the reference policy shape: public paths
   open; 401 without a token; masked vs self vs payroll reads;
   destructive ops (`delete`, `/erase`, `/sweep`) admin-only; the
   subject-access export refused to masked callers; 360 report
   comments withheld from masked callers.

Known engine limits a deployment must plan around (stated, not
hidden): employee **self-service writes** need a coarse `write` allow
to pass the blanket guard (the guard evaluates without record
attributes), so grant them via a subject attribute (e.g.
`access=write` for staff) and rely on the record-level `$sub` checks
on ownership-enforcing handlers; department-scoped manager rules are
written per department (`{"manager": ["true"],
"resource.department": ["engineering"]}`) because subject-vs-resource
attribute equality has no template.

## Keycloak as the identity provider (WPM-G2)

By default WPM never talks to an identity provider itself — it
verifies PASETO tokens (`WPM_PASETO_KEYS_URL`/`WPM_PASETO_KEYS` above,
the `paseto` Cargo feature, on by default) minted by the sibling
**authentication service**
(authentication-sessions),
and every front-end (this one included) reaches that service only
through its own BFF server, per `AGENTS.md` §3 — no OIDC/Keycloak
library in this repo or the Rust service. Pointing that authentication
service at Keycloak as its upstream IdP is entirely its own
configuration; this section exists so a deployment doesn't have to
rediscover the claim mapping from scratch.

A second, independent integration point exists for a deployment that
wants this service to skip the sibling authentication service
entirely and verify a Keycloak-issued JWT itself: the `keycloak` Cargo
feature (mutually exclusive with `paseto` — see
`workforce-planning-management-service-with-rust/Cargo.toml`'s
`[features]` table and `src/auth/keycloak.rs`'s module docs for its
own `WPM_KEYCLOAK_*` environment). It uses the *same* claim-mapping
table below, so a deployment can switch which component talks to
Keycloak without re-deriving the realm-mapper configuration.

1. **Register a confidential client** in the Keycloak realm for the
   authentication service (not per front-end — WPM's front-ends only
   ever redirect to the authentication service's own
   `/api/auth/sso/start`, never to Keycloak directly). Redirect URI is
   the authentication service's own callback, not any WPM app's.
2. **Configure the authentication service's upstream IdP** via its own
   `AUTH_OIDC_*` environment:
   - `AUTH_OIDC_ISSUER_URL` — the realm issuer, e.g.
     `https://keycloak.example/realms/wpm`.
   - `AUTH_OIDC_CLIENT_ID` / `AUTH_OIDC_CLIENT_SECRET` — the
     confidential client from step 1.
   - `AUTH_OIDC_SCOPES` — `openid email profile`, plus whatever scope
     carries the realm/client roles and group path used below (Keycloak
     ships these on `email profile` by default; a custom scope is only
     needed for the organization mapper).
3. **Map Keycloak claims to the `attrs` this policy already reads**
   (client scope → *Mapper Type* in Keycloak's admin console; every
   mapper below is a **User Realm Role**, **User Client Role**, **Group
   Membership**, or **User Attribute** mapper writing into a named
   token claim, which the authentication service then copies verbatim
   into the PASETO's `attrs` map — it does not invent new attribute
   names, only relays Keycloak's):

   | WPM `attrs` key | Keycloak source | Mapper shape |
   |---|---|---|
   | `hr` | realm role `wpm-hr` | role present ⇒ `attrs.hr = ["true"]` |
   | `payroll` | realm role `wpm-payroll` | role present ⇒ `attrs.payroll = ["true"]` |
   | `svc` | realm role `wpm-svc` | role present ⇒ `attrs.svc = ["true"]` (granted only to a service-account client, never a human realm user) |
   | `access` | realm role `wpm-admin` | role present ⇒ `attrs.access = ["admin"]`; otherwise a plain authenticated human gets `attrs.access = ["write"]` (the self-service-write allow the engine limits above call for) |
   | `department` | group path, e.g. `/org/engineering` | group membership ⇒ the path's leaf segment, `attrs.department = ["engineering"]` (one row per department group the user is in) |
   | `organization_ref` | a custom user/group attribute holding the org's URN (see [`memberships.rs`](../workforce-planning-management-service-with-rust/src/models/memberships.rs) for the multi-organization membership model this feeds) | attribute mapper ⇒ `attrs.organization_ref = ["organization:<uuid>", …]`, multi-valued for a person in several organizations |

   `resource.person = $sub` self-rules need no mapper: `$sub` is
   Keycloak's own `sub` claim, relayed as-is.
4. **Verify** the same way as the activation runbook above: mint a
   token for a Keycloak test user with each role/group combination and
   run `cargo test --test enforcement -- --ignored` against it, plus a
   manual round trip through `/signin` → *Sign in with SSO* → Keycloak
   login → back at `/`, confirming the front-end's dashboard shows the
   organizations and role the group/attribute mappers above should
   produce.

**Not yet wired**: today only this front-end links to
`/signin/sso`; every sibling front-end in the family still shows only
the magic-link form and needs the same link added (`../signin/sso`
redirect route + the button, both trivial per-app copies of this
app's `src/routes/signin/+page.svelte` and
`src/routes/signin/sso/+server.ts`) — tracked as a follow-up, not
attempted here since those repos aren't part of this change.
