# Integrations

WPM is a consumer of the Main X Index family; it holds **EntityRef
URNs** and never duplicates upstream records.

| Service | Used for | How |
|---|---|---|
| person-service | the human | `person:<pid>` on Employee and (optionally) Candidate; display names resolved best-effort |
| worker-service | professional identity | `worker:<pid>` on Employee; interviewers |
| organization-service | the employer | `organization:<pid>` on Employee, PayrollRun |
| course-service | training | `course:` / `courseinstance:` URNs on TrainingEnrollment |
| authentication-service | SSO + ABAC attrs | offline PASETO via `authentication-verifier`; persona attributes |

Client modules follow the stub-first pattern (patient-flow
`clients.rs`): display-name lookups are read-only, cached,
best-effort, and never block writes. The wellbeing engine adds a
second best-effort person lookup — the **birth date** (for age-banded
entitlement rules, WPM-R25): cached, used only to derive a whole-year
age, **never stored** in a WPM table; an unknown birth date honestly
fails an age-banded rule rather than guessing.

## The `employed_by` edge

The registry's cross-service link `employed_by` (worker →
organization, with `role` and validity dates) is the identity-level
assertion the family's link-graph aggregates. WPM's Employee record
is the operational layer above it. **Roadmap**: on hire/termination
WPM emits the corresponding `entity_links` write (or a `linked` /
`unlinked` event) so the registry edge tracks employment facts
automatically — until then the two layers are maintained
independently, and this file is the record of that gap.

## Events

Family envelope via the `WPM_EVENT_TRANSPORT` seam (default
`memory`; Postgres outbox rows share the mutation's transaction under
`outbox`). Event kinds are listed in [audit.md](audit.md).

## Proposed additions (WPM-R149–R151; spec round WPM-T268, 2026-10-24)

Three ways for other systems to read WPM or be told by it. None adds a read or a field the API does not already have, and none
carries a personal field outward (WPM-D101, WPM-D102, WPM-D106). Delivery: WPM-T275 to WPM-T277.

### WPM-R149 — A read-only MCP server

- **What.** A [Model Context Protocol](https://modelcontextprotocol.io/) server that exposes existing reads as tools (my details,
  my time off, plan totals, aggregates). **Every tool maps to a documented route** of this API, and a test fails if one does not
  (WPM-T275); there is no tool without an endpoint and no write tool.
- **Whose access.** The tool call is made **under the caller's own token**, passed to the API, so the API's policy decides, not the
  server: a caller without access is refused exactly as on the endpoint, and there is **no service account** with wider rights
  (WPM-D101). Each call is audited with the tool name.
- **Transport and authorization.** MCP over HTTP is an OAuth 2.1 resource server: the MCP spec made servers resource servers in its
  2025-06-18 revision and added OpenID Connect discovery in 2025-11-25, and the authorization section was reported revised again in
  2026 (unconfirmed). WPM-T275 **pins one dated revision** of the MCP specification at build time and says which; a later revision is
  a change, not a surprise.
- **Not done.** No tool that returns what the API would mask; no cross-caller cache (a result is for one token).

**Built (WPM-T275).** `POST /mcp`, Streamable HTTP of the pinned revision **2025-11-25** (the tools and transports pages were read
on 2026-10-24): JSON replies, no session, no server-sent events (`GET` and `DELETE` answer 405), a notification answers 202, an
`MCP-Protocol-Version` other than the pinned one answers 400, and an `Origin` header is refused (403) unless listed in
`WPM_MCP_ALLOWED_ORIGINS` (an agent sends none). Methods: `initialize`, `ping`, `tools/list`, `tools/call`; the only capability is
`tools`. Ten tools in `rules::mcp::TOOLS`, each one `GET` route (my contact details, my time off, plans and a plan's forecast, the
workforce overview, metrics, headcount history, the human capital report, the AI register). A call goes **in process through the
API's own router** with the caller's `Authorization` header and the same sign-in and need-to-know layers, so the API's refusal is the
tool's (a tool error naming the status), and a value that is not a date, a UUID, a URN or a number is refused before any request
is made, so no argument can steer a call to another route. Each call writes an audit row with the tool name before it is made; if
that cannot be written the call is not made (WPM-D107). Tests: every tool is a documented `GET` of the OpenAPI document and is
classified in the need-to-know table; no tool name reads like a write; an enforcement test with real tokens (anonymous 401, a worker
refused the report by the API itself, HR given it, a worker given the AI register, a path-traversal argument refused).
**Not built:** OAuth protected-resource metadata and token audience binding (the endpoint takes the deployment's existing bearer
token, as every route does); resources, prompts, sampling and pagination; `outputSchema` for the tools; a rate limit of its own
beyond the service's (a tool call is one request to the endpoint, not counted again inside).

### WPM-R150 — SCIM 2.0 outbound provisioning

- **What.** Joiners, movers and leavers are pushed as SCIM 2.0 `User` resources (RFC 7643 and RFC 7644) with the **Enterprise User
  extension** (`employeeNumber`, `manager`, `department`, and `costCenter` where known) to an identity provider that consumes SCIM
  (Entra ID and Okta do). The push is a task over the event outbox (below), retried, and idempotent.
- **Employment attributes only.** The payload has employee number, display name as the person service resolves it, title,
  department, manager, active state and dates. **No contact detail leaves** (WPM-D23, WPM-D102): a test fails if a contact field is
  reachable from the mapping. The leaver's **deprovision date is the last day** from the handover ([joiners-and-leavers.md](joiners-and-leavers.md)), not
  the date WPM learned of it.
- **Testing.** Against a stub SCIM server. Whether Keycloak can be a SCIM target (it has no SCIM endpoint in its core as far as
  this round found; third-party extensions exist) is **unconfirmed**; WPM-T276 says what it tested.

**Built (WPM-T276).** The task `scim_push` (`rules::scim` is the pure mapping, `tasks/scim_push.rs` the push). **It reconciles
rather than replays the outbox**: for each worker it works out the `User` that should exist today, compares a SHA-256 fingerprint with
what was last sent (`scim_pushes`: the provider's id, the fingerprint, the retry state, no body), and creates (`POST /Users`) or
replaces (`PUT /Users/{id}`) only when it differs. That is idempotent, mends itself after an outage and does not depend on
`WPM_EVENT_TRANSPORT` (the outbox exists under one transport only). A `409` is resolved by finding the provider's user by
`externalId` and replacing it. A failure is recorded as `HTTP <status>` (never the provider's body) and retried with a back-off of
one minute doubling to a day. Managers are sent first so the Enterprise `manager` carries the provider's own id. Mapping: `userName`
and `employeeNumber` are the employee number (WPM holds no login), `externalId` is `worker:<pid>`, plus display name, title,
`department` and `active`. **No `costCenter`** (WPM holds no cost centre) and **no `name` parts, emails, phone numbers or addresses**.
`active` follows the headcount definition: not before the first day, and not on or after the termination date. A deleted
record is deactivated if it was ever sent. Settings: `WPM_SCIM_BASE_URL` (https, or http to this machine), `WPM_SCIM_TOKEN`,
`WPM_SCIM_ORGANIZATION`. Erasure deletes the `scim_pushes` row and the subject export lists it. **Not built:** a
schedule (run `cargo loco task scim_push` from the deployment's scheduler); a deprovision on the handover's last day (WPM uses
the termination date, by the headcount definition, so a leaver is deactivated from that date, which may be a day before the last
day worked); `PATCH`; groups; the token held by reference (it is read from the environment); a test against Keycloak (it has no SCIM
endpoint in its core as far as this round found) or any real identity provider: **only a stub server was used**.

### WPM-R151 — Outbound webhooks

- **What.** Subscribers register an endpoint and the event kinds they want (hire, movement, leave approved, last day). Each delivery
  is **signed** with an HMAC of the body under a per-subscriber secret kept by reference, **retried with back-off**, and listed in a
  **delivery log that holds no payload body** (status, attempt, time).
- **Payload.** URNs and event kinds only (`worker:<pid>`, `leave_approved`), **never a personal field** (WPM-D106). A subscriber that
  needs more reads it through the API under its own access. A test fails if a payload field outside the allow-list is added.

**Built (WPM-T277).** Administrators (`access=admin`; HR is refused) manage `POST/GET /api/webhook-subscriptions`,
`PUT/DELETE /api/webhook-subscriptions/{pid}` and read `GET .../{pid}/deliveries`. A subscription has an endpoint, kinds from the
offered list (`worker_hired`, `worker_activated`, `worker_terminated`, `worker_retired`, `leave_approved`), a **secret reference**
(the name of an environment variable, `WPM_WEBHOOK_SECRET_<NAME>`, so it cannot point at any other setting) and an on/off flag.
The endpoint is `https` to a host that is not an internal address literal, or `http` to this machine, with no credentials, query or
fragment. **Deliveries are queued in the transaction of the change** (`streaming::emit_on` writes one row per matching active
subscription), so a change that rolls back tells nobody. The task `webhook_deliver` sends them: the payload is `{id, kind, subject,
occurred_at}` where `subject` is the URN (`worker:<pid>`, `leave_request:<pid>`), signed `X-WPM-Signature: t=<unix>,v1=<HMAC-SHA256
of "<t>.<body>">` with the timestamp inside the signature, no redirects followed, a 10 second timeout. 2xx is delivered; a network
failure, `408`, `429` and `5xx` are retried with a back-off of one minute doubling to a day, ten tries in all; any other answer is a
refusal and is not repeated. The log keeps the HTTP status and a short reason; **there is no column for a body**. Erasure deletes the
deliveries that name the worker. Settings: the secret variables only. **Not built:** a schedule (run `cargo loco task
webhook_deliver` every minute or so); the other events in the spec's list ("movement": WPM emits no movement event on the
transport); DNS-rebinding protection (a host name that later resolves to an internal address is not caught, only address literals
are); the secret rotated without a gap; a screen; a replay of a failed delivery.

### Decisions

- **WPM-D101 — An agent sees what the caller sees.** The MCP server is an adapter over existing endpoints under the caller's token.
  No tool exists without an endpoint, and there is no service account.
- **WPM-D102 — Provisioning sends employment facts only.** SCIM carries employee number, manager, department, title and active
  state; contact details stay with the person service (WPM-D23).
- **WPM-D106 — Events leave as references.** Webhook and SCIM payloads carry URNs and kinds; a subscriber that needs more reads it
  through the API with its own access.

