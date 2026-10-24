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

### WPM-R151 — Outbound webhooks

- **What.** Subscribers register an endpoint and the event kinds they want (hire, movement, leave approved, last day). Each delivery
  is **signed** with an HMAC of the body under a per-subscriber secret kept by reference, **retried with back-off**, and listed in a
  **delivery log that holds no payload body** (status, attempt, time).
- **Payload.** URNs and event kinds only (`worker:<pid>`, `leave_approved`), **never a personal field** (WPM-D106). A subscriber that
  needs more reads it through the API under its own access. A test fails if a payload field outside the allow-list is added.

### Decisions

- **WPM-D101 — An agent sees what the caller sees.** The MCP server is an adapter over existing endpoints under the caller's token.
  No tool exists without an endpoint, and there is no service account.
- **WPM-D102 — Provisioning sends employment facts only.** SCIM carries employee number, manager, department, title and active
  state; contact details stay with the person service (WPM-D23).
- **WPM-D106 — Events leave as references.** Webhook and SCIM payloads carry URNs and kinds; a subscriber that needs more reads it
  through the API with its own access.

