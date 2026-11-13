import { error } from "@sveltejs/kit";
import type { EntryGenerator, PageServerLoad } from "./$types";
import { BRIEF_LOCALES, isBriefLocale, type BriefLocale } from "#lib/site.js";
import { productionAlternates, renderSegments } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () => BRIEF_LOCALES.map((locale) => ({ locale }));

// What production use would mean and an honest list of what stands between WPM and it,
// `locales/<locale>/path-to-production/index.md`, each item with the author's readiness estimate on a slider.
export const load: PageServerLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  if (!isBriefLocale(locale)) error(404, "Not found");
  const path = `locales/${locale}/path-to-production/index.md`;
  const { title, segments } = renderSegments(path, href, locale);
  return { title, segments, path, locale, alternates: productionAlternates() };
};
