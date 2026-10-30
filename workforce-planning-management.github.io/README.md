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

**Status (2026-10-30): built, not published.** No `github-pages` remote is
configured, the exported repository does not exist yet, and nothing has been
pushed. The steps to publish are below.

## What it shows

| Route | Document |
| --- | --- |
| `/` | [`index.md`](../index.md) |
| `/readme/` | [`README.md`](../README.md) |
| `/install/` | [`INSTALL.md`](../INSTALL.md) |
| `/family-bridges/` | [`docs/tutorials/family-bridges.md`](../docs/tutorials/family-bridges.md) |
| `/spec/` | [`spec/index.md`](../spec/index.md) |
| `/worldwide/` | [`spec/worldwide.md`](../spec/worldwide.md) |
| `/<locale>/for/` | a locale's list of role briefs, [`locales/<locale>/for/index.md`](../locales/en-001/for/index.md) (21 locales) |
| `/<locale>/for/<role>/` | a role brief, `locales/<locale>/for/<role>/index.md` (11 roles in each locale; the directory name is translated) |

- A link between two of these documents stays on the site; a link to any other
  file in the repository goes to that file on GitHub. Raw HTML in the Markdown is
  shown as text.
- On a list or a brief, the header's language button (Lily `LocalePicker`) opens
  the same page in another locale, matched by its `.locale-peer-id` (the directory
  names differ: `chief-risk-officer`, `prif-swyddog-risg`, `رئيس-المخاطر`).
  `<html lang dir>` are the page's locale from the first paint, so an Arabic or
  Urdu page is laid out from the right; the CSS uses logical properties only, as
  the WPM UI does (WPM-T377).
- An ISO date in running text ("2026-10-11") is shown as the WPM UI's
  `locale-format.ts` shows one, in the page's locale ("11 Oct 2026",
  "11.10.2026"), and two dates joined by "to" or a dash as one range
  (`formatRange`, the locale's own separator), inside `<time datetime>`. A date in
  code is left as written. The site's English pages use `en-001`.
- The theme follows the system's light or dark preference until the reader picks
  (Lily `ThemePicker`, the `light` and `dark` themes only); the colour tokens are
  the WPM UI's. Text size uses Lily `TextSizePicker`.

## Content

The site does not read `..` when it builds: the exported repository has no
parent. `scripts/sync-content.mjs` copies the documents (and each brief's
`.locale-peer-id`) into `src/content/docs.json`, which is committed. After editing
any of the documents above, run in this directory:

```sh
pnpm run sync          # rewrite src/content/docs.json
pnpm run sync:check    # CI runs this: fails if the copy is stale
```

The Lily theme stylesheets are copied from the installed `@lilydesignsystem/themes`
into `static/assets/themes/` on every install (`scripts/sync-themes.mjs`, as in the
UI) and are not committed.

## Build and test

Node 26 and pnpm.

```sh
pnpm install
pnpm run check                                  # svelte-check
pnpm run build                                  # prerenders every page into build/
PW_PORT=4197 CI=true pnpm exec playwright test  # every page, axe in light and dark, locale switch, right to left, dates
```

Another project on the same machine may hold Playwright's default port, 4173;
`PW_PORT` picks a free one. The monorepo's CI job `site` runs the same commands.
`BASE_PATH` (empty by default) serves the site from a sub-path if it is ever
published as a project site rather than the organization site.

## Publishing

The owner of the `workforce-planning-management` GitHub organization does this
once; nothing in this repository has done any of it.

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

- The site's own words (navigation, picker labels, footer) are English only; the
  lists and briefs are in their locales, and "back to the list" uses the list's
  own title.
- The Bengali briefs write their dates in Bengali digits, not ISO, so those dates
  are left as written.
- Only the `light` and `dark` Lily themes are offered and audited; the UI offers
  all of Lily's.
- Before the theme stylesheet loads (the first paint, or with JavaScript off) the
  page uses the light fallbacks of the tokens, whatever the system preference.
- GitHub Pages cannot send HTTP headers, so the content security policy is a
  `<meta http-equiv>` tag, which cannot carry `frame-ancestors`.
- Axe audits every document and list but one brief per locale (21 of 231).
- The deploy workflow has never run, and the published site has not been seen.
