// SSO callback (BFF, Keycloak via the authentication service). The
// browser lands here once the authentication service has completed the
// Keycloak exchange; `code` is a single-use grant it issued — this app
// never sees a Keycloak token. Mirrors `../+page.server.ts` (the
// sibling magic-link flow) exactly: same session-cookie handoff,
// different front door.

import type { PageServerLoad } from "./$types";
import { redirect } from "@sveltejs/kit";
import {
  SESSION_COOKIE,
  SESSION_COOKIE_OPTIONS,
  sessionIdFromResponse,
} from "$lib/server/session";
import { verifySso } from "$lib/server/auth";

// `page.data.title` convention (see `../../+layout.svelte`): mirrors this
// route's own <svelte:head><title> so SharePicker gets the right title
// without reading the DOM.
const TITLE = "Sign-in link — Workforce Planning Management";

export const load: PageServerLoad = async ({ url, fetch, cookies }) => {
  const code = url.searchParams.get("code");
  if (!code) {
    return { error: "missingToken" as const, title: TITLE };
  }
  // See `../+page.server.ts` for why a network-level failure is caught
  // separately from an upstream-rejected code.
  let upstream: Response;
  try {
    upstream = await verifySso(fetch, code);
  } catch {
    return { error: "serviceUnavailable" as const };
  }
  if (!upstream.ok) {
    return { error: "invalidToken" as const, title: TITLE };
  }
  const sid = sessionIdFromResponse(upstream);
  if (!sid) {
    return { error: "noSession" as const, title: TITLE };
  }
  cookies.set(SESSION_COOKIE, sid, SESSION_COOKIE_OPTIONS);
  redirect(303, "/");
};
