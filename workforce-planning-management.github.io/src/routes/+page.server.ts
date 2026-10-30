import type { PageServerLoad } from "./$types";
import { render } from "#lib/server/content.js";
import { href } from "#lib/href.js";
import { SITE_LOCALE } from "#lib/site.js";

export const load: PageServerLoad = () => {
  const { title, html } = render("index.md", href, SITE_LOCALE);
  return { title, html, path: "index.md" };
};
