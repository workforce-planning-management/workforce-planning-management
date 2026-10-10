# DTAC v2.0 assessment of Workforce Planning Management (WPM)

An assessment of this software against the questions in [checklist.md](checklist.md) (Digital Technology
Assessment Criteria, form v2.0, 24 February 2026). It is the answers a manufacturer would give, written from the
repository as it stood on **2026-10-11**, for **version 0.1.0** of the service and the web client.

> ⚠️ **This is a self-assessment, written by the maintainer's AI coding assistant, and nobody independent has
> reviewed it.** It is **not** a submitted DTAC form and does not say the product passes. Every answer cites the
> file, requirement or test it rests on; where it cannot be answered from the repository, it says
> **manufacturer to complete** rather than invent a company, a certificate or a date. Under this repository's own
> rule ([WPM-D61](../../../../spec/operations-and-governance.md)), an answer without evidence is not "met".

## How to read it

| Assessment | Meaning |
| --- | --- |
| **Met** | The repository holds the evidence cited. |
| **Partly met** | Some of it is evidenced; the gap is stated. |
| **Not met** | The requirement is not satisfied, and this says so plainly. |
| **Not applicable** | The question does not apply to the product as built, with the reason (a commissioner may challenge it). |
| **Manufacturer to complete** | A fact about a company, a certificate or a date that only the manufacturer holds. |

## The product in one paragraph

WPM is a **workforce planning and HR management** service for an organization's own staff: employment
records, leave and rota, recruitment, learning, performance, payroll (illustrative stubs), strategic workforce
planning and delivery capacity. It holds **no patient data, takes no clinical input and gives no clinical
output.** It is open-source software the buyer deploys and operates (a Rust JSON API and a SvelteKit web client
over PostgreSQL); the maintainer does not host it, has no access to any deployment, and receives no telemetry.
It is **demonstration software** ([regulatory.md](../../../../spec/regulatory.md)): not a production HR or payroll
system, and its own [readiness assessment](../../../../spec/readiness-assessment.md) says it is **not ready**
for production use in a large medical or governmental organization.

## Where it stands

| Section | Result |
| --- | --- |
| A. Company information | **Mostly manufacturer to complete.** There is no company: the maintainer is one individual. |
| B. Value proposition | Described; the benefits are **not validated**. |
| C1. Clinical safety | Answered **not in scope** (workforce records, no direct care). That answer can be challenged, and **no DCB0129 documentation exists**. |
| C2. Data protection | The manufacturer holds and processes **no** deployed data. A DPIA template, data-flow diagram and record of processing are provided; **no ICO registration evidence, no privacy notice**; storage location is the deployer's. |
| C3. Technical security | **Would not pass as it stands:** no Cyber Essentials certificate, the Cyber Security Charter is unsigned, and **no independent penetration test** has been done. Several controls are built and tested. |
| C4. Interoperability | JSON/OpenAPI API; **no patient-identity exchange** (no NHS number, PDS or NHS login), so the identity questions fall away. |
| D1. Usability and accessibility | Scored, not pass/fail: **no user testing, no WCAG audit**, no availability figure. |

## A. Company information (non-assessed)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| A1 | None. WPM is maintained by an individual, Joel Parker Henderson, as sole maintainer | Manufacturer to complete | [MAINTAINERS.md](../../../../MAINTAINERS.md). No company is named anywhere in the repository |
| A2 | Workforce Planning Management (WPM): the service and its web client | Met | [README.md](../../../../README.md) |
| A3 | 0.1.0 (both subprojects). Pre-release: the release workflow is written and has not run | Met | `Cargo.toml`, `package.json`, [production-readiness.md](../../../../spec/production-readiness.md) (WPM-R87) |
| A4 | **Other:** open-source software that the buyer deploys and operates (API, web client, PostgreSQL). The maintainer does not offer it as software as a service | Met | [INSTALL.md](../../../../INSTALL.md), [deployment.md](../../../../spec/operations/deployment.md) |
| A5 | Joel Parker Henderson, sole maintainer | Met | [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| A6 | joel@joelparkerhenderson.com | Met | [SECURITY.md](../../../../SECURITY.md) |
| A7 | Not published | Manufacturer to complete | |
| A8 | Not published | Manufacturer to complete | |
| A9 | Not published | Manufacturer to complete | |
| A10 | None known. No company or charity number is recorded | Manufacturer to complete | |
| A11 | Not applicable: the maintainer provides no regulated care activity. **The manufacturer must confirm** | Not applicable | |
| A12 | Not applicable | Not applicable | |

## B. Value proposition (non-assessed)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| B1 | **Workforce Support or Management** | Met | [purpose.md](../../../../spec/purpose.md) |
| B2 | A workforce planning and HR service for an organization's staff: the employee lifecycle from recruitment to leaving, leave and rota, performance and learning, strategic workforce planning, delivery capacity by skill pool, and illustrative payroll. People can also see and change their own details, ask for flexible working, log a resignation and see their time-off. Two optional features, equality monitoring and workplace health requirements, are **off unless the deployer records a lawful basis** | Met | [README.md](../../../../README.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| B3 | Intended users: HR, line managers, workforce planners, payroll and employees. **The benefits are not validated:** there has been no evaluation, trial or user study. The README describes what is built and tested, not outcomes | Not met | No evaluation exists |
| B4 | A data-flow diagram and its table are provided. **A user-journey map is not provided** | Partly met | [data-flow.md](../../../../spec/governance/data-flow.md) |

## C1. Clinical safety (assessed)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| C1.1.1 | **No.** No component has a medical purpose: it holds workforce records, takes no clinical input and produces no clinical output. This is the manufacturer's statement, not a regulator's view. An integrator who gives it a medical purpose changes it | Met (as a statement) | [regulatory.md](../../../../spec/regulatory.md); [assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) rows 2.1 and 2.4 |
| C1.1.2 | **No** | Met (as a statement) | As C1.1.1 |
| C1.2 | **No**, with the caveat in C1.2.1 | Partly met | |
| C1.2.1 | WPM holds staff records and plans workforce. It carries no patient information and does not influence, support or manage direct care. **The caveat a commissioner may press:** it produces staffing information, such as an on-call rota, who covers whom, and a delivery-capacity view, that a care organization could use to staff a clinical service. Those outputs are advisory (WPM-D19, WPM-D28, WPM-D50), nothing is assigned or alerted in real time, and a person decides. **An organization that uses them to staff a service in near real time should treat that use as in scope and ask for a hazard assessment, which does not exist.** | Partly met | [design.md](../../../../spec/design.md), [delivery-capacity.md](../../../../spec/delivery-capacity.md), assessment-checklist row 2.2 |
| C1.2.2 to C1.2.5 | Not answered, because C1.2 is No. **For completeness: none of this exists.** There is no DCB0129 clinical risk management system, no Clinical Safety Case Report, no hazard log and no Clinical Safety Officer. **If a commissioner decides DCB0129 applies, C1 is Not met** | Not applicable (see C1.2.1) | [assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) rows 2.2 and 2.3 |

## C2. Data protection (assessed)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| C2.1 | **No access to patient data or national NHS System.** The maintainer has no access to any deployment: no hosting, no remote support, no telemetry. The software's only outbound requests are to URLs the deployer configures (token signing keys, and optional name look-ups against the deployer's own services, off by default) | Met | `src/clients.rs` (stub mode by default), `src/auth/keycloak.rs`. The importers were read for network use: they make none |
| C2.2 | **Yes, the product processes personal data**, namely staff and applicants' data. The manufacturer does not operate or host it and has no means of access, so the deployer is the controller. (The form lets a manufacturer in that position answer No; this answers Yes so that nothing is understated) | Met | [record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| C2.2.1 | **Not provided.** The maintainer is not a controller or processor of any deployment. Whether the maintainer pays an ICO fee is not recorded | Manufacturer to complete | |
| C2.2.2 | **Provided as a template** completed for the demo. Coverage of the required content: see below | Partly met | [dpia.md](../../../../spec/governance/dpia.md) |
| C2.2.3 | **Not provided.** A deployer writes their own from the record of processing | Not met | assessment-checklist row 3.10 |
| C2.2.4 | Not applicable as a user-data agreement: the software is licensed under open-source licences and the manufacturer processes no user data. It is **not** a substitute for the buyer's own terms with their staff | Not applicable | [LICENSE.md](../../../../LICENSE.md) |
| C2.2.5 | **The deployer's choice.** The software stores data only in the PostgreSQL database the deployer connects it to. One third-party component, the content manager at `/admin`, loads a script from a public CDN with no pinned version or integrity hash (a known gap, WPM-R73) | Manufacturer to complete | [production-readiness.md](../../../../spec/production-readiness.md) |
| C2.2.6 | Not answerable by the manufacturer: depends on where the deployer hosts. The software makes no transfer of its own | Manufacturer to complete | [data-flow.md](../../../../spec/governance/data-flow.md) |

### How the DPIA template covers what C2.2.2 requires

| Required content | Covered? | Where |
| --- | --- | --- |
| Summary of the product and how it processes data | Yes | dpia.md sections 1 and 2 |
| List of data fields required | Partly: by category, not field by field | [record-of-processing.md](../../../../spec/governance/record-of-processing.md), [domain-model.md](../../../../spec/domain-model.md) |
| How data flows into, within and out of the product | Yes | [data-flow.md](../../../../spec/governance/data-flow.md) |
| End-user security controls: on/off-boarding and access limits | Yes | [auth.md](../../../../spec/auth.md): sign-in enforced by default, need-to-know on every read (WPM-R114), self-service allow-list |
| Measures for data in transit and at rest | Partly: TLS to the database is required in production (WPM-R116); backups are encrypted (WPM-R117); encryption of the database at rest is the deployer's | [production-readiness.md](../../../../spec/production-readiness.md), [backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Countries where data is stored or flows | The deployer's | |
| Whether the manufacturer's staff can access personal data | Yes: they cannot | See C2.1 |
| Who is controller for each element | The deployer for all of it | dpia.md section 2 |
| Retention and disposal | Yes: a schedule per record kind, a sweep, an erasure ledger that survives a restore | [retention-schedule.md](../../../../spec/governance/retention-schedule.md), WPM-R89, WPM-R91 |
| Processors and sub-processors, with binding agreements | The deployer's | |
| Confidentiality, availability and integrity risks and mitigations | Yes | dpia.md section 5 (R1 to R14) |
| How the product supports data subject rights | Yes: a subject-access export (with special-category data given to the worker alone) and erasure | `GET /api/workers/{pid}/subject-access`, [regulatory.md](../../../../spec/regulatory.md) |

## C3. Technical security (assessed)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| C3.1 | **No Cyber Essentials certificate.** The maintainer is an individual and has none | **Not met** | |
| C3.2 | **No**, the Cyber Security Charter for Suppliers is not signed | Not met | So C3.3 to C3.6 are answered below |
| C3.3 | **Not provided. No independent penetration test has been done**, and the external dynamic scan (OWASP ZAP baseline) is written in CI but has never run. The author's own [threat model](../../../../spec/governance/threat-model.md) is not a substitute | **Not met** | [readiness-assessment.md](../../../../spec/readiness-assessment.md) section 4; WPM-T201 is open |
| C3.4 | **Unable to confirm.** Practices that map to the Code's four themes exist, and the gaps are named. *Secure design and development:* threat model, need-to-know on every read, pure-core rules tested, mutation-checked security tests, `cargo deny`. *Build environment:* CI is written and has **never run on a hosting platform**; actions are pinned by tag, not commit; no secret scanning. *Deployment and maintenance:* production refuses an unauthenticated or plaintext-database start, encrypted backups, a tested restore. *Communication with customers:* a security policy, a changelog, a bill of materials on release (not yet run). **Not independently assessed against the Code** | Partly met | [threat-model.md](../../../../spec/governance/threat-model.md), [production-readiness.md](../../../../spec/production-readiness.md), SECURITY.md; open: WPM-T194, T195 |
| C3.5 | **Yes, by identity federation.** WPM implements no passwords or accounts of its own: it verifies tokens issued by the deployer's identity provider (Microsoft Entra ID or Keycloak), so MFA is enforced there for every account type. The plan is documented | Partly met | [entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md). MFA itself is the identity provider's and was **not exercised** by this repository's tests |
| C3.5.1 | Not applicable: the supplier has no access to any deployment | Not applicable | See C2.1 |
| C3.6 | **Yes.** Every mutation and every sensitive read writes an audit entry naming the actor; the trail is **append-only and hash-chained in the database**, verifiable by an endpoint and a task, and checked by every restore drill. Requirements for what is logged are written | Met | [audit.md](../../../../spec/audit.md), WPM-R115, `GET /api/audits/verify`, `tests/requests/audit_chain.rs` |

## C4. Interoperability (assessed)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| C4.1 | **Yes:** a JSON REST API, for the buyer's own systems | Met | `/api-docs/openapi.json` |
| C4.1.1 | HTTP and JSON; OpenAPI 3.0.3; bearer tokens (PASETO or OIDC JWT); ISO 8601 dates; ISO 4217 currencies; opaque entity references as URNs. **No health-care standard is used** (no HL7 FHIR, for example) because the product holds workforce, not clinical, records | Met | `src/openapi.rs`, `crates/entity-ref` |
| C4.1.2 | **Cannot confirm** conformance with GDS Open API best practice; the API is openly documented and the code is public | Not met (cannot confirm) | The OpenAPI document is hand-written and its coverage is spot-checked by a test, not generated, so it can lag the routes |
| C4.1.3 | The basis on which third parties can build integrations: the source and the OpenAPI document are public under open-source licences; there is no commercial gate or partner agreement | Met | [LICENSE.md](../../../../LICENSE.md), `/api-docs/openapi.json` |
| C4.2 | **No.** WPM does not share or receive data with national or local care systems where patient identity is relevant: it holds no patient identity | Met | C2.1 |
| C4.2.1 to C4.2.6 | Not asked, because C4.2 is No. For information: **WPM does not use the NHS number, does not integrate with the Personal Demographics Service and does not use NHS login.** People sign in through the deployer's identity provider | Not applicable | [entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |

## D1. Usability and accessibility (scored, not pass/fail)

| Code | Answer | Assessment | Evidence or gap |
| --- | --- | --- | --- |
| D1.1 | **Not provided** as a user journey. The specification describes personas and screens; no journey or instructions for use fit WPM into a care pathway | Not met | [purpose.md](../../../../spec/purpose.md) |
| D1.2 | **No.** No testing with intended users has been done | Not met | |
| D1.3 | **Cannot confirm** that the Accessible Information Standard has been read and considered. It concerns communications with patients, which WPM has none of | Not met (cannot confirm) | |
| D1.4 | **Yes**, a web application | Met | `workforce-planning-management-ui-with-svelte` |
| D1.4.1 | **No.** Compliance with WCAG 2.2 AA is **not verified**: there has been no audit, no assistive-technology testing and no automated accessibility test. There is a **plan** (an independent audit, WPM-T202) but no timeline | Not met | [accessibility-statement.md](../../../../spec/governance/accessibility-statement.md) |
| D1.4.2 | **No timescale has been set** | Not met | WPM-T202 is open |
| D1.4.3 | The statement is [accessibility-statement.md](../../../../spec/governance/accessibility-statement.md). It says plainly that the software has not been audited. It is a template for a deployer's own; **it is not published at a public address** | Partly met | |
| D1.5 | **Not applicable and not measured:** the manufacturer operates no service, so there is no availability figure. A deployer measures their own | Not applicable | |

## Supporting documents

| Code | Document | Status |
| --- | --- | --- |
| A12 | CQC report | Not applicable |
| B4 | User journeys and data flows | Data flow: [data-flow.md](../../../../spec/governance/data-flow.md). **User journeys: not provided** |
| C1.1.1 | Pre-acquisition questionnaire | Not applicable (C1.1.1 is No) |
| C1.2.3, C1.2.4 | Clinical risk management system, Clinical Safety Case Report, Hazard Log | **Do not exist** |
| C2.2.1 | ICO registration | **Not provided** |
| C2.2.2 | DPIA | Template: [dpia.md](../../../../spec/governance/dpia.md) |
| C2.2.3 | Transparency information | **Not provided** |
| C2.2.4 | Terms on use of user data | Not applicable: [LICENSE.md](../../../../LICENSE.md) |
| C3.1 | Cyber Essentials certificate | **Not provided** |
| C3.3 | External penetration test summary | **Not provided** |
| D1.1 | User journeys | **Not provided** |

## What it would take to pass

The assessed sections (C1 to C4) must all be met. As submitted, **C3 fails on three points the maintainer cannot
close in code**, and C2 and C1 each depend on a decision for someone else.

1. **C3.1 Cyber Essentials.** A certificate for the manufacturer (or the hosting supplier, if the buyer buys it
   as a service). Needs an organization to certify.
2. **C3.3 Independent penetration test** of a deployed instance covering the OWASP Top 10, with no finding at or above
   CVSS 7.0. Open as [WPM-T201](../../../../spec/tasks.md). Alternatively **C3.2:** sign the Cyber Security Charter
   for Suppliers to the NHS, which removes C3.3 to C3.6.
3. **C2.2.1 and C2.2.3.** ICO registration evidence if the manufacturer is a controller or processor of any
   deployment (not if only the deployer is), and a transparency notice the deployer completes from the record of
   processing.
4. **C1.** Settle with the commissioner whether any use of the rota or capacity view staffs a service in near real
   time. If it does, DCB0129 applies and the clinical risk management system, safety case, hazard log and Clinical
   Safety Officer must exist; none does.
5. **D1.** An independent WCAG 2.2 AA audit ([WPM-T202](../../../../spec/tasks.md)) and user testing, for the score.
6. **A.** A legal entity, address and phone number, or a statement that the buyer is contracting with an
   individual.

Separately, the product's own [readiness assessment](../../../../spec/readiness-assessment.md) records that it is not
ready for production use in a large medical or governmental organization, whatever DTAC says.

## Limits of this assessment

- Written from the repository only. It did not see a deployment, a hosting arrangement, a certificate or a contract.
- It rests on the maintainer's own tests and documents. No external party has reviewed either.
- It follows the checklist as converted in [checklist.md](checklist.md). The original form, and its guidance on
  evidence, were not available to the assessor; the pass criteria are as that file states them.
- Where the checklist links to a published standard (DCB0129, the Software Security Code of Practice, DAPB3051,
  the Accessible Information Standard), the standard itself was not read in full for this assessment.
