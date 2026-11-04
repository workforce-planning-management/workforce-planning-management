import { defineParams } from "@sveltejs/kit/params";
import { COMPARE_SLUGS, DOC_PAGES, isBriefLocale } from "#lib/site.ts";

// Route parameters: [slug=doc] is one of the documents and [locale=locale] a role-brief locale.
// A brief's [role] is its directory name in that locale (translated, so not one list); only the briefs that exist are
// prerendered, and any other name is a 404.
export const params = defineParams({
  doc: (p: string) => (DOC_PAGES.some((d) => d.slug === p) ? p : undefined),
  locale: (p: string) => (isBriefLocale(p) ? p : undefined),
  compare: (p: string) => ((COMPARE_SLUGS as readonly string[]).includes(p) ? p : undefined),
});
