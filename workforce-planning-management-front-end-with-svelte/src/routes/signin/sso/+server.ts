// SSO login entry point (BFF, Keycloak via the authentication service).
// A GET here has no page of its own — it immediately redirects to the
// authentication service's SSO start endpoint, which owns the whole
// Keycloak exchange; only it, never this app, talks to the identity
// provider directly. Mirrors the magic-link flow's `return_url`
// convention (`../+page.server.ts`): the authentication service brings
// the browser back to THIS app's own `/verify/sso`.

import { redirect } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { ssoStartUrl } from "$lib/server/auth";

export const GET: RequestHandler = ({ url }) => {
  redirect(303, ssoStartUrl(`${url.origin}/verify/sso`));
};
