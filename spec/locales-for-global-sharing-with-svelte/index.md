# Locales for major projects with SvelteKit

Translate content into multiple locales.

How this site supports multiple locales end to end: content, web
routing, UI chrome, and bugs.

Read locales via file `locales.tsv`.

Locale code priority order:

- en
- cy
- zh
- sp
- ar

## .locale-peer.id file

`.locale-peer-id` file is a byte-identical 32-character hexadecimal lowercase
number then newline, across every locale's version of "the same" topic,
regardless of slug.

`.locale-peer-id` id is how the project resolves "this page, in locale X".

## Guidance

- en-us: consistent American spelling; fix any stray en-gb forms (organisation→organization, licence→license, programme→program, cancelled→canceled, analogue→analog).

- en-gb: the -ize/-ise family (optimise, realise, organise, prioritise, utilise, etc.), -or/-our (colour, behaviour, favour, labour, neighbours), -er/-re (centre, theatre for the metaphorical sense), -ense/-ce (defence, licence), doubled-L forms (modelled, labelled, cancelled, enrol/enrolment), analogue, programme, and math→maths.

- en-gb-oxendict: use en-gb then revert just the -ise family back to Oxford -ize spelling (optimize, realise→realize, organise→organize, etc.), while correctly keeping -yse forms (analyse/analysable) unchanged, since Oxford style never uses -yze, and keeping all other British forms (colour, centre, defence, licence, programme, maths, modelled) intact.

## Guard against corruption

Keep proper nouns unconverted. Example: "Hospital Readmissions Reduction Program" (a real United States federal program name).

## Verify

For each locale subdirectory:

- File exists: `index.md`
- Symlink exists: `README.md`
- Locale peer id tracking file exists: `.locale-peer-id`

Then:

- Fix any broken internal links
- Fix any residual wrong-dialect spellings
- Update `./spec/locale/index.md`

## Content structure (book side)

Each locale is `locales/<code>/` in the book repo, containing:

- `locales/<code>/topics/<slug>/index.md` + `.locale-peer-id` — one per topic.
  `README.md` is a symlink to `index.md`.
- `locales/<code>/index.md` + `.locale-peer-id` + `README.md` symlink — the
  locale's own translated README (site home/contents page source). Every
  locale gets this file scaffolded (matching the topic-file pattern) even
  before it has a translation; it starts empty.

## Slugs

Slugs are per-locale, not shared.** Translated locales rename topic directories
to native-script/accented slugs.

Example: `es-001` `año-de-vida-ajustado-por-calidad`, `ur-001` `صحت-ایڈجسٹڈ-متوقع-زندگی`.

Nothing in the site assumes slugs match across locales.

## Locale picker (labels + ordering)

- Labels live in `locales.js`'s `LOCALE_LABELS`, one entry per code, in that
  language (e.g. `'fr-001': 'Français (Monde)'`). Falls back to the raw code
  via `localeLabel()` if a code has no label yet.
- Header `PickerBar` order comes from `content.js`'s `locales()` (sorted by
  code) — the `-001` suffix happens to sort before any letter-starting
  regional suffix, so variants already come first there.
- Home page's locale list (`+page.server.js`) sorts explicitly: default
  locale first, then grouped by language name (label text before the `(`),
  with the `-001`/World variant sorted before its regional siblings within
  each group, then alphabetically by label. This does NOT fall out of
  alphabetical-by-label sort on its own (e.g. "España" < "Mundo") — it needs
  the explicit `-001` check.

## Bug fixes (regression watch-list)

### Bug: ASCII-only `\w` regexes broke every non-Latin/non-accented slug

Bug: matched topic slugs with `[\w.-]+` (ASCII word chars only). Any locale with
an accented or native-script slug (Spanish, French, Russian, Chinese, Arabic,
Welsh, Hindi, Bengali, Portuguese, Indonesian, Urdu) silently failed peer-id
resolution and cross-topic links.

Fix by widening the slug capture group to `[^/]+`.

### Bug: Every locale's home/contents page showed canonical English content

Bug: code and content always read a single top-level `/README.md` for title,
intro, "New here?" picks, part headings, and blurbs — only topic _links_ were
ever localized.

Fix: populate the previously-empty `locales/<code>/index.md` per locale.

## Bug: Link extraction was hardcoded to literal English phrase

Bug: link silently found nothing once the README was translated.

Fix: extract all links from the whole pre-`##` intro block instead of
regex-matching the English sentence.

### Bug: UI chrome was hardcoded English in the `.svelte` templates

Bug: nav labels, subtitles, page titles, intros, breadcrumbs, topic position,
pagination, picker/share labels.

Fix: add `i18n.js` and threading `ui(locale)` through every locale-scoped route
and `+layout.svelte`.

### Bug: header/footer brand wordmark stayed English

Bug: wordmark came only from the root (locale-agnostic) `+layout.server.js`,
which deliberately never picks a locale.

Fix: have `locales/[locale]/+layout.server.js` supply this locale's own title,
which overrides the root layout's canonical one via SvelteKit's merged
`page.data` on any route under `/locales/<locale>/` — the root picker and
`/about/` (no locale in the URL) correctly keep the canonical English title.

## How WPM applies this (WPM-R50, WPM-D37)

The front-end (`workforce-planning-management-ui-with-svelte`) applies the
guidance above to an **application**, not a book — the UI strings are the
content.

- **Content locales** (17): `ar-001 bn-001 cy-001 de-001 de-de en-001 en-gb
  en-us es-001 es-es fr-001 hi-001 id-001 pt-001 ru-001 ur-001 zh-001`, one
  directory each at `content/locales/<locale>/ui.json`. A `<language>-001`
  file is complete (the parity test pins every key); a **regional** locale
  (`en-gb`, `en-us`, `de-de`, `es-es`) holds only its **overrides** and falls
  back to its `-001` base, then `en-001`. The files nest on the key dots
  (`{"nav": {"workers": "…"}}`) because that is the shape the CMS edits;
  the app flattens them at load.
- **Routes:** every page is served under its locale — `/en-001/workers`,
  `/cy-001/workers` — by a SvelteKit `reroute` hook (`src/hooks.ts`); the
  route tree is unchanged. A bare language **alias** (`/en/…`, `/cy/…`)
  **301-redirects** to its `-001` locale (one canonical address per locale);
  an unprefixed URL **302-redirects** to the remembered
  (`wpm-locale` cookie) or `Accept-Language` locale, else `en-001`. `/api`,
  `/_app`, `/assets`, files with an extension, and the SSO and sign-out
  endpoints are exempt. The **URL is the source of truth** for the UI
  language; the picker navigates to the same page under the new prefix; the
  server sets `<html lang dir>`.
- **Sveltia CMS** at `/<locale>/admin/` (e.g. `/de-001/admin/`) edits the
  strings in the repository. `static/admin/config.yml` is **generated** by
  `pnpm cms-config` from `en-001` (i18n `multiple_folders`; every field
  optional so a regional locale can hold just its overrides) and a unit test
  fails if it is stale. Sveltia has no setting for its own interface
  language, so the shell page writes the `sveltia-cms.prefs` value from the
  URL's locale, mapped by `CMS_UI_LOCALE` onto the 29 languages Sveltia ships
  (Bengali, Welsh, Hindi, Indonesian and Urdu have none: English).
- **Dynamic text:** server-derived findings (insights, WPM-R49) arrive as a
  `code` plus `params`; the UI renders them from
  `insights.<code>.observation|suggestion` strings and falls back to the
  server's English for a code it does not know.
- **Verification:** `pnpm test` pins the parity of every `-001` locale,
  the alias and prefix logic (`src/lib/locales.ts`) and the CMS config;
  Playwright pins the redirect, the picker navigation and RTL direction.
- **Known limits:** the strings were translated by an AI assistant and have
  had no native-speaker review; `/admin/` needs a GitHub login and has not
  been exercised end to end in a browser.
