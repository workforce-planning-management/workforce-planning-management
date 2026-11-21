// `/robots.txt` — points crawlers at the sitemap and away from the API, the CMS
// and sign-out (see `$lib/sitemap`).

import type { RequestHandler } from "./$types";
import { WORKFORCE_PLANNING_MANAGEMENT_PUBLIC_URL } from "#lib/server/config.js";
import { renderRobots } from "#lib/sitemap.js";

export const GET: RequestHandler = ({ url }) =>
  new Response(
    renderRobots(WORKFORCE_PLANNING_MANAGEMENT_PUBLIC_URL || url.origin),
    {
      headers: {
        "content-type": "text/plain; charset=utf-8",
        "cache-control": "public, max-age=3600",
      },
    },
  );
