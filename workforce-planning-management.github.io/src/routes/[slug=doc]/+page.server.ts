import { error } from "@sveltejs/kit";
import type { EntryGenerator, PageServerLoad } from "./$types";
import { DOC_PAGES, SITE_LOCALE } from "#lib/site.js";
import { render } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () =>
  DOC_PAGES.filter((d) => d.slug !== "").map((d) => ({ slug: d.slug }));

export const load: PageServerLoad = ({ params }) => {
  const doc = DOC_PAGES.find((d) => d.slug === params.slug);
  if (!doc) error(404, "Not found");
  const { title, html } = render(doc.path, href, SITE_LOCALE);
  return { title, html, path: doc.path };
};
