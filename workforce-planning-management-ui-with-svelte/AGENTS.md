# AGENTS.md — working agreements

A pocket guide for human and AI collaborators working in this
subproject. Read this **before** opening a PR. (Updated 2026-10-30,
through WPM-T396.)

## What this project is

A **SvelteKit browser client** for the
[Loco JSON API sibling](../workforce-planning-management-service-with-rust/): hiring
boards, onboarding, time/leave/rotas, the employee record (the **self-service
hub**) and org chart, reviews and 360°s, learning, skills, succession, payroll,
benchmarking, wellbeing and privacy — and, since WPM-T69: `/metrics` (with
insights), `/directory`, `/rota`, `/announcements`, `/skill-gaps`,
`/movements`, `/planning`, `/roles`, `/groups` the one-screen `/ceo`
dashboard and, since WPM-T118, `/capacity`, `/resilience`, `/oversight` (with the monthly
pack), `/cutover` and `/warnings`; later the pay pages (`/pay-gap`, `/pay-information`,
`/pay-range`, `/pay-scales`), `/positions`, `/job-levels`, `/bargaining`, `/consultations`,
`/policies`, `/ai-register`, `/expenses`, `/contingent`, `/decision-gates` and the `/demo`
tour: 56 pages in all (2026-10-30). The bridges to the sibling apps have no screen; the UI
calls none of their routes. Money, numbers, dates and plurals go through the page's locale
(`src/lib/money.ts`, `src/lib/locale-format.ts`, `tn`; [worldwide](../spec/worldwide.md)). The app owns no data; every page round-trips through the API, and
masked fields render as first-class masked states, never errors or fake zeros.

> ⚠️ Demo software, not a production HR system. See
> [regulatory](../spec/regulatory.md).

## Ground rules

1. **Spec first.** [`../spec/`](../spec/index.md) is the single source of truth
   ([auth](../spec/auth.md), [architecture](../spec/architecture.md)); the
   queue is [`../spec/tasks.md`](../spec/tasks.md). A change is requirement +
   design + task + code + tests; the task entry says what was verified and what
   was not.
2. **Stack.** SvelteKit, **Svelte 5 runes only** (no legacy stores or `$:`),
   TypeScript strict, SPA mode (`ssr = false`) with a same-origin BFF proxy.
   Drift from sibling front-ends is accepted; no shared package.
3. **BFF auth.** The server holds the cookie session and exchanges it for
   short-lived PASETO tokens; **no token in browser JS, no localStorage
   credentials**. Browser calls go to `/api/proxy/*`.
4. **Masking honesty.** Salary, payslip amounts and review content may arrive
   masked; render it as a state. **A figure that cannot be loaded or is unknown
   is "—", never `0`** (`format.rate` / `format.mean` return `null`).
5. **Money.** Minor units + currency through `src/lib/money.ts` (`money`,
   `moneyWhole`, `toMinor` for typed amounts): the exponent is the currency's
   own (JPY 0, KWD 3), never a fixed 100, the format is the page's locale, and
   no form starts with a currency (WPM-T373, WPM-T376). It is shared in
   behaviour with the sibling PPM UI's `money.ts`; change both together.
   **Other numbers and dates** go through `src/lib/locale-format.ts`
   (`formatNumber`, `formatPercent`, `formatDate`, `parseNumber`, …), never
   `toFixed`, `toLocale*`, a glued `%` or a raw ISO date in markup; a scan in
   `tests/unit/locale-format.test.ts` fails otherwise (WPM-T375). A decimal
   input is text with `inputmode="decimal"`, read with `parseNumber`.
   A range of dates is `formatDateRange` and a "becomes" arrow is `arrow()`, never a literal "→"; styles
   are logical (`text-align: start`, `margin-inline-*`), and `tests/unit/right-to-left.test.ts` fails on
   a physical left/right (WPM-T377). Today is `todayIso()` (the reader's time zone, never the UTC date),
   and a calendar's week start and weekend come from `weekInfo()` in `src/lib/reader-calendar.ts`
   (WPM-T378, WPM-D149).
6. **Localization is content, and the URL carries the locale** (WPM-R50,
   WPM-D37). Strings live in `content/locales/<locale>/ui.json` (17 locales;
   regional ones hold overrides only), loaded through `src/lib/i18n.svelte.ts`;
   `t(key)` for text, `tp(key, params)` for server-derived text with `{name}`
   placeholders. **Internal links and `goto` use `l("/path")`** (or
   `localePath`) so they keep the `/<locale>/` prefix; do not hard-code
   `href="/workers"`. Add a key to **every `-001` locale** (the parity test
   fails otherwise) and run `pnpm cms-config` (a test fails if
   `static/admin/config.yml` is stale). `en-001` is the source; translations are
   AI-written and unreviewed.
7. **Tests.** vitest for the API path map (add every new client function to
   `tests/unit/wpm.test.ts`, in call order), parity, locale logic and helpers;
   Playwright over a `page.route`-stubbed API (**unstubbed = 404-loud**) — one
   spec per new area; screenshots go to `test-results/` and **should be looked
   at**.
8. **Charts and dashboards** follow the data-viz method: one series colour per
   job from the validated palette tokens (light and dark), thin marks, direct
   labels, a visually hidden table, status as icon + label, never colour alone.
   The CEO dashboard is sized for an iPad (9th gen) and **tested for fit and for
   tile clipping at two viewports** — keep it passing when tiles change.

## Gotchas

- **Global CSS collides**: `.tile`, `.hero`, `.chip`, `.panel`, `.muted` are
  styled in `src/app.css`; component-local classes that reuse those names pick up
  unwanted styles — prefix them (the dashboard uses `cx-`).
- **A hidden table does not clip**: `overflow: hidden` is ignored on
  `<table>`; wrap it in a clipping `div` (it stretched the page's scroll height).
- With `noUncheckedIndexedAccess`, indexed reads are `T | undefined`.
- `ssr = false` means a `.server.ts` load's `fetch` is invisible to Playwright;
  keep data loads in universal loads/`$effect`s.
- `$effect` loaders: return a cleanup that marks the run stale so a late
  response for an older input is dropped.
- Playwright can fail nearly everything if started straight after another
  build; re-run it.

## Running

```bash
pnpm install
pnpm dev                    # expects the Loco sibling on :5150
pnpm check                  # svelte-kit sync && svelte-check (0 errors)
pnpm lint                   # eslint, then prettier --check
pnpm test                   # vitest (308 in 37 files, 2026-10-30)
PW_PORT=4193 pnpm exec playwright test   # 335 pass, 19 skip, stubbed API (2026-10-30); pick a free port
WPM_LIVE_API=1 pnpm exec playwright test -c playwright.live.config.ts   # 23 tests in 10 files against a REAL service (tests/live/README.md)
pnpm build
# The demo (guided tour, fictitious data): see INSTALL.md "Try the demo". WPM_DEMO_API_URL names a SEPARATE
# service instance (WPM_REQUIRE_AUTH=0, nothing real in it, filled by scripts/demo-data.py); signing in as
# example@example.com then opens /demo. Unset = no demo.
pnpm cms-config             # regenerate static/admin/config.yml from en-001
```
