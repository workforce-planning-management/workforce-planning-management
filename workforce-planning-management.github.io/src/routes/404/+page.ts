import type { PageLoad } from "./$types";
import { ui } from "#lib/i18n.js";

// Prerendered to build/404.html, the file GitHub Pages serves for an address it does not have.
export const trailingSlash = "never";

export const load: PageLoad = () => ({ title: ui(null).notFound.title });
