# The path to production

WPM is demonstration software. This page says what production use would mean for it, and lists what stands between WPM and that use. Each item carries an estimate of how ready it is, from 0 to 100 per cent.

## What production means

Production use means an organisation relies on WPM for real work:

- the records are those of real people: employees, applicants and their emergency contacts;
- pay calculated by WPM is paid, and the law of the place of work applies to it;
- people sign in every day, and some of what they read is sensitive, such as absence for sickness;
- the organisation answers to a regulator, an auditor and the people in the records for how it is run.

This page measures WPM against the hardest case it was assessed for: a large medical or governmental organisation ([the readiness assessment](../../../spec/readiness-assessment.md)). A smaller organisation may need less of some items, but it needs the same kinds of evidence.

Production needs three kinds of evidence, and software alone supplies only the first:

1. **Software** that does the job safely, with tests that show it.
2. **A deployer** who hosts and operates it: backups, monitoring, support and incident response.
3. **People other than the author** who have checked it: a penetration tester, an accessibility auditor, translators and the deployer's data protection officer.

## How to read the estimates

- An estimate is the author's judgement on 2026-11-13, made from the specification and the tests. It is not a measurement and not a certification.
- 0 per cent means nothing exists yet. 100 per cent means built, tested and checked by someone independent on a real deployment. No item is at 100 per cent.
- Each item says what its estimate rests on and what remains.
- The estimates are not averaged. One item at a low estimate is enough to keep WPM out of production.

## The software

- **75%** **Sign-in and need-to-know access.** Sign-in is on by default, and the service refuses to start in production without it (WPM-R72). Reads are limited to people with a need to know (WPM-T188), and a token has a maximum age (WPM-T200). Rests on: the enforcement tests that run in CI. Remains: there is no token revocation list, and no independent penetration test has been run.
- **70%** **A complete and unaltered audit trail.** The audit trail is append-only and hash-chained, and can be exported to a monitoring system (WPM-T189, WPM-T197). Rests on: tests that were broken on purpose to prove that they fail. Remains: a truncated tail of the chain is not detected, and a database owner can still disable the triggers.
- **70%** **Data protection by design.** People are identified by references, there are no demographic columns, and there is subject access, erasure and retention for each kind of record. Rests on: the tests, and section 1 of the readiness assessment. Remains: the deployer's own assessment and legal basis (below).
- **75%** **Encrypted connections and backups.** In production the service refuses a plaintext database connection and times out a request that hangs (WPM-T191), and backups are encrypted (WPM-T192). Rests on: the tests of both. Remains: the management of keys and certificates belongs to the deployer.
- **10%** **Statutory calculations.** The statutory calculations are illustrative, and payroll tax is a stub ([the regulatory position](../../../spec/regulatory.md)). Rests on: the specification, which says so. Remains: the rules of each place of work, written and checked by a qualified person. Do not pay anyone from WPM.
- **50%** **Scale.** On one laptop, the directory pages and the capacity view do not fail at 50,000 workers, and the planning views refuse more than 10,000 (WPM-T196, WPM-T230). Rests on: one measured run ([BENCHMARKS.md](../../../BENCHMARKS.md)). Remains: the capacity view still grows in proportion to the workforce, and nothing has been measured on a deployer's hardware.
- **65%** **Supply chain.** Every CI action is pinned to a commit and the history is scanned for secrets. The container image is signed and carries its provenance, and this was run for release v0.1.2 (WPM-T194, WPM-T195). Rests on: that release run. Remains: the image package is private, so nobody outside the project can verify it yet.
- **60%** **Monitoring.** Every response carries a request id, and log lines carry no query string or personal identifier (WPM-T197). Rests on: the tests of the request log. Remains: dashboards and alerts belong to the deployer, and rate limits apply to one instance only (WPM-T198).
- **60%** **Backup and restore.** Encrypted backups, a restore drill that runs in CI, and a runbook. Rests on: the drill. Remains: a restore drill on the deployer's own platform within the last 90 calendar days (gate WPM-G4).
- **40%** **Accessibility checks.** Automated checks against WCAG 2.2 AA run on the application and on this site, in light and dark themes. Rests on: the Playwright and axe tests. Remains: the independent audit (below); no screen reader has been used.
- **20%** **Translations.** The application has 25 locales and the role briefs have 21. Rests on: the text exists. Remains: an AI wrote the translations, and nobody has reviewed them professionally (WPM-T155).

## Checks by people other than the author

- **0%** **Independent penetration test.** No penetration test of a deployed instance has been run (WPM-T201). Rests on: the task is open. Remains: the test, and fixing what it finds.
- **0%** **Independent accessibility audit.** No audit against WCAG 2.2 AA with people who use assistive technology has been done (WPM-T202). Rests on: the task is open. Remains: the audit, and fixing what it finds.
- **25%** **Threat model.** A starting threat model exists ([threat-model.md](../../../spec/governance/threat-model.md)), written by the author. Rests on: the document. Remains: an independent team redoes it for a real deployment.

## The deployer's part

- **20%** **Data protection impact assessment.** A pack to adapt exists: an impact assessment, a data flow, a record of processing and a retention schedule ([spec/governance](../../../spec/governance/dpia.md)). Rests on: the pack. Remains: there is no deployer, so no data protection officer has signed off an assessment, a legal basis, a retention schedule or a processor agreement (gate WPM-G5).
- **15%** **Equality impact assessments.** A template and a register exist ([equality-impact](../../../spec/equality-impact/template.md)). Rests on: the template. Remains: the deployer writes and approves an assessment for its own use.
- **40%** **Hosting and operation.** A production configuration, a container image, and documents for deployment and upgrades exist. Rests on: those documents and the restore drill. Remains: nothing has been run on a hardened host, and a deployer has to operate the service.
- **10%** **Support.** There is an issue tracker and a security policy ([SECURITY.md](../../../SECURITY.md)). Rests on: those two. Remains: there is no support contract, service level, on-call rota or organisation behind WPM.

## What this page does not claim

The page records no decision to go live. That decision belongs to a deployer and is made outside WPM. The estimates cover only the items listed, and the readiness assessment's own list of what it did not check applies here too. Where a clinical system is connected, a clinical safety review is the deployer's decision.
