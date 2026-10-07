# Workforce Planning Management — SvelteKit front-end

The browser client for the
[Loco JSON API sibling](../workforce-planning-management-api-with-rust/):
HR, manager, and employee self-service views over the full
employment lifecycle — hiring boards, onboarding, time and leave,
rotas with working-time and ergonomic-issue panels, the employee
record and org chart, wellbeing prompts and the anonymous pulse,
reviews and 360° appraisals, notifications, reasonable adjustments,
training and learning, succession, payroll runs, salary
benchmarking, and the privacy/retention admin area — and, since the original
delivery, workforce metrics and insights, an employee directory, an on-call
rota, announcements, skill gaps with a training plan, joiners and leavers, and a
**one-screen CEO dashboard** sized for an iPad.

> ⚠️ **Demo software.** Not a production HR system; synthetic data
> only. See [spec/regulatory](../spec/regulatory.md).

**Status: implemented through WPM-T97 (2026-07-18 → 2026-10-07).**
svelte-check clean (0 errors, 0 warnings); **80 vitest + 45 Playwright specs**
pass (`page.route`-stubbed — runs without the Rust service); the production
build is green. Quick start: `pnpm install && pnpm dev` (expects the Loco
sibling on :5150; `pnpm test` / `pnpm exec playwright test`).

## Environment variables

Server-side only (`src/lib/server/config.ts`); the browser never
sees these — every request goes through the same-origin BFF proxy.
See `.env.example`.

| Variable | Default | Meaning |
| --- | --- | --- |
| `WPM_API_URL` | `http://localhost:5150` | workforce-planning-management-service base URL the proxy forwards to |
| `AUTH_API_URL` | `http://localhost:5150` | authentication-service base URL — magic-link login + session→PASETO exchange |
| `WPM_PUBLIC_URL` | _(blank)_ | public address of this site for `/sitemap.xml` and `/robots.txt`; blank = the request origin (set it behind a proxy) |

Both default to the same port because every family service defaults
to `:5150`; point them at distinct ports/hosts when running the
auth service and this service side by side.

## Stack

SvelteKit 2 · Svelte 5 runes · TypeScript strict · SPA mode with a
same-origin BFF proxy (session cookie → short-lived PASETO; no token
in browser JS) · **17 content locales under `/<locale>/` routes** (e.g.
`/en-001/workers`, `/cy-001/workers`; `/en/…` redirects to `/en-001/…`) with
strings as content in `content/locales/<locale>/ui.json` edited through
**Sveltia CMS** at `/<locale>/admin/` · Lily Design System (headless +
ThemePicker + LocalePicker) · inline-SVG charts following the data-viz method ·
vitest + Playwright (`page.route`-stubbed).

### Sitemap and robots.txt

`/sitemap.xml` is generated per request: every public page (`/`, `/tour`,
`/signin`) in every locale — 17 × 3 URLs — each with `hreflang` alternates for its
sibling locales and `x-default` → `en-001` (a `<language>-001` locale is its bare
language; `en-gb` is `en-GB`). `/robots.txt` names it and keeps crawlers off the
API, the CMS and sign-out. Signed-in pages are never listed (they redirect to
sign-in); the list of public pages is shared with the sign-in gate
(`src/lib/publicPages.ts`), so the two cannot drift. Code: `src/lib/sitemap.ts`
(pure, unit-tested) and the two `+server.ts` routes.

### Localization in practice

```sh
pnpm dev                         # then open /en-001/ , /cy-001/ , /de-001/ …
pnpm cms-config                  # regenerate static/admin/config.yml from en-001
pnpm cms-config:check            # fails if it is stale (a unit test does too)
```

A new UI string goes in `content/locales/en-001/ui.json` **and** every other
`-001` locale (the parity test fails otherwise); regional locales (`en-gb`,
`en-us`, `de-de`, `es-es`) hold only overrides. Use `t("key")` for text,
`tp("key", { n })` for server-derived text with `{n}` placeholders, and
`l("/path")` for every internal link so the locale prefix is kept. See the
[locales spec](../spec/locales-for-global-sharing-with-svelte/index.md).

## Views

| Area               | Views                                                                       |
| ------------------ | --------------------------------------------------------------------------- |
| Talent acquisition | requisition board (SVAR Kanban), application pipeline, candidate pool, onboarding tracker |
| Workforce          | team approvals, department rota, working-time flags, ergonomic issues       |
| HR core            | employee list (SVAR grid) + profile (masked salary unless entitled), org chart, benefits |
| Wellbeing          | `/wellbeing`: entitlement rules + aggregate uptake & conversion + pulse results; `/privacy`: retention report + sweep |
| Development        | review panel, `/learning` (skills matrix, analytics, paths), `/mentorship`, succession + gap report |
| Payroll            | run screen (draft → calculated → approved), payslips, benchmarking table    |
| People and cover   | `/directory` (search; away / covered by / on call), `/rota` (on-call rotation, swaps, swap requests), `/announcements` (feed, editors' post form), emergency contacts + backups + my on-call on the profile |
| Skills and training | `/skill-gaps` (workforce gaps, training time, catalogue editor), "My skill gaps" + training plan on the profile, `/skills`, `/roles`, `/planning`, `/groups`, `/cpd` |
| Joiners and leavers | `/movements` (open records with progress), `/movements/{pid}` (dated checklist, last-day handover, audit trail) |
| Leadership         | `/ceo` (six tiles, one screen, no scrolling — 2160 × 1620 px, an iPad 9th gen), `/metrics` (shared metrics + insights, period picker) |
| Self-service (profile) | my record + payslips + leave + reviews, wellbeing prompts, pulse card, notifications, my 360 requests, 360 panel, ergonomics checklist, reasonable adjustments, "Download my data", erase (terminated only) |

The employee **profile page is the self-service hub** — most
per-person features surface there as panels; masked fields render as
first-class masked states (an em dash / "Hidden"), never as errors
or fake zeros.
