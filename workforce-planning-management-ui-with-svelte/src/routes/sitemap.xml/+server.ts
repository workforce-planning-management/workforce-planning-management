// `/sitemap.xml` — the public pages in every locale with hreflang alternates
// (see `$lib/sitemap`). Generated per request from the locale list, so adding a
// locale or a public page needs no separate step. Set `WPM_PUBLIC_URL` when the
// app sits behind a proxy and `url.origin` is not the public address.

import type { RequestHandler } from "./$types";
import { WPM_PUBLIC_URL } from "#lib/server/config.js";
import { renderSitemap } from "#lib/sitemap.js";

export const GET: RequestHandler = ({ url }) =>
  new Response(renderSitemap(WPM_PUBLIC_URL || url.origin), {
    headers: {
      "content-type": "application/xml; charset=utf-8",
      "cache-control": "public, max-age=3600",
    },
  });
