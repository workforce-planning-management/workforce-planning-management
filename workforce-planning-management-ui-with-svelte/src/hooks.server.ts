import { redirect } from "@sveltejs/kit";
import type { Handle } from "@sveltejs/kit/hooks";

// BFF entry point: read the httpOnly session cookie on every request
// and expose the opaque session id to server endpoints via `locals`.
// The browser never reads the cookie; only the SvelteKit server does,
// and only the server talks to the services (WPM-T18, per
// `agents/share/authentication-sessions.md`).
import { SESSION_COOKIE } from "#lib/server/session.js";
import {
  isRtl,
  localePath,
  negotiateLocale,
  normaliseLocale,
  splitLocale,
} from "#lib/locales.js";

/** Cookie remembering the last locale, to pick one for an unprefixed visit. */
const LOCALE_COOKIE = "wpm-locale";

/** Paths that are never locale-prefixed: the BFF proxy, build assets, the
 *  CMS, and the server-only endpoints the identity flow redirects through. */
const UNPREFIXED = ["/api/", "/_app/", "/assets/", "/signin/sso", "/signout"];

function isUnprefixed(pathname: string): boolean {
  return (
    UNPREFIXED.some((p) => pathname.startsWith(p)) ||
    /\.[A-Za-z0-9]+$/.test(pathname)
  );
}

export const handle: Handle = async ({ event, resolve }) => {
  event.locals.sessionId = event.cookies.get(SESSION_COOKIE) ?? null;

  const { pathname, search } = event.url;
  const { locale } = splitLocale(pathname);
  const pageRequest =
    event.request.method === "GET" || event.request.method === "HEAD";

  const remembered = normaliseLocale(event.cookies.get(LOCALE_COOKIE));
  const acceptLanguage = event.request.headers.get("accept-language");
  // A bare `/` with no remembered locale is left to the browser: the root
  // layout redirects by `navigator.languages` (see `routes/+layout.ts`).
  // Everything else unprefixed is sent to the remembered or negotiated locale.
  const browserDecides = pathname === "/" && remembered === null;
  if (!locale && pageRequest && !isUnprefixed(pathname) && !browserDecides) {
    const preferred = remembered ?? negotiateLocale(acceptLanguage);
    redirect(302, `${localePath(preferred, pathname)}${search}`);
  }
  if (locale) {
    event.cookies.set(LOCALE_COOKIE, locale, {
      path: "/",
      maxAge: 60 * 60 * 24 * 365,
      sameSite: "lax",
      httpOnly: false,
    });
  }

  return resolve(event, {
    transformPageChunk: ({ html }) =>
      locale
        ? html
            .replace('lang="en"', `lang="${locale}"`)
            .replace("%dir%", isRtl(locale) ? "rtl" : "ltr")
        : html.replace("%dir%", "ltr").replace(
            "</head>",
            // Without JavaScript the browser cannot decide: the server's
            // `Accept-Language` pick is the fallback.
            browserDecides
              ? `<noscript><meta http-equiv="refresh" content="0;url=${localePath(negotiateLocale(acceptLanguage), "/")}"></noscript></head>`
              : "</head>",
          ),
  });
};
