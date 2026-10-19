# For the Chief Technology Officer

**The question you are likely asking:** is it well built, can my team change it safely, and how do I know?

## What you get

| Concern | What exists | Read |
| --- | --- | --- |
| Stack | Rust with Loco (Axum, SeaORM) and PostgreSQL 18 for the API; SvelteKit 2 with Svelte 5 and TypeScript for the client | [architecture.md](../../../../spec/architecture.md) |
| Design rule | A pure core: rules are database-free and clock-free, tested exhaustively, then wired into controllers | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Spec first | A change is a requirement, a decision, a task, code and tests together; each task entry says what was **not** verified | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Tests | Unit tests on the rules, database-backed request tests, enforcement binaries run with sign-in on, front-end unit tests and browser tests. Mutation checks: break the code on purpose and confirm a test fails | [testing.md](../../../../spec/testing.md) |
| Counts | Generated from the files, never copied by hand | [implementation-status.md](../../../../spec/implementation-status.md) |
| Supply chain | `cargo deny` for licences and bans, version-pinned shared crates, a software bill of materials on release | [production-readiness.md](../../../../spec/production-readiness.md) |
| API contract | OpenAPI, and a test that fails if a registered route is undocumented | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Access control | The policy decides by action only; handlers and a classified route table decide need to know, and a source scan fails on an unclassified read route | [auth.md](../../../../spec/auth.md) |
| Performance | [BENCHMARKS.md](../../../../BENCHMARKS.md) records what was measured; no load test exists | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Known engineering debt

- Several views load whole organizations into memory.
- Row locks on conversion plans are reasoned about, not proven by a test.
- CI, release and the dynamic scan are written but have not run on the hosting platform.
- Actions are pinned by tag, not by digest, and images are not signed.
- Some password and TLS Redis tests need variables and a certificate authority, so CI does not run them.
- The `spec/` tree is large; a generated status file is the way to see what is declared against what passes.

## How to contribute

Read [AGENTS.md](../../../../AGENTS.md) first: five rules, then a map by task.

See also the [Chief Information Officer](../chief-information-officer/index.md) page for the operations view.

## How current this is

Written on 2026-10-11 from the specification, not from a run of the software for this page. Figures that change
(tests, migrations, tasks done) are not copied here; read them in
[implementation-status.md](../../../../spec/implementation-status.md), which a script generates. WPM is demonstration software
with synthetic data: this page says what it does and does not do, and is not a claim that it is fit for your
organization. Nothing here is legal advice.
