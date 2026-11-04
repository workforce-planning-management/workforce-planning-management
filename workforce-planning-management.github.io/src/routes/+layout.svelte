<script lang="ts">
  import "../app.css";
  import type { Snippet } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import type { ComponentProps } from "svelte";
  import PickerBar from "@lilydesignsystem/svelte-picker-bar";
  import { href } from "#lib/href.js";
  import { formatInteger } from "#lib/locale-format.js";
  import { fill, ui } from "#lib/i18n.js";
  import {
    DEFAULT_BRIEF_LOCALE,
    DOC_PAGES,
    LOCALE_LABELS,
    REPO_URL,
    ROOT_HOME_LOCALE,
    SITE_LOCALE,
    hasHome,
    isBriefLocale,
    localeLanding,
    isRtl,
    pickerLocales,
  } from "#lib/site.js";
  import type { LayoutProps } from "./$types";

  // The WPM UI's storage keys, so the choices read the same way (each origin keeps its own).
  const THEME_KEY = "mxi.wpm.theme";
  const TEXT_SIZE_KEY = "mxi.wpm.text-size";

  let { data, children }: LayoutProps & { children: Snippet } = $props();

  // The role-brief locale of this page (null on the English documents).
  const locale = $derived(page.data.locale ?? null);
  const dir = $derived(locale && isRtl(locale) ? "rtl" : "ltr");
  const alternates = $derived(page.data.alternates ?? {});
  const pickable = $derived(pickerLocales(locale));
  // The site's own words, in the language of the page's locale (`ui`, English where there is no catalogue yet).
  const s = $derived(ui(locale));

  // Keep <html lang dir> in step on client-side navigation; the prerendered HTML already carries them
  // (hooks.server.ts).
  $effect(() => {
    document.documentElement.lang = locale ?? "en";
    document.documentElement.dir = dir;
  });

  // The locale switcher opens the same page in the chosen locale: the brief with the same `.locale-peer-id`, or the
  // locale's list. A page with no translation of itself (the English documents) opens the locale's home, or its list
  // where it has no home yet. The search page keeps the query. The picker also reports the locale it starts on, which
  // is this page's own: nothing to do then.
  function switchLocale(code: string) {
    if (!isBriefLocale(code) || code === (locale ?? SITE_LOCALE)) return;
    void goto(`${href(alternates[code] ?? localeLanding(code))}${page.url.search}`);
  }

  const home = $derived(locale && hasHome(locale) ? `/${locale}/` : "/");
  const nav = $derived([
    { label: s.nav.home, route: home },
    { label: s.nav.start, route: `/${locale && hasHome(locale) ? locale : ROOT_HOME_LOCALE}/start/` },
    { label: s.nav.briefs, route: `/${locale ?? DEFAULT_BRIEF_LOCALE}/for/` },
    { label: s.nav.compare, route: "/compare/" },
    { label: s.nav.docs, route: "/docs/" },
  ]);
  // The header names only the few places a newcomer wants; the pages picker lists them all.
  const allPages = $derived([
    ...nav,
    ...DOC_PAGES.filter((d) => d.key !== "docs").map((d) => ({ label: s.nav[d.key], route: `/${d.slug}/` })),
  ]);
  function current(route: string): boolean {
    if (route.endsWith("/for/")) return /\/for\//.test(page.url.pathname);
    if (route === "/compare/") return /\/compare\//.test(page.url.pathname);
    if (route.endsWith("/start/")) return /\/start\/$/.test(page.url.pathname);
    return page.url.pathname === new URL(href(route), page.url.href).pathname;
  }

  const links = $derived(
    allPages.map((item) => ({ id: item.route, label: item.label, href: href(item.route), current: current(item.route) })),
  );

  // The bar sends `<action>?<query>`; /search/ reads `?q=`, so a result page has an address worth sharing.
  function searchTo(destination: string) {
    const query = decodeURIComponent(destination.split("?")[1] ?? "");
    void goto(`${href(locale ? `/${locale}/search/` : "/search/")}?q=${encodeURIComponent(query)}`);
  }

  // The destinations the picker bar spec names, as the WPM UI builds them; Lily ships no third-party URLs. SharePicker
  // uses the browser's share sheet instead wherever there is one.
  type ShareTarget = NonNullable<ComponentProps<typeof PickerBar>["shareTargets"]>[number];
  const shareTargets: ShareTarget[] = $derived([
    {
      id: "email",
      label: s.share.email,
      href: (url, title) => `mailto:?subject=${encodeURIComponent(title)}&body=${encodeURIComponent(url)}`,
      newTab: false,
    },
    {
      id: "linkedin",
      label: s.share.linkedin,
      href: (url) => `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`,
    },
    {
      id: "reddit",
      label: s.share.reddit,
      href: (url, title) =>
        `https://www.reddit.com/submit?url=${encodeURIComponent(url)}&title=${encodeURIComponent(title)}`,
    },
    {
      id: "bluesky",
      label: s.share.bluesky,
      href: (url, title) => `https://bsky.app/intent/compose?text=${encodeURIComponent(`${title} ${url}`)}`,
    },
    {
      id: "mastodon",
      label: s.share.mastodon,
      href: (url, title) =>
        `https://mastodonshare.com/?text=${encodeURIComponent(title)}&url=${encodeURIComponent(url)}`,
    },
  ]);
  const pageTitle = $derived(page.data.title ?? s.brand);

  const themesUrl = $derived(`${href("/")}assets/themes/`.replace(/\/{2,}assets/, "/assets"));
</script>

<a class="skip" href="#main" lang={s.locale}>{s.skip}</a>
<header class="site-header" lang={s.locale} {dir}>
  <a class="brand" href={href(home)}>{s.brand}</a>
  <nav aria-label={s.siteNav}>
    <ul>
      {#each nav as item (item.route)}
        <li>
          <a href={href(item.route)} aria-current={current(item.route) ? "page" : undefined}>{item.label}</a>
        </li>
      {/each}
    </ul>
  </nav>
  <div class="pickers">
    <PickerBar
      {links}
      linkProps={{ navigate: (to: string) => void goto(to) }}
      searchProps={{ navigate: searchTo }}
      labels={{
        link: s.pickers.pages,
        search: s.pickers.search,
        searchInput: s.pickers.searchInput,
        searchSubmit: s.pickers.searchSubmit,
        theme: s.pickers.theme,
        locale: s.pickers.language,
        textSize: s.pickers.textSize,
        share: s.pickers.share,
      }}
      {themesUrl}
      themeProps={{ storageKey: THEME_KEY, detectFromSystem: true, defaultValue: "light" }}
      locales={pickable}
      localeProps={{
        value: locale ?? SITE_LOCALE,
        localeLabels: LOCALE_LABELS,
        applyDir: false,
        onChange: switchLocale,
      }}
      textSizeProps={{ storageKey: TEXT_SIZE_KEY, sizeLabels: s.textSizes }}
      {shareTargets}
      shareProps={{
        title: pageTitle,
        copyLabel: s.share.copy,
        copiedLabel: s.share.copied,
        copyFailedLabel: s.share.copyFailed,
      }}
    />
  </div>
</header>

<main id="main" tabindex="-1">{@render children()}</main>

<footer class="site-footer" lang={s.locale} dir="ltr">
  <p>
    {s.footer.builtFrom} <a href={REPO_URL}>{s.footer.repository}</a>:
    <span data-testid="site-counts"
      >{fill(s.footer.counts, {
        documents: formatInteger(data.counts.documents, s.locale),
        briefs: formatInteger(data.counts.briefs, s.locale),
        locales: formatInteger(data.counts.locales, s.locale),
      })}</span
    >. {s.footer.notice}
    {fill(s.footer.export, { directory: "workforce-planning-management.github.io/", tool: "git subtree" })}
  </p>
</footer>

<style>
  /* The wrapper div in app.html: a class, not an inline style, for the CSP. */
  :global(.app-root) {
    display: contents;
  }
  .skip {
    position: absolute;
    inset-inline-start: 0.5rem;
    inset-block-start: -3rem;
    padding: 0.5rem 0.75rem;
    background: var(--accent-fill);
    color: var(--accent-fg);
    border-radius: var(--radius);
    z-index: 50;
  }
  .skip:focus {
    inset-block-start: 0.5rem;
  }
  .site-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 1.25rem;
    padding: 0.75rem 1.25rem;
    border-block-end: 1px solid var(--line);
    background: var(--panel);
  }
  .brand {
    font-weight: 700;
    color: var(--ink);
  }
  nav ul {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 1rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  nav a {
    display: inline-block;
    min-height: 24px;
    padding-block: 0.125rem;
  }
  nav a[aria-current="page"] {
    font-weight: 700;
    text-decoration: underline;
  }
  /* The Lily picker bar: the theme stylesheet styles the buttons and places each popup; as in the WPM UI, this only
     lines the buttons up and sizes the popups so they stay inside the screen. */
  .pickers {
    margin-inline-start: auto;
    position: relative;
  }
  .pickers :global(.picker-bar) {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
  }
  .pickers :global(:is(.link-picker, .search-picker, .theme-picker, .locale-picker, .text-size-picker, .share-picker)) {
    display: inline-flex;
    align-items: center;
  }
  .pickers
    :global(
      :is(
        .link-picker-button,
        .search-picker-button,
        .theme-picker-button,
        .locale-picker-button,
        .text-size-picker-button,
        .share-picker-button
      )
    ) {
    box-sizing: border-box;
    width: 2.5rem;
    height: 2.5rem;
    margin: 0;
  }
  .pickers
    :global(
      :is(
        .link-picker-list,
        .search-picker-panel,
        .theme-picker-list,
        .locale-picker-list,
        .text-size-picker-list,
        .share-picker-list
      )
    ) {
    z-index: 40;
    box-sizing: border-box;
    min-width: 0;
    width: max-content;
    max-width: calc(100vw - 2rem);
    max-height: 70vh;
    overflow-y: auto;
  }
  /* A long name (the NHS themes) wraps inside a popup at phone width instead of running out of it. */
  .pickers :global(:is(.theme-picker-option, .locale-picker-option, .text-size-picker-option)) {
    overflow-wrap: anywhere;
  }
  /* The share button's icon is an arrow pointing forward (Lily draws it pointing right); on a right-to-left page
     forward is to the left, as in the WPM UI (WPM-T377). */
  :global(:root[dir="rtl"] .share-picker-icon) {
    transform: scaleX(-1);
  }
  /* SharePicker always renders its live region: visually hidden, not display: none, which would silence it. Left in
     flow, it knocks the share button out of line with the others. */
  .pickers :global(.share-picker-status) {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
    border: 0;
  }
  /* The pickers' hover tooltips, as in the WPM UI: out of flow under the bar, so a hover never widens a picker's box
     and moves the button from under the pointer; never a click target. */
  .pickers :global([class$="-picker-tooltip"]) {
    position: absolute;
    inset-inline-end: 0;
    inset-block-start: 100%;
    z-index: 50;
    pointer-events: none;
    white-space: nowrap;
    padding: 0.15rem 0.5rem;
    font-size: 0.75rem;
    border-radius: 0.25rem;
    background: var(--ink);
    color: var(--panel);
  }
  main {
    max-width: 60rem;
    margin-inline: auto;
    padding: 1.5rem 1.25rem 3rem;
  }
  main:focus {
    outline: none;
  }
  .site-footer {
    max-width: 60rem;
    margin-inline: auto;
    padding: 1rem 1.25rem 2rem;
    color: var(--muted);
    font-size: 0.875rem;
    border-block-start: 1px solid var(--line);
  }
</style>
