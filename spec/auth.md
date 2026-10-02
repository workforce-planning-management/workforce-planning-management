# Authentication & authorization

The family stack unchanged
([authentication-sessions](../../agents/share/authentication-sessions.md),
[authorization-attributes](../../agents/share/authorization-attributes.md)):
cookie sessions + BFF for humans, offline PASETO v4.public for
services, blanket guard `WPM_REQUIRE_AUTH` (default **off** — the
family activation gate; any real deployment MUST activate before
exposure, and HR data makes that non-negotiable), shared ABAC engine.

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

Every read of a highest/high-tier record is **audited** (see
[audit.md](audit.md)).

## Activation runbook (WPM-G1)

The shipped default is **wide open** (family posture,
`agents/share/security.md` §4). Activation is a release gate, not a
config tweak:

1. **Mount a policy.** Start from the shipped reference,
   [`config/abac-policy.reference.json`](../workforce-planning-management-api-with-rust/config/abac-policy.reference.json)
   — svc/admin do everything, `payroll=true` reads unmasked,
   `hr=true` writes and reads **masked** (salary stays payroll+self),
   `resource.person = $sub` reads the own record unmasked, and every
   other authenticated caller gets the masked-read fallback. Point
   `WPM_ABAC_POLICY_FILE` at your copy (it hot-reloads on change) or
   inline it via `WPM_ABAC_POLICY`.
2. **Point at the keys.** `WPM_PASETO_KEYS_URL` (boot-fetched +
   refreshed) or `WPM_PASETO_KEYS`; set `WPM_TOKEN_ISSUER` /
   `WPM_TOKEN_AUDIENCE` if they differ from the defaults.
3. **Flip the flag.** `WPM_REQUIRE_AUTH=1` (read once at boot —
   restart to change).
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
([authentication-sessions](../../agents/share/authentication-sessions.md)),
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
`workforce-planning-management-api-with-rust/Cargo.toml`'s
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
   | `organization_ref` | a custom user/group attribute holding the org's URN (see [`memberships.rs`](../workforce-planning-management-api-with-rust/src/models/memberships.rs) for the multi-organization membership model this feeds) | attribute mapper ⇒ `attrs.organization_ref = ["organization:<uuid>", …]`, multi-valued for a person in several organizations |

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
