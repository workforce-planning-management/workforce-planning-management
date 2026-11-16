# Lessons from a Rust rewrite (WPM, proposed 2026-11-16)

> **Status: proposed, not built.** This file is an assessment and a plan. Nothing in it changes how WPM behaves today. The
> requirements (WPM-R248 to WPM-R252) and decisions (WPM-D197 to WPM-D200) are proposed, and each is the owner's to confirm.
> The tasks are WPM-T508 to WPM-T521 in [tasks.md](tasks.md).

## Where the ideas come from

An integration hub for HL7 v2 messages, being rewritten in Rust from Python, in a separate repository that is not
published. It is spec-driven, as WPM is, and it has had to prove that the rewrite behaves like the system it replaces.
That pressure produced engineering practice WPM can use. Its domain (clinical messages, a message bus, transformers) does
not transfer and is left out here.

The repository was read on 2026-11-16: its plan, its agent guides, its testing, configuration, observability, health,
message-store, replay, deployment and local-CI specifications, its cutover runbook and deviation sign-off, its gate task,
its fuzz layout, and the documentation of eight of its crates. Every claim below about what WPM lacks was checked against
WPM's `spec/` and the service's `src/` on the same day.

## Advice, in one paragraph

Take the cheap safety rules first, because each one closes a gap already visible in WPM:

1. a time limit on every CI job;
2. every configuration error reported at start, all at once, and no setting read while a request is being served;
3. a test on every error path that a planted marker never reaches a response or a log.

Then take the three practices that make the specification checkable:

1. a script that lists every requirement no test cites;
2. property fuzzing of the pure code that reads input WPM does not control;
3. golden files for what WPM sends to other systems.

Then build the one missing operational feature the hub shows how to do well: replay of failed webhook deliveries as a
batch an operator chooses, with a visible "gave up" state. Fold the hub's single gate and run record into the local-first
CI work WPM has already planned ([local-ci-cd.md](local-ci-cd.md), WPM-D171) rather than starting it again. Leave the
hub's health check, bus inspector and dashboard: WPM's own are already stronger.

## Ideas assessed

| # | Idea | How the hub does it | What WPM has | Fit | Task |
| --- | --- | --- | --- | --- | --- |
| 1 | A time limit on every CI job | Every hosted job has a time limit, after one run hung for about six hours. | No job in `ci.yml` or `release.yml` sets `timeout-minutes`. | Strong | WPM-T508 |
| 2 | Configuration checked at start, every error at once | Blank means absent for every type. A bad number stops the start instead of falling back to a default. Every problem is printed, one per line, before any port opens. A secret prints as `<redacted>`. Parsers take values, not variable names, so tests leave the process environment alone. | `app.rs` returns at the first bad setting. Three settings are read during a request: `controllers/padr.rs:249`, `controllers/mandatory_learning.rs:74` and `clients.rs:135`. Blank is already treated as absent in `compat.rs`. | Strong | WPM-T509, WPM-T510 |
| 3 | A planted marker never leaks | Each crate that can render text taken from content pushes a distinctive marker through every failure path. The test asserts the marker is in neither the output nor the captured logs. | A few such tests (`tests/request_log.rs`, `tests/requests/webhooks.rs`, `scim_push`); no rule says every module has one. | Strong | WPM-T511 |
| 4 | Every requirement cited by a test | There is no coverage percentage. Each observable clause has a test that names it, and a script checks this. | About 925 citations of WPM-R, WPM-D and WPM-T in `src/`, but nothing lists a requirement that no test cites. `status.py` counts tests, not traced requirements. | Strong | WPM-T512, WPM-T513 |
| 5 | Property fuzzing, with crash seeds kept | The fuzz crate sits outside the workspace (it needs nightly, so it is outside the MSRV). One target for each surface that reads untrusted input and must not panic. Targets assert properties ("returns", "leaks nothing", "idempotent"), never outputs. Each crash is fixed and its input kept as a seed. | None: no fuzz or property tests. The threat model says "not fuzzed". | Strong | WPM-T514 |
| 6 | Golden files for outputs other systems read | Captured once, committed, compared byte for byte or in canonical form. Changing a golden file only to make a test pass is forbidden unless it is a recorded deviation or a spec change. "Golden corpora beat remembered constants": two of four constants written from memory were wrong. | None. The OpenAPI copy test (`openapi::tests::the_ui_copy_is_current`) is the one thing of the kind. | Strong for payroll exports, SCIM and webhook payloads, and statutory examples | WPM-T515 |
| 7 | Replay as a batch an operator chooses | A one-shot job that exits 0 or 1. A person picks the batch beforehand; the job never selects. The original correlation id is kept and a replay id added. Re-running the job is the recovery. A timeout warns that the item may arrive twice. Payloads are never logged. | [integrations.md](integrations.md) lists "a replay of a failed delivery" as not built; [cross-app-bridges.md](cross-app-bridges.md) lists replay of missed events as not done. | Strong | WPM-T516, WPM-T517 |
| 8 | One failure vocabulary | Three outcomes: accepted, rejected (will never succeed, so never retried) and deferred (may succeed later). A message that was not delivered is never marked complete. | Webhooks retry ten times, and any other answer is a refusal. No state, screen or count shows a delivery that ran out of retries. | Partial | WPM-T516 |
| 9 | Telemetry names frozen as a contract | Event and metric names and labels are frozen, because dashboards and alerts depend on them. New ones may be added; renaming is forbidden; tests assert the exact strings. No label ever comes from content, which would leak personal data and blow up cardinality. | [rust-tracing-stack.md](rust-tracing-stack.md) lists the kinds of metric but no names; there is no catalogue. | Partial | WPM-T518 |
| 10 | A register of deliberate departures, with sign-off | Each departure from the old system says what it was, what it is now, why, and where it is specified. The review pack ends each entry with an empty "confirmed by, date, decision" table, because the owning team signs, not the author. | WPM-D decisions record choices. Nothing records a departure from a statute or a standard together with who must accept it. | Partial | WPM-T519 |
| 11 | The MSRV read from its declaration | A task prints the declared `rust-version`, and CI checks with that toolchain, so the check cannot drift from the declaration. | `ci.yml` writes `1.98` by hand, in a comment and a command. | Partial | WPM-T520 |
| 12 | A runbook you follow with the console open | It starts from the property that must hold throughout: rollback needs no data migration. Then, in order: the least costly failure first, one unit at a time, a hold for a full business day, and the old version switched off but never deleted. It gives real commands, says "roll back rather than debug forward", and ends with a "before you call it done" checklist. | [operations/runbook.md](operations/runbook.md) and the upgrade notes, not in this shape. [cutover-readiness.md](cutover-readiness.md) is a report inside the product, not a runbook. | Partial | WPM-T521 |
| 13 | One gate, with a record of each run | Hooks, developers and CI all call one gate task, so none of them keeps its own list of steps. It runs the cheapest steps first, stops at the first failure, and writes a JSON record of each run (sha, clean tree, tiers, durations, toolchain). A local result is self-attested, so the hosted gate stays required. | Proposed already: WPM-T437 (one entry point) and WPM-T441 (the record) in [local-ci-cd.md](local-ci-cd.md). | Strong; already planned | WPM-T437, WPM-T441 |
| 14 | Test isolation learned from flaky runs | Ids unique to each run. Database assertions scoped to the run's own keys, with a row count, after a prefix match was both polluted and vacuous. A two-second check that a container is up, not an eight-minute retry. | Integration suites run one test at a time, which isolates by serialising. | Partial | WPM-T511 (scoped assertions) |
| 15 | Phases gated on probes of risky bets | Phase 0 tests the two technical bets everything rests on, and nothing else starts until both pass. | Phases exist; none is gated on a probe. | Weak now; use it for the next risky dependency | none |
| 16 | A single project risk register and a definition of done | Risks rated by likelihood and impact, each with a mitigation, including process risks such as "a local gate is self-attested". | Risk tables in several topic files; no single register. [spec-driven-delivery](../AGENTS/spec-driven-delivery.md) says what lands with a change. | Weak now | none |

Not taken: the hub's health check, which checks only that a TCP port answers (WPM's `/_health/live`, `/ready` and
`/startup` already do more). Its read-only message inspector: the one idea worth keeping from it is that a debug view
with no redaction says "must not be deployed near real data" in the spec, in the code and on the page. Its dashboard,
which starts with no configuration as WPM's demo mode already does.

## Requirements (proposed)

- **WPM-R248** the service checks its whole configuration before it opens a port: every problem is reported at once, one
  per line, naming the setting and never its value when the setting is a secret; a value that does not parse stops the
  start rather than falling back to a default; and no setting is read from the environment while a request is served.
- **WPM-R249** a webhook delivery that runs out of retries is marked as given up. It is never shown as delivered, and an
  administrator can see the given-up deliveries for each subscription, with the reason, the last answer and the number of
  attempts, but not the payload.
- **WPM-R250** an administrator can replay a batch of given-up deliveries that they choose: each is sent again with its
  original event id and a new replay id, the replay is audited, re-running it is safe, and a payload is never written to
  a log.
- **WPM-R251** every WPM-R that is marked built is cited by at least one test, and a check in CI lists any that are not.
- **WPM-R252** every metric and log event the service emits is listed by name and label in the specification; a name is
  never renamed once published, and no label value is taken from personal data or free text.

## Decisions (proposed)

- **WPM-D197 — Three outcomes for anything WPM sends.** Accepted; rejected, which will never succeed and is not retried;
  and deferred, which may succeed later and is retried with a limit. A delivery is complete only when accepted. When the
  retries run out, a deferred delivery is given up, never dropped. *Rejected alternative:* keep "refusal" for every
  answer that is not success. That retries a request that can never succeed, ten times, and hides the ones that might.
- **WPM-D198 — Replay is chosen by a person, never by the job.** The administrator picks the deliveries, and the job
  sends them and stops. *Rejected alternative:* retry every given-up delivery automatically. A delivery is usually given
  up because the receiver is broken, and sending again on a timer only adds to that receiver's load.
- **WPM-D199 — Settings are read once, at start, and all their errors reported together.** *Rejected alternative:* read a
  setting where it is used. A typo then shows up only when the first request reaches that code, possibly days later, and
  a restart loop shows one error per restart.
- **WPM-D200 — Fuzzing asserts properties, not outputs, and lives outside the workspace.** It needs nightly Rust, so it
  stays outside the MSRV. A target says "returns", "never panics", "leaks no marker" or "parses what it prints". Every crash
  is fixed, and its input is kept as a seed. *Rejected alternative:* fuzzing that compares outputs. That turns the fuzzer
  into a slow unit test and fails on every intended change.

## Risks

- **A traced requirement is not a tested requirement.** A test can name WPM-R123 and check nothing. The trace (WPM-R251)
  finds what is missing; it does not prove what is there. A review still reads the test.
- **Golden files can freeze a mistake.** The hub's rule applies: a golden file changes only with a recorded decision or
  a spec change, never only to make a test pass.
- **Fuzzing costs machine time.** Run it on demand and on a schedule, not on every push. CI replays only the seeds.

## Decision

Proposed. The owner confirms or changes each of WPM-R248 to WPM-R252 and WPM-D197 to WPM-D200 before its task starts.
Four tasks change no behaviour of the product, and can start once the owner says so: WPM-T508 (CI time limits),
WPM-T512 (the trace as a report), WPM-T519 (the register) and WPM-T520 (the MSRV).
