import type { Reroute } from "@sveltejs/kit/hooks";
import { splitLocale } from "#lib/locales.js";

// Locale content routes: `/en-001/workers`, `/cy-001/workers`, and the
// bare aliases `/en/…`, `/cy/…` (which the server hook redirects to the
// canonical prefix) all resolve to the one route tree under `/workers`.
// Only route resolution is rewritten — `url` still carries the prefix,
// which is how the root layout learns the locale.
export const reroute: Reroute = ({ url }) => {
  const { locale, rest } = splitLocale(url.pathname);
  return locale ? rest : undefined;
};
