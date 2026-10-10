# Readiness for production use in large medical and governmental organizations (assessment of 2026-10-11)

Requirements WPM-R114–R123, decisions WPM-D71–D73, tasks WPM-T187–T203 (see [plan.md](plan.md)
section L and [tasks.md](tasks.md) Phase 20). Generic by design: no organization is named, and the
standards named are public examples a deployer's own assurance team may replace.

> ⚠️ **Verdict: not ready.** This is demonstration software, and this assessment says what stands between
> it and a deployment where the records are those of patients' carers and public employees. One finding
> (section 2) was a **blocker** and is fixed in WPM-T188; the others are listed with evidence, and the
> things only people can do are listed as gates. Nothing here is legal advice or a certification.

## 1. What an assessor will find in the repository's favour

Each of these is a fact about the code or a document, checked when it was written; none is a claim that the
deployment meets a standard.

- **Sign-in is enforced by default** and production refuses to start without it or without a token key
  source (WPM-R72). Security headers, an explicit CORS list and rate limiting exist (WPM-R73, R74).
- **Data minimisation by design:** identities are URNs, there are no demographic columns, scores advise
  and do not decide, and roll-ups name no one (see [privacy-and-data-rules](../AGENTS/privacy-and-data-rules.md)).
- **Subject access, erasure and retention per record kind**, with an erasure ledger so a restore does not
  bring an erased person back (WPM-R89, R91, D63).
- **A tested restore** (the drill in CI), a runbook, deployment and upgrade documents (WPM-R89, R90).
- **A governance pack** to adapt: data protection impact assessment, data flow, record of processing,
  retention, an assessment checklist, an accessibility statement (WPM-R92). Honest about gaps (WPM-D61).
- **Dependency policy:** `cargo deny`, a software bill of materials on release, vendored shared crates.
- **Sign-in with an enterprise directory** (OIDC, Entra-compatible) tested in-process (WPM-R93).

## 2. The blocker found by this assessment: reads were not need-to-know

The blanket policy decides by **action**, and the shipped reference policy lets *every signed-in caller*
read, masking only money. Record-level checks exist in some controllers and not in others. A probe was run
against the real service under enforcement with a caller who holds **no attributes at all** (the plainest
possible signed-in user) and a synthetic worker:

| Read by a signed-in stranger | Result before WPM-T188 |
| --- | --- |
| `GET /api/workers/{pid}/leave-requests` | `200`, including `kind: sick` **and the free-text reason** |
| `GET /api/workers/{pid}/time-entries` | `200`, including the notes ("hospital appointment" in the probe) |
| `GET /api/candidates` | `200`, names and e-mail addresses of applicants |
| `GET /api/audits/recent`, `GET /api/events/recent` | `200`, every recent audit entry and event |
| `GET /api/workers/{pid}` | `200`, a colleague's employment record (salary masked) |
| `leave-entitlements`, `development-plans`, `assessment-profile`, `wellbeing-prompts`, … | `200` |
| `emergency-contacts`, `subject-access`, `job-level`, `pay-position`, `engagement` | `403` (these had record-level checks) |

For an organization that holds health-related absence and applicants' personal data, a regime of
"any employee may read any colleague's sickness record" is disqualifying, and a regulator would call it a
failure of access control and of data protection by design. The cause was structural: the policy engine
sees an action and an entity, not the record, so only code that loads the record can decide need-to-know,
and many routes did not. WPM-T188 adds a guard with a table in code (WPM-D71).

## 3. Gaps, with evidence and what is planned

| # | Gap | Evidence | Planned |
| --- | --- | --- | --- |
| 1 | **Fixed (WPM-T188):** reads were not need-to-know | Section 2 | **WPM-R114, T188 (built)** |
| 2 | **Fixed, with limits (WPM-T189):** the audit trail could be altered or deleted by anyone with database write access; nothing proves it is complete | `audit_logs` has no trigger, rule or hash; the migration is a plain table | WPM-R115, T189 |
| 3 | **Fixed (WPM-T190):** a diagnosis could be stored: sick-leave `reason` is free text | The probe above | WPM-R118, D73, T190 |
| 4 | **Fixed (WPM-T191):** the database connection could be plaintext, and a request may hang | `config/production.yaml` has no TLS setting or request timeout; deployment docs do not mention `sslmode` | WPM-R116, T191 |
| 5 | **Fixed (WPM-T192):** backups were protected by file permissions only | `scripts/backup.sh` says to encrypt at rest and does not | WPM-R117, T192 |
| 6 | No threat model | None in `spec/` | **WPM-T193 (built):** [governance/threat-model.md](governance/threat-model.md) |
| 7 | Supply chain: actions pinned by tag, no secret scanning, no image signature or provenance, base images by tag | `.github/workflows/*.yml` | WPM-R119, T194, T195 |
| 8 | No evidence the service holds up under load; several views load an organization's records into memory | `BENCHMARKS.md` records no measurement; `controllers/capacity.rs` and `planning.rs` load whole organizations | WPM-R120, T196 |
| 9 | Observability: a request id is on every response (confirmed on the running service, WPM-T191), but there is no check that logs hold no personal data, audit is not exportable to a security monitoring system | No test; loco's default request id not confirmed | WPM-R122, T197 |
| 10 | Rate limits are per instance | Documented in WPM-R74 | WPM-T198 (document the limit; shared store is deployer's) |
| 11 | The application connects with whatever role `DATABASE_URL` names; no least-privilege role set is provided | `spec/operations/deployment.md` | WPM-R123, T199 |
| 12 | Token revocation and maximum token age are the identity provider's, undocumented here | `spec/auth.md` | WPM-T200 (document and check the maximum age) |
| 13 | Human assurance has not happened | Section 4 | WPM-R121, T201–T203 |

## 4. What only people can do (gates, not code)

These stay open until a person records them done, and the repository must not look more finished than it is
(WPM-D61, D64).

- An **independent penetration test** of a deployed instance, and remediation of what it finds.
- An **independent accessibility audit** against WCAG 2.2 AA with assistive technology users.
- The deployer's **data protection impact assessment** signed off by their data protection officer, and their
  lawful-basis mapping, retention schedule and processor agreements (WPM-G5).
- A **professional review of the translations** (WPM-T155).
- **CI, release and the dynamic scan run on the hosting platforms**, and branch protection set (they are
  written, not yet run).
- A **restore drill on the deployer's platform** within the last 90 calendar days (WPM-G4).
- **Clinical or domain safety review** only where a deployer's use needs it. WPM holds workforce records, not
  clinical records, and takes no clinical decision; a deployer that connects it to a clinical system decides.
- **Equality impact assessments** approved by the deployer ([equality-impact/](equality-impact/template.md)).

## 5. What this assessment did not check

Dependency vulnerabilities against a live advisory feed; behaviour under load; the container image's runtime
behaviour on a hardened host; the UI under assistive technology; the correctness of any statutory
calculation (payroll tax is a stub, WPM-D5); and any regime that applies to a particular jurisdiction or
sector. Each is the deployer's or a later task's.

## 6. Status at the end of the first round (2026-10-11)

**Built and verified** (each with tests that were broken on purpose to prove they fail): the need-to-know guard
(T188), the append-only hash-chained audit trail (T189), no diagnosis in sick leave (T190), refusal of a plaintext
database connection and a request timeout in production (T191), encrypted backups (T192), and a threat model
(T193). **Open and written down:** supply-chain pinning and signing (T194, T195), load evidence (T196),
observability checks (T197), the multi-instance and database-role guidance (T198, T199), token age (T200), and
the independent tests and audits (T201–T203). **The verdict stands:** the one blocker is fixed, but the gates in
section 4 are still open, and a first round by the author is not an independent review.

