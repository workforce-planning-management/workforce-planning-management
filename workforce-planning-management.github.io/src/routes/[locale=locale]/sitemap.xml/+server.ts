import type { EntryGenerator, RequestHandler } from "./$types";
import { sitemapRoutes, sitemapXml } from "#lib/server/content.js";
import { BRIEF_LOCALES, SITE_URL, type BriefLocale } from "#lib/site.js";

export const prerender = true;
export const trailingSlash = "never";

export const entries: EntryGenerator = () => BRIEF_LOCALES.map((locale) => ({ locale }));

export const GET: RequestHandler = ({ params }) =>
  new Response(sitemapXml(sitemapRoutes(params.locale as BriefLocale), SITE_URL), {
    headers: { "content-type": "application/xml" },
  });
