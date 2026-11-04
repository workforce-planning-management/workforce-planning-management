import type { RequestHandler } from "./$types";
import { sitemapRoutes, sitemapXml } from "#lib/server/content.js";
import { SITE_URL } from "#lib/site.js";

export const prerender = true;
export const trailingSlash = "never";

export const GET: RequestHandler = () =>
  new Response(sitemapXml(sitemapRoutes(), SITE_URL), { headers: { "content-type": "application/xml" } });
