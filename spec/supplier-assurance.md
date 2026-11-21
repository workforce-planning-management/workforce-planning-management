# Supplier assurance: hosting and supporting WPM for a health body (NHS Wales as the worked example)

Proposed 2026-11-20 from security-audit preparation notes the owner supplied ([plan.md](plan.md) section Y). Requirements
**WPM-R267–R274** and **WPM-R277**, decisions **WPM-D214–D220**, tasks **WPM-T574–T595** and **WPM-T603** in [tasks.md](tasks.md). **Nothing here is built;
every decision is the owner's to confirm.**

> ⚠️ Demo software ([regulatory.md](regulatory.md), [the path to production](../locales/en-001/path-to-production/index.md)).
> NHS Wales is the **worked example**. The same shape fits any buyer that assesses a supplier, such as an NHS England trust
> using DTAC and the DSPT, a European public body under NIS2, or a US health system under HIPAA (WPM-R214). Nothing here is legal
> advice. The supplier's data protection officer and counsel confirm the contract terms.

## The situation

An organization that hosts WPM, licenses or supplies it as a service, and supports it becomes the buyer's **processor** under
UK GDPR Article 28 and a **third-party supplier**. The buyer assesses that organization, not the open-source project. So most of
what a buyer asks is about **the supplier**: its certifications, staff, contracts, and operations. Software can only make the
supplier's answers true and checkable. This file separates the two:

- **What the software must do** (requirements below), so a supplier can give honest answers.
- **What the supplier must do** (tasks marked *supplier's task*). These cannot be met by code, and the repository records only
  that they exist and which evidence they need.

The repository stays generic. A supplier's own pack (its certificates, its cloud account, the buyer's configuration) belongs in
the supplier's private records, never in this public repository (WPM-D214).

## Sources and how far they were checked

The owner's notes are a secondary summary. They were checked on 2026-11-20 as far as search excerpts reached:

| Claim in the notes | Found | Status |
| --- | --- | --- |
| WHC (2017) 025 sets supplier cyber and IG requirements | Welsh Government page "Cyber security and information governance (WHC/2017/025)", first published 4 July 2017, on suppliers that access and handle patient data | **Exists**; its full text, and whether it names Cyber Essentials Plus, **not read** |
| Cyber Essentials Plus as the minimum, with ISO/IEC 27001 preferred | A health board's (Hywel Dda) third-party supplier policy grades it by agreement level: ISO 27001 and Cyber Essentials for level 2 and above, Cyber Essentials Plus for level 3 | **One board's policy**, not national; varies by board |
| The Welsh Information Governance Toolkit (WIGTK) | DHCW maintains it; the 2026/27 toolkit is due 31 March 2027. DHCW has tendered a third-party provider accreditation service | **Exists**; how a supplier is assessed under it **not found** |
| Breach notice to the buyer within 24 to 48 hours | Not found in any primary source | **Unverified.** Article 33 gives the controller 72 hours to notify the regulator; the processor's duty is "without undue delay", and the contract sets the hours |
| Critical patches within 14 calendar days | Not found for NHS Wales | **Unverified**; a common contract term and Cyber Essentials' window for high and critical updates. Confirm against the current Cyber Essentials requirements and the contract |
| BPSS or DBS vetting and UK-based support for HR data | Not found for NHS Wales | **Unverified**; plausible contract terms |
| CHECK or CREST IT Health Check, yearly | Not found for NHS Wales | **Unverified**; a common buyer requirement |
| NHS Wales Records Management Code of Practice for retention | Not checked | **Unverified**: which code applies in Wales, and which version |
| NADEX as the NHS Wales directory, federated through Entra ID | Not checked | **Unverified**; WPM already signs in through Entra over OIDC ([entra-sign-in](operations/entra-sign-in.md)) |

WPM-T574 reads each primary source and corrects this table.

## Where WPM stands against each ask

| The buyer asks | WPM today | Gap | Here |
| --- | --- | --- | --- |
| A licence that permits hosted commercial supply | Service `MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only OR OGL-UK-3.0 OR CC-BY-NC-SA-4.0`; UI and site `MIT OR Apache-2.0` ([LICENSE.md](../LICENSE.md)); dependency licences checked (`check-licenses.py`). No AGPL anywhere | None in the code. A supplier chooses MIT or Apache-2.0, under which hosting creates no duty to disclose changes. SFIA content is licensed separately (WPM-D69) | WPM-T575 |
| UK-only hosting, backups, and recovery copies | Backups encrypted; the target is configured (WPM-T192); nothing records or checks a region | **No declared residency** that a check can compare against | R272 |
| A sub-processor register | None | Supplier's document; the software's own outbound calls (webhooks, SCIM, upstream lookups) are not listed in one place | R272, WPM-T584 |
| Support staff never see live sensitive records unmonitored | No vendor or support role. Administrators are the deployer's people; `svc` is a service account only | **No time-limited, customer-approved support access** | R267 |
| No shared vendor backdoor accounts | Sign-in only through the identity provider; no local passwords; production refuses to start without sign-in (WPM-R72) | None in the code; to evidence in the pack | WPM-T589 |
| Single-tenant isolation | One database can hold several organizations, scoped per row | **Nothing pins a deployment to one buyer**, and **nothing stops a service starting against another buyer's database** | R268, R277, WPM-D215, WPM-D220 |
| SSO with MFA through Entra ID, OIDC or SAML | OIDC with Entra, tested ([entra-sign-in](operations/entra-sign-in.md)); MFA is the identity provider's | **WPM does not check that MFA happened**; no SAML until a customer needs it (Entra serves OIDC; WPM-D222) | R269, WPM-D222 |
| Field-level separation of sickness, disciplinary, and pay | A sick-leave request **cannot carry a reason** (WPM-D73); no diagnosis or condition column (WPM-D24); **no disciplinary case record exists**; pay masked by policy; leave readable only by the worker, line managers, and HR or payroll (WPM-R114); the enforcement persona matrix tests it | The strongest position: the field that does not exist cannot leak. The persona matrix has **no support persona** | R267, WPM-T586 |
| TLS 1.2 or 1.3, and AES-256 at rest | TLS at the deployer's proxy with HSTS; plaintext database connections refused in production (WPM-T191); encrypted backups (WPM-T192) | Encryption at rest is the platform's. **No application-level encryption** of the most sensitive columns | R270, WPM-D217 |
| Tamper-evident logs of every read and export of sensitive records, to the buyer's SIEM | Append-only, hash-chained audit (WPM-R115); sensitive reads audited; `GET /api/audits/export` as JSON Lines (WPM-R122) | **A truncated tail goes undetected**, and a database owner can disable the triggers. No IP address is recorded (by design: WPM-D54 spirit) | R271, WPM-D218 |
| Export of all records at exit, then certified erasure | Per-person subject-access export; erasure as anonymization; backups expire (WPM-D53) | **No whole-organization export, and no destruction record** | R273, R274 |
| Retention aligned with the buyer's records code | A schedule per record kind with replaceable defaults, and a sweep ([retention-schedule](governance/retention-schedule.md)) | Not mapped to a Welsh code | WPM-T583 |
| Patch SLA and automated dependency and container scanning | `cargo deny`, Dependabot, `pnpm audit`, a signed image with provenance, and an SBOM at release | **No container image scan**; no written SLA | WPM-T580, WPM-T581 |
| An independent penetration test, including role bypass | Not done (WPM-T201) | The scope should name the support role and the tenant pin | WPM-T587 |
| A 24/7 incident response plan, tested | A runbook section "A suspected breach" ([runbook](operations/runbook.md)) | Supplier's plan and rota; no exercise | WPM-T590 |
| Inputs to the buyer's DPIA and DPA | A DPIA pack, a record of processing, a data flow ([governance](governance/dpia.md)) | No processor's view: what the supplier does with the data, where, and under which controls | WPM-T585 |
| Cyber Essentials Plus, ISO/IEC 27001, WIGTK onboarding, vetting, insurance | None (assessment checklist 4.14: not met) | **Supplier's tasks** | WPM-T591–T593 |

## Requirements

### WPM-R267 — Support access is granted by the buyer, limited in time and scope, and recorded

*As the buyer's administrator I decide when a supplier's engineer may see our data, what they may see, and for how long. Every
read they make is recorded where I can see it.*

- A **support** role exists in the policy. It has no access to records by default: with no grant, a support engineer reaches
  health, version, and the configuration check, and nothing that holds personal data.
- A **support access grant** is requested by the support engineer, with a reason and a ticket reference, and **approved by a
  buyer administrator**, never by the same person and never by the supplier. It names the scope (record kinds, an organization,
  optionally a department or one record), an expiry of at most 8 hours, and whether masked fields stay masked (the default).
- While a grant is live, the engineer's reads are audited with the grant's id. The buyer's administrators see the live grants
  and can end any of them at once. An expired grant is refused on the next request, not at the next sign-in.
- **Break glass**: where the buyer configures it, an engineer may self-grant for at most 1 hour during an incident. Every
  administrator is told in-app immediately, and the grant needs a buyer administrator's review afterwards. It is off by default.
- Sensitive categories (pay, payslips, leave of the sickness kind, adjustments, the injury record, equality data, and review
  content) are outside a grant unless it names them one by one.
- The persona matrix adds a support persona: with no grant, a grant of each scope, an expired grant, and a revoked grant.

### WPM-R268 — A deployment can be pinned to one buyer

*As a supplier I can show that a buyer's deployment serves that buyer only.*

- `WORKFORCE_PLANNING_MANAGEMENT_ORGANIZATION_ALLOWLIST` names the organizations a deployment serves. When set, a token whose organization is outside it
  is refused with 403, a write naming another organization is refused, and the service refuses to start if the database holds
  rows of an organization outside the list.
- The configuration check (WPM-R248) reports the pin. The supported model for a health buyer is **one deployment, one database,
  and one key set per buyer** (WPM-D215).

### WPM-R277 — One deployment, one database

*As a supplier hosting WPM for several buyers, I know that a buyer's service can only start against that buyer's database.*

When each buyer has its own database (WPM-D215), the only thing that links a running service to its database is `DB_URL` (or
`DATABASE_URL`). Suppose an operator copies the wrong configuration or makes a typo in a deploy script. Buyer B's service then
starts against buyer A's database, and B's staff see and change A's people, pay, absence and allocations. Nothing stops that
today. The sibling contact-relationship-management application has the same requirement (CRM-R105, "One customer, one
database"), and this one follows it so the family behaves alike.

- **Naming a deployment.** `WORKFORCE_PLANNING_MANAGEMENT_DEPLOYMENT_ID` names the deployment: 1 to 64 characters from lower-case letters, digits and
  hyphens, starting with a letter (for example `acme-example`). Any other value stops the start, and the message names the rule,
  not the value.
- **The first start** with a name, against a database that has none: the service writes the name into a new one-row table
  `deployment` (the name and when it was bound), audits `deployment_bound`, and starts.
- **Every later start**: the service compares its configured name with the stored one. If they are equal, it starts. If they
  differ, it **refuses to start**. The log names both deployments, because names identify deployments, not people; it never
  shows the connection string. Nothing in the database is read or written beyond that one row.
- **No name configured** *(inferred; the owner's text stops at the third step)*:
  - In the `production` environment, the service refuses to start, as it does without sign-in (WPM-D52).
  - Elsewhere it starts and logs a warning once.
  - If the database already holds a name, a service with none refuses to start in every environment, because it cannot show
    that it is that deployment.
- **Changing the name** is a deliberate act: a task `deployment_rename --from <old> --to <new>` rewrites the row only if `<old>`
  matches, and audits `deployment_renamed`. There is no way to clear the name through the API.
- **The check runs before migrations and before the port opens**, so a wrong database is never migrated, and never served even
  for one request. The configuration check (WPM-R248) reports the bound name. `GET /api/admin/processing-destinations`
  (WPM-R272) shows it beside the database destination.
- **What it does not do:** it does not stop two deployments that share a name from sharing a database; naming them apart is the
  supplier's part. It does not authenticate the database server: TLS (WPM-T191) and the credentials do that. The organization
  pin (WPM-R268) is a separate check on what the rows contain, and the two together cover a wrong database and a wrong row.

### WPM-R269 — WPM can require that the sign-in used MFA

*As the buyer I can make WPM refuse a token that did not come from a multi-factor sign-in.*

- `WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH_CONTEXT` lists the `amr` values (for example `mfa`) or the `acr` value a token must carry. A token without it
  is refused with 401, naming the requirement but not the token. It can apply to every request or only to high-tier reads.
- Entra's claims are checked against a real tenant before the setting is documented as working for Entra (WPM-T579).

### WPM-R270 — The most sensitive columns are encrypted in the application

*As the buyer I know that a database dump or a misdirected backup does not expose the most sensitive free text and
special-category fields.*

- Envelope encryption with AES-256-GCM: a data key per organization, wrapped by a key in the deployer's key management service
  (in the declared region, R272). Keys are rotated with re-wrapping, never by re-encrypting in place.
- **Scope** (WPM-D217): free-text and special-category fields that are never sorted, filtered, or summed in SQL. These include
  adjustment details, the injury record's narrative, emergency contacts, contact details, equality data, bank and payroll
  details, review content, and reasons on requests. **Salary and other figures used in arithmetic stay in the clear.** They are
  protected by masking, need-to-know reads, and storage encryption, because encrypting them would move every aggregate out of the
  database.
- A missing or unreachable key stops the service from starting, rather than serving ciphertext or the clear text.

### WPM-R271 — The audit chain's head is anchored outside the database

*As the buyer's security team I can tell when the audit trail has been cut short or rewritten, even by a database owner.*

- At a configured interval (default 15 minutes) the service publishes the chain head (sequence and hash), signed with a key the
  database does not hold. It goes to a configured sink: the SIEM, an object store with object lock, or both.
- The verifier compares the chain with the latest anchor. A chain shorter than an anchor, or a hash that differs at an anchored
  sequence, is reported as `truncated` or `rewritten`, with the sequence.
- The SIEM export (WPM-R122) carries the grant id (R267) on support reads. **No IP address is stored in the audit trail**
  (WPM-D218). The request id joins an entry to the proxy's log, which holds the address under the proxy's own retention.

### WPM-R272 — Data residency is declared and checked

*As a supplier I can show where the data is, and the software tells me when a configured place disagrees.*

- `WORKFORCE_PLANNING_MANAGEMENT_DATA_RESIDENCY` declares the permitted regions (for example `uk`) and each outbound destination. These include the
  database, backup and recovery storage, the key service, the audit anchor sink, webhook subscribers, the SCIM target, and
  upstream services. Each destination carries its declared region.
- At start, the configuration check refuses a destination with no declared region, and one whose region is outside the permitted
  list. It cannot prove where a provider stores bytes. It makes the supplier's statement explicit and checkable, and the pack
  carries the provider's attestation (WPM-D216).
- `GET /api/admin/processing-destinations` (administrators) lists every destination with its purpose, region, and the
  personal data it receives. That list is the software's half of the sub-processor register.

### WPM-R273 — The whole of an organization's data can be exported at exit

*As the buyer I get all of our records back in a structured, documented form when the contract ends.*

- An administrator's export of one organization: a JSON Lines file and a CSV per table, a schema document, the audit trail with
  its last anchor, and a manifest with row counts and SHA-256 checksums. It covers every table that holds that organization's
  rows, including soft-deleted and erased (anonymized) ones as they stand.
- It runs as a job with progress, writes to the buyer's chosen destination, which must be declared (R272), and is audited.
  It needs two buyer administrators (maker and checker), because it is the whole of the data.

### WPM-R274 — Destruction at exit is a record, not a claim

*As the buyer I receive a dated record of what was destroyed and when, including the backups that expire later.*

- After the buyer confirms receipt of the export, a destruction run deletes the organization's data and **destroys its data keys**
  (R270). Crypto-erasure makes the encrypted columns unreadable in every backup at once.
- A **destruction record** lists the database, storage, and key ids destroyed, with times. It names each backup set with the date it
  expires, because a backup that cannot be edited is destroyed only when it expires (WPM-D53, WPM-D219). The supplier signs the
  record; the software does not certify.

## Design decisions (proposed)

- **WPM-D214 The repository holds templates; the supplier holds its pack.** Certificates, the cloud account, vetting records,
  and the buyer's configuration are private. Committing them to a public repository would itself be a disclosure.
- **WPM-D215 A health buyer gets its own deployment.** Row-level scoping by organization is right for one employer's group, not
  for unrelated buyers. One deployment, database, and key set per buyer makes isolation a fact of the infrastructure. The pin
  (R268) catches a misconfiguration. Rejected: a shared database with row-level security as the isolation boundary.
- **WPM-D220 A database knows whose it is.** The deployment's name is stored in the database it serves and checked at every
  start, so a configuration mistake fails closed, before migrations, rather than serving one buyer's records to another. The
  name lives in the database, not in a file beside the service, because the database is the thing that must not be swapped.
  Rejected: checking the host name in `DB_URL`, because one server can hold several buyers' databases, and a host name changes
  when a database is moved or restored. Follows CRM-R105 in the sibling CRM application, so the family behaves alike.
- **WPM-D216 Residency is declared, not detected.** Software cannot see where a provider keeps bytes. It can refuse an
  undeclared destination and list the declared ones, so the supplier's statement has one source.
- **WPM-D217 Encrypt what is never computed on.** Application-level encryption of free text and special-category fields gives
  real protection against dumps. Encrypting salary would break aggregates and the honest arithmetic they rest on. Salary keeps
  masking, need-to-know, and storage encryption. Reconsider if a buyer requires it, at the cost of moving aggregates into the
  service.
- **WPM-D218 The audit trail holds no IP address.** An address is personal data, and the trail is kept for years. The proxy
  already logs it, under its own shorter retention, and the request id joins the two. A buyer who requires the address in the
  trail can be offered it as a setting, with the retention consequence stated.
- **WPM-D219 Erasure of a backup happens when the backup expires.** The destruction record says when each will expire. Crypto-
  erasure covers the encrypted columns immediately; the rest of a backup is unreadable only once its own key or retention ends.

## Shared responsibility (worked example: NHS Wales)

| Area | Supplier (hosts, supplies, and supports WPM) | Buyer (health board or trust, the controller) | What WPM provides |
| --- | --- | --- | --- |
| Controller duties, DPIA | Processor inputs: architecture, controls, sub-processors (WPM-T585) | Completes and signs the DPIA (DPO, SIRO); lawful basis | The DPIA pack, record of processing, data flow |
| Contract (Article 28) | Signs the DPA; breach notice within the agreed hours; flows terms down to sub-processors | Sets instructions, hours, and audit rights | The processing-destinations list (R272) |
| Hosting, backups, recovery | UK regions; hardening; monitoring; restore drills (WPM-G4) | Approves the location; reviews assurance | Encrypted backups, restore drill, residency check (R272) |
| Code and patching | Tracks advisories; patches within the agreed window; yearly independent test | Reviews test summaries; approves maintenance windows | Pinned supply chain, SBOM, signed image, scans |
| Identity | Maintains the OIDC integration and role templates | Accounts, MFA, leavers in Entra ID; assigns WPM roles | OIDC, token age, MFA requirement (R269) |
| Sensitive fields | Enforces the software's controls; never self-approves support access | Decides who reads what; approves support grants | Need-to-know policy, masking, no reason fields, support grants (R267), column encryption (R270) |
| Audit and incidents | 24/7 monitoring; contains incidents; notifies the buyer | Notifies the ICO and staff | Hash chain, anchors (R271), SIEM export |
| Retention and exit | Runs the sweep; executes export and destruction | Sets the schedule; verifies the export | Retention sweep, organization export (R273), destruction record (R274) |

## Not done, not decided

The open questions are in [plan.md](plan.md) section Y.
