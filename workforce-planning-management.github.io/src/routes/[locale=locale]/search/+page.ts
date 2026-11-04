import type { EntryGenerator, PageLoad } from "./$types";
import { ui } from "#lib/i18n.js";
import { BRIEF_LOCALES, searchAlternates, type BriefLocale } from "#lib/site.js";

export const entries: EntryGenerator = () => BRIEF_LOCALES.map((locale) => ({ locale }));

// The URL carries the locale, so a search from a Welsh page stays Welsh.
export const load: PageLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  return { title: ui(locale).search.title, locale, alternates: searchAlternates() };
};
