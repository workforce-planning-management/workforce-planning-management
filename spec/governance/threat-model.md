# Threat model (WPM-T193, assessed 2026-10-11)

A starting threat model for a deployer's security team to adapt, not a security review. It lists what could go
wrong at each boundary, the control that answers it **as built**, and where there is a gap. Every control
named exists and was exercised in the session that wrote this; a control the author did not check is marked
**not assessed**. Written alongside [the readiness assessment](../readiness-assessment.md), whose verdict is
that the software is not ready for production use in a high-assurance organization.

> ⚠️ **Demo software.** A deployer completes their own model for their environment, and has it tested by
> people who did not write the software (WPM-T201).

## 1. What is protected

| Asset | Why it matters | Where it is |
| --- | --- | --- |
| Employment and pay records | Personal data; pay is sensitive | PostgreSQL |
| Health-adjacent data: leave kinds, adjustments, ergonomics, wellbeing acknowledgements | Special-category data under UK and EU law when it reveals health | PostgreSQL |
| Applicants' details | Personal data of people who are not employees | PostgreSQL |
| The audit trail | Evidence of who did what; the basis of accountability | PostgreSQL, hash-chained |
| Token keys, database credentials, backup keys | Anyone holding them acts as the service or reads everything | The deployer's secret store; environment |
| Backups | A complete copy of everything | Off-host storage |
| The software supply chain | A change here reaches every deployment | Repository, CI, registry |

## 2. Actors

A signed-in employee; a line manager; HR; payroll; a service peer; an administrator; an **insider with
database access**; an **outsider on the network**; an applicant; the identity provider; a contributor or a
compromised dependency; the operator of the host.

## 3. Trust boundaries

1. **Browser ↔ interface server ↔ API.** Callers present a bearer token the API verifies; the interface adds
   security headers and a content security policy.
2. **API ↔ identity provider.** The API fetches signing keys (short timeout, no redirects) and verifies tokens;
   it never holds a client secret or an authorization code (WPM-D62).
3. **API ↔ database.** One connection pool. In production the connection must use TLS across a network
   (WPM-R116).
4. **Database ↔ backups.** Encrypted at write (WPM-R117).
5. **Repository ↔ CI ↔ registry ↔ deployment.**

## 4. Threats, controls and gaps

Categories are STRIDE: spoofing, tampering, repudiation, information disclosure, denial of service,
elevation of privilege.

| # | Cat. | Threat | Control as built | Gap / status |
| --- | --- | --- | --- | --- |
| T1 | S | A request without a valid identity | Sign-in on by default; production refuses to start with it off or without a key source; a bad, expired or tampered token is `401` (WPM-R72) | Token lifetime and revocation are the identity provider's and undocumented here (WPM-T200) |
| T2 | S | A forged token | Signature, issuer, audience and expiry checked; keys refreshed; tested with forged, expired and tampered tokens | Algorithm confusion and key-rotation races: **not assessed** |
| T3 | I | A signed-in person reads records they have no need for | The need-to-know guard classifies every `GET` and a test fails on an unclassified route (WPM-R114, D71); record-level checks where the controller decides; masking of pay | **Reads only.** A route classified `Open` or `ControllerDecides` is as safe as that classification; the table is reviewed by people, not proven |
| T4 | I | A manager reads a report's most personal records | Worker records are split into in-scope, with-managers and self-only; the manager is excluded from the last | Depends on `manager_pid` being right; a wrong reporting line widens access |
| T5 | I | A diagnosis or reason is stored where many can read it | Sick-leave requests refuse a reason (WPM-D73); adjustments and ergonomics record the barrier and the change, never the cause | Free-text fields elsewhere (time-entry notes, adjustment words) can still hold health detail; the service cannot tell |
| T6 | E | A signed-in person writes what only HR may | The policy gates every write by action; destructive operations need `access=admin` | Per-record write rules are in controllers; self-service writes are proposed with an explicit allow-list (WPM-R130) |
| T7 | E | An insider with a SQL prompt rewrites or removes an audit entry | `UPDATE` and `DELETE` on `audit_logs` are refused by the database; entries are hash-chained and verifiable (WPM-R115) | A superuser can disable the trigger. Cutting the **tail** is invisible without a head hash recorded elsewhere. `TRUNCATE` is not blocked |
| T8 | R | A person denies an action | Every mutation and sensitive read writes an audit row naming the actor, in the same transaction | An actor is the token's subject; a shared account defeats it. Some reads of lower-tier data are not audited |
| T9 | T | Data changed in transit to the database | TLS is required across a network in production; `verify-full` recommended (WPM-R116) | `require` encrypts without verifying the server; the override exists and is logged |
| T10 | I | A stolen backup | Backups are encrypted as written, with the plaintext never on disk (WPM-R117) | The key's custody is the deployer's. The cipher is not authenticated encryption, which the tag compensates for. Backups made before this change are plaintext |
| T11 | I | An erased person returns from a backup | The erasure ledger is replayed after a restore (WPM-D63) | Backups older than the retention are deleted by the script, not by the platform |
| T12 | T | Injection into the database | The data layer uses bound parameters. The erasure statements are built with `format!` from a typed UUID, not from request text | The raw statements were read, not fuzzed; **not assessed** with a scanner |
| T13 | T | Cross-site scripting | A restrictive content security policy, nonces, and the framework's escaping; strings come from a catalogue | A scanner baseline is written in CI and **has not run**; the content manager's page loads an unpinned script (a known gap in WPM-R73) |
| T14 | S | Cross-site request forgery | The API uses bearer tokens, not cookies, so a forged cross-site request carries no credential | How the interface server stores and forwards the token: **not assessed** |
| T15 | I | Server-side request forgery through fetched keys | The few outbound fetches use short timeouts and no redirects | The fetched URL is the operator's configuration; **not assessed** beyond that |
| T16 | D | A flood of requests | Rate limits per network address, a 2 MB body limit, a request timeout (WPM-R74, R116) | Limits are per instance; the views that load an organization's records have no cap and no load evidence (WPM-T196) |
| T17 | D | A slow request holds a connection | The production configuration times requests out | A request that hits the limit was not exercised in a test |
| T18 | D | Audit writes serialise and deadlock under load | Chosen for correctness (WPM-D72) and documented | Throughput not measured |
| T19 | I | Personal data in logs | Logs are JSON; the limiter keeps no address; a request identifier is on every response | A test that logs hold no query string or personal data does not exist (WPM-T197) |
| T20 | E | A malicious or compromised dependency | `cargo deny`, vendored shared crates, a software bill of materials on release, Dependabot | Actions are pinned by tag, not commit; no secret scanning; no image signature or provenance (WPM-T194, T195) |
| T21 | E | A malicious change merged | A single maintainer with machine gates; the AI statement says how AI is used | No second reviewer by design; branch protection is a setting not yet applied |
| T22 | I | Equality or union data misused | The service stores neither today; if added they are bounded and off by default (WPM-D68, D74) | Proposed, not built |
| T23 | I | A score used to discriminate | Scores advise; an equality impact register exists | The register's assessments are unapproved drafts; the scoring register is not in code (WPM-T166) |
| T24 | E | An insider abuses a privileged attribute | Privilege is an attribute or an `hr_admin`/`payroll_admin` role, reads of pay are audited | No break-glass procedure, no periodic access review, no separation of duties between granting and using |

## 5. What this model leaves out

Physical and host security, network segmentation and the reverse proxy; the identity provider's own security;
the interface's session handling; denial of service at the network layer; and any threat specific to a
deployer's sector. It also leaves out the people threats a technical model cannot reach: training, joiner and
leaver discipline, and the care taken with exports.

## 6. Using it

Add a row for every new route class, table and integration. A threat with no control is a task, not a footnote.
Review it when the readiness assessment changes, and before any production use.
