// A walkthrough for one role (WPM-R244). Public, like the tour it sits under. An unknown role is a 404, which the error page
// answers with a link to help.

import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { pageTitle } from "#lib/title.js";
import { isGuideRole } from "#lib/guides.js";

export const load: PageLoad = ({ params, url }) => {
  if (!isGuideRole(params.role)) error(404, "No such walkthrough");
  return { role: params.role, title: pageTitle("titles.guides", url) };
};
