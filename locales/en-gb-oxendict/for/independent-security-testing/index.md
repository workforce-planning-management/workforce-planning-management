# Independent security testing

**The question you are likely asking:** how do we get people who did not write this software to attack it, what should
we point them at, and what do we do with what they find?

No independent test has been done. The [threat model](../../../../spec/governance/threat-model.md) was written by the
same assistant that wrote the software and was reviewed by no one else, and every control it names was exercised by
that author. The [readiness assessment](../../../../spec/readiness-assessment.md) lists an independent penetration test
as a gate only people can close (WPM-T201, **not done**). This page is a brief for whoever commissions one. It is not a
test plan from a tester and it does not replace one.

## What to give the testers

| Item | Where it is | Notes |
| --- | --- | --- |
| A deployment to attack | Yours: [deployment guide](../../../../spec/operations/deployment.md) | Production settings with sign-in **on**, a test identity provider, and **synthetic data only**. Do not test against real people's records |
| The source | This repository | A test with the source is more thorough than a black-box one; say which you want |
| The threat model | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Twenty-four threats, the control as built, and the gaps. Give it to them **after** a first black-box pass if you want to see what they find unprompted |
| The API description | `GET /api-docs/openapi.json` | Every route, with the `Idempotency-Key` on every operation that changes something |
| The sign-in design | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personas, attributes, and what the policy decides by action only |
| Test identities | You create them | At least: a worker with no attributes, a line manager, a manager outside the chain, HR, payroll, an administrator, and a service peer |

## What to ask them to look at

The author could not check these. Each row names the threat model's own entry, so a finding can be traced.

| Area | What to try | Threat |
| --- | --- | --- |
| Tokens | Algorithm confusion, a key set that rotates while requests are in flight, an expired or not-yet-valid token, a token for another audience | T1, T2 |
| Need to know | Every read route as a caller with no attributes and as a neighbour; a route the classification table does not know; a write that the policy allows but the handler should refuse | T3, T4, T6 |
| Privileged attributes | Whether an attribute in a token can be added, replayed or inherited; who can mint one; a shared or long-lived administrator | T24 |
| Injection | Fuzz every parameter, path segment and body field; the erasure and retention statements are built from typed identifiers and were read, not fuzzed | T12 |
| The browser client | Session handling by the interface server, how it exchanges a session for a token, cross-site scripting through stored text, a content security policy bypass, the content manager's page that loads a script from a public network without a pinned version | T13, T14 |
| Outbound requests | Server-side request forgery through any configured URL (key sets, the sibling services, the shared cache) | T15 |
| Limits | The body limit, the request timeout, the rate limits per instance, and the views that load a whole organization's records into memory | T16, T17 |
| Logs | Whether any personal data, token or query string reaches a log | T19 |
| Idempotency keys | A key used by two callers; the same key with a changed body, path or query; a key that is almost valid; a very large body; two requests racing under one key (the claim is reasoned, not raced by a test); whether a stored response can be read by anyone but its caller | new |
| The audit trail | Any way to change, remove or skip an entry; the chain verification endpoint | T7 |
| Backups | Restore from a tampered or wrongly keyed backup; whether an erased person returns | T10, T11 |
| Supply chain | Pinning of build actions, secret scanning, image signing | T20, T21 |

## Rules to settle before they start

Fill these in with your own security and legal teams. None is decided here.

- **Authority:** who signs that the test is authorised, and for which systems and addresses.
- **Window:** the dates and hours, counted in calendar days, and who to call to stop it.
- **Scope:** what is in (the API, the interface, the sign-in flow, the backup script) and what is out (the identity
  provider's own service, your network, anything shared with other systems).
- **Data:** synthetic only. If a tester finds real personal data, who is told and how it is handled.
- **Destructive tests:** whether denial of service, bulk writes and erasure are allowed, and where.
- **Reporting:** the format, the severity scale, who receives it, how long they may keep it, and how secrets they
  find are handled.
- **Retest:** that the contract includes checking the fixes.

## What to do with the findings

1. Record each as a task in [tasks.md](../../../../spec/tasks.md) with the tester's reference, the evidence, and the
   severity they gave. A finding that is disputed stays recorded as disputed, with the reason.
2. Fix with a test that fails first. Where the fix is a rule, add it to the threat model's gap column and close it
   there only when the retest passes.
3. Do not mark WPM-T201 done until the report is in hand, every finding has an owner, and a retest is complete. Update
   the [readiness assessment](../../../../spec/readiness-assessment.md) and the gates list (WPM-T203) in the same change.
4. Keep the report out of the repository if it holds anything exploitable; link to where it is held.

## What a clean report does and does not mean

A test with no findings shows that these testers, in this window, with this scope and these identities, found nothing.
It does not show the absence of flaws, it does not cover a configuration you changed afterwards, and it does not
replace the other gates: an accessibility audit, your data protection impact assessment signed off, and a restore drill on
your platform.

## What this page does not do

It does not name a supplier, set a price or promise a standard. Which scheme or accreditation your sector expects of a
tester is for you to decide.

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Counts that change are in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration
software with synthetic data. Nothing here is legal advice.
