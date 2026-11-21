# Security policy

Version 2.1.0, status is active.

Canonical location is `SECURITY.md` at the repository root. A machine-readable
pointer to it is at <https://workforce-planning-management.github.io/.well-known/security.txt>
([RFC 9116](https://www.rfc-editor.org/rfc/rfc9116)).

Review at least once a year, and at every release that changes a supported
version (§3).

Abstract: this document says how to report a vulnerability in this software, what
happens to a report, how long each step should take, what is in and out of scope,
which versions receive fixes, how a fix is disclosed and credited, and what a
good-faith researcher can rely on. It also says what a deployer must configure,
and which protections are built in. It is written for security researchers, for
deployers, and for evaluators doing supplier due diligence. It changes in the same
commit as the practice it describes.

The key words **shall**, **should** and **may** are used as ISO/IEC Directives
Part 2 defines them: requirement, recommendation, permission.

> ⚠️ **Demo software, not yet ready for production use.** Synthetic data only.
> One person maintains this software, with no paid support and no service-level
> agreement. The times in §5 are what the maintainer commits to try for, not a
> contract. [The path to production](locales/en-001/path-to-production/index.md)
> says what production use would need.

## 1. Terms

- **Vulnerability:** a weakness in this repository's code, configuration, build
  or release that lets someone break confidentiality, integrity or availability
  beyond what [spec/auth.md](spec/auth.md) allows. Examples: reading a masked field,
  naming a person in a roll-up, writing as another actor, or getting past the
  audit trail.
- **Reporter:** the person who sends a report.
- **Maintainer:** the person named in [MAINTAINERS.md](MAINTAINERS.md).
- **Deployer:** an organization that runs this software. A deployer is
  responsible for its own environment ([spec/governance/threat-model.md](spec/governance/threat-model.md)).
- **Advisory:** a published GitHub security advisory for this repository.

## 2. Scope

**In scope:**

| Component | Where |
| --- | --- |
| The service (the JSON API) | `workforce-planning-management-service-with-rust/`, including the vendored `crates/entity-ref` |
| The browser client | `workforce-planning-management-ui-with-svelte/` |
| The worker-record matcher crate | `workforce-planning-management-matcher-with-rust/`, and its copy on crates.io |
| The documentation site | `workforce-planning-management.github.io/`, and <https://workforce-planning-management.github.io/> as built from it |
| The build and release | `.github/workflows/`, `compose.yaml`, `deploy/`, `scripts/`, and the container image the release workflow publishes to GHCR |
| The specification | `spec/`, where a stated control is wrong or a documented setting is unsafe |

**Out of scope.** Report these elsewhere, or not at all:

- The sibling applications PPM (project-portfolio-management) and CRM
  (contact-relationship-management). They are separate repositories with their own
  policies.
- A deployment this project does not run. Report it to the organization that
  runs it.
- A vulnerability in a dependency that this software does not make worse. Report
  it upstream. Please also tell us if this software uses the affected path.
- `compose.yaml` and the demo setup with sign-in turned off
  (`WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH=0`). These are open on purpose, and the documentation says
  they must never be exposed. A way to turn sign-in off by accident, or a
  production start with it off, is in scope.
- Findings that need physical access to a deployer's machine, a compromised
  administrator account, or social engineering of the maintainer.
- A volumetric denial of service against GitHub, GHCR, crates.io or the site's
  host. A request that costs the service far more than it should is in scope.
- Missing hardening with no exploit, such as a header the site cannot send because
  GitHub Pages sets the headers. These are welcome as ordinary issues.

## 3. Supported versions

This is pre-1.0 software. Only the versions below receive security fixes. An
older version shall not receive a backported fix.

| Component | Supported | Not supported |
| --- | --- | --- |
| The service and its container image | The latest `v0.1.x` tag, and `main` | Any earlier tag |
| The matcher crate | The latest version published on crates.io, and `main` | Any earlier version |
| The browser client | `main` (it has no published release) | — |
| The documentation site | The published site, and `main` | — |

## 4. How to report

Use either private route:

- **GitHub:** the repository's Security tab, **Report a vulnerability**
  (<https://github.com/workforce-planning-management/workforce-planning-management/security/advisories/new>).
  The report is a draft advisory that only the reporter and the maintainer can see.
- **Email:** <joel@joelparkerhenderson.com> with **"Security:"** at the start of
  the subject line.

Then:

- A reporter **shall not** open a public issue, discussion, pull request or
  commit for a vulnerability that has not been disclosed (§6).
- A report **should** include: the affected component (§2) and version or commit;
  what an attacker could do; steps to reproduce, or a proof of concept; and the
  configuration used, for example whether `WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH` was on and which
  ABAC policy was mounted.
- A report **shall not** contain real personal data. This software holds only
  synthetic data, and a reproduction needs nothing else. If real data was
  exposed, describe it without copying it.
- There is no PGP key and no bug bounty. For a report that must not travel by
  email, use GitHub's private reporting above.

## 5. What happens to a report

| Step | Target time, from receipt |
| --- | --- |
| Acknowledge the report, from a person | 5 business days |
| Triage: in or out of scope, a severity, and a plan, sent to the reporter | 10 business days |
| A fix released for a **critical** or **high** issue | 30 calendar days |
| A fix released for a **medium** issue | 60 calendar days |
| A fix released for a **low** issue | 90 calendar days |
| Status updates to the reporter, while the issue is open | At least every 14 calendar days |

- **Severity** is the CVSS v4.0 base score (critical 9.0–10.0, high 7.0–8.9,
  medium 4.0–6.9, low 0.1–3.9). This project raises a score in two cases. Any
  disclosure of a masked field, or of a named person in a roll-up, is at least
  **high**. So is any way to change the audit trail without detection. This is
  because the data is pay, health-adjacent and employment records
  ([spec/governance/dpia.md](spec/governance/dpia.md)).
- The fix lands as a requirement, decision, task, code and tests together, as
  every change does ([AGENTS/spec-driven-delivery.md](AGENTS/spec-driven-delivery.md)).
  The tests include one that fails without the fix. The public commit shall not
  describe the exploit before §6 allows it.
- If a target will be missed, the maintainer shall tell the reporter before it
  passes, with a reason and a new date.
- A report that is not a vulnerability gets an answer saying why. With the
  reporter's agreement, it may become an ordinary public issue.

## 6. Disclosure

This project uses coordinated disclosure, following ISO/IEC 29147.

- The details are published when a fix is released, or **90 calendar days after
  the report**, whichever comes first. The reporter and the maintainer may agree
  to a different date. An issue already being exploited in public may be
  disclosed sooner, with whatever mitigation exists.
- Each fixed vulnerability gets an advisory on this repository. The advisory
  names the affected and fixed versions, the severity and the mitigation. It also
  gets a CVE id, requested through GitHub as the CNA, when the issue affects a
  published release or the matcher crate on crates.io. A matcher crate issue is
  also reported to the RustSec advisory database.
- The fix is listed in [CHANGELOG.md](CHANGELOG.md) and in the affected
  subproject's changelog under "Security".
- The reporter is credited by name, or by a handle, in the advisory and the
  changelog, unless they ask not to be.

## 7. Safe harbour

The maintainer shall not pursue or support legal action against a researcher who
acts in good faith under this policy. Good faith means the researcher:

- tests a copy they run themselves, with the synthetic data this repository
  provides, and not a deployment run by someone else;
- tests the published site only with ordinary requests, and does not attack
  GitHub's infrastructure;
- does not access, change or keep data that is not theirs, and stops and reports
  at once if they reach some;
- causes no lasting harm, and does not degrade a service for other people;
- gives the maintainer the time in §6 before disclosing; and
- does not ask for payment in return for not disclosing.

This covers only the maintainer's own rights. It does not bind a deployer, GitHub
or any other party.

## 8. What deployers must know

**Sign-in is enforced by default.** `WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH` gates authentication and
authorization in the service, and it defaults to **on**. Only an explicit
`0`/`false`/`no`/`off` turns it off. While it is off, the service logs a warning
and reports it at `GET /_posture`. The `production` environment refuses to start
with it off, or with no token key source.

A deployment exposed to untrusted callers **shall** configure
`WORKFORCE_PLANNING_MANAGEMENT_PASETO_KEYS[_URL]` (or `WORKFORCE_PLANNING_MANAGEMENT_KEYCLOAK_JWKS_URL`) and mount a real ABAC policy
before it is reachable. The container demo in `compose.yaml` turns sign-in off on
purpose and must never be exposed. See [spec/auth.md](spec/auth.md) for how to
turn sign-in on, and [spec/operations/deployment.md](spec/operations/deployment.md)
for every setting. [spec/regulatory.md](spec/regulatory.md) says what production
would also require.

No real personal data belongs in the repository, its seeds, or any deployment
used for demonstration ([spec/regulatory.md](spec/regulatory.md)).

A deployer should subscribe to this repository's security advisories, and should
complete its own threat model from [spec/governance/threat-model.md](spec/governance/threat-model.md).

## 9. Protections in place

**In the software:**

- Security headers on every response: CSP with `frame-ancestors 'none'`,
  `nosniff`, referrer, permissions and cross-origin policies, HSTS over TLS, and
  `no-store` on API responses. Both the API and the UI send them (WPM-R73).
- A rate limit for each caller, with a stricter class for the erase, sweep,
  import, merge and token routes. The answer is `429` with `Retry-After`. The
  counters are in memory and keep no address or subject (WPM-R74, WPM-D54).
- CORS is an allow-list in production (`WORKFORCE_PLANNING_MANAGEMENT_CORS_ORIGIN`), never `*`. The metrics
  endpoint needs a bearer token.
- No `unsafe` in the service crate (`#![forbid(unsafe_code)]`).
- Salary, payslips, review content, 360 reports, assessment scores, the words of
  an adjustment request, and succession plans are masked under ABAC. Their reads
  are audited, and they are never logged. See [spec/auth.md](spec/auth.md) and
  [spec/audit.md](spec/audit.md).
- The audit trail is append-only and hash-chained in the database. An endpoint and
  a task verify it.
- What must not be stored gets no column. No health cohort, symptom, diagnosis or
  pulse-author column exists (WPM-D17, D20, D24, D25).
- The front-end holds the session cookie on the server and exchanges it for
  short-lived PASETO tokens. No token or credential reaches browser JavaScript or
  `localStorage`.

**In the build and release** ([.github/workflows/](.github/workflows/)):

- Every action and container image is pinned to a digest, and
  `scripts/check-actions-pinned.py` fails the build if one is not. Every job has
  a time limit (`scripts/check-job-timeouts.py`).
- Every push runs `cargo deny` (advisories, licences, sources), gitleaks over the
  whole history, zizmor on the workflows, and an OWASP ZAP baseline scan of the
  running service.
- The service's security, enforcement and least-privilege database suites run on
  every push. Many of their checks were mutation-checked: the code was broken on
  purpose to confirm a test fails ([spec/testing.md](spec/testing.md)).
- A release builds without a cache. It attaches a CycloneDX bill of materials,
  signs the image with cosign (keyless), and adds SLSA build provenance. Both are
  verified before the release is created.

## 10. Credentials and publishing

The release workflow uses only the run's own `GITHUB_TOKEN`, and its OIDC
identity for signing. It holds no long-lived secret. No registry token for
crates.io or npm is held in CI.

The service crate and the matcher crate may be published to crates.io, and the
front-end package to npm. Under [AI_STATEMENT.md](AI_STATEMENT.md) §5 and §6, AI
tooling may run `cargo publish` or `npm publish` for a version bump that is
already merged and already decided, once the §6 checklist is met. Whoever runs it,
maintainer or AI, uses a credential stored locally: a crates.io API token or an
npm auth token. These are long-lived secrets. They shall stay out of the
repository, out of logs and out of CI.

## 11. Review and change

The maintainer owns this policy. It is reviewed at least once a year, and revised
off-cycle when a claim in it stops being true, or when a supported version, the
reporting route or a target time changes. A change lands as a commit like any
other. The version, Annex A, Annex B and the `Expires` field of `security.txt`
change in the same commit.

## 12. References

**Normative for this project** (these bind the practice described here):
[GOVERNANCE.md](GOVERNANCE.md), [MAINTAINERS.md](MAINTAINERS.md),
[CONTRIBUTING.md](CONTRIBUTING.md), [AI_STATEMENT.md](AI_STATEMENT.md),
[spec/auth.md](spec/auth.md), [spec/audit.md](spec/audit.md),
[spec/regulatory.md](spec/regulatory.md),
[spec/governance/threat-model.md](spec/governance/threat-model.md).

**Informative** (the structure and positions here draw on these): ISO/IEC
29147:2018 (vulnerability disclosure) and ISO/IEC 30111:2019 (vulnerability
handling); RFC 9116 (`security.txt`); CVSS v4.0 (FIRST); the OpenSSF guide to
coordinated vulnerability disclosure for open-source projects; the CISA and
disclose.io safe-harbour wording; the ISO/IEC Directives Part 2 verbal forms.

## Annex A. Change log

| Version | Date | Change |
| --- | --- | --- |
| 1.0.0 | 2026-09-28 | First issue: the reporting address, the scope, what deployers must know, the protections in place, publishing. |
| 2.0.0 | 2026-11-17 | Made formal (WPM-T525): terms; scope that covers the matcher crate, the site and the release, with what is out of scope; supported versions; what a report contains; target times and severity; coordinated disclosure, advisories, CVE ids and credit; safe harbour; the build and release protections; review; references; `security.txt`. Corrected "no CI-driven publish step": the release workflow publishes the container image, using only `GITHUB_TOKEN`. |
| 2.1.0 | 2026-11-17 | GitHub's private vulnerability reporting is on, beside email (WPM-T537): §4, Annex B and `security.txt`. |

## Annex B. Machine-readable summary

The prose above is authoritative where the two could ever disagree.

```yaml
security-policy:
  version: 2.1.0
  last-updated: 2026-11-17
  contact: mailto:joel@joelparkerhenderson.com
  subject-prefix: "Security:"
  private-vulnerability-reporting: true
  encryption: none
  bug-bounty: false
  supported:
    service: latest v0.1.x tag, and main
    matcher-crate: latest on crates.io, and main
    ui: main
    site: published, and main
  targets-business-days:
    acknowledge: 5
    triage: 10
  targets-calendar-days:
    fix-critical-or-high: 30
    fix-medium: 60
    fix-low: 90
    update-interval: 14
    disclosure-deadline: 90
  severity: cvss-4.0-base, with masked-field, named-person and audit-tamper issues at least high
  disclosure: coordinated, iso-iec-29147
  advisories: github-security-advisories
  cve: github-cna
  credit: by default, unless declined
  safe-harbour: true
```
