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

The fake session cookie only passes the UI's sign-in gate; with enforcement off the API ignores it.
