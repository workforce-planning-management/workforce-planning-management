// `page.data.title` convention (see `../+layout.svelte`): mirrors this
// route's own <svelte:head><title> so SharePicker gets the right title
// without reading the DOM.

import type { PageLoad } from "./$types";
import { pageTitle } from "#lib/title.js";

export const load: PageLoad = ({ url }) => {
  return { title: pageTitle("titles.movements", url) };
};
