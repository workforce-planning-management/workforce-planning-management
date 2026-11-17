# The GitHub Pages website

WPM-R217, WPM-R231, WPM-D150, WPM-D175 (themes: WPM-R247, WPM-D196; what is kept: WPM-R253, WPM-D201); tasks WPM-T454, WPM-T392 (built) and WPM-T399 (published 2026-11-04, at
<https://workforce-planning-management.github.io/>). The family pattern is [monorepo-github-pages](monorepo-github-pages/index.md);
this file says how WPM applies it. The site itself is
[`workforce-planning-management.github.io/`](../workforce-planning-management.github.io/README.md).

## WPM-R217 — A static documentation site, exported by subtree

**As** a reader who does not clone the repository (a chief officer, an assessor, a
contributor deciding whether to), **I want** WPM's own documents and role briefs as
a website in my language, **so that** I can read them without GitHub's file view.

Acceptance criteria:

1. A SvelteKit subproject `workforce-planning-management.github.io/` inside this
   repository, built with `adapter-static` into plain files (every page
   prerendered, no server), Svelte 5 runes only, with the full Lily Design System
   `PickerBar` ([lily-design-system-svelte-with-picker-bar](lily-design-system-svelte-with-picker-bar/index.md)):
   pages, search, every Lily theme, language, every text size and share. Search is a
   prerendered page reading a static index; there is no time zone picker, because the
   site shows no times.
2. Pages from the root [`index.md`](../index.md) (at `/docs/`; the home page is
   WPM-R231's), `README.md`,
   `INSTALL.md`, [`docs/tutorials/family-bridges.md`](../docs/tutorials/family-bridges.md),
   [`spec/index.md`](index.md), [`spec/worldwide.md`](worldwide.md), each locale's privacy notice
   (`locales/<locale>/privacy/index.md` at `/<locale>/privacy/`, WPM-R253), and every
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
6. Axe finds no serious or critical WCAG 2.2 AA violation in light and dark; on
   the home page and `/docs/` each other Lily theme loads, and its axe result is reported, not failed
   ([themes-and-contrast.md](themes-and-contrast.md), WPM-R247);
   a content security policy with no inline script except SvelteKit's own, by
   hash, and no `style` attribute.
7. `make publish-github-pages` runs the POSIX script `bin/publish-github-pages`, which runs
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
- **The site's own words are in catalogues.** Superseded in part by WPM-D175: English,
  Welsh, Mandarin, Arabic and Hindi have a catalogue; every other locale's pages keep English
  chrome around their documents.
- **All of Lily's themes.** Superseded by WPM-D175: the picker bar offers all 63 (57 before WPM-T505), and
  the tests audit each one; since WPM-D196 only light and dark fail the run.
- **The CSP is a meta tag.** GitHub Pages sends no custom headers, so the policy
  is SvelteKit's `<meta http-equiv>` (no `frame-ancestors`). It is stricter than the
  UI's in one way: no `unsafe-inline` styles.
- **Published at the owner's direction.** Creating the organization's
  `*.github.io` repository, adding the `github-pages` remote and the first push are
  the maintainer's decision; the maintainer asked for them on 2026-11-04 and an agent
  ran them (WPM-T399). The site's README lists the steps. The exported
  repository, and any sibling clone of it, is read-only: it changes only through the
  subtree.

## WPM-R231 — A welcoming website: landing pages, a tutorial and comparisons

**As** a visitor who has never heard of WPM, **I want** to be told plainly what it is,
why it might matter to me and how it compares, in my language, **so that** I can decide
whether to try it.

Acceptance criteria:

1. `/` and `/<locale>/` are landing pages: a hero (headline, subheadline, one paragraph,
   the demo notice, actions and a "Translations:" list of `-001` endonyms), benefits,
   features, a comparison, who it is for and how to start.
2. `/<locale>/start/` is a guided tutorial of the demo tour.
3. `/compare/` and `/compare/<slug>/` compare WPM with each named product; each page says
   when to choose the other product, that WPM is a demo, what it rests on, and that WPM
   is not affiliated with the product's owner.
4. Those pages exist for `en-001`, `en-us`, `en-gb`, `en-gb-oxendict`, `cy-gb` (copied to
   `cy-001`), `zh-cn` (copied to `zh-001`), `ar-001` and `hi-in` (copied to `hi-001`), as `locales/<code>/index.md` and
   `start/index.md`, each with a `README.md` symlink and a `.locale-peer-id`; `/compare/`
   is English.
5. On `cy-*`, `zh-*`, `ar-*` and `hi-*` pages the site's own words and wordmark are in that language, and
   `/<locale>/search/` keeps the reader in it. On an Arabic page the header, footer, source line and not-found message read right to left; code reads left to right in every locale.
6. `/sitemap.xml` lists the site and `/<locale>/sitemap.xml` a locale's pages.
7. `/404.html` is the site's own not-found page, in each catalogue's language, because the address does not say which locale the reader wants.

## WPM-D175 — Promotion that stays honest; content stays Markdown

- **A neutral register.** The site's own pages read as a business analyst's description:
  no greetings, slogans, exclamation marks or emoji; a claim is stated with its limit.
- **Promotion does not outrun the evidence.** The demo notice is in every hero, and the
  comparison says the alternatives are for real HR today and WPM is not. A claim about a
  named product rests on [COMPARISONS.md](../COMPARISONS.md) and the dated scan in
  `.sota/`; a commercial product was not code-scanned, its page says so, and no page
  states its price, roadmap or feature count.
- **Content is Markdown, not a template.** A translator edits `locales/<code>/index.md`;
  CSS makes a section's list cards, and the renderer marks a paragraph of only links as a
  row of buttons.
- **The header names four or five places**; the pages picker lists them all.
- **A filled control is the theme's text and panel colours inverted**, and link and muted
  text are mostly the text colour, because a theme's own primary does not hold 4.5:1 in
  all 63 themes.
- **`/` shows `en-001`'s landing page**; the repository's document index moved to `/docs/`.

## WPM-R246 — The path to production, with a readiness estimate on every item

**As** a reader deciding whether WPM could carry real work (a chief officer, an assessor, a deployer), **I want**
one page that says what production use would mean and lists, honestly, what stands between WPM and it, **so
that** "demo software" is a list I can check rather than a label.

Acceptance criteria:

1. `/<locale>/path-to-production/` for every locale (all 21), from
   `locales/<locale>/path-to-production/index.md`, with a `README.md` symlink and a `.locale-peer-id`; the
   language picker opens the same page in another locale, and the build fails if a locale has none.
2. The page says what production use means (real people's records, pay that is paid, daily sign-in to
   sensitive reads, accountability to a regulator), the case it is measured against (the
   [readiness assessment](readiness-assessment.md)) and the three kinds of evidence (software, a deployer,
   people other than the author), before it lists anything.
3. Every item of the honest list carries the author's readiness estimate from 0 to 100 per cent, what the
   estimate rests on and what remains. No item is at 100 per cent, and the estimates are not averaged.
4. Each item has a Lily `Slider` (`@lilydesignsystem/svelte-headless`) from 0 to 100 in steps of 5 that starts
   at the author's estimate, with an accessible name that names the item and a value text in the page's
   locale. A reader may move it; the page then shows the reader's value beside the author's. Nothing is saved
   or sent, and a reload restores the estimates.
5. Every notice that says WPM is not production software says "not yet ready for production use" and links to
   the page: the locale homes, the root documents, the subproject READMEs and agent guides, the comparison
   pages, the DTAC notes, the readiness assessment, the threat model and `llms.txt`.
6. Axe finds no serious or critical violation on every locale's page in light and dark, and the Arabic page
   fits a phone.

## WPM-D194 — Estimates are Markdown, judged and dated; the slider is the reader's

- **The estimate is in the document, not the code.** An item is `- **75%** **Name.** …`: the estimate in bold
  ASCII digits, then the item's name in bold. It reads the same on GitHub as on the site, a translator keeps
  the number and translates the rest, and the build fails on an estimate above 100 or an item with no name.
- **An estimate is a judgement, said to be one.** The page dates it, says what each rests on, and says it is
  neither a measurement nor a certification (rule 3: an estimate says what it rests on). It is the author's,
  so a reader's own view needs a place: the slider.
- **The slider changes nothing.** It is for a reader to try their own estimate while reading. Storing it
  would make the page a record of something nobody reviewed, so it keeps nothing and sends nothing.
- **No average.** One item at 0 per cent keeps WPM out of production whatever the others say, so the page
  shows no total.
- **The in-app notice is not changed here.** The application's own home notice (`home.demoNotice`, 13
  locales) links nowhere yet; every one of its locales now has a site page it could link to.
- **The site's own words stay in five catalogues.** On a locale without a catalogue (WPM-D175) the note above
  the list and the slider's name prefix are English around the translated page.

## WPM-R253 — The site keeps a reader's choices only when they make them, and its privacy notice says what and why

**As** a reader of the website, **I want** my theme and text size to stay as I chose them, and to be told
exactly what the site keeps in my browser and why, **so that** the site reads the way I asked and I can trust
that it keeps nothing about me.

Acceptance criteria:

1. The theme picker is Lily's `ThemePicker` (in `PickerBar`, 0.4.0 or later). It keeps a chosen theme in
   `localStorage` (`mxi.wpm.theme`, the WPM UI's key) **and** in a first-party cookie (`wpm-theme`: `Path=/`,
   `SameSite=Lax`, `Secure` on HTTPS, 400 calendar days), and reads the cookie first, then `localStorage`, then
   the browser's light or dark setting, then light. Either store alone brings the theme back; a cookie that
   names no theme is ignored.
2. A chosen text size is kept in `localStorage` (`mxi.wpm.text-size`) and comes back on the next page; a kept
   value that names no size is ignored.
3. **Nothing is kept until the reader chooses.** A first visit writes no theme, no text size and no cookie. The
   language and locale are in the address and are not kept.
4. `/<locale>/privacy/` in every locale (all 21), from `locales/<locale>/privacy/index.md` with a `README.md`
   symlink and one `.locale-peer-id` ([the English notice](../locales/en-001/privacy/index.md)), says what is
   kept, for the website and for the application, in two tables: each choice and each storage name, the
   application's language keys and sign-in cookie, the content editor's settings, and SvelteKit's per-tab
   `sveltekit:` session keys for scroll positions. It says why, which item stands for the reader (only the
   application's sign-in cookie), who receives it (a cookie goes to the server that set it; no program of
   WPM's reads the website's), how long each lasts, how to remove it, why there is no consent banner, and that
   the records the application holds are the deployer's to give notice of. Every page's footer links to the
   notice in the page's locale (English on the English documents), in the page's catalogue language; the
   pages picker lists it, the language picker opens the same notice in another locale, and the build fails if
   a locale has none. Every translation has the English notice's headings, list items, table rows, links and
   backticked names (checked by a test).
6. **The application's footer links to it.** Every page of the UI ends with a footer whose link, "Privacy
   notice" (`footer.privacyNotice`, in every catalogue), opens `/<locale>/privacy/` on the website in the
   app's locale (`privacyNoticeUrl` in `src/lib/help.ts`). A unit test fails if an app locale has no notice
   on the website, or if a storage key or cookie name in the app's code is missing from the notice.
5. A test chooses a theme and a text size, collects every `localStorage`, `sessionStorage` and cookie name the
   site then holds, and fails if the notice does not name each one; so a new key cannot be added without the
   notice.

## WPM-D201 — Keep a choice in two places, only when it is made, and name every key

- **Cookie and `localStorage`, both.** A browser may clear one and keep the other (a cookie-only clean-up, or
  storage partitioning in a private window). Keeping the theme in both means either alone restores it, and
  the cookie is what a server would need to send a page in the reader's theme before any script runs. GitHub
  Pages runs no server program, so on this site the cookie only restores; it costs a few bytes per request.
  Text size is kept in `localStorage` only, because Lily's `TextSizePicker` offers no cookie.
- **Only a choice is kept.** A value kept on a first visit is not the reader's choice, and keeping it would
  make the site's storage something nobody asked for. `ThemePicker` 0.4.0 writes only on a change.
  `TextSizePicker` 0.3.2 also writes when it first applies a size, so the site reads and writes the text-size
  key itself (`keptTextSize`, `keepTextSize` in `src/routes/+layout.svelte`) until a Lily release stops that.
- **The notice is tested against the browser.** A notice written from memory goes stale the first time a
  dependency adds a key. The test reads what the browser actually holds after the choices are made and checks
  the notice names each key. SvelteKit's `sveltekit:` session keys are listed because they are there, though
  no reader chose them.
- **No consent banner.** Each item is kept at the reader's request to show the site the way they asked, holds
  only the name of a theme or a size, and identifies no one: the user-interface customisation that the
  ePrivacy Directive and the UK's Privacy and Electronic Communications Regulations treat as strictly
  necessary. The notice says this is the project's reading, not legal advice.
- **Rejected:** keeping nothing (every page would start in the default theme and size, which fails readers who
  need a larger size or a dark page); a cookie only (a reader who clears cookies loses the choice, and the
  UI's `localStorage` key would differ); a consent banner (it asks a question about storage that holds
  nothing about the reader, on every first visit); a hand-written list of keys (stale on the next upgrade).
- **One notice for the website and the application, on the website.** Both keep the same kinds of choices,
  and a reader of either should find what is kept in one place, in their language. The application links to
  the website rather than carrying its own copy, so there is one text to translate and keep current; a
  deployment that writes its own notice points `privacyNoticeUrl` at it. The notice covers only what is kept
  in a browser: the records the application holds are the deployer's
  ([assessment checklist](governance/assessment-checklist.md), question 3.10).
- **The application is described as it is.** It still uses the earlier picker bar, so it keeps the theme and
  text size on a first visit, and it keeps the language of each page opened. The notice says so rather than
  claiming the website's behaviour for both.
- **Every locale has the notice, like the path to production.** It is a page in `locales/<locale>/` with a
  peer id, so the language picker, sitemap and search treat it like every other translated page. The
  translations were written by an AI assistant and are not reviewed (WPM-T155).

## Not done

Machine translations (Welsh, Mandarin, Arabic, Hindi) have had no native-speaker review. The other 10
locales have no landing page or tutorial, and English chrome around their briefs; README,
Install, the specification and `/compare/` are English. Theme names are English.

See the site's README, "Not done", and WPM-T392 in [tasks.md](tasks.md).
