<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  import { untrack } from "svelte";
  import { goto } from "$app/navigation";
  import {
    i18n,
    isRtl,
    l,
    t,
    splitLocale,
    localePath,
    normaliseLocale,
    LOCALES,
    LOCALE_LABELS,
  } from "#lib/i18n.svelte.js";
  import PickerBar from "@lilydesignsystem/svelte-picker-bar";
  import { Drawer } from "@lilydesignsystem/svelte-headless";
  import type { ShareTarget } from "@lilydesignsystem/svelte-share-picker";

  let { children } = $props();

  // Left nav drawer (hamburger menu): the section links used to sit in
  // the header itself; moved into a drawer to keep the header from
  // wrapping to a second line at anything narrower than a wide desktop.
  let navOpen = $state(false);
  let hamburgerButton: HTMLButtonElement | undefined = $state();
  let drawerNav: HTMLElement | undefined = $state();

  const NAV_LINKS = [
    ["/workers", "nav.workers"],
    ["/directory", "nav.directory"],
    ["/org-chart", "nav.orgChart"],
    ["/groups", "nav.groups"],
    ["/requisitions", "nav.requisitions"],
    ["/workforce", "nav.workforce"],
    ["/development", "nav.development"],
    ["/learning", "nav.learning"],
    ["/me", "nav.me"],
    ["/roles", "nav.roles"],
    ["/skills", "nav.skills"],
    ["/cpd", "nav.cpd"],
    ["/planning", "nav.planning"],
    ["/change", "nav.change"],
    ["/mentorship", "nav.mentorship"],
    ["/wellbeing", "nav.wellbeing"],
    ["/privacy", "nav.privacy"],
    ["/payroll", "nav.payroll"],
    ["/benchmarks", "nav.benchmarks"],
    ["/metrics", "nav.metrics"],
  ] as const;

  function closeNav() {
    navOpen = false;
    hamburgerButton?.focus();
  }

  // Move focus into the drawer on open (APG dialog pattern); the Lily
  // Drawer primitive only sets tabindex="-1" on its own root and leaves
  // focus management to the consumer.
  $effect(() => {
    if (navOpen)
      drawerNav
        ?.querySelector<HTMLAnchorElement>("a")
        ?.focus();
  });

  // Share destinations for the Lily SharePicker. Lily ships no
  // third-party URLs — each `href` builder is ours. `url`/`title` are
  // supplied by SharePicker at share time (current page URL; the leaf
  // page's title, sourced from `page.data.title` below — the
  // `page.data.title` convention, set per-route by each route's load
  // function so it stays in sync with that page's own <svelte:head>
  // <title> without SharePicker having to read the DOM).
  const SHARE_TARGETS: ShareTarget[] = [
    {
      id: "email",
      label: "Email",
      href: (url, title) =>
        `mailto:?subject=${encodeURIComponent(title)}&body=${encodeURIComponent(url)}`,
      newTab: false,
    },
    {
      id: "linkedin",
      label: "LinkedIn",
      href: (url) =>
        `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`,
    },
    {
      id: "reddit",
      label: "Reddit",
      href: (url, title) =>
        `https://www.reddit.com/submit?url=${encodeURIComponent(url)}&title=${encodeURIComponent(title)}`,
    },
    {
      id: "bluesky",
      label: "Bluesky",
      href: (url, title) =>
        `https://bsky.app/intent/compose?text=${encodeURIComponent(`${title} ${url}`)}`,
    },
    {
      id: "mastodon",
      label: "Mastodon",
      href: (url, title) =>
        `https://mastodonshare.com/?text=${encodeURIComponent(title)}&url=${encodeURIComponent(url)}`,
    },
  ];

  // The `page.data.title` convention: each route's own load function
  // (`+page.ts`/`+page.server.ts`) returns a plain `title` string that
  // mirrors what that route's `<svelte:head><title>` renders, so the
  // layout — which does not know which leaf page is active — can read
  // it here for SharePicker without scraping `document.title`. Falls
  // back to the brand name for the rare route that sets none.
  const pageTitle = $derived(page.data?.title ?? t("brand.name"));

  // localStorage key ThemePicker persists the chosen theme under, and
  // its pre-rename spelling (2026-07-23, `HCM` -> `WPM`).
  // NOTE to a future renamer: THEME_KEY_LEGACY is deliberately the OLD
  // name — a blanket search-and-replace must not "fix" it.
  const THEME_KEY = "mxi.wpm.theme";
  const THEME_KEY_LEGACY = "mxi.hcm.theme";

  // Adopt a returning user's saved theme once, before ThemePicker reads
  // the key: without this the rename silently resets everyone to the
  // default theme. Runs at module scope (not `onMount`) so it lands
  // before the component initialises; guarded for SSR and for a blocked
  // or full store, neither of which may stop the app rendering.
  if (typeof localStorage !== "undefined") {
    try {
      const legacy = localStorage.getItem(THEME_KEY_LEGACY);
      if (legacy !== null && localStorage.getItem(THEME_KEY) === null) {
        localStorage.setItem(THEME_KEY, legacy);
        localStorage.removeItem(THEME_KEY_LEGACY);
      }
    } catch {
      // Ignore: a missing theme preference is cosmetic.
    }
  }

  // The URL's locale prefix is the source of truth: follow it, so a
  // direct visit to `/cy-001/workers` (or back/forward between locales)
  // retranslates the chrome.
  $effect(() => {
    const { locale } = splitLocale(page.url.pathname);
    if (locale && locale !== untrack(() => i18n.locale)) i18n.set(locale);
  });

  // Switching language navigates to the same page under the new prefix;
  // the effect above then applies it, so the URL stays the source of truth.
  function switchLocale(code: string) {
    const next = normaliseLocale(code);
    if (!next) return;
    const { rest } = splitLocale(page.url.pathname);
    void goto(`${localePath(next, rest)}${page.url.search}${page.url.hash}`);
  }

  $effect(() => {
    document.documentElement.lang = i18n.locale;
    document.documentElement.dir = isRtl(i18n.locale) ? "rtl" : "ltr";
  });
</script>

<nav class="top">
  <button
    bind:this={hamburgerButton}
    type="button"
    class="hamburger-menu hamburger-menu-button"
    aria-label={t("nav.menu")}
    aria-haspopup="dialog"
    aria-expanded={navOpen}
    onclick={() => (navOpen = true)}
  >
    <svg viewBox="0 0 16 16" aria-hidden="true" width="1.05rem" height="1.05rem" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
      <path d="M2 4h12M2 8h12M2 12h12" />
    </svg>
  </button>
  <a class="brand" href={l("/")}>{t("brand.name")}</a>
  <span class="spacer"></span>
  <a href={l("/tour")}>{t("nav.tour")}</a>
  <a href={l("/signin")}>{t("nav.signin")}</a>
  <div class="chrome">
    <PickerBar
      labels={{
        theme: "Theme",
        locale: t("chrome.language"),
        textSize: t("nav.text_size"),
        share: t("nav.share"),
      }}
      themesUrl="/assets/themes/"
      themeProps={{ storageKey: THEME_KEY }}
      locales={[...LOCALES]}
      localeProps={{
        value: i18n.locale,
        localeLabels: LOCALE_LABELS,
        applyDir: false,
        onChange: switchLocale,
      }}
      textSizeProps={{
        storageKey: "mxi.wpm.text-size",
      }}
      shareTargets={SHARE_TARGETS}
      shareProps={{
        title: pageTitle,
        copyLabel: t("share.copy_link"),
        copiedLabel: t("share.copied"),
        copyFailedLabel: t("share.copy_failed"),
      }}
    />
  </div>
</nav>

{#if navOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <div class="overlay-container" aria-hidden="true" onclick={closeNav}></div>
{/if}
<Drawer label={t("nav.menu")} side="left" bind:open={navOpen}>
  <div class="drawer-head">
    <span class="brand">{t("brand.name")}</span>
    <button
      type="button"
      class="hamburger-menu hamburger-menu-button"
      aria-label={t("nav.closeMenu")}
      onclick={closeNav}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true" width="1.05rem" height="1.05rem" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
        <path d="M3 3l10 10M13 3L3 13" />
      </svg>
    </button>
  </div>
  <nav class="menu drawer-links" aria-label={t("nav.menu")} bind:this={drawerNav}>
    {#each NAV_LINKS as [href, key] (href)}
      <a class="menu-item" href={l(href)} onclick={closeNav}>
        {t(key)}
      </a>
    {/each}
  </nav>
</Drawer>

<main>
  {@render children()}
</main>

<style>
  .spacer {
    flex: 1;
  }

  /* The active Lily theme styles .hamburger-menu/-button and .drawer/
     .overlay-container/.menu/.menu-item already (colours, borders,
     shadow, hover). Two gaps it leaves for the consumer: .drawer
     defaults to the inline-end edge (a right-hand filters/settings
     panel is its assumed use), and it has no slide transition at all. */
  :global(.drawer[data-side="left"]) {
    inset-inline-end: auto;
    inset-inline-start: 0;
    border-inline-start: 0;
    border-inline-end: var(--border, 1px) solid var(--lily-border, var(--line));
  }
  :global(.drawer) {
    transition: transform 0.15s ease-out;
  }
  .drawer-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  .drawer-links {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .chrome {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .chrome :global(.picker-bar) {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }
  .chrome :global(.theme-picker-button),
  .chrome :global(.locale-picker-button),
  .chrome :global(.text-size-picker-button),
  .chrome :global(.share-picker-button) {
    font: inherit;
    padding: 0.15rem 0.3rem;
    max-width: 11rem;
  }

  /* The active Lily theme (static/assets/themes/*.css) already styles
     every *-picker-list: position: absolute, min-width: max-content,
     max-height, colours, [hidden] handling — all of it. Its one default
     we override is the anchor side: it anchors inset-inline-start: 0 on
     each *-picker's own (button-sized) root, which is fine for a picker
     with room to its right, but this bar sits flush against the nav's
     right edge, so ThemePicker's list — wide enough to hold the longest
     NHS theme name — has nowhere to grow and overflows the viewport,
     dragging the whole page into a horizontal scroll via
     scroll-into-view-on-focus (the very "page jump" every Lily
     *-picker's styling guide warns about).
     Anchoring inset-inline-end instead fixes the direction, but only if
     the containing block is wide enough to solve `start: auto` against —
     tried against each *-picker's own ~40px icon-button box, Chromium's
     abspos solver collapses back to the start edge instead of growing
     left (verified empirically: explicit width/min-width overrides on
     the list were silently ignored while anchored to that box). Routing
     the containing block up to nav.top — hundreds of px wide, comfortably
     wider than any list's content — avoids that edge case entirely, and
     doesn't need `top` set: the list's static position (right below its
     own button, in normal flow) is unaffected by which ancestor governs
     the inline axis. inset-inline-* (not left/right) keeps the anchor
     flipping correctly under the app's RTL locales. */
  nav.top {
    position: relative;
  }
  .chrome :global(.theme-picker-list),
  .chrome :global(.locale-picker-list),
  .chrome :global(.text-size-picker-list),
  .chrome :global(.share-picker-list) {
    inset-inline-start: auto;
    inset-inline-end: 0;
  }

  /* Every other *-picker leaves its status line to the consumer, so
     nothing else is rendered here; SharePicker always renders its own
     <p class="share-picker-status">, which — unhidden — adds invisible
     block-height to .share-picker alone and knocks the SharePicker
     button a couple of pixels out of vertical alignment with its three
     siblings under .chrome's align-items: center. Visually hidden
     (not display: none, which would silence the live region) removes
     it from layout entirely. */
  .chrome :global(.share-picker-status) {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
