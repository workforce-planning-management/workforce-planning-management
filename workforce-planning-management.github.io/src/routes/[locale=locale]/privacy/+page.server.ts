import { error } from "@sveltejs/kit";
import type { EntryGenerator, PageServerLoad } from "./$types";
import { BRIEF_LOCALES, isBriefLocale, type BriefLocale } from "#lib/site.js";
import { privacyAlternates, render } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () => BRIEF_LOCALES.map((locale) => ({ locale }));

// The privacy notice, `locales/<locale>/privacy/index.md`: what the website and the application keep in a browser,
// and why (WPM-R253).
export const load: PageServerLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  if (!isBriefLocale(locale)) error(404, "Not found");
  const path = `locales/${locale}/privacy/index.md`;
  const { title, html } = render(path, href, locale);
  return { title, html, path, locale, alternates: privacyAlternates() };
};
