import type { EntryGenerator, PageServerLoad } from "./$types";
import { BRIEF_LOCALES, type BriefLocale } from "#lib/site.js";
import { alternates, briefsOf, render } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () => BRIEF_LOCALES.map((locale) => ({ locale }));

// A locale's list of briefs is its own `locales/<locale>/for/index.md`, in its language.
export const load: PageServerLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  const path = `locales/${locale}/for/index.md`;
  const { title, html } = render(path, href, locale);
  return { title, html, path, locale, count: briefsOf(locale).length, alternates: alternates(locale, null) };
};
