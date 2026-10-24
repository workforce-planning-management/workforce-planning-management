// Sign-out (BFF, WPM-T18): best-effort server-side session revocation
// at the authentication service, then clear the httpOnly cookie and
// send the browser home.

import type { RequestHandler } from "./$types";
import { redirect } from "@sveltejs/kit";
import { signout } from "#lib/server/auth.js";
import { tokens } from "#lib/server/token-cache.js";
import { isDemoSession } from "#lib/server/demo.js";
import { clearSessionCookie } from "#lib/server/session.js";

export const POST: RequestHandler = async ({ locals, cookies, fetch, url }) => {
  if (locals.sessionId && !isDemoSession(locals.sessionId)) {
    // Best effort: whatever the authentication service does, the visitor leaves.
    try {
      await signout(fetch, locals.sessionId);
    } catch {
      // The cookie below is cleared regardless.
    }
    tokens.forget(locals.sessionId);
  }
  clearSessionCookie(cookies, url);
  redirect(303, "/");
};
