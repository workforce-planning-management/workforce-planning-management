# The GitHub Pages website

WPM-R217, WPM-R231, WPM-D150, WPM-D175 (themes: WPM-R247, WPM-D196); tasks WPM-T454, WPM-T392 (built) and WPM-T399 (published 2026-11-04, at
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
6. Axe finds no serious or critical WCAG 2.2 AA violation in light and dark; on
   the home page and `/docs/` each other Lily theme loads, and its axe result is reported, not failed
   ([themes-and-contrast.md](themes-and-contrast.md), WPM-R247);
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
- **The site's own words are in catalogues.** Superseded in part by WPM-D175: English,
  Welsh, Mandarin, Arabic and Hindi have a catalogue; every other locale's pages keep English
  chrome around their documents.
- **All of Lily's themes.** Superseded by WPM-D175: the picker bar offers all 57, and
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
  all 57 themes.
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

## Not done

Machine translations (Welsh, Mandarin, Arabic, Hindi) have had no native-speaker review. The other 10
locales have no landing page or tutorial, and English chrome around their briefs; README,
Install, the specification and `/compare/` are English. Theme names are English.

See the site's README, "Not done", and WPM-T392 in [tasks.md](tasks.md).
