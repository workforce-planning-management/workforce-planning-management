// `page.data.title` convention (see `../+layout.svelte`): the page's own <title> and the
// title SharePicker offers, in the locale the address carries.

import type { PageLoad } from "./$types";
import { navTitle } from "#lib/title.js";

export const load: PageLoad = ({ url }) => {
  return { title: navTitle("nav.orgChart", url) };
};
