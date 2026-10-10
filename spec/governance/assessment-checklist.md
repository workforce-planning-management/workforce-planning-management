# Digital technology assessment checklist (WPM-R92, WPM-D61)

A checklist a buyer's information governance lead can complete their own assessment from.
It follows the structure of published digital technology assessment criteria (DTAC-style):
company, domain safety, data protection, technical security, interoperability, and
usability and accessibility. It is not an official form of any scheme, and answering it
does not certify anything.

**How to read an answer.** Each answer is one of:

- **Met**: the software does this, and the evidence column names a requirement, a test or
  a document that shows it;
- **Partly met**: some of it, and the evidence column says what is missing;
- **Not met**: it does not, and the evidence column says so plainly;
- **Deployer to answer**: only the organization deploying the software can answer.

An answer without evidence is not "met" (WPM-D61). The software is **demo software** and
the maintainers have not been independently assessed. Evidence paths are relative to the
repository root; "T" numbers are in [tasks.md](../tasks.md).

## 1. Company information

| # | Question | Answer | Evidence or gap |
| --- | --- | --- | --- |
| 1.1 | Who is the supplier, and is it a registered organization? | Deployer to answer | An open-source project; the deployer decides who supports it for them, and who the supplier is |
| 1.2 | Is there a named support and security contact? | Partly met | `SECURITY.md` and `MAINTAINERS.md` name contacts. There is no service level, and no support organization |
| 1.3 | Is the license clear and compatible with our use? | Met | `LICENSE.md` and the texts in `LICENSE/`; a CI check keeps the manifests in step (T125) |
| 1.4 | Is the software maintained, and is its history visible? | Partly met | Public history and changelogs; a pre-release project with no stability promise |
| 1.5 | Insurance, financial standing, subcontractors | Deployer to answer | Not applicable to the code; applies to whoever supplies it |

## 2. Domain safety

| # | Question | Answer | Evidence or gap |
| --- | --- | --- | --- |
| 2.1 | Is the product intended for clinical use? | Met (answered) | No. It is an HR and workforce planning tool with synthetic data only (`spec/regulatory.md`). It has no clinical safety case and must not be used as a clinical system |
| 2.2 | Are there functions that could cause harm if wrong? | Partly met | Working-time guardrails and the on-call rota are advisory (WPM-D19); cutover readiness is planned, not built. A deployer should hold a hazard log for any use where staffing affects safety |
| 2.3 | Is there a named safety officer and a hazard log? | Deployer to answer | None is provided |
| 2.4 | Do automated outputs make decisions about people? | Met | No. Forecasts and gap analyses state their assumptions; the levers they list are suggestions and a person decides (WPM-D27, D28, D50). Nothing ranks or selects individuals |

## 3. Data protection

| # | Question | Answer | Evidence or gap |
| --- | --- | --- | --- |
| 3.1 | Is there a documented lawful basis for each purpose? | Deployer to answer | The [record of processing](record-of-processing.md) lists purposes with the basis column left empty |
| 3.2 | Has a data protection impact assessment been done? | Partly met | A [template completed for the demo](dpia.md); the deployer must complete their own and have their data protection officer advise |
| 3.3 | Is the data collected limited to what is needed? | Met | Identities are URNs, not copied demographics; "what must not be stored gets no column" (WPM-D17, D20, D24, D25); `spec/regulatory.md` |
| 3.4 | Is special category data handled with extra protection? | Partly met | No diagnosis, symptom or cohort column exists; adjustments record the barrier and the change only; the pulse is anonymous by construction (WPM-D20, D24, D25). A sick-leave request cannot carry a reason (WPM-D73), and a worker's leave is readable only by the worker, their line managers and HR or payroll (WPM-R114, `spec/auth.md`). Leave kinds (sick leave) can still imply health and are high-tier. The deployer must identify their Article 9 condition |
| 3.5 | Is access limited by role, and are sensitive fields masked? | Met | Attribute-based policy with masking; the enforcement matrix test (`tests/enforcement.rs`, `tests/enforcement_expenses.rs`) |
| 3.6 | Is access to sensitive records logged? | Met | Every mutation and every read of a high-tier record is audited without the sensitive value (`spec/audit.md`) |
| 3.7 | Are retention periods defined for each kind of record? | Partly met | A [schedule per record kind](retention-schedule.md) (WPM-R91, T147) with defaults to be replaced. The audit log has no retention rule; backups expire after 35 calendar days by default |
| 3.8 | Can a person see their data (subject access)? | Met | `GET /api/workers/{pid}/subject-access`, with its exclusions named; tested (`tests/requests/privacy.rs`). Upstream identity services hold their own copies |
| 3.9 | Can a person's data be erased, and does a restore undo it? | Met | Erasure anonymises and is refused while employment is open; payroll rows stay under statutory retention. An erasure ledger and replay stop a restore bringing a person back (WPM-D63, T148); tested in the request suite and in the restore drill |
| 3.10 | Is there a privacy notice for the people whose data is held? | Not met | None is provided; the deployer writes it from the record of processing |
| 3.11 | Are processors and international transfers identified? | Deployer to answer | The software makes no transfer of its own; the [data-flow diagram](data-flow.md) shows where one may arise |
| 3.12 | Is there a personal data breach procedure? | Partly met | The [runbook](../operations/runbook.md) has a template; notification duties and decisions are the deployer's |
| 3.13 | Is there a data protection officer? | Deployer to answer | |

## 4. Technical security

| # | Question | Answer | Evidence or gap |
| --- | --- | --- | --- |
| 4.1 | Is sign-in enforced by default, and can it be switched off in production? | Met | On by default; production refuses to start with it off or with no token key source (WPM-D52, T126); unit and live tests, mutation-checked |
| 4.2 | How are users authenticated? | Met | Offline-verified signed tokens from an identity provider (PASETO or OIDC, including Entra ID); the UI holds no token in browser JavaScript (WPM-D62, `SECURITY.md`) |
| 4.3 | Is multi-factor authentication supported? | Deployer to answer | It is the identity provider's feature; WPM neither adds nor blocks it |
| 4.4 | Are security headers and a content security policy set? | Met | API and UI (WPM-R73, T127); a test asserts them. `style-src` allows inline styles; the content manager loads an unpinned script from a public CDN (a known gap) |
| 4.5 | Is there protection against brute force and abuse? | Partly met | Per-address rate limiting with a stricter class (WPM-R74, T128), tested. Per instance only; behind a proxy it needs `WPM_TRUST_FORWARDED` and a proxy that overwrites the header |
| 4.6 | Is data encrypted in transit and at rest? | Deployer to answer | TLS is terminated by the deployer's proxy (HSTS is sent); encryption at rest is the database and backup storage's |
| 4.7 | Is there an independent penetration test? | Not met | None. A ZAP baseline scan is written for CI but has not run (T124) |
| 4.8 | Are vulnerabilities in dependencies found and fixed? | Partly met | `cargo deny check` (passes locally), Dependabot and an advisory `pnpm audit`, all written for CI; CI has not run on the hosting platform (T124) |
| 4.9 | Is there a software bill of materials? | Partly met | A CycloneDX bill is generated on release (T144, written, not yet run); generated locally once |
| 4.10 | Are secrets kept out of the repository? | Partly met | Production secrets are environment-only. The repository holds a test-only key and a placeholder development secret, labelled as such |
| 4.11 | Is there backup, restore and disaster recovery? | Partly met | Scripts, targets and a CI-run [restore drill](../operations/backup-and-restore.md) (WPM-R89). WAL archiving and point-in-time recovery are documented, not tested; times are measured on a small database only |
| 4.12 | Are changes tested and reviewed before release? | Partly met | Unit, database, enforcement and end-to-end suites; CI written (T124). Branch protection is a repository setting the deployer's fork must enable |
| 4.13 | Are accounts and access reviewed? | Deployer to answer | At the identity provider |
| 4.14 | Certifications (Cyber Essentials, ISO 27001, SOC 2) | Not met | None |

## 5. Interoperability

| # | Question | Answer | Evidence or gap |
| --- | --- | --- | --- |
| 5.1 | Is there a documented, versioned API? | Met | OpenAPI at `/api-docs/openapi.json`, Swagger UI, header-based versioning |
| 5.2 | Are open standards used for identity? | Met | OpenID Connect and signed tokens (WPM-R93) |
| 5.3 | Can data be exported in a usable form? | Partly met | Per-person JSON subject-access export; no bulk export of an organization |
| 5.4 | Domain standards (for example HL7 FHIR) | Not met | Not applicable to an HR tool; no such interface |
| 5.5 | Are integrations configurable rather than hard-coded? | Met | Upstream lookups by URN with stub and http modes; oversight levels and measures are configuration |

## 6. Usability and accessibility

| # | Question | Answer | Evidence or gap |
| --- | --- | --- | --- |
| 6.1 | Is there an accessibility statement against WCAG 2.2 AA? | Partly met | [One is provided](accessibility-statement.md), and it says the software has **not** been audited |
| 6.2 | Does it meet WCAG 2.2 AA? | Not met (not verified) | No audit, no automated accessibility test in the suite, no skip link. See the statement for what is and is not known |
| 6.3 | Is the interface available in the users' languages? | Partly met | 13 languages, with right-to-left support; machine-written and unreviewed until a person records a review ([statement](official-language-statement.md), WPM-R94) |
| 6.4 | Has it been tested with real users? | Not met | No user research has been done |
| 6.5 | Can users adapt text size and appearance? | Met | A text-size picker and a choice of themes including accessible variants (`spec/index.md`) |

## Summary

The count of answers by kind (met, partly met, not met, deployer to answer) is generated
from the tables above into [implementation-status.md](../implementation-status.md), so it
cannot drift from them. The honest reading is that the software has a strong design position
on data minimisation, access control, audit and erasure, and a thin one on independent
assurance (no penetration test, no certification, no accessibility audit, no user
research), and that a large share of the questions are the deployer's to answer.
