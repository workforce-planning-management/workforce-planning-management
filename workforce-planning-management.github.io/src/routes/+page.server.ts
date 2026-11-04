import type { PageServerLoad } from "./$types";
import { homeAlternates, render, splitHero } from "#lib/server/content.js";
import { href } from "#lib/href.js";
import { ROOT_HOME_LOCALE } from "#lib/site.js";

export const load: PageServerLoad = () => {
  const path = `locales/${ROOT_HOME_LOCALE}/index.md`;
  const { title, html } = render(path, href, ROOT_HOME_LOCALE);
  return { title, ...splitHero(html), path, alternates: homeAlternates() };
};
