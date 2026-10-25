# Live tests (real service, no stubs)

`live.spec.ts` drives the built UI against a running Loco API. It is skipped unless `WPM_LIVE_API=1`.

1. Start Postgres and the API with enforcement off (a demo posture, never for production):
   `WPM_REQUIRE_AUTH=0 DATABASE_URL=… cargo run -- start` (port 5150).
2. Seed it: `cargo run -- task seed`, then add contingent data through the API: at least ten contractors and ten fixed-term
   people in one department, each with a programme funding (`PUT /api/workers/{pid}/funding`), a plan with demand lines of
   more than one basis and a programme, a fixed-term requisition with a duration, and three
   `cargo run -- task snapshot_headcount as_of:YYYY-MM-DD` runs at least 60 calendar days apart.
   `scripts/` of the scratch run is not kept; the spec reads whatever is there and asserts shapes and rules, not exact numbers.
3. `WPM_LIVE_API=1 PW_PORT=4510 pnpm exec playwright test -c playwright.live.config.ts`

`oversight.spec.ts` (capacity, oversight with a pack, cutover, early warnings, resilience) needs no seed data: each run uses its own
organization reference, typed in because enforcement is off. The service rate-limits writes, so a run straight after another can meet
`429` (the page shows it); wait about forty seconds and rerun that spec.

The fake session cookie only passes the UI's sign-in gate; with enforcement off the API ignores it.

`pay-range.spec.ts` needs `PAY_RANGE_ORG=<organization ref>` naming an organization with at least ten workers on one band with pay and
fewer than ten on another (pay is set in the database; there is no API to set a salary). With enforcement off the service reports no
organization scope, so the spec stubs that one call to name the organization; the report is real. `PAGE_SHOTS=<dir>` also writes a
light, a dark and a phone screenshot. Do not run the service's own request tests against the same database while you work: they truncate its tables
(`cargo run -- start` itself does not).

`pay-gap.spec.ts` (`PAY_GAP_ORG`) and `pay-information.spec.ts` (`PAY_INFO_ORG`) need the service started with
`WPM_EQUALITY_MONITORING_BASIS` and `WPM_EQUALITY_CATEGORIES='{"sex":["female","male"]}'`, workers with pay and a sex answer (set in the
database), and two pay categories. A pay information request is made by the worker through `/api/me`, which needs a token, so the
spec expects two rows inserted in the database: one overdue from a woman in a category with at least ten of each sex, one open from a
man in a category with fewer than ten men. It answers one of them, so seed again to rerun.

`joint-assessment.spec.ts` uses the same set-up as `pay-gap.spec.ts` (`PAY_GAP_ORG`; a category whose gap is at the threshold and one
whose gap is withheld). It writes assessment records, so clear them (`DELETE FROM pay_assessment_records`) or seed again to rerun.
