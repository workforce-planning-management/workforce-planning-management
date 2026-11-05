// `page.data.title` convention (see `../../+layout.svelte`): mirrors this route's own <svelte:head><title>.

import type { PageLoad } from "./$types";
import { pageTitle } from "#lib/title.js";

export const load: PageLoad = ({ url }) => {
  return { title: pageTitle("titles.myPadrs", url) };
};
