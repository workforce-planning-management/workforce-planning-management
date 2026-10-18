// Sign-in action (BFF, per-app magic-link login, WPM-T18): the server
// asks the authentication service for a magic link that returns to
// THIS app's own `/verify` (via `return_url` = this origin, honoured
// by the auth service's allow-list). No credential is held
// client-side.

import type { Actions, PageServerLoad } from "./$types";
import { redirect } from "@sveltejs/kit";
import { localePath, splitLocale } from "#lib/locales.js";
import { pageTitle } from "#lib/title.js";
import { requestMagicLink } from "#lib/server/auth.js";
import {
  demoAvailable,
  isDemoEmail,
  newDemoSessionId,
} from "#lib/server/demo.js";
import { setSessionCookie } from "#lib/server/session.js";

// `page.data.title` convention (see `../+layout.svelte`): mirrors this
// route's own <svelte:head><title> so SharePicker gets the right title
// without reading the DOM.
export const load: PageServerLoad = ({ url }) => {
  return {
    title: pageTitle("titles.signin", url),
    demo: demoAvailable(),
  };
};

export const actions: Actions = {
  default: async ({ request, fetch, url, cookies }) => {
    const form = await request.formData();
    const email = String(form.get("email") ?? "").trim();
    if (!email) {
      return { sent: false, error: "email-required" };
    }
    // The example address opens the demo (fictitious data, a guided tour) where one is offered.
    if (demoAvailable() && isDemoEmail(email)) {
      setSessionCookie(cookies, url, newDemoSessionId());
      redirect(
        303,
        localePath(splitLocale(url.pathname).locale ?? "en-001", "/demo"),
      );
    }
    const ok = await requestMagicLink(fetch, email, url.origin);
    return ok ? { sent: true, error: null } : { sent: false, error: "failed" };
  },
};
