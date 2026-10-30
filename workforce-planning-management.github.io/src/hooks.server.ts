import type { Handle } from "@sveltejs/kit/hooks";
import { isRtl, localeOfPath } from "#lib/site.js";

// `lang` and `dir` on <html> come from the page's locale (the first path segment of a role brief), so the
// prerendered HTML is right before any script runs: an Arabic or Urdu brief is laid out from the right from its first
// paint. Every other page is English. The locale code is a valid BCP 47 tag as written (`ar-001`, `en-gb-oxendict`),
// as in the WPM UI.
export const handle: Handle = async ({ event, resolve }) => {
  const locale = localeOfPath(event.url.pathname);
  const lang = locale ?? "en";
  const dir = locale && isRtl(locale) ? "rtl" : "ltr";
  return resolve(event, {
    transformPageChunk: ({ html }) => html.replace("%lang%", lang).replace("%dir%", dir),
  });
};
