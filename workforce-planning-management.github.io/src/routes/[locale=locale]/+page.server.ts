import { error } from "@sveltejs/kit";
import type { EntryGenerator, PageServerLoad } from "./$types";
import { HOME_LOCALES, hasHome, type BriefLocale } from "#lib/site.js";
import { homeAlternates, render, splitHero } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () => HOME_LOCALES.map((locale) => ({ locale }));

// A locale's own home, `locales/<locale>/index.md`, in its language. A locale without one has no page here.
export const load: PageServerLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  if (!hasHome(locale)) error(404, "Not found");
  const path = `locales/${locale}/index.md`;
  const { title, html } = render(path, href, locale);
  return { title, ...splitHero(html), path, locale, alternates: homeAlternates() };
};
