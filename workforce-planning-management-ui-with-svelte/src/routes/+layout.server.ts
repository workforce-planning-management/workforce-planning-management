// Root sign-in gate (WPM-T38): every page except the public
// sign-in/verify routes — and now `/`, the marketing splash a
// signed-out visitor lands on — requires a session. Before this, a
// visitor with no `locals.sessionId` reached every page — profile,
// payroll, wellbeing, all of them — and only discovered they were
// signed out once an API call silently failed through the BFF proxy,
// rather than being redirected up front.
//
// Gated at the root (not per-mutation-page, unlike person-front-end's
// narrower guard): WPM's pages mix read content with embedded actions
// (the retention sweep button on `/privacy`, payroll lifecycle actions
// on `/payroll/[id]`, …) rather than separating reads and writes onto
// dedicated routes, so there is no small "mutation-only" subset to
// gate instead.
//
// `/` is exact-matched, not prefix-matched like the others: every path
// starts with "/", so a naive `startsWith("/")` would make the whole
// app public. `+page.svelte` itself renders the splash for a
// signed-out visitor and the HR dashboard for a signed-in one, keyed
// off `signedIn` below.
//
// `locals.sessionId` is presence-only (set from the httpOnly cookie in
// `hooks.server.ts`, never re-validated here) — a UX convenience in
// front of the backend's real ABAC enforcement, not a substitute for
// it, matching the family's `requireSignedIn` convention.
//
// This is the *only* thing that belongs in a `.server.ts` load here:
// it's the one piece of data (the httpOnly session cookie) that only
// the server can see. Everything else — including the multi-org
// membership fetch in the sibling `+layout.ts` — stays in a universal
// load, because `ssr=false` (below) means a `.server.ts` load's
// `fetch` makes a real, Playwright-invisible server-to-server call
// (verified: it 500s against nothing being there and does not appear
// in any `page.route` stub), while a universal load's `fetch` runs in
// the browser under this app's SPA mode, exactly like every other
// data fetch here — the same reason `+layout.ts` gives for `ssr=false`
// existing at all.

import { redirect } from "@sveltejs/kit";
import { localePath, splitLocale } from "#lib/locales.js";
import { isPublicPath } from "#lib/publicPages.js";
import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = ({ locals, url }) => {
  // The URL carries a locale prefix (`/cy-001/workers`); the gate is
  // keyed on the path beneath it.
  const { locale, rest } = splitLocale(url.pathname);
  const isPublic = isPublicPath(rest);
  if (!isPublic && locals.sessionId === null) {
    redirect(303, localePath(locale ?? "en-001", "/signin"));
  }
  return { signedIn: locals.sessionId !== null };
};
