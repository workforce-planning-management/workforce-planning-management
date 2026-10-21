// The Sveltia CMS shell, one per locale: `/en-001/admin/`, `/cy-001/admin/`. The
// rest is the same CMS and `/admin/config.yml`; what the locale changes
// is the CMS's own interface language and the page's `lang`/`dir`.
//
// Sveltia has no config option for its interface language: it reads the
// `locale` field of the `sveltia-cms.prefs` localStorage entry, else the
// browser's language. So the page writes that entry from the URL's
// locale before the CMS script runs — the URL stays the source of truth,
// as in the app. (This replaces the old static `admin/index.html`.)
//
// SvelteKit's `kit.csp` covers only pages it renders, so this response sends its own policy (WPM-T249): the CMS
// script at an exact version with its integrity hash, the inline script by its hash, and the hosts Sveltia's
// security page lists plus the GitHub backend's.

import type { RequestHandler } from "./$types";
import { CMS_SCRIPT, adminCsp, prefsScript } from "#lib/server/admin-page.js";
import {
  CMS_UI_LOCALE,
  DEFAULT_LOCALE,
  isRtl,
  splitLocale,
} from "#lib/locales.js";

export const GET: RequestHandler = ({ url }) => {
  const locale = splitLocale(url.pathname).locale ?? DEFAULT_LOCALE;
  const ui = CMS_UI_LOCALE[locale];
  const inline = prefsScript(ui);
  const html = `<!doctype html>
<html lang="${locale}" dir="${isRtl(locale) ? "rtl" : "ltr"}">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="robots" content="noindex" />
    <title>Content manager — WPM</title>
    <link href="/admin/config.yml" type="text/yaml" rel="cms-config-url" />
  </head>
  <body>
    <script>${inline}</script>
    <script src="${CMS_SCRIPT.src}" integrity="${CMS_SCRIPT.integrity}" crossorigin="anonymous"></script>
  </body>
</html>
`;
  return new Response(html, {
    headers: {
      "content-type": "text/html; charset=utf-8",
      "content-security-policy": adminCsp(inline),
    },
  });
};
