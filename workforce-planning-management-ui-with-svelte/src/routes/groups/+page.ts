// `page.data.title` convention (see `../+layout.svelte`).

import type { PageLoad } from "./$types";
import { pageTitle } from "#lib/title.js";

export const load: PageLoad = ({ url }) => {
  return { title: pageTitle("titles.groups", url) };
};
