# Request for comments

What this project wants to learn, and the feedback that helps most.
Send comments to <joel@joelparkerhenderson.com> or open an issue.

## Open questions we want outside views on

- **Statutory calculation correctness.** Payroll tax tables and the
  48-hour/11-hour working-time flags are illustrative stubs (UK WTR
  shape). What would a real jurisdiction-correct implementation need
  to get right first? See [spec/payroll-compensation.md](spec/payroll-compensation.md)
  and [spec/workforce-management.md](spec/workforce-management.md).
- **Equality-law review of scoring.** Application screening and
  succession-readiness scoring are areas where real deployments would
  need equality-law review. What should the spec require before any
  such scoring ships beyond demo status?
- **Subject rights coordination.** Access/erasure/retention (WPM-R30)
  is implemented within WPM's own tables; a real deployment would need
  this coordinated with the upstream identity services (person,
  worker, organization, course). What does that coordination contract
  need to look like? See [spec/regulatory.md](spec/regulatory.md).
- **`employed_by` edge emission.** Should hire/termination write the
  registry's cross-service link automatically, and at what
  consistency guarantee? See [spec/roadmap.md](spec/roadmap.md) and
  [spec/integrations.md](spec/integrations.md).
- **Comparisons.** Systems missing from [COMPARISONS.md](COMPARISONS.md)
  that this should be evaluated against.

## Feedback that is always useful

- A spec claim you found to be untrue in code (the repo treats this as
  a first-class defect).
- A masked field rendered as an error or a fake zero instead of a
  first-class masked state — that is a bug, not a display quirk.
- Deployment experience reports: what broke, what was unclear.
