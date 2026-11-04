import type { EntryGenerator, PageServerLoad } from "./$types";
import { render } from "#lib/server/content.js";
import { href } from "#lib/href.js";
import { COMPARE_SLUGS, SITE_LOCALE } from "#lib/site.js";

export const entries: EntryGenerator = () => COMPARE_SLUGS.map((slug) => ({ slug }));

export const load: PageServerLoad = ({ params }) => {
  const path = `docs/compare/${params.slug}/index.md`;
  const { title, html } = render(path, href, SITE_LOCALE);
  return { title, html, path };
};
