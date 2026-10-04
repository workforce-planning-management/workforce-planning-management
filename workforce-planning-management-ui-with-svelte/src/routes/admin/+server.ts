// The Sveltia CMS shell, one per locale: `/en-001/admin/`, `/cy-001/admin/`
// (and the `/en/admin/` aliases, which the server hook redirects). The
// rest is the same CMS and `/admin/config.yml`; what the locale changes
// is the CMS's own interface language and the page's `lang`/`dir`.
//
// Sveltia has no config option for its interface language: it reads the
// `locale` field of the `sveltia-cms.prefs` localStorage entry, else the
// browser's language. So the page writes that entry from the URL's
// locale before the CMS script runs — the URL stays the source of truth,
// as in the app. (This replaces the old static `admin/index.html`.)

import type { RequestHandler } from "./$types";
import {
  CMS_UI_LOCALE,
  DEFAULT_LOCALE,
  isRtl,
  splitLocale,
} from "#lib/locales.js";

export const GET: RequestHandler = ({ url }) => {
  const locale = splitLocale(url.pathname).locale ?? DEFAULT_LOCALE;
  const ui = CMS_UI_LOCALE[locale];
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
    <script>
      try {
        const key = "sveltia-cms.prefs";
        const prefs = JSON.parse(localStorage.getItem(key) || "{}") || {};
        prefs.locale = ${JSON.stringify(ui)};
        localStorage.setItem(key, JSON.stringify(prefs));
      } catch {
        // A blocked store only costs the language choice.
      }
    </script>
    <script src="https://unpkg.com/@sveltia/cms/dist/sveltia-cms.js"></script>
  </body>
</html>
`;
  return new Response(html, {
    headers: { "content-type": "text/html; charset=utf-8" },
  });
};
