import type { PageLoad } from "./$types";
import { ui } from "#lib/i18n.js";
import { searchAlternates } from "#lib/site.js";

export const load: PageLoad = () => ({ title: ui(null).search.title, alternates: searchAlternates() });
