# Localization

The URL carries the locale, and the UI strings are content. Full description:
[spec/locales-for-global-sharing-with-svelte](../spec/locales-for-global-sharing-with-svelte/index.md)
("How WPM applies this", WPM-R50 / WPM-D37).

## Locales

17 content locales in
`workforce-planning-management-ui-with-svelte/content/locales/<locale>/ui.json`:
`ar-001 bn-001 cy-001 de-001 de-de en-001 en-gb en-us es-001 es-es fr-001 hi-001
id-001 pt-001 ru-001 ur-001 zh-001`.

- **Every directory under `content/locales/` is `<language>-<region>`** — lower-case ISO 639
  language, then an ISO 3166-1 alpha-2 country or the UN numeric `001`; never a bare
  language (`en`, `cy`), and a bare language is not a URL either. A unit test enforces the directories.
- A **`<language>-001`** file must contain **every key** (the parity test fails
  otherwise). `en-001` is the source.
- A **regional** locale (`en-gb`, `en-us`, `de-de`, `es-es`) holds only its
  **overrides** and falls back to its `-001` base, then `en-001`.
- Files nest on the key dots (`{"nav": {"workers": "…"}}`); the app flattens them.

## Adding or changing a string

1. Add the key to `en-001/ui.json` (nested).
2. Add the **translation to every other `-001` locale**. (A script that loads each
   JSON, sets the nested keys and writes it back with `ensure_ascii=False,
   indent=2` keeps the files consistent; never hand-edit one locale only.)
3. Use it: `t("area.key")`; for server-derived text with placeholders,
   `tp("insights.<code>.observation", params)` (`{name}` placeholders; returns
   `null` for an unknown code so you can fall back to the server's English).
4. `pnpm cms-config` to regenerate `static/admin/config.yml` — a unit test fails
   if it is stale.
5. `pnpm test` (parity) and Playwright (the locale-switch spec).

The translations were written by an AI assistant and have **not** had a
native-speaker review — say so if you add more, and do not present them as
reviewed. Right-to-left locales (`ar`, `ur`) flip `<html dir>`; use logical CSS
(`inset-inline-*`, `margin-inline-*`) in new styles.

## Adding a locale

Create `content/locales/<code>/ui.json` (a `-001` file complete, or a regional
file with overrides), add the code to `LOCALES`, `LOCALE_LABELS` and (for the CMS)
`CMS_UI_LOCALE` in `src/lib/locales.ts`, and rerun `pnpm cms-config`. Check
`CMS_UI_LANGUAGES` for the nearest language Sveltia CMS ships.

## Routing

- Every page: `/<locale>/…` (a `reroute` hook in `src/hooks.ts`; the route tree
  is unchanged). **Links and `goto` use `l("/path")`.**
- **A locale has one address: its full code.** A bare language (`/en/…`) is **not** a route
  and is **not forwarded** (the old `/en/` → `/en-001/` 301 was removed, WPM-D44); a bare
  language still *matches a tag* (cookie, `Accept-Language`, `navigator`) via `LANGUAGE_LOCALE`.
  An unprefixed path
  **302s** to the `wpm-locale` cookie, else `Accept-Language`, else `en-001`.
  **A bare `/` is the exception:** with no cookie the *browser* decides —
  `routes/+layout.ts` redirects by `navigator.languages` (`localeFromNavigator`; `cy_GB`
  → `cy-001`, or `cy-gb` if the app ever has one), falling back to the server's
  `Accept-Language` pick (`fallbackLocale`, and a `<noscript>` refresh for no-JS). A
  cookie still wins. A test must make `navigator` and `Accept-Language` *disagree* to prove
  it, since Playwright sets both from `locale`.
  `/api`, `/_app`, `/assets`, files with an extension, `/signin/sso` and
  `/signout` are exempt (`src/hooks.server.ts`).
- The URL drives `i18n.locale` (the layout's `$effect`); the picker navigates.
- The CMS shell is `src/routes/admin/+server.ts` (writes `sveltia-cms.prefs` from
  the URL's locale); `/admin/config.yml` stays static.
- Playwright pins `locale: "en"` so an unprefixed visit lands on `en-001`.

## Known limits

`/admin/` needs a GitHub login and has not been exercised end to end in a
browser. Spelling variants (`en-gb` "organisation") are overrides, not a
separate translation.
