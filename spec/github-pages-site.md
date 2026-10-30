# The GitHub Pages documentation site

WPM-R217, WPM-D150; tasks WPM-T392 (built, not published) and WPM-T399 (publish,
open). The family pattern is [monorepo-github-pages](monorepo-github-pages/index.md);
this file says how WPM applies it. The site itself is
[`workforce-planning-management.github.io/`](../workforce-planning-management.github.io/README.md).

## WPM-R217 — A static documentation site, exported by subtree

**As** a reader who does not clone the repository (a chief officer, an assessor, a
contributor deciding whether to), **I want** WPM's own documents and role briefs as
a website in my language, **so that** I can read them without GitHub's file view.

Acceptance criteria:

1. A SvelteKit subproject `workforce-planning-management.github.io/` inside this
   repository, built with `adapter-static` into plain files (every page
   prerendered, no server), Svelte 5 runes only, with the Lily Design System
   pickers the WPM UI uses (`ThemePicker`, `LocalePicker`, `TextSizePicker`).
2. Pages from the root [`index.md`](../index.md) (the home page), `README.md`,
   `INSTALL.md`, [`docs/tutorials/family-bridges.md`](../docs/tutorials/family-bridges.md),
   [`spec/index.md`](index.md), [`spec/worldwide.md`](worldwide.md), and every
   locale's list and role briefs under `locales/<locale>/for/` (21 locales, 11
   briefs each, as of 2026-10-30).
3. A locale switcher on every list and brief that opens **the same page** in the
   chosen locale, matched by its `.locale-peer-id`, because a brief's directory
   name is translated.
4. `<html lang dir>` are the page's locale in the prerendered HTML; an Arabic or
   Urdu page is laid out from the right by logical CSS only (as the UI, WPM-T377).
5. Dates in running text are shown as the UI's `locale-format.ts` shows them
   (WPM-T375, WPM-T377): `formatDate` in the page's locale, two dates as one
   `formatDateRange`, inside `<time datetime>`; a date in code is left as written.
   The site's own figures are `formatInteger`.
6. Axe finds no serious or critical WCAG 2.2 AA violation in light and dark;
   a content security policy with no inline script except SvelteKit's own, by
   hash, and no `style` attribute.
7. `make github-pages` runs the POSIX script `bin/make-github-pages`, which runs
   `git subtree push --prefix=workforce-planning-management.github.io github-pages main`
   after checking that the remote exists, that the document copy is current and
   that the directory has no uncommitted change.
8. CI builds the site and runs its Playwright and axe checks, including `ar-001`.

## WPM-D150 — The site carries a copy, is English around the documents, and is published by the owner

- **A committed copy of the documents.** `git subtree` exports the directory alone,
  so the exported repository has no `../README.md` to read. A script copies the
  documents into `src/content/docs.json`; CI fails if the copy is stale, and the
  publish script refuses a stale copy. One more file to keep current is the price of
  a site that builds by itself.
- **Links.** A link between two documents the site shows stays on the site; any
  other repository link goes to the file on GitHub, so nothing on the site is a
  dead end. Raw HTML in the Markdown is shown as text, never passed through.
- **The site's own words are English.** The navigation, picker labels and footer
  are a handful of strings; translating them would add 21 unreviewed catalogues for
  very little. The documents themselves are in their locales, and the link back to
  a locale's list uses that list's own title.
- **Two themes.** Only Lily's `light` and `dark` are offered, because only those are
  audited; the UI offers all of Lily's.
- **The CSP is a meta tag.** GitHub Pages sends no custom headers, so the policy
  is SvelteKit's `<meta http-equiv>` (no `frame-ancestors`). It is stricter than the
  UI's in one way: no `unsafe-inline` styles.
- **Published by the owner, not by an agent.** Creating the organization's
  `*.github.io` repository, adding the `github-pages` remote and the first push are
  the maintainer's (WPM-T399); the site's README lists the steps. The exported
  repository, and any sibling clone of it, is read-only: it changes only through the
  subtree.

## Not done

See the site's README, "Not done", and WPM-T392 in [tasks.md](tasks.md).
