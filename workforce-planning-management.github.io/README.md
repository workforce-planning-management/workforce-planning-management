# workforce-planning-management.github.io

The Workforce Planning Management documentation site for GitHub Pages: a static
SvelteKit build (`adapter-static`, Svelte 5 runes, Lily Design System widgets) of
the repository's own documents and role briefs.

This directory lives inside the monorepo,
`workforce-planning-management/workforce-planning-management`, and is
**exported** to its own repository,
`workforce-planning-management/workforce-planning-management.github.io`, with
`git subtree`, as [`../spec/monorepo-github-pages/`](../spec/monorepo-github-pages/index.md)
says. Change the site here; never work in the exported repository. What the site
does, and why, is [`../spec/github-pages-site.md`](../spec/github-pages-site.md)
(WPM-R217, WPM-D150, WPM-T392).

**Status (2026-11-04): published** at
<https://workforce-planning-management.github.io/>. To publish a change, see
"Publishing" below.

## What it shows

| Route | Document |
| --- | --- |
| `/` | the landing page, [`locales/en-001/index.md`](../locales/en-001/index.md) |
| `/<locale>/` | a locale's landing page, `locales/<locale>/index.md` (8 locales) |
| `/<locale>/start/` | the guided tutorial, `locales/<locale>/start/index.md` (8 locales) |
| `/compare/`, `/compare/<slug>/` | [`docs/compare/`](../docs/compare/index.md) (11 products, English) |
| `/docs/` | [`index.md`](../index.md) |
| `/readme/` | [`README.md`](../README.md) |
| `/install/` | [`INSTALL.md`](../INSTALL.md) |
| `/family-bridges/` | [`docs/tutorials/family-bridges.md`](../docs/tutorials/family-bridges.md) |
| `/spec/` | [`spec/index.md`](../spec/index.md) |
| `/worldwide/` | [`spec/worldwide.md`](../spec/worldwide.md) |
| `/<locale>/for/` | a locale's list of role briefs (21 locales) |
| `/<locale>/for/<role>/` | a role brief (11 roles in each locale; the directory name is translated) |
| `/search/`, `/<locale>/search/` | search, in the browser, over the static `/search-index.json` |
| `/sitemap.xml`, `/<locale>/sitemap.xml` | the whole site, and one locale |
| `/404.html` | the not-found page, in each catalogue's language |

- A link between two of these documents stays on the site; a link to any other
  file in the repository goes to that file on GitHub. Raw HTML in the Markdown is
  shown as text.
- The header is the full Lily `PickerBar`: pages, search, theme (all 57), language,
  text size (all 7) and share. Choosing a language opens the same page in that
  locale, matched by its `.locale-peer-id`; a page with no translation of itself opens
  the locale's landing page, or its list of briefs.
- The site's own words are in `src/content/locales/` (`en-001`, `cy-001`, `zh-001`) and
  read through `ui(locale)` in `src/lib/i18n.ts`; a test checks that every catalogue has
  every key. Other locales read English. The translations were written by an AI
  assistant and have not been reviewed by a native speaker.
- A landing page is plain Markdown: a section's list becomes cards, a numbered list
  steps, and a paragraph of only links a row of buttons.
- `<html lang dir>` are the page's locale from the first paint; the CSS uses logical
  properties only (WPM-T377). An ISO date in running text is shown in the page's locale
  inside `<time datetime>`; a date in code is left as written.
- The colour tokens hold 4.5:1 in all of Lily's themes; the tests sweep each one.

## Content

The site does not read `..` when it builds: the exported repository has no
parent. `scripts/sync-content.mjs` copies the documents (and each brief's
`.locale-peer-id`) into `src/content/docs.json`, which is committed. After editing
any of the documents above, run in this directory:

```sh
pnpm run sync          # rewrite src/content/docs.json
pnpm run sync:check    # CI runs this: fails if the copy is stale
```

All 57 Lily theme stylesheets are copied from the installed `@lilydesignsystem/themes`
into `static/assets/themes/` on every install (`scripts/sync-themes.mjs`, as in the
UI) and are not committed.

## Build and test

Node 26 and pnpm.

```sh
pnpm install
pnpm run check                                  # svelte-check
pnpm run build                                  # prerenders every page into build/
PW_PORT=4197 CI=true pnpm exec playwright test  # every page, axe, every picker, every theme, locales, search, sitemaps
```

Another project on the same machine may hold Playwright's default port, 4173;
`PW_PORT` picks a free one. The monorepo's CI job `site` runs the same commands.
`BASE_PATH` (empty by default) serves the site from a sub-path if it is ever
published as a project site rather than the organization site.

## Publishing

Steps 1 to 3 were done once, on 2026-11-04 (WPM-T399); a new clone needs only
step 3. Every later publish is step 4.

1. On GitHub, create the empty public repository
   `workforce-planning-management/workforce-planning-management.github.io` (no
   README, licence or `.gitignore`, so the first push is not rejected).
2. In that repository, open Settings, then Pages, and set Source to
   **GitHub Actions**. The workflow `.github/workflows/deploy.yml` in this
   directory becomes that repository's root workflow; it builds the site and
   deploys it.
3. In the monorepo clone, add the remote (the `.git/config` entry the spec names):

   ```sh
   git remote add github-pages git@github.com:workforce-planning-management/workforce-planning-management.github.io.git
   ```

4. Commit any change to this directory, then from the monorepo root run:

   ```sh
   make github-pages
   ```

   which runs `bin/make-github-pages`: it checks that the remote exists, that
   `src/content/docs.json` is current and that this directory has no uncommitted
   change, then runs
   `git subtree push --prefix=workforce-planning-management.github.io github-pages main`.
5. Watch the "Deploy to GitHub Pages" run in the exported repository; the site is
   then at <https://workforce-planning-management.github.io/>.
6. If you want the read-only sibling export on disk, as the spec describes, clone
   it beside the monorepo and only ever pull into it:

   ```sh
   git clone git@github.com:workforce-planning-management/workforce-planning-management.github.io.git ~/git/workforce-planning-management/workforce-planning-management.github.io
   ```

Every later publish is step 4 again (and a `git pull` in the sibling export).

## Not done

- The site's own words are in English, Welsh and Mandarin only. The other 13 locales
  have no landing page or tutorial, and README, Install, the specification and
  `/compare/` are English. Theme names are English.
- The tutorial's commands come from `INSTALL.md` and have not been run end to end.
- The Bengali briefs write their dates in Bengali digits, not ISO, so those dates
  are left as written.
- Before the theme stylesheet loads (the first paint, or with JavaScript off) the
  page uses the light fallbacks of the tokens, whatever the system preference.
- GitHub Pages cannot send HTTP headers, so the content security policy is a
  `<meta http-equiv>` tag, which cannot carry `frame-ancestors`.
- Axe audits every document and list but one brief per locale (21 of 231).
