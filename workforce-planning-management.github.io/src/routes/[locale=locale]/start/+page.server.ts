import { error } from "@sveltejs/kit";
import type { EntryGenerator, PageServerLoad } from "./$types";
import { HOME_LOCALES, hasHome, type BriefLocale } from "#lib/site.js";
import { render, startAlternates } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () => HOME_LOCALES.map((locale) => ({ locale }));

// The guided tutorial, `locales/<locale>/start/index.md`, in the locale's language.
export const load: PageServerLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  if (!hasHome(locale)) error(404, "Not found");
  const path = `locales/${locale}/start/index.md`;
  const { title, html } = render(path, href, locale);
  return { title, html, path, locale, alternates: startAlternates() };
};
