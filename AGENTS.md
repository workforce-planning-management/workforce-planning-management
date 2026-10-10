# AGENTS.md — start here

Working agreements for humans and AI agents on this repository. This file is
the short entry point; the detail is in [`AGENTS/`](AGENTS/). Each subproject
also has its own [API](workforce-planning-management-service-with-rust/AGENTS.md)
and [UI](workforce-planning-management-ui-with-svelte/AGENTS.md) guide for
stack-level detail. (Updated 2026-10-07, through WPM-T97.)

> ⚠️ **Demo software.** Not a production HR or payroll system; synthetic data
> only. See [spec/regulatory.md](spec/regulatory.md).

## The five rules everything else follows from

1. **Spec first.** [`spec/`](spec/index.md) is the single source of truth. A
   behavioural change is a requirement + a design decision + a task + code +
   tests, landed together. → [spec-driven-delivery](AGENTS/spec-driven-delivery.md)
2. **Pure core first.** Rules are DB-free and clock-free, tested exhaustively,
   *then* wired into controllers. → [backend-rust](AGENTS/backend-rust.md)
3. **What must not be stored gets no column; what must not be disclosed gets no
   endpoint.** An unknown is never a zero; roll-ups name no one; an estimate says
   what it rests on. → [privacy-and-data-rules](AGENTS/privacy-and-data-rules.md)
4. **Verify, then say exactly what you verified.** Run the suites; look at
   screenshots; record what was *not* checked. →
   [testing-and-verification](AGENTS/testing-and-verification.md)
5. **The URL carries the locale and strings are content.** →
   [localization](AGENTS/localization.md), [frontend-svelte](AGENTS/frontend-svelte.md)

## Map

| File | Read it when you… |
| --- | --- |
| [AGENTS/spec-driven-delivery.md](AGENTS/spec-driven-delivery.md) | start any change; write a requirement, decision or task entry |
| [AGENTS/backend-rust.md](AGENTS/backend-rust.md) | add or change anything in the service |
| [AGENTS/frontend-svelte.md](AGENTS/frontend-svelte.md) | add or change anything in the UI |
| [AGENTS/privacy-and-data-rules.md](AGENTS/privacy-and-data-rules.md) | add a table, a field, an aggregate or a read |
| [AGENTS/localization.md](AGENTS/localization.md) | add a string, a locale, or touch routing or the CMS |
| [AGENTS/testing-and-verification.md](AGENTS/testing-and-verification.md) | run, add or report tests |
| [AGENTS/git-and-releases.md](AGENTS/git-and-releases.md) | branch, commit, push, merge or publish |
| [AGENTS/docs-and-agent-files.md](AGENTS/docs-and-agent-files.md) | update documentation, `llms.txt`, or these files |

## Where things are

- `spec/` — requirements (`WPM-R*`), decisions (`WPM-D*`), tasks (`WPM-T*`), topic files.
- `workforce-planning-management-service-with-rust/` — Loco JSON API (`src/rules/`, `src/controllers/`, `migration/`, `tests/`).
- `workforce-planning-management-ui-with-svelte/` — SvelteKit client (`src/routes/`, `src/lib/`, `content/locales/`, `tests/`).
- `.sota/` — the 2026-10-05 benchmark scan record (`last-scan.json`, the rubric).
- [`llms.txt`](llms.txt) / [`llms.json`](llms.json) — a map of the repo for agents.
