# Frontend (SvelteKit, Svelte 5)

`workforce-planning-management-ui-with-svelte` — SvelteKit, **Svelte 5 runes
only**, TypeScript strict, SPA mode (`ssr = false`) behind a same-origin BFF
proxy. Stack-level agreements:
[the UI's AGENTS.md](../workforce-planning-management-ui-with-svelte/AGENTS.md).

## Order of work for a new view

1. **API client** in `src/lib/api/wpm.ts` (+ types in `types.ts`). One function
   per endpoint, exact proxied path (`/api/proxy/...`).
2. **Path-map test** — add the call and its expected URL to
   `tests/unit/wpm.test.ts` **in the same order** (the test compares call order).
3. **Page** `src/routes/<area>/+page.svelte` (+ `+page.ts` returning
   `{ title }` — the SharePicker reads `page.data.title`), or a **component** in
   `src/lib/components/` for a panel on `/me` and `/workers/[pid]`.
4. **Nav link** in `NAV_LINKS` (`src/routes/+layout.svelte`) if it is a
   top-level area.
5. **Strings** in every `-001` locale ([localization](localization.md)).
6. **Playwright spec** over stubbed endpoints in `tests/e2e/smoke.spec.ts`.

## Conventions

- **Runes only**: `$state`, `$derived`, `$effect`, `$props`; no stores, no `$:`.
- **Data loads** go in `$effect`s (SPA mode: a `.server.ts` load's `fetch` is
  invisible to Playwright). A loader that depends on inputs returns a cleanup that
  marks the run stale so a late response is dropped.
- **A figure that cannot be loaded or is unknown is "—", never `0`** (`rate()` and
  `mean()` return `null`; render `?? "—"`). Failure of a secondary panel hides the
  panel rather than showing an error over the page.
- **Links** use `l("/path")` (or `localePath`) — never a hard-coded `href="/…"`.
- **Plain text only** for user-authored text; Svelte escapes it. Do not use
  `{@html}` for anything user- or server-authored. Render links only if the URL
  starts with `https://`.
- Masked values render as a state ("Hidden"), never an error.
- Status is **icon + label**, never colour alone (▲ Attention, ✓ Done, ● Info).
- Forms: `data-testid` on inputs and buttons the e2e spec drives; labels from `t()`.

## Charts and dashboards

Follow the data-viz method: one series colour per job from the validated palette
tokens (`--viz-*`, light and dark under `prefers-color-scheme` **and**
`[data-theme]`), thin marks (2 px line, ≥ 8 px markers with a surface ring),
direct labels, a hairline grid, a hover/touch crosshair, and a **visually hidden
data table** — in a clipping `div` (an `overflow: hidden` `<table>` does not
clip). Validate any categorical palette with the dataviz validator before
shipping; check text contrast (≥ 4.5:1).

The CEO dashboard (`/ceo`) is sized for **an iPad, 9th generation: 2160 × 1620
device pixels = 1080 × 810 CSS pixels at 2×**, in `em` off the viewport's short
side. Its Playwright spec checks **two viewports** for page scroll, every tile
inside the screen, **and no tile clipping its own content** (tiles hide
overflow). Keep it green when tiles change, and **look at the screenshots** in
`test-results/`.

## Gotchas

- **Global CSS collisions**: `.tile`, `.hero`, `.chip`, `.panel`, `.muted`,
  `.label` are styled in `src/app.css`. A component-local class with the same name
  inherits those styles — prefix yours (`cx-`).
- Putting a class on `<body>` from a page: use `onMount`/`$effect` with
  `document.body.classList` and a cleanup (there is no `svelte:body` class).
- With `noUncheckedIndexedAccess`, `array[i]` is `T | undefined`; use
  `.at(-1)`, `{#each x.slice(0, n)}` or an `{#if}` guard.
- Playwright strict mode: two controls with the same label on a page break
  `getByLabel`; scope to a form (`getByTestId("…-form")`) or add a `data-testid`.
- Do not run `prettier --write` over a whole existing file just to format a small
  edit — it produces a large unrelated diff.
- **Playwright reuses whatever is already listening on its port** (4173), so a
  preview server from another project makes the specs — and any screenshots —
  run against the wrong app (it once showed a 404 page from an unrelated site).
  Check with `lsof -nP -iTCP:4173 -sTCP:LISTEN`; do not kill a process you did
  not start — set `PW_PORT=4181` (any free port) instead. Specs must not
  hard-code the origin.
- **Look at screenshots, in light *and* dark, at phone width.** The theme is the
  Lily theme picker, not `prefers-color-scheme`: seed it with
  `localStorage["mxi.wpm.theme"] = "dark"` (`addInitScript`). Global `th` styles
  upper-case their text (a band code `8a` shows as `8A`: override per cell) and
  tables wider than the screen need a `overflow-x: auto` wrapper, not squeezed
  columns. Measure `scrollWidth - innerWidth` rather than trusting your eye.
- Playwright can fail nearly everything if started straight after another build
  (its preview server races); re-run it.
