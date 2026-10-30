import { error } from "@sveltejs/kit";
import type { EntryGenerator, PageServerLoad } from "./$types";
import type { BriefLocale } from "#lib/site.js";
import { allBriefs, alternates, findBrief, render, titleOf } from "#lib/server/content.js";
import { href } from "#lib/href.js";

export const entries: EntryGenerator = () => allBriefs();

export const load: PageServerLoad = ({ params }) => {
  const locale = params.locale as BriefLocale;
  const brief = findBrief(locale, params.role);
  if (!brief) error(404, "Not found");
  const { title, html } = render(brief.path, href, locale);
  // The link back to the list is the list's own title, in the brief's language.
  const listTitle = titleOf(`locales/${locale}/for/index.md`);
  return { title, html, path: brief.path, locale, role: params.role, listTitle, alternates: alternates(locale, params.role) };
};
